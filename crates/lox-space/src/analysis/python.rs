// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use crate::analysis::assets::{AssetId, GroundStation, Spacecraft};
use crate::analysis::visibility::{
    DynPass, ElevationMask, ElevationMaskError, PairType, Pass, VisibilityAnalysis,
    VisibilityError, VisibilityResults,
};
use crate::bodies::DynOrigin;
use crate::bodies::python::PyOrigin;
use crate::comms::python::PyCommunicationSystem;
use crate::ephem::python::PySpk;
use crate::orbits::ground::Observables;
use crate::orbits::python::{PyGroundLocation, PyInterval, PyTrajectory};
use crate::time::deltas::TimeDelta;
use crate::time::python::deltas::PyTimeDelta;
use crate::time::python::time::PyTime;
use crate::units::python::{PyAngle, PyAngularRate, PyDistance, PyVelocity};
use lox_time::intervals::TimeInterval;
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
    #[pyo3(signature = (id, location, mask, communication_systems=None))]
    fn new(
        id: String,
        location: PyGroundLocation,
        mask: PyElevationMask,
        communication_systems: Option<Vec<PyCommunicationSystem>>,
    ) -> Self {
        let mut gs = GroundStation::new(id, location.0, mask.0);
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

/// A named spacecraft for visibility analysis.
///
/// Wraps a trajectory with an identifier.
///
/// Args:
///     id: Unique identifier for this spacecraft.
///     trajectory: Spacecraft trajectory.
///     max_slew_rate: Optional maximum slew rate (angular rate) for this
///         spacecraft's antenna/gimbal.
///     communication_systems: Optional list of communication systems.
#[pyclass(name = "Spacecraft", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PySpacecraft(pub Spacecraft);

#[pymethods]
impl PySpacecraft {
    #[new]
    #[pyo3(signature = (id, trajectory, max_slew_rate=None, communication_systems=None))]
    fn new(
        id: String,
        trajectory: PyTrajectory,
        max_slew_rate: Option<PyAngularRate>,
        communication_systems: Option<Vec<PyCommunicationSystem>>,
    ) -> Self {
        let mut asset = Spacecraft::new(id, trajectory.0);
        if let Some(rate) = max_slew_rate {
            asset = asset.with_max_slew_rate(rate.0);
        }
        if let Some(systems) = communication_systems {
            for system in systems {
                asset = asset.with_communication_system(system.0);
            }
        }
        PySpacecraft(asset)
    }

    /// Return the asset identifier.
    fn id(&self) -> String {
        self.0.id().as_str().to_string()
    }

    /// Return the spacecraft trajectory.
    fn trajectory(&self) -> PyTrajectory {
        PyTrajectory(self.0.trajectory().clone())
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
        let traj = self.trajectory();
        format!("Spacecraft(\"{}\", {})", self.id(), traj.__repr__(),)
    }
}

/// Computes ground-station-to-spacecraft visibility.
///
/// Args:
///     ground_assets: List of GroundStation objects.
///     space_assets: List of Spacecraft objects.
///     occulting_bodies: Optional list of bodies for LOS checking.
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
    ground_assets: Vec<GroundStation>,
    space_assets: Vec<Spacecraft>,
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
    #[pyo3(signature = (ground_assets, space_assets, occulting_bodies=None, step=None, min_pass_duration=None, inter_satellite=false, min_range=None, max_range=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        ground_assets: Vec<PyGroundStation>,
        space_assets: Vec<PySpacecraft>,
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
            ground_assets: ground_assets.into_iter().map(|g| g.0).collect(),
            space_assets: space_assets.into_iter().map(|s| s.0).collect(),
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

    /// Compute visibility intervals for all (ground, space) pairs.
    ///
    /// Args:
    ///     start: Start time of the analysis period.
    ///     end: End time of the analysis period.
    ///     ephemeris: SPK ephemeris data.
    ///
    /// Returns:
    ///     VisibilityResults containing intervals for all pairs.
    fn compute(
        &self,
        py: Python<'_>,
        start: PyTime,
        end: PyTime,
        ephemeris: &Bound<'_, PySpk>,
    ) -> PyResult<PyVisibilityResults> {
        let ephemeris = &ephemeris.get().0;
        let interval = TimeInterval::new(start.0, end.0);

        let results = py.detach(|| {
            let mut analysis =
                VisibilityAnalysis::new(&self.ground_assets, &self.space_assets, ephemeris)
                    .with_occulting_bodies(self.occulting_bodies.clone())
                    .with_step(self.step);
            if let Some(mpd) = self.min_pass_duration {
                analysis = analysis.with_min_pass_duration(mpd);
            }
            if self.inter_satellite {
                analysis = analysis.with_inter_satellite();
            }
            if let Some(min_range) = self.min_range {
                analysis = analysis.with_min_range(min_range);
            }
            if let Some(max_range) = self.max_range {
                analysis = analysis.with_max_range(max_range);
            }
            analysis.compute(interval)
        });

        Ok(PyVisibilityResults {
            results: results.map_err(PyVisibilityError)?,
            ground_assets: self.ground_assets.clone(),
            space_assets: self.space_assets.clone(),
            step: self.step,
        })
    }

    fn __repr__(&self) -> String {
        if self.inter_satellite {
            format!(
                "VisibilityAnalysis({} ground assets, {} space assets, inter_satellite=True)",
                self.ground_assets.len(),
                self.space_assets.len(),
            )
        } else {
            format!(
                "VisibilityAnalysis({} ground assets, {} space assets)",
                self.ground_assets.len(),
                self.space_assets.len(),
            )
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
    ground_assets: Vec<GroundStation>,
    space_assets: Vec<Spacecraft>,
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
            .map(|intervals| intervals.iter().map(|i| PyInterval(*i)).collect())
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
                    intervals.iter().map(|i| PyInterval(*i)).collect(),
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
                    intervals.iter().map(|i| PyInterval(*i)).collect(),
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
                    intervals.iter().map(|i| PyInterval(*i)).collect(),
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

        let gs = self.ground_assets.iter().find(|g| g.id() == &gs_id);
        let sc = self.space_assets.iter().find(|s| s.id() == &sc_id);
        match (gs, sc) {
            (Some(gs), Some(sc)) => {
                let passes = self
                    .results
                    .to_passes(
                        &gs_id,
                        &sc_id,
                        gs.location(),
                        gs.mask(),
                        sc.trajectory(),
                        self.step,
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
        let gs_map: HashMap<&AssetId, &GroundStation> =
            self.ground_assets.iter().map(|g| (g.id(), g)).collect();
        let sc_map: HashMap<&AssetId, &Spacecraft> =
            self.space_assets.iter().map(|s| (s.id(), s)).collect();

        self.results
            .ground_space_pair_ids()
            .into_iter()
            .filter_map(|(gs_id, sc_id)| {
                let gs = gs_map.get(gs_id)?;
                let sc = sc_map.get(sc_id)?;
                let intervals = self.results.intervals_for(gs_id, sc_id)?;
                let passes: Vec<PyPass> = intervals
                    .iter()
                    .filter_map(|interval| {
                        DynPass::from_interval(
                            *interval,
                            self.step,
                            gs.location(),
                            gs.mask(),
                            sc.trajectory(),
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
