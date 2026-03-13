// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use crate::analysis::assets::{AssetId, DynScenario, GroundStation, Spacecraft};
use crate::analysis::visibility::{
    DynPass, ElevationMask, ElevationMaskError, PairType, Pass, VisibilityAnalysis,
    VisibilityError, VisibilityResults,
};
use crate::bodies::DynOrigin;
use crate::bodies::python::PyOrigin;
use crate::comms::python::PyCommunicationSystem;
use crate::ephem::python::PySpk;
use crate::frames::python::PyFrame;
use crate::orbits::ground::Observables;
use crate::orbits::python::{
    PyGroundLocation, PyInterval, PyJ2Propagator, PySgp4, PyTrajectory, PyVallado,
};
use crate::time::deltas::TimeDelta;
use crate::time::python::deltas::PyTimeDelta;
use crate::time::python::time::PyTime;
use crate::units::python::{PyAngle, PyAngularRate, PyDistance, PyVelocity};
use lox_frames::DynFrame;
use lox_frames::providers::DefaultRotationProvider;
use lox_orbits::orbits::Ensemble;
use lox_orbits::propagators::OrbitSource;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::Tai;
use lox_units::{Angle, Distance, Velocity};

use numpy::{PyArray1, PyArrayMethods};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;

struct PyVisibilityError(VisibilityError);

impl From<PyVisibilityError> for PyErr {
    fn from(err: PyVisibilityError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

pub struct PyElevationMaskError(pub ElevationMaskError);

impl From<PyElevationMaskError> for PyErr {
    fn from(err: PyElevationMaskError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// A named ground station for visibility analysis.
///
/// Wraps a ground location and elevation mask with an identifier.
///
/// Args:
///     id: Unique identifier for this ground station.
///     location: Ground station location.
///     mask: Elevation mask defining minimum elevation constraints.
///     communication_systems: Optional list of communication systems.
#[pyclass(name = "GroundStation", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyGroundStation(pub GroundStation);

#[pymethods]
impl PyGroundStation {
    #[new]
    #[pyo3(signature = (id, location, mask, body_fixed_frame=None, communication_systems=None))]
    fn new(
        id: String,
        location: PyGroundLocation,
        mask: PyElevationMask,
        body_fixed_frame: Option<PyFrame>,
        communication_systems: Option<Vec<PyCommunicationSystem>>,
    ) -> Self {
        let mut gs = GroundStation::new(id, location.0, mask.0);
        if let Some(frame) = body_fixed_frame {
            gs = gs.with_body_fixed_frame(frame.0);
        }
        if let Some(systems) = communication_systems {
            for system in systems {
                gs = gs.with_communication_system(system.0);
            }
        }
        PyGroundStation(gs)
    }

    /// Return the asset identifier.
    fn id(&self) -> String {
        self.0.id().as_str().to_string()
    }

    /// Return the ground location.
    fn location(&self) -> PyGroundLocation {
        PyGroundLocation(self.0.location().clone())
    }

    /// Return the elevation mask.
    fn mask(&self) -> PyElevationMask {
        PyElevationMask(self.0.mask().clone())
    }

    /// Return the body-fixed frame.
    fn body_fixed_frame(&self) -> PyFrame {
        PyFrame(self.0.body_fixed_frame())
    }

    /// Return the communication systems.
    fn communication_systems(&self) -> Vec<PyCommunicationSystem> {
        self.0
            .communication_systems()
            .iter()
            .map(|s| PyCommunicationSystem(s.clone()))
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "GroundStation(\"{}\", {}, {})",
            self.id(),
            self.location().__repr__(),
            self.mask().__repr__(),
        )
    }
}

/// Extract an OrbitSource from a Python object (SGP4, Vallado, J2, or Trajectory).
fn extract_orbit_source(obj: &Bound<'_, PyAny>) -> PyResult<OrbitSource> {
    if let Ok(sgp4) = obj.extract::<PySgp4>() {
        return Ok(OrbitSource::Sgp4(sgp4.inner));
    }
    if let Ok(vallado) = obj.extract::<PyVallado>() {
        return Ok(OrbitSource::Vallado(vallado.0));
    }
    if let Ok(j2) = obj.extract::<PyJ2Propagator>() {
        return Ok(OrbitSource::J2(j2.0));
    }
    if let Ok(traj) = obj.extract::<PyTrajectory>() {
        return Ok(OrbitSource::Trajectory(traj.0));
    }
    Err(PyValueError::new_err(
        "expected an SGP4, Vallado, J2, or Trajectory object",
    ))
}

/// A named spacecraft for visibility analysis.
///
/// Wraps an orbit source (propagator or pre-computed trajectory) with an
/// identifier.
///
/// Args:
///     id: Unique identifier for this spacecraft.
///     orbit: Orbit source — an SGP4, Vallado, J2 propagator, or a
///         pre-computed Trajectory.
///     max_slew_rate: Optional maximum slew rate (angular rate) for this
///         spacecraft's antenna/gimbal.
///     communication_systems: Optional list of communication systems.
#[pyclass(name = "Spacecraft", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PySpacecraft(pub Spacecraft);

#[pymethods]
impl PySpacecraft {
    #[new]
    #[pyo3(signature = (id, orbit, max_slew_rate=None, communication_systems=None))]
    fn new(
        id: String,
        orbit: &Bound<'_, PyAny>,
        max_slew_rate: Option<PyAngularRate>,
        communication_systems: Option<Vec<PyCommunicationSystem>>,
    ) -> PyResult<Self> {
        let orbit_source = extract_orbit_source(orbit)?;
        let mut asset = Spacecraft::new(id, orbit_source);
        if let Some(rate) = max_slew_rate {
            asset = asset.with_max_slew_rate(rate.0);
        }
        if let Some(systems) = communication_systems {
            for system in systems {
                asset = asset.with_communication_system(system.0);
            }
        }
        Ok(PySpacecraft(asset))
    }

    /// Return the asset identifier.
    fn id(&self) -> String {
        self.0.id().as_str().to_string()
    }

    /// Return the maximum slew rate, if set.
    fn max_slew_rate(&self) -> Option<PyAngularRate> {
        self.0.max_slew_rate().map(PyAngularRate)
    }

    /// Return the communication systems.
    fn communication_systems(&self) -> Vec<PyCommunicationSystem> {
        self.0
            .communication_systems()
            .iter()
            .map(|s| PyCommunicationSystem(s.clone()))
            .collect()
    }

    fn __repr__(&self) -> String {
        format!("Spacecraft(\"{}\")", self.id())
    }
}

/// A scenario grouping spacecraft, ground stations, and a time interval.
///
/// Args:
///     start: Start time of the scenario.
///     end: End time of the scenario.
///     spacecraft: List of Spacecraft objects.
///     ground_stations: List of GroundStation objects.
#[pyclass(name = "Scenario", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyScenario(pub DynScenario);

#[pymethods]
impl PyScenario {
    #[new]
    #[pyo3(signature = (start, end, spacecraft=None, ground_stations=None))]
    fn new(
        start: PyTime,
        end: PyTime,
        spacecraft: Option<Vec<PySpacecraft>>,
        ground_stations: Option<Vec<PyGroundStation>>,
    ) -> Self {
        let tai_start = start.0.to_scale(Tai);
        let tai_end = end.0.to_scale(Tai);
        let mut scenario = DynScenario::new(tai_start, tai_end, DynOrigin::Earth, DynFrame::Icrf);
        if let Some(sc) = spacecraft {
            let sc_vec: Vec<Spacecraft> = sc.into_iter().map(|s| s.0).collect();
            scenario = scenario.with_spacecraft(&sc_vec);
        }
        if let Some(gs) = ground_stations {
            let gs_vec: Vec<GroundStation> = gs.into_iter().map(|g| g.0).collect();
            scenario = scenario.with_ground_stations(&gs_vec);
        }
        PyScenario(scenario)
    }

    /// Propagate all spacecraft, returning an Ensemble.
    ///
    /// Trajectories are transformed to ICRF using the default rotation
    /// provider.
    fn propagate(&self, py: Python<'_>) -> PyResult<PyEnsemble> {
        let ensemble = py.detach(|| self.0.propagate(&DefaultRotationProvider));
        Ok(PyEnsemble(
            ensemble.map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }

    /// Return the start time.
    fn start(&self) -> PyTime {
        PyTime(self.0.interval().start().into_dyn())
    }

    /// Add a constellation to the scenario, converting all its satellites
    /// to spacecraft using the constellation's selected propagator.
    fn with_constellation(
        &self,
        constellation: crate::constellations::python::PyConstellation,
    ) -> PyResult<Self> {
        let scenario = self
            .0
            .clone()
            .with_constellation(constellation.0)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PyScenario(scenario))
    }

    /// Return the end time.
    fn end(&self) -> PyTime {
        PyTime(self.0.interval().end().into_dyn())
    }

    fn __repr__(&self) -> String {
        format!(
            "Scenario({} spacecraft, {} ground stations)",
            self.0.spacecraft().len(),
            self.0.ground_stations().len(),
        )
    }
}

/// A collection of propagated trajectories keyed by spacecraft id.
#[pyclass(name = "Ensemble", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyEnsemble(pub Ensemble<AssetId, Tai, DynOrigin, DynFrame>);

#[pymethods]
impl PyEnsemble {
    /// Return the trajectory for a given spacecraft id.
    fn get(&self, id: &str) -> Option<PyTrajectory> {
        self.0
            .get(&AssetId::new(id))
            .map(|t| PyTrajectory(t.clone().into_dyn()))
    }

    fn __len__(&self) -> usize {
        self.0.len()
    }

    fn __repr__(&self) -> String {
        format!("Ensemble({} trajectories)", self.0.len())
    }
}

/// Computes ground-station-to-spacecraft and inter-satellite visibility.
///
/// Args:
///     scenario: Scenario containing spacecraft, ground stations, and
///         time interval.
///     ephemeris: SPK ephemeris data.
///     ensemble: Optional pre-computed Ensemble. If not provided, the
///         scenario is propagated automatically.
///     occulting_bodies: Optional list of additional occulting bodies for
///         LOS checking. For inter-satellite visibility, the scenario's
///         central body is always checked automatically.
///     step: Optional time step for event detection (default: 60s).
///     min_pass_duration: Optional minimum pass duration. Passes shorter
///         than this value may be missed. Enables two-level stepping for faster
///         detection.
///     inter_satellite: If True, also compute inter-satellite visibility
///         for all unique spacecraft pairs (default: False).
///     min_range: Optional minimum range constraint for inter-satellite pairs.
///     max_range: Optional maximum range constraint for inter-satellite pairs.
#[pyclass(name = "VisibilityAnalysis", module = "lox_space", frozen)]
pub struct PyVisibilityAnalysis {
    scenario: DynScenario,
    ensemble: Option<Ensemble<AssetId, Tai, DynOrigin, DynFrame>>,
    occulting_bodies: Vec<DynOrigin>,
    step: TimeDelta,
    min_pass_duration: Option<TimeDelta>,
    inter_satellite: bool,
    min_range: Option<Distance>,
    max_range: Option<Distance>,
}

#[pymethods]
impl PyVisibilityAnalysis {
    #[new]
    #[pyo3(signature = (scenario, ensemble=None, occulting_bodies=None, step=None, min_pass_duration=None, inter_satellite=false, min_range=None, max_range=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        scenario: PyScenario,
        ensemble: Option<PyEnsemble>,
        occulting_bodies: Option<Vec<Bound<'_, PyAny>>>,
        step: Option<PyTimeDelta>,
        min_pass_duration: Option<PyTimeDelta>,
        inter_satellite: bool,
        min_range: Option<PyDistance>,
        max_range: Option<PyDistance>,
    ) -> PyResult<Self> {
        let occulting_bodies: Vec<DynOrigin> = occulting_bodies
            .unwrap_or_default()
            .iter()
            .map(|b| Ok(PyOrigin::try_from(b)?.0))
            .collect::<PyResult<_>>()?;
        Ok(Self {
            scenario: scenario.0,
            ensemble: ensemble.map(|e| e.0),
            occulting_bodies,
            step: step
                .map(|s| s.0)
                .unwrap_or_else(|| TimeDelta::from_seconds_f64(60.0)),
            min_pass_duration: min_pass_duration.map(|d| d.0),
            inter_satellite,
            min_range: min_range.map(|d| d.0),
            max_range: max_range.map(|d| d.0),
        })
    }

    /// Compute visibility intervals for all pairs.
    ///
    /// If no ensemble was provided at construction, the scenario is
    /// propagated automatically (trajectories transformed to ICRF).
    ///
    /// Args:
    ///     ephemeris: SPK ephemeris data.
    ///
    /// Returns:
    ///     VisibilityResults containing intervals for all pairs.
    fn compute(
        &self,
        py: Python<'_>,
        ephemeris: &Bound<'_, PySpk>,
    ) -> PyResult<PyVisibilityResults> {
        let ephemeris = &ephemeris.get().0;
        let step = self.step;
        let scenario = &self.scenario;

        // Auto-propagate if no ensemble was provided.
        let auto_ensemble;
        let ensemble = match &self.ensemble {
            Some(e) => e,
            None => {
                auto_ensemble = scenario
                    .propagate(&DefaultRotationProvider)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                &auto_ensemble
            }
        };

        let occulting_bodies = self.occulting_bodies.clone();
        let min_pass_duration = self.min_pass_duration;
        let inter_satellite = self.inter_satellite;
        let min_range = self.min_range;
        let max_range = self.max_range;

        let results = py.detach(|| {
            let mut analysis = VisibilityAnalysis::new(scenario, ensemble, ephemeris)
                .with_occulting_bodies(occulting_bodies)
                .with_step(step);
            if let Some(mpd) = min_pass_duration {
                analysis = analysis.with_min_pass_duration(mpd);
            }
            if inter_satellite {
                analysis = analysis.with_inter_satellite();
            }
            if let Some(min_range) = min_range {
                analysis = analysis.with_min_range(min_range);
            }
            if let Some(max_range) = max_range {
                analysis = analysis.with_max_range(max_range);
            }
            analysis.compute()
        });

        Ok(PyVisibilityResults {
            results: results.map_err(PyVisibilityError)?,
            scenario: self.scenario.clone(),
            ensemble: ensemble.clone(),
            step: self.step,
        })
    }

    fn __repr__(&self) -> String {
        let sc_count = self.scenario.spacecraft().len();
        let gs_count = self.scenario.ground_stations().len();
        if self.inter_satellite {
            format!(
                "VisibilityAnalysis({gs_count} ground assets, {sc_count} space assets, inter_satellite=True)",
            )
        } else {
            format!("VisibilityAnalysis({gs_count} ground assets, {sc_count} space assets)",)
        }
    }
}

/// Results of a visibility analysis.
///
/// Provides lazy access to visibility intervals and passes. Intervals
/// (time windows) are computed eagerly; observables-rich Pass objects are
/// computed on demand to avoid unnecessary work.
#[pyclass(name = "VisibilityResults", module = "lox_space", frozen)]
pub struct PyVisibilityResults {
    results: VisibilityResults,
    scenario: DynScenario,
    ensemble: Ensemble<AssetId, Tai, DynOrigin, DynFrame>,
    step: TimeDelta,
}

#[pymethods]
impl PyVisibilityResults {
    /// Return visibility intervals for a specific pair.
    ///
    /// Args:
    ///     id1: First asset identifier (ground or space).
    ///     id2: Second asset identifier (space).
    ///
    /// Returns:
    ///     List of Interval objects, or empty list if pair not found.
    fn intervals(&self, id1: &str, id2: &str) -> Vec<PyInterval> {
        let id1 = AssetId::new(id1);
        let id2 = AssetId::new(id2);
        self.results
            .intervals_for(&id1, &id2)
            .map(|intervals| {
                intervals
                    .iter()
                    .map(|i| {
                        PyInterval(TimeInterval::new(i.start().into_dyn(), i.end().into_dyn()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Return all intervals for all pairs.
    ///
    /// Returns:
    ///     Dictionary mapping (id1, id2) to list of Interval objects.
    fn all_intervals(&self) -> HashMap<(String, String), Vec<PyInterval>> {
        self.results
            .all_intervals()
            .iter()
            .map(|((id1, id2), intervals)| {
                (
                    (id1.as_str().to_string(), id2.as_str().to_string()),
                    intervals
                        .iter()
                        .map(|i| {
                            PyInterval(TimeInterval::new(i.start().into_dyn(), i.end().into_dyn()))
                        })
                        .collect(),
                )
            })
            .collect()
    }

    /// Return intervals for ground-to-space pairs only.
    ///
    /// Returns:
    ///     Dictionary mapping (ground_id, space_id) to list of Interval objects.
    fn ground_space_intervals(&self) -> HashMap<(String, String), Vec<PyInterval>> {
        self.results
            .ground_space_pair_ids()
            .into_iter()
            .filter_map(|(gs_id, sc_id)| {
                let intervals = self.results.intervals_for(gs_id, sc_id)?;
                Some((
                    (gs_id.as_str().to_string(), sc_id.as_str().to_string()),
                    intervals
                        .iter()
                        .map(|i| {
                            PyInterval(TimeInterval::new(i.start().into_dyn(), i.end().into_dyn()))
                        })
                        .collect(),
                ))
            })
            .collect()
    }

    /// Return intervals for inter-satellite pairs only.
    ///
    /// Returns:
    ///     Dictionary mapping (sc1_id, sc2_id) to list of Interval objects.
    fn inter_satellite_intervals(&self) -> HashMap<(String, String), Vec<PyInterval>> {
        self.results
            .inter_satellite_pair_ids()
            .into_iter()
            .filter_map(|(sc1_id, sc2_id)| {
                let intervals = self.results.intervals_for(sc1_id, sc2_id)?;
                Some((
                    (sc1_id.as_str().to_string(), sc2_id.as_str().to_string()),
                    intervals
                        .iter()
                        .map(|i| {
                            PyInterval(TimeInterval::new(i.start().into_dyn(), i.end().into_dyn()))
                        })
                        .collect(),
                ))
            })
            .collect()
    }

    /// Compute passes with observables for a specific ground-to-space pair.
    ///
    /// This is more expensive than `intervals()` as it computes azimuth,
    /// elevation, range, and range rate for each time step.
    ///
    /// Raises ValueError for inter-satellite pairs since ground-station
    /// observables are not meaningful for them.
    ///
    /// Args:
    ///     ground_id: Ground asset identifier.
    ///     space_id: Space asset identifier.
    ///
    /// Returns:
    ///     List of Pass objects, or empty list if pair not found.
    fn passes(&self, ground_id: &str, space_id: &str) -> PyResult<Vec<PyPass>> {
        let gs_id = AssetId::new(ground_id);
        let sc_id = AssetId::new(space_id);

        // Check if this is an inter-satellite pair before looking up assets.
        if self.results.pair_type(&gs_id, &sc_id) == Some(PairType::InterSatellite) {
            return Err(PyValueError::new_err(format!(
                "passes are not supported for inter-satellite pair ({}, {}): use intervals() instead",
                ground_id, space_id,
            )));
        }

        let gs = self
            .scenario
            .ground_stations()
            .iter()
            .find(|g| g.id() == &gs_id);
        let sc_traj = self.ensemble.get(&sc_id);
        match (gs, sc_traj) {
            (Some(gs), Some(sc_traj)) => {
                let dyn_traj = sc_traj.clone().into_dyn();
                let passes = self
                    .results
                    .to_passes(
                        &gs_id,
                        &sc_id,
                        gs.location(),
                        gs.mask(),
                        &dyn_traj,
                        self.step,
                        gs.body_fixed_frame(),
                    )
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(passes.into_iter().map(PyPass).collect())
            }
            _ => Ok(vec![]),
        }
    }

    /// Compute passes for all ground-to-space pairs.
    ///
    /// Inter-satellite pairs are skipped since ground-station observables
    /// are not meaningful for them.
    ///
    /// Returns:
    ///     Dictionary mapping (ground_id, space_id) to list of Pass objects.
    fn all_passes(&self) -> HashMap<(String, String), Vec<PyPass>> {
        let gs_map: HashMap<&AssetId, &GroundStation> = self
            .scenario
            .ground_stations()
            .iter()
            .map(|g| (g.id(), g))
            .collect();

        self.results
            .ground_space_pair_ids()
            .into_iter()
            .filter_map(|(gs_id, sc_id)| {
                let gs = gs_map.get(gs_id)?;
                let sc_traj = self.ensemble.get(sc_id)?;
                let dyn_traj = sc_traj.clone().into_dyn();
                let intervals = self.results.intervals_for(gs_id, sc_id)?;
                let passes: Vec<PyPass> = intervals
                    .iter()
                    .filter_map(|interval| {
                        let dyn_interval = TimeInterval::new(
                            interval.start().into_dyn(),
                            interval.end().into_dyn(),
                        );
                        DynPass::from_interval(
                            dyn_interval,
                            self.step,
                            gs.location(),
                            gs.mask(),
                            &dyn_traj,
                            gs.body_fixed_frame(),
                        )
                    })
                    .map(PyPass)
                    .collect();
                Some((
                    (gs_id.as_str().to_string(), sc_id.as_str().to_string()),
                    passes,
                ))
            })
            .collect()
    }

    /// Return all pair identifiers.
    fn pair_ids(&self) -> Vec<(String, String)> {
        self.results
            .pair_ids()
            .map(|(id1, id2)| (id1.as_str().to_string(), id2.as_str().to_string()))
            .collect()
    }

    /// Return pair identifiers for ground-to-space pairs only.
    fn ground_space_pair_ids(&self) -> Vec<(String, String)> {
        self.results
            .ground_space_pair_ids()
            .into_iter()
            .map(|(id1, id2)| (id1.as_str().to_string(), id2.as_str().to_string()))
            .collect()
    }

    /// Return pair identifiers for inter-satellite pairs only.
    fn inter_satellite_pair_ids(&self) -> Vec<(String, String)> {
        self.results
            .inter_satellite_pair_ids()
            .into_iter()
            .map(|(id1, id2)| (id1.as_str().to_string(), id2.as_str().to_string()))
            .collect()
    }

    /// Return the total number of pairs.
    fn num_pairs(&self) -> usize {
        self.results.num_pairs()
    }

    /// Return the total number of visibility intervals across all pairs.
    fn total_intervals(&self) -> usize {
        self.results.total_intervals()
    }

    fn __repr__(&self) -> String {
        format!(
            "VisibilityResults({} pairs, {} intervals)",
            self.results.num_pairs(),
            self.results.total_intervals(),
        )
    }
}

/// Defines elevation constraints for visibility analysis.
///
/// An elevation mask specifies the minimum elevation angle required for
/// visibility at different azimuth angles. Can be either fixed (constant
/// minimum elevation) or variable (azimuth-dependent).
///
/// Args:
///     azimuth: Array of azimuth angles in radians (for variable mask).
///     elevation: Array of minimum elevations in radians (for variable mask).
///     min_elevation: Fixed minimum elevation in radians.
#[pyclass(name = "ElevationMask", module = "lox_space", frozen, eq)]
#[derive(Debug, Clone, PartialEq)]
pub struct PyElevationMask(pub ElevationMask);

#[pymethods]
impl PyElevationMask {
    #[new]
    #[pyo3(signature = (azimuth=None, elevation=None, min_elevation=None))]
    fn new(
        azimuth: Option<&Bound<'_, PyArray1<f64>>>,
        elevation: Option<&Bound<'_, PyArray1<f64>>>,
        min_elevation: Option<PyAngle>,
    ) -> PyResult<Self> {
        if let Some(min_elevation) = min_elevation {
            return Ok(PyElevationMask(ElevationMask::with_fixed_elevation(
                min_elevation.0.to_radians(),
            )));
        }
        if let (Some(azimuth), Some(elevation)) = (azimuth, elevation) {
            let azimuth = azimuth.to_vec()?;
            let elevation = elevation.to_vec()?;
            return Ok(PyElevationMask(
                ElevationMask::new(azimuth, elevation).map_err(PyElevationMaskError)?,
            ));
        }
        Err(PyValueError::new_err(
            "invalid argument combination, either `min_elevation` or `azimuth` and `elevation` arrays need to be present",
        ))
    }

    /// Create a fixed elevation mask with constant minimum elevation.
    ///
    /// Args:
    ///     min_elevation: Minimum elevation angle as Angle.
    ///
    /// Returns:
    ///     ElevationMask with fixed minimum elevation.
    #[classmethod]
    fn fixed(_cls: &Bound<'_, PyType>, min_elevation: PyAngle) -> Self {
        PyElevationMask(ElevationMask::with_fixed_elevation(
            min_elevation.0.to_radians(),
        ))
    }

    /// Create a variable elevation mask from azimuth-dependent data.
    ///
    /// Args:
    ///     azimuth: Array of azimuth angles in radians.
    ///     elevation: Array of minimum elevations in radians.
    ///
    /// Returns:
    ///     ElevationMask with variable minimum elevation.
    #[classmethod]
    fn variable(
        _cls: &Bound<'_, PyType>,
        azimuth: &Bound<'_, PyArray1<f64>>,
        elevation: &Bound<'_, PyArray1<f64>>,
    ) -> PyResult<Self> {
        let azimuth = azimuth.to_vec()?;
        let elevation = elevation.to_vec()?;
        Ok(PyElevationMask(
            ElevationMask::new(azimuth, elevation).map_err(PyElevationMaskError)?,
        ))
    }

    fn __getnewargs__(&self) -> (Option<Vec<f64>>, Option<Vec<f64>>, Option<PyAngle>) {
        (self.azimuth(), self.elevation(), self.fixed_elevation())
    }

    /// Return the azimuth array (for variable masks only).
    fn azimuth(&self) -> Option<Vec<f64>> {
        match &self.0 {
            ElevationMask::Fixed(_) => None,
            ElevationMask::Variable(series) => Some(series.x().to_vec()),
        }
    }

    /// Return the elevation array (for variable masks only).
    fn elevation(&self) -> Option<Vec<f64>> {
        match &self.0 {
            ElevationMask::Fixed(_) => None,
            ElevationMask::Variable(series) => Some(series.y().to_vec()),
        }
    }

    /// Return the fixed elevation value (for fixed masks only).
    fn fixed_elevation(&self) -> Option<PyAngle> {
        match &self.0 {
            ElevationMask::Fixed(min_elevation) => Some(PyAngle(Angle::radians(*min_elevation))),
            ElevationMask::Variable(_) => None,
        }
    }

    /// Return the minimum elevation at the given azimuth.
    ///
    /// Args:
    ///     azimuth: Azimuth angle as Angle.
    ///
    /// Returns:
    ///     Minimum elevation as Angle.
    fn min_elevation(&self, azimuth: PyAngle) -> PyAngle {
        PyAngle(Angle::radians(self.0.min_elevation(azimuth.0.to_radians())))
    }

    fn __repr__(&self) -> String {
        match &self.0 {
            ElevationMask::Fixed(min_elevation) => {
                format!(
                    "ElevationMask(min_elevation={})",
                    PyAngle(Angle::radians(*min_elevation)).__repr__(),
                )
            }
            ElevationMask::Variable(series) => {
                let n = series.x().len();
                format!("ElevationMask({n} azimuth/elevation pairs)")
            }
        }
    }
}

/// Observation data from a ground station to a target.
///
/// Observables contain the geometric relationship between a ground station
/// and a spacecraft, including angles and range information.
///
/// Args:
///     azimuth: Azimuth angle as Angle.
///     elevation: Elevation angle as Angle.
///     range: Distance to target as Distance.
///     range_rate: Rate of change of range as Velocity.
#[pyclass(name = "Observables", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyObservables(pub Observables);

#[pymethods]
impl PyObservables {
    #[new]
    fn new(
        azimuth: PyAngle,
        elevation: PyAngle,
        range: PyDistance,
        range_rate: PyVelocity,
    ) -> Self {
        PyObservables(Observables::new(
            azimuth.0.to_radians(),
            elevation.0.to_radians(),
            range.0.to_meters(),
            range_rate.0.to_meters_per_second(),
        ))
    }

    /// Return the azimuth angle.
    fn azimuth(&self) -> PyAngle {
        PyAngle(Angle::radians(self.0.azimuth()))
    }

    /// Return the elevation angle.
    fn elevation(&self) -> PyAngle {
        PyAngle(Angle::radians(self.0.elevation()))
    }

    /// Return the range (distance).
    fn range(&self) -> PyDistance {
        PyDistance(Distance::meters(self.0.range()))
    }

    /// Return the range rate.
    fn range_rate(&self) -> PyVelocity {
        PyVelocity(Velocity::meters_per_second(self.0.range_rate()))
    }

    fn __repr__(&self) -> String {
        format!(
            "Observables({}, {}, {}, {})",
            self.azimuth().__repr__(),
            self.elevation().__repr__(),
            self.range().__repr__(),
            self.range_rate().__repr__(),
        )
    }
}

/// Represents a visibility pass between a ground station and spacecraft.
///
/// A Pass contains the visibility interval (start and end times) along with
/// observables computed at regular intervals throughout the pass.
#[pyclass(name = "Pass", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyPass(pub DynPass);

#[pymethods]
impl PyPass {
    #[new]
    fn new(
        interval: PyInterval,
        times: Vec<PyTime>,
        observables: Vec<PyObservables>,
    ) -> PyResult<Self> {
        let times: Vec<crate::time::DynTime> = times.into_iter().map(|t| t.0).collect();
        let observables: Vec<Observables> = observables.into_iter().map(|o| o.0).collect();

        let pass = Pass::try_new(interval.0, times, observables)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(PyPass(pass))
    }

    /// Return the visibility interval for this pass.
    fn interval(&self) -> PyInterval {
        PyInterval(*self.0.interval())
    }

    /// Return the time samples during this pass.
    fn times(&self) -> Vec<PyTime> {
        self.0.times().iter().map(|&t| PyTime(t)).collect()
    }

    /// Return the observables at each time sample.
    fn observables(&self) -> Vec<PyObservables> {
        self.0
            .observables()
            .iter()
            .map(|o| PyObservables(o.clone()))
            .collect()
    }

    /// Interpolate observables at a specific time within the pass.
    ///
    /// Args:
    ///     time: Time to interpolate at.
    ///
    /// Returns:
    ///     Interpolated Observables, or None if time is outside the pass.
    fn interpolate(&self, time: PyTime) -> Option<PyObservables> {
        self.0.interpolate(time.0).map(PyObservables)
    }

    fn __repr__(&self) -> String {
        format!(
            "Pass(interval={}, {} observables)",
            self.interval().__repr__(),
            self.0.observables().len(),
        )
    }
}
