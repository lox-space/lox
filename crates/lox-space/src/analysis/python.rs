// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::{HashMap, HashSet};

use crate::analysis::assets::{AssetId, ConstellationId, DynScenario, GroundStation, Spacecraft};
use crate::analysis::events::{Event, ZeroCrossing};
use crate::analysis::imaging::{
    AccessError, Aoi, AoiId, LookSide, OpticalAccessAnalysis, OpticalPayload, SarAccessAnalysis,
    SarPayload,
};
use crate::analysis::power::{
    PowerBudgetAnalysis, PowerBudgetResults, PowerError, SpacecraftFilter,
};
use crate::analysis::sun::AnalyticalSunEphemeris;
use crate::analysis::visibility::{
    DynPass, ElevationMask, ElevationMaskError, PairType, Pass, VisibilityAnalysis,
    VisibilityError, VisibilityResults,
};
use crate::bodies::DynOrigin;
use crate::bodies::python::PyOrigin;
use crate::comms::python::{
    build_rx_terminal, build_tx_terminal, rx_terminal_to_py, tx_terminal_to_py,
};
use crate::ephem::python::PySpk;
use crate::ephem::spk::parser::Spk;
use crate::frames::python::PyFrame;
use crate::orbits::ground::Observables;
use crate::orbits::python::{
    PyGroundLocation, PyJ2Propagator, PyJ4Propagator, PyNumericalPropagator, PySgp4, PyTrajectory,
    PyVallado,
};
use crate::time::deltas::TimeDelta;
use crate::time::python::deltas::PyTimeDelta;
use crate::time::python::intervals::PyInterval;
use crate::time::python::time::PyTime;
use crate::time::python::time_series::PyTimeSeries;
use crate::units::python::{PyAngle, PyAngularRate, PyDistance, PyVelocity};
use lox_frames::DynFrame;
use lox_frames::providers::DefaultRotationProvider;
use lox_orbits::orbits::Ensemble;
use lox_orbits::propagators::OrbitSource;
use lox_time::intervals::TimeInterval;
use lox_time::series::TimeSeries;
use lox_time::time_scales::{DynTimeScale, Tai};
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

/// Error wrapper converting `ElevationMaskError` into a Python `ValueError`.
pub struct PyElevationMaskError(pub ElevationMaskError);

impl From<PyElevationMaskError> for PyErr {
    fn from(err: PyElevationMaskError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// Represents a detected event (zero-crossing of a function).
///
/// Events are detected when a monitored function crosses zero during
/// trajectory analysis. The crossing direction indicates whether the
/// function went from negative to positive ("up") or positive to negative ("down").
///
/// Args:
///     time: The time of the event.
///     crossing: The crossing direction ("up" or "down").
#[pyclass(name = "Event", module = "lox_space", frozen, from_py_object)]
#[derive(Clone, Debug)]
pub struct PyEvent(pub Event<DynTimeScale>);

#[pymethods]
impl PyEvent {
    #[new]
    fn new(time: PyTime, crossing: &str) -> PyResult<Self> {
        let crossing = match crossing {
            "up" => ZeroCrossing::Up,
            "down" => ZeroCrossing::Down,
            _ => return Err(PyValueError::new_err("crossing must be 'up' or 'down'")),
        };
        Ok(PyEvent(Event::new(time.0, crossing)))
    }

    fn __repr__(&self) -> String {
        format!("Event({}, \"{}\")", self.time().__repr__(), self.crossing(),)
    }

    fn __str__(&self) -> String {
        format!(
            "Event - {}crossing at {}",
            self.crossing(),
            self.time().__str__()
        )
    }

    /// Return the time of this event.
    fn time(&self) -> PyTime {
        PyTime(self.0.time())
    }

    /// Return the crossing direction ("up" or "down").
    fn crossing(&self) -> String {
        self.0.crossing().to_string()
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
///     tx_terminals: Optional dict of named transmit terminals (TxChain or EirpModel).
///     rx_terminals: Optional dict of named receive terminals (RxChain or GtModel).
#[pyclass(name = "GroundStation", module = "lox_space", frozen, from_py_object)]
#[derive(Clone, Debug)]
pub struct PyGroundStation(pub GroundStation);

#[pymethods]
impl PyGroundStation {
    #[new]
    #[pyo3(signature = (id, location, mask, body_fixed_frame=None, network_id=None, tx_terminals=None, rx_terminals=None))]
    fn new(
        id: String,
        location: PyGroundLocation,
        mask: PyElevationMask,
        body_fixed_frame: Option<PyFrame>,
        network_id: Option<String>,
        tx_terminals: Option<HashMap<String, Bound<'_, PyAny>>>,
        rx_terminals: Option<HashMap<String, Bound<'_, PyAny>>>,
    ) -> PyResult<Self> {
        let mut gs = GroundStation::new(id, location.0, mask.0);
        if let Some(frame) = body_fixed_frame {
            gs = gs.with_body_fixed_frame(frame.0);
        }
        if let Some(nid) = network_id {
            gs = gs.with_network_id(nid);
        }
        for (name, terminal) in tx_terminals.unwrap_or_default() {
            gs = gs.with_tx_terminal(name, build_tx_terminal(&terminal)?);
        }
        for (name, terminal) in rx_terminals.unwrap_or_default() {
            gs = gs.with_rx_terminal(name, build_rx_terminal(&terminal)?);
        }
        Ok(PyGroundStation(gs))
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

    /// Return the network identifier, if assigned.
    fn network_id(&self) -> Option<String> {
        self.0.network_id().map(|id| id.as_str().to_string())
    }

    /// Return the body-fixed frame.
    fn body_fixed_frame(&self) -> PyFrame {
        PyFrame(self.0.body_fixed_frame())
    }

    /// Return the named transmit terminals as a dict.
    fn tx_terminals<'py>(&self, py: Python<'py>) -> HashMap<String, Bound<'py, PyAny>> {
        self.0
            .tx_terminals()
            .iter()
            .map(|(name, terminal)| (name.clone(), tx_terminal_to_py(py, terminal)))
            .collect()
    }

    /// Return the named receive terminals as a dict.
    fn rx_terminals<'py>(&self, py: Python<'py>) -> HashMap<String, Bound<'py, PyAny>> {
        self.0
            .rx_terminals()
            .iter()
            .map(|(name, terminal)| (name.clone(), rx_terminal_to_py(py, terminal)))
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
    if let Ok(n) = obj.extract::<PyNumericalPropagator>() {
        return Ok(OrbitSource::Numerical(n.0));
    }
    if let Ok(p) = obj.extract::<PyJ2Propagator>() {
        return Ok(OrbitSource::J2(p.0));
    }
    if let Ok(p) = obj.extract::<PyJ4Propagator>() {
        return Ok(OrbitSource::J4(p.0));
    }
    if let Ok(traj) = obj.extract::<PyTrajectory>() {
        return Ok(OrbitSource::Trajectory(traj.0));
    }
    Err(PyValueError::new_err(
        "expected a propagator (SGP4, Vallado, Numerical, J2, J4) or Trajectory object",
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
///     tx_terminals: Optional dict of named transmit terminals (TxChain or EirpModel).
///     rx_terminals: Optional dict of named receive terminals (RxChain or GtModel).
#[pyclass(name = "Spacecraft", module = "lox_space", frozen, from_py_object)]
#[derive(Clone, Debug)]
pub struct PySpacecraft(pub Spacecraft);

#[pymethods]
impl PySpacecraft {
    #[new]
    #[pyo3(signature = (id, orbit, max_slew_rate=None, constellation_id=None, optical_payload=None, sar_payload=None, tx_terminals=None, rx_terminals=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        id: String,
        orbit: &Bound<'_, PyAny>,
        max_slew_rate: Option<PyAngularRate>,
        constellation_id: Option<String>,
        optical_payload: Option<PyOpticalPayload>,
        sar_payload: Option<PySarPayload>,
        tx_terminals: Option<HashMap<String, Bound<'_, PyAny>>>,
        rx_terminals: Option<HashMap<String, Bound<'_, PyAny>>>,
    ) -> PyResult<Self> {
        let orbit_source = extract_orbit_source(orbit)?;
        let mut asset = Spacecraft::new(id, orbit_source);
        if let Some(rate) = max_slew_rate {
            asset = asset.with_max_slew_rate(rate.0);
        }
        if let Some(cid) = constellation_id {
            asset = asset.with_constellation_id(cid);
        }
        if let Some(payload) = optical_payload {
            asset = asset.with_optical_payload(payload.0);
        }
        if let Some(payload) = sar_payload {
            asset = asset.with_sar_payload(payload.0);
        }
        for (name, terminal) in tx_terminals.unwrap_or_default() {
            asset = asset.with_tx_terminal(name, build_tx_terminal(&terminal)?);
        }
        for (name, terminal) in rx_terminals.unwrap_or_default() {
            asset = asset.with_rx_terminal(name, build_rx_terminal(&terminal)?);
        }
        Ok(PySpacecraft(asset))
    }

    /// Return the asset identifier.
    fn id(&self) -> String {
        self.0.id().as_str().to_string()
    }

    /// Return the constellation identifier, if assigned.
    fn constellation_id(&self) -> Option<String> {
        self.0.constellation_id().map(|id| id.as_str().to_string())
    }

    /// Return the maximum slew rate, if set.
    fn max_slew_rate(&self) -> Option<PyAngularRate> {
        self.0.max_slew_rate().map(PyAngularRate)
    }

    /// Return the optical payload, if set.
    fn optical_payload(&self) -> Option<PyOpticalPayload> {
        self.0.optical_payload().map(PyOpticalPayload)
    }

    /// Return the SAR payload, if set.
    fn sar_payload(&self) -> Option<PySarPayload> {
        self.0.sar_payload().map(PySarPayload)
    }

    /// Return the named transmit terminals as a dict.
    fn tx_terminals<'py>(&self, py: Python<'py>) -> HashMap<String, Bound<'py, PyAny>> {
        self.0
            .tx_terminals()
            .iter()
            .map(|(name, terminal)| (name.clone(), tx_terminal_to_py(py, terminal)))
            .collect()
    }

    /// Return the named receive terminals as a dict.
    fn rx_terminals<'py>(&self, py: Python<'py>) -> HashMap<String, Bound<'py, PyAny>> {
        self.0
            .rx_terminals()
            .iter()
            .map(|(name, terminal)| (name.clone(), rx_terminal_to_py(py, terminal)))
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
#[pyclass(name = "Scenario", module = "lox_space", frozen, from_py_object)]
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
#[pyclass(name = "Ensemble", module = "lox_space", frozen, from_py_object)]
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
///     ground_space_filter: Optional callable ``(GroundStation, Spacecraft) -> bool``
///         that receives a ground station and spacecraft and returns whether the
///         pair should be evaluated. Called once per candidate pair before the
///         parallel phase.
///     inter_satellite_filter: Optional callable ``(Spacecraft, Spacecraft) -> bool``
///         that receives two spacecraft and returns whether the pair should be
///         evaluated. Called once per candidate pair before the parallel phase.
///         When provided, inter-satellite visibility is automatically enabled.
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
    ground_space_filter: Option<Py<PyAny>>,
    inter_satellite_filter: Option<Py<PyAny>>,
    min_range: Option<Distance>,
    max_range: Option<Distance>,
}

#[pymethods]
impl PyVisibilityAnalysis {
    #[new]
    #[pyo3(signature = (scenario, ensemble=None, occulting_bodies=None, step=None, min_pass_duration=None, inter_satellite=false, ground_space_filter=None, inter_satellite_filter=None, min_range=None, max_range=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python<'_>,
        scenario: PyScenario,
        ensemble: Option<PyEnsemble>,
        occulting_bodies: Option<Vec<Bound<'_, PyAny>>>,
        step: Option<PyTimeDelta>,
        min_pass_duration: Option<PyTimeDelta>,
        inter_satellite: bool,
        ground_space_filter: Option<Py<PyAny>>,
        inter_satellite_filter: Option<Py<PyAny>>,
        min_range: Option<PyDistance>,
        max_range: Option<PyDistance>,
    ) -> PyResult<Self> {
        let occulting_bodies: Vec<DynOrigin> = occulting_bodies
            .unwrap_or_default()
            .iter()
            .map(|b| Ok(PyOrigin::try_from(b)?.0))
            .collect::<PyResult<_>>()?;
        if let Some(ref f) = ground_space_filter
            && !f.bind(py).is_callable()
        {
            return Err(PyValueError::new_err(
                "ground_space_filter must be callable",
            ));
        }
        if let Some(ref f) = inter_satellite_filter
            && !f.bind(py).is_callable()
        {
            return Err(PyValueError::new_err(
                "inter_satellite_filter must be callable",
            ));
        }
        Ok(Self {
            scenario: scenario.0,
            ensemble: ensemble.map(|e| e.0),
            occulting_bodies,
            step: step
                .map(|s| s.0)
                .unwrap_or_else(|| TimeDelta::from_seconds_f64(60.0)),
            min_pass_duration: min_pass_duration.map(|d| d.0),
            inter_satellite,
            ground_space_filter,
            inter_satellite_filter,
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
    ///     ephemeris: SPK ephemeris data. Required when ``occulting_bodies``
    ///         is non-empty; optional otherwise.
    ///
    /// Returns:
    ///     VisibilityResults containing intervals for all pairs.
    ///
    /// Raises:
    ///     ValueError: if occulting bodies are configured but no
    ///         ephemeris is provided.
    #[pyo3(signature = (ephemeris=None))]
    fn compute(
        &self,
        py: Python<'_>,
        ephemeris: Option<&Bound<'_, PySpk>>,
    ) -> PyResult<PyVisibilityResults> {
        if !self.occulting_bodies.is_empty() && ephemeris.is_none() {
            return Err(PyValueError::new_err(
                "ephemeris is required when occulting_bodies is set",
            ));
        }

        let ephemeris_ref: Option<&Spk> = ephemeris.map(|e| &e.get().0);
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

        // Eagerly evaluate Python filters while we hold the GIL, collecting
        // accepted pairs into plain sets that can cross into the GIL-free
        // parallel section.
        let gs_accepted: Option<HashSet<(AssetId, AssetId)>> =
            if let Some(ref filter) = self.ground_space_filter {
                let ground_stations = scenario.ground_stations();
                let spacecraft = scenario.spacecraft();
                let mut set = HashSet::new();
                for gs in ground_stations {
                    for sc in spacecraft {
                        let py_gs = PyGroundStation(gs.clone());
                        let py_sc = PySpacecraft(sc.clone());
                        let accept: bool = filter.call1(py, (py_gs, py_sc))?.extract(py)?;
                        if accept {
                            set.insert((gs.id().clone(), sc.id().clone()));
                        }
                    }
                }
                Some(set)
            } else {
                None
            };

        let isl_accepted: Option<HashSet<(AssetId, AssetId)>> =
            if let Some(ref filter) = self.inter_satellite_filter {
                let spacecraft = scenario.spacecraft();
                let n = spacecraft.len();
                let mut set = HashSet::new();
                for i in 0..n {
                    for j in (i + 1)..n {
                        let py_sc1 = PySpacecraft(spacecraft[i].clone());
                        let py_sc2 = PySpacecraft(spacecraft[j].clone());
                        let accept: bool = filter.call1(py, (py_sc1, py_sc2))?.extract(py)?;
                        if accept {
                            set.insert((spacecraft[i].id().clone(), spacecraft[j].id().clone()));
                        }
                    }
                }
                Some(set)
            } else {
                None
            };

        let results = py.detach(|| -> Result<VisibilityResults, VisibilityError> {
            let analysis = VisibilityAnalysis::new(scenario, ensemble).with_step(step);
            let analysis = match min_pass_duration {
                Some(d) => analysis.with_min_pass_duration(d),
                None => analysis,
            };
            let analysis = if let Some(ref accepted) = gs_accepted {
                analysis.with_ground_space_filter(move |gs, sc| {
                    accepted.contains(&(gs.id().clone(), sc.id().clone()))
                })
            } else {
                analysis
            };
            let analysis = if let Some(ref accepted) = isl_accepted {
                analysis.with_inter_satellite_filter(move |sc1, sc2| {
                    accepted.contains(&(sc1.id().clone(), sc2.id().clone()))
                })
            } else if inter_satellite {
                analysis.with_inter_satellite()
            } else {
                analysis
            };
            let analysis = match min_range {
                Some(r) => analysis.with_min_range(r),
                None => analysis,
            };
            let analysis = match max_range {
                Some(r) => analysis.with_max_range(r),
                None => analysis,
            };

            if let Some(eph) = ephemeris_ref {
                analysis
                    .with_occulting_bodies(eph, occulting_bodies)
                    .compute()
            } else {
                analysis.compute()
            }
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
        let mut extras = Vec::new();
        if self.ground_space_filter.is_some() {
            extras.push("ground_space_filter=True".to_string());
        }
        if self.inter_satellite_filter.is_some() {
            extras.push("inter_satellite_filter=True".to_string());
        } else if self.inter_satellite {
            extras.push("inter_satellite=True".to_string());
        }
        if extras.is_empty() {
            format!("VisibilityAnalysis({gs_count} ground assets, {sc_count} space assets)")
        } else {
            format!(
                "VisibilityAnalysis({gs_count} ground assets, {sc_count} space assets, {})",
                extras.join(", "),
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
#[pyclass(
    name = "ElevationMask",
    module = "lox_space",
    frozen,
    eq,
    from_py_object
)]
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
#[pyclass(name = "Observables", module = "lox_space", frozen, from_py_object)]
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
#[pyclass(name = "Pass", module = "lox_space", frozen, from_py_object)]
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

// ---------------------------------------------------------------------------
// PowerBudgetAnalysis / PowerBudgetResults Python bindings
// ---------------------------------------------------------------------------

struct PyPowerError(PowerError);

impl From<PyPowerError> for PyErr {
    fn from(err: PyPowerError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// Power budget analysis for spacecraft in a scenario.
///
/// Computes eclipse intervals, sun beta angle, and solar flux for each
/// spacecraft.  The shadow model is cylindrical (umbra only) — penumbra
/// is **not** modelled.
///
/// Args:
///     scenario: Scenario containing spacecraft and time interval.
///     ensemble: Optional pre-computed Ensemble. If not provided, the
///         scenario is propagated automatically.
///     step: Optional time step for sampling / event detection (default: 60s).
///     spacecraft_ids: Optional list of spacecraft ids to analyse. Mutually
///         exclusive with ``constellation_id``.
///     constellation_id: Optional constellation id — only spacecraft belonging
///         to this constellation are analysed. Mutually exclusive with
///         ``spacecraft_ids``.
#[pyclass(name = "PowerBudgetAnalysis", module = "lox_space", frozen)]
pub struct PyPowerBudgetAnalysis {
    scenario: DynScenario,
    ensemble: Option<Ensemble<AssetId, Tai, DynOrigin, DynFrame>>,
    step: TimeDelta,
    filter: Option<SpacecraftFilter>,
}

#[pymethods]
impl PyPowerBudgetAnalysis {
    #[new]
    #[pyo3(signature = (scenario, ensemble=None, step=None, spacecraft_ids=None, constellation_id=None))]
    fn new(
        scenario: PyScenario,
        ensemble: Option<PyEnsemble>,
        step: Option<PyTimeDelta>,
        spacecraft_ids: Option<Vec<String>>,
        constellation_id: Option<String>,
    ) -> PyResult<Self> {
        let filter = match (spacecraft_ids, constellation_id) {
            (Some(_), Some(_)) => {
                return Err(PyValueError::new_err(
                    "spacecraft_ids and constellation_id are mutually exclusive",
                ));
            }
            (Some(ids), None) => Some(SpacecraftFilter::Ids(
                ids.into_iter().map(AssetId::new).collect(),
            )),
            (None, Some(cid)) => Some(SpacecraftFilter::Constellation(ConstellationId::new(cid))),
            (None, None) => None,
        };
        Ok(Self {
            scenario: scenario.0,
            ensemble: ensemble.map(|e| e.0),
            step: step
                .map(|s| s.0)
                .unwrap_or_else(|| TimeDelta::from_seconds_f64(60.0)),
            filter,
        })
    }

    /// Compute the power budget analysis.
    ///
    /// Args:
    ///     ephemeris: Optional SPK ephemeris for Sun position. When omitted,
    ///         an analytical model is used (valid for Earth-centred scenarios).
    ///
    /// Returns:
    ///     PowerBudgetResults with eclipse intervals, beta angles, and
    ///     solar flux for each spacecraft.
    #[pyo3(signature = (ephemeris=None))]
    fn compute(
        &self,
        py: Python<'_>,
        ephemeris: Option<&Bound<'_, PySpk>>,
    ) -> PyResult<PyPowerBudgetResults> {
        let scenario = &self.scenario;
        let step = self.step;
        let filter = self.filter.clone();

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

        let results = if let Some(spk_bound) = ephemeris {
            let spk = &spk_bound.get().0;
            py.detach(|| {
                let mut a = PowerBudgetAnalysis::new(scenario, ensemble, spk).with_step(step);
                if let Some(f) = &filter {
                    a = a.with_filter(f.clone());
                }
                a.compute()
            })
        } else {
            let analytical = AnalyticalSunEphemeris;
            py.detach(|| {
                let mut a =
                    PowerBudgetAnalysis::new(scenario, ensemble, &analytical).with_step(step);
                if let Some(f) = &filter {
                    a = a.with_filter(f.clone());
                }
                a.compute()
            })
        };

        Ok(PyPowerBudgetResults {
            results: results.map_err(PyPowerError)?,
        })
    }

    fn __repr__(&self) -> String {
        let sc_count = self.scenario.spacecraft().len();
        match &self.filter {
            Some(SpacecraftFilter::Ids(ids)) => format!(
                "PowerBudgetAnalysis({sc_count} spacecraft, filtered to {} ids)",
                ids.len()
            ),
            Some(SpacecraftFilter::Constellation(cid)) => {
                format!("PowerBudgetAnalysis({sc_count} spacecraft, constellation=\"{cid}\")",)
            }
            None => format!("PowerBudgetAnalysis({sc_count} spacecraft)"),
        }
    }
}

/// Convert a `TimeSeries<Tai>` to a `PyTimeSeries` (which uses `DynTimeScale`).
fn to_py_time_series(ts: &TimeSeries<Tai>) -> PyTimeSeries {
    let dyn_ts = TimeSeries::new(ts.epoch().into_dyn(), ts.series().clone());
    PyTimeSeries(dyn_ts)
}

/// Results of a power budget analysis.
///
/// Provides access to eclipse intervals, eclipse/sunlit fractions,
/// beta-angle time series, and solar-flux time series for each spacecraft.
#[pyclass(name = "PowerBudgetResults", module = "lox_space", frozen)]
pub struct PyPowerBudgetResults {
    results: PowerBudgetResults,
}

#[pymethods]
impl PyPowerBudgetResults {
    /// Eclipse intervals for a given spacecraft.
    ///
    /// Args:
    ///     id: Spacecraft identifier.
    ///
    /// Returns:
    ///     List of Interval objects, or empty list if id not found.
    fn eclipse_intervals(&self, id: &str) -> Vec<PyInterval> {
        let asset_id = AssetId::new(id);
        self.results
            .eclipse_intervals_for(&asset_id)
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

    /// Eclipse fraction for a given spacecraft (0 = fully sunlit, 1 = always eclipsed).
    ///
    /// Args:
    ///     id: Spacecraft identifier.
    ///
    /// Returns:
    ///     Eclipse fraction as float, or None if id not found.
    fn eclipse_fraction(&self, id: &str) -> Option<f64> {
        self.results.eclipse_fraction(&AssetId::new(id))
    }

    /// Sunlit fraction for a given spacecraft (1 - eclipse_fraction).
    ///
    /// Args:
    ///     id: Spacecraft identifier.
    ///
    /// Returns:
    ///     Sunlit fraction as float, or None if id not found.
    fn sunlit_fraction(&self, id: &str) -> Option<f64> {
        self.results.sunlit_fraction(&AssetId::new(id))
    }

    /// Beta-angle time series for a given spacecraft (radians).
    ///
    /// Args:
    ///     id: Spacecraft identifier.
    ///
    /// Returns:
    ///     TimeSeries of beta angles in radians, or None if id not found.
    fn beta_angles(&self, id: &str) -> Option<PyTimeSeries> {
        let ts = self.results.beta_angles_for(&AssetId::new(id))?;
        Some(to_py_time_series(ts))
    }

    /// Solar-flux time series for a given spacecraft (W/m²).
    ///
    /// Args:
    ///     id: Spacecraft identifier.
    ///
    /// Returns:
    ///     TimeSeries of solar flux in W/m², or None if id not found.
    fn solar_flux(&self, id: &str) -> Option<PyTimeSeries> {
        let ts = self.results.solar_flux_for(&AssetId::new(id))?;
        Some(to_py_time_series(ts))
    }

    fn __repr__(&self) -> String {
        let n = self.results.all_eclipse_intervals().len();
        format!("PowerBudgetResults({n} spacecraft)")
    }
}

// ---------------------------------------------------------------------------
// Imaging Python bindings (optical + SAR)
// ---------------------------------------------------------------------------

struct PyAccessError(AccessError);

impl From<PyAccessError> for PyErr {
    fn from(err: PyAccessError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// An area of interest (AOI) defined as a geographic polygon.
///
/// Coordinates follow GeoJSON convention: longitude/latitude in degrees.
///
/// Args:
///     coords: List of (longitude, latitude) tuples in degrees forming the
///         polygon exterior ring. The ring should be closed (first == last).
#[pyclass(name = "Aoi", module = "lox_space", frozen, from_py_object)]
#[derive(Clone, Debug)]
pub struct PyAoi(pub Aoi);

#[pymethods]
impl PyAoi {
    #[new]
    fn new(coords: Vec<(f64, f64)>) -> Self {
        let line_string = geo::LineString::from(coords);
        let polygon = geo::Polygon::new(line_string, vec![]);
        PyAoi(Aoi::new(polygon))
    }

    /// Parse an AOI from a GeoJSON string.
    ///
    /// Expects a GeoJSON Polygon geometry, Feature containing a Polygon,
    /// or FeatureCollection containing a Feature with a Polygon.
    ///
    /// Args:
    ///     geojson: GeoJSON string.
    ///
    /// Returns:
    ///     Aoi parsed from the GeoJSON.
    #[classmethod]
    fn from_geojson(_cls: &Bound<'_, PyType>, geojson: &str) -> PyResult<Self> {
        let aoi = Aoi::from_geojson(geojson).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PyAoi(aoi))
    }

    fn __repr__(&self) -> String {
        let n = self.0.polygon().exterior().0.len();
        format!("Aoi({n} vertices)")
    }
}

/// Optical imaging payload describing a spacecraft's ground coverage capability.
///
/// Defines the sensor's swath width and optional off-nadir pointing capability.
/// Assign to a spacecraft via the ``optical_payload`` parameter.
#[pyclass(name = "OpticalPayload", module = "lox_space", frozen, from_py_object)]
#[derive(Clone, Debug)]
pub struct PyOpticalPayload(pub OpticalPayload);

#[pymethods]
impl PyOpticalPayload {
    /// Create parameters for a nadir-only sensor.
    ///
    /// Args:
    ///     swath_width: Full swath width as Distance.
    ///
    /// Returns:
    ///     OpticalPayload for nadir-only imaging.
    #[classmethod]
    fn nadir_only(_cls: &Bound<'_, PyType>, swath_width: PyDistance) -> Self {
        PyOpticalPayload(OpticalPayload::nadir_only(swath_width.0))
    }

    /// Create parameters for a sensor with off-nadir pointing capability.
    ///
    /// Args:
    ///     swath_width: Full swath width as Distance.
    ///     max_off_nadir: Maximum off-nadir angle as Angle.
    ///
    /// Returns:
    ///     OpticalPayload for off-nadir imaging.
    #[classmethod]
    fn off_nadir(
        _cls: &Bound<'_, PyType>,
        swath_width: PyDistance,
        max_off_nadir: PyAngle,
    ) -> Self {
        PyOpticalPayload(OpticalPayload::off_nadir(swath_width.0, max_off_nadir.0))
    }

    fn __repr__(&self) -> String {
        "OpticalPayload(...)".to_string()
    }
}

/// AOI optical access analysis: computes imaging windows for spacecraft over AOIs.
///
/// Optical payloads are read from each spacecraft; spacecraft without an
/// optical payload are skipped.
///
/// Args:
///     scenario: Scenario containing spacecraft and time interval.
///         Spacecraft must have an ``optical_payload`` assigned.
///     aois: List of (id, Aoi) tuples defining the areas of interest.
///     ensemble: Optional pre-computed Ensemble. If not provided, the
///         scenario is propagated automatically.
///     step: Optional time step for event detection (default: 60s).
///     body_fixed_frame: Optional body-fixed frame override (e.g. "ITRF").
///         Defaults to IAU frame of the scenario's origin.
#[pyclass(name = "OpticalAccessAnalysis", module = "lox_space", frozen)]
pub struct PyOpticalAccessAnalysis {
    scenario: DynScenario,
    aois: Vec<(AoiId, Aoi)>,
    ensemble: Option<Ensemble<AssetId, Tai, DynOrigin, DynFrame>>,
    step: TimeDelta,
    body_fixed_frame: Option<DynFrame>,
}

#[pymethods]
impl PyOpticalAccessAnalysis {
    #[new]
    #[pyo3(signature = (scenario, aois, ensemble=None, step=None, body_fixed_frame=None))]
    fn new(
        scenario: PyScenario,
        aois: Vec<(String, PyAoi)>,
        ensemble: Option<PyEnsemble>,
        step: Option<PyTimeDelta>,
        body_fixed_frame: Option<PyFrame>,
    ) -> Self {
        let aois = aois
            .into_iter()
            .map(|(id, aoi)| (AoiId::new(id), aoi.0))
            .collect();
        Self {
            scenario: scenario.0,
            aois,
            ensemble: ensemble.map(|e| e.0),
            step: step
                .map(|s| s.0)
                .unwrap_or_else(|| TimeDelta::from_seconds_f64(60.0)),
            body_fixed_frame: body_fixed_frame.map(|f| f.0),
        }
    }

    /// Compute optical access intervals for all (spacecraft, AOI) pairs.
    ///
    /// If no ensemble was provided at construction, the scenario is
    /// propagated automatically (trajectories transformed to ICRF).
    ///
    /// Returns:
    ///     AccessResults containing intervals for all pairs.
    fn compute(&self, py: Python<'_>) -> PyResult<PyAccessResults> {
        let scenario = &self.scenario;
        let step = self.step;
        let body_fixed_frame = self.body_fixed_frame;

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

        let aois = self.aois.clone();

        let results = py.detach(|| {
            let mut analysis = OpticalAccessAnalysis::new(scenario, ensemble, aois).with_step(step);
            if let Some(frame) = body_fixed_frame {
                analysis = analysis.with_body_fixed_frame(frame);
            }
            analysis.compute()
        });

        Ok(PyAccessResults {
            results: results.map_err(PyAccessError)?,
        })
    }

    fn __repr__(&self) -> String {
        let sc_count = self.scenario.spacecraft().len();
        let aoi_count = self.aois.len();
        let aoi_label = if aoi_count == 1 { "AOI" } else { "AOIs" };
        format!("OpticalAccessAnalysis({sc_count} spacecraft, {aoi_count} {aoi_label})")
    }
}

/// Direction of orbital motion at the time of an access window.
#[pyclass(
    name = "PassDirection",
    module = "lox_space",
    eq,
    eq_int,
    hash,
    frozen,
    from_py_object
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PyPassDirection {
    /// Spacecraft is moving from south to north at the access midpoint.
    Ascending,
    /// Spacecraft is moving from north to south at the access midpoint.
    Descending,
}

impl From<PyPassDirection> for crate::analysis::imaging::PassDirection {
    fn from(d: PyPassDirection) -> Self {
        match d {
            PyPassDirection::Ascending => Self::Ascending,
            PyPassDirection::Descending => Self::Descending,
        }
    }
}

impl From<crate::analysis::imaging::PassDirection> for PyPassDirection {
    fn from(d: crate::analysis::imaging::PassDirection) -> Self {
        match d {
            crate::analysis::imaging::PassDirection::Ascending => Self::Ascending,
            crate::analysis::imaging::PassDirection::Descending => Self::Descending,
        }
    }
}

/// A single access window: time interval + pass direction at the midpoint.
#[pyclass(name = "AccessWindow", module = "lox_space", frozen, from_py_object)]
#[derive(Clone, Copy)]
pub struct PyAccessWindow(pub crate::analysis::imaging::AccessWindow);

#[pymethods]
impl PyAccessWindow {
    /// The access time interval.
    fn interval(&self) -> PyInterval {
        PyInterval(TimeInterval::new(
            self.0.interval.start().into_dyn(),
            self.0.interval.end().into_dyn(),
        ))
    }

    /// The spacecraft pass direction at the interval midpoint.
    fn direction(&self) -> PyPassDirection {
        self.0.direction.into()
    }

    fn __repr__(&self) -> String {
        let dir = match self.0.direction {
            crate::analysis::imaging::PassDirection::Ascending => "Ascending",
            crate::analysis::imaging::PassDirection::Descending => "Descending",
        };
        format!(
            "AccessWindow({} → {}, {dir})",
            self.0.interval.start(),
            self.0.interval.end(),
        )
    }
}

/// Results of an imaging access analysis (optical or SAR).
///
/// Provides access windows for each (spacecraft, AOI) pair.
#[pyclass(name = "AccessResults", module = "lox_space", frozen)]
pub struct PyAccessResults {
    results: crate::analysis::imaging::AccessResults,
}

#[pymethods]
impl PyAccessResults {
    /// Return access windows for a specific (spacecraft, AOI) pair.
    ///
    /// Args:
    ///     spacecraft_id: Spacecraft identifier.
    ///     aoi_id: AOI identifier.
    ///
    /// Returns:
    ///     List of AccessWindow objects, or empty list if pair not found.
    fn windows(&self, spacecraft_id: &str, aoi_id: &str) -> Vec<PyAccessWindow> {
        let sc_id = AssetId::new(spacecraft_id);
        let aoi_id = AoiId::new(aoi_id);
        self.results
            .windows(&sc_id, &aoi_id)
            .iter()
            .map(|w| PyAccessWindow(*w))
            .collect()
    }

    /// Return all access windows for all (spacecraft, AOI) pairs.
    ///
    /// Returns:
    ///     Dictionary mapping (spacecraft_id, aoi_id) to list of AccessWindow objects.
    fn all_windows(&self) -> HashMap<(String, String), Vec<PyAccessWindow>> {
        self.results
            .all_windows()
            .iter()
            .map(|((sc_id, aoi_id), windows)| {
                (
                    (sc_id.as_str().to_string(), aoi_id.as_str().to_string()),
                    windows.iter().map(|w| PyAccessWindow(*w)).collect(),
                )
            })
            .collect()
    }

    fn __repr__(&self) -> String {
        let n = self.results.num_pairs();
        let label = if n == 1 { "pair" } else { "pairs" };
        format!("AccessResults({n} {label})")
    }
}

// ---------------------------------------------------------------------------
// SAR Python bindings: LookSide, SarPayload, SarAccessAnalysis
// ---------------------------------------------------------------------------

/// Which side of the ground track a SAR payload can image.
#[pyclass(
    name = "LookSide",
    module = "lox_space",
    eq,
    eq_int,
    hash,
    frozen,
    from_py_object
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PyLookSide {
    /// SAR payload images to the left of the ground track.
    Left,
    /// SAR payload images to the right of the ground track.
    Right,
    /// SAR payload can image on either side of the ground track.
    Either,
}

impl From<PyLookSide> for LookSide {
    fn from(s: PyLookSide) -> Self {
        match s {
            PyLookSide::Left => LookSide::Left,
            PyLookSide::Right => LookSide::Right,
            PyLookSide::Either => LookSide::Either,
        }
    }
}

impl From<LookSide> for PyLookSide {
    fn from(s: LookSide) -> Self {
        match s {
            LookSide::Left => PyLookSide::Left,
            LookSide::Right => PyLookSide::Right,
            LookSide::Either => PyLookSide::Either,
        }
    }
}

/// SAR (Synthetic Aperture Radar) payload — side-looking annular access geometry.
///
/// Construct via :meth:`with_look_angles` (look angle at the satellite) or
/// :meth:`with_incidence_angles` (incidence angle at the ground point).
///
/// Assign to a spacecraft via the ``sar_payload`` parameter.
///
/// ```python
/// import lox_space as lox
/// payload = lox.SarPayload.with_incidence_angles(29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right)
/// sc = lox.Spacecraft("sat1", orbit, sar_payload=payload)
/// ```
#[pyclass(name = "SarPayload", module = "lox_space", frozen, from_py_object)]
#[derive(Clone, Copy)]
pub struct PySarPayload(pub SarPayload);

#[pymethods]
impl PySarPayload {
    /// Constructs a SAR payload from a look-angle envelope.
    ///
    /// Args:
    ///     min: Minimum look angle (off-nadir at the satellite).
    ///     max: Maximum look angle (off-nadir at the satellite).
    ///     side: Which side of the ground track the payload can image.
    ///
    /// Returns:
    ///     SarPayload for the given envelope.
    ///
    /// Raises:
    ///     ValueError: if min ≥ max or either angle is outside [0°, 90°).
    #[classmethod]
    fn with_look_angles(
        _cls: &Bound<'_, PyType>,
        min: PyAngle,
        max: PyAngle,
        side: PyLookSide,
    ) -> PyResult<Self> {
        SarPayload::with_look_angles(min.0, max.0, side.into())
            .map(PySarPayload)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Constructs a SAR payload from an incidence-angle envelope.
    ///
    /// Args:
    ///     min: Minimum incidence angle (off-vertical at the ground point).
    ///     max: Maximum incidence angle (off-vertical at the ground point).
    ///     side: Which side of the ground track the payload can image.
    ///
    /// Returns:
    ///     SarPayload for the given envelope.
    ///
    /// Raises:
    ///     ValueError: if min ≥ max or either angle is outside [0°, 90°).
    #[classmethod]
    fn with_incidence_angles(
        _cls: &Bound<'_, PyType>,
        min: PyAngle,
        max: PyAngle,
        side: PyLookSide,
    ) -> PyResult<Self> {
        SarPayload::with_incidence_angles(min.0, max.0, side.into())
            .map(PySarPayload)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Returns the configured looking side.
    fn side(&self) -> PyLookSide {
        self.0.side().into()
    }

    fn __repr__(&self) -> String {
        "SarPayload(...)".to_string()
    }
}

/// AOI SAR access analysis: computes imaging windows for SAR spacecraft over AOIs.
///
/// SAR payloads are read from each spacecraft; spacecraft without a SAR
/// payload are skipped.
///
/// Args:
///     scenario: Scenario containing spacecraft and time interval.
///         Spacecraft must have a ``sar_payload`` assigned.
///     aois: List of (id, Aoi) tuples defining the areas of interest.
///     ensemble: Optional pre-computed Ensemble. If not provided, the
///         scenario is propagated automatically.
///     step: Optional time step for event detection (default: 60s).
///     body_fixed_frame: Optional body-fixed frame override (e.g. "ITRF").
///         Defaults to IAU frame of the scenario's origin.
#[pyclass(name = "SarAccessAnalysis", module = "lox_space", frozen)]
pub struct PySarAccessAnalysis {
    scenario: DynScenario,
    aois: Vec<(AoiId, Aoi)>,
    ensemble: Option<Ensemble<AssetId, Tai, DynOrigin, DynFrame>>,
    step: TimeDelta,
    body_fixed_frame: Option<DynFrame>,
}

#[pymethods]
impl PySarAccessAnalysis {
    #[new]
    #[pyo3(signature = (scenario, aois, ensemble=None, step=None, body_fixed_frame=None))]
    fn new(
        scenario: PyScenario,
        aois: Vec<(String, PyAoi)>,
        ensemble: Option<PyEnsemble>,
        step: Option<PyTimeDelta>,
        body_fixed_frame: Option<PyFrame>,
    ) -> Self {
        let aois = aois
            .into_iter()
            .map(|(id, aoi)| (AoiId::new(id), aoi.0))
            .collect();
        Self {
            scenario: scenario.0,
            aois,
            ensemble: ensemble.map(|e| e.0),
            step: step
                .map(|s| s.0)
                .unwrap_or_else(|| TimeDelta::from_seconds_f64(60.0)),
            body_fixed_frame: body_fixed_frame.map(|f| f.0),
        }
    }

    /// Compute SAR access intervals for all (spacecraft, AOI) pairs.
    ///
    /// If no ensemble was provided at construction, the scenario is
    /// propagated automatically (trajectories transformed to ICRF).
    ///
    /// Returns:
    ///     AccessResults containing intervals for all pairs.
    fn compute(&self, py: Python<'_>) -> PyResult<PyAccessResults> {
        let scenario = &self.scenario;
        let step = self.step;
        let body_fixed_frame = self.body_fixed_frame;

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

        let aois = self.aois.clone();

        let results = py.detach(|| {
            let mut analysis = SarAccessAnalysis::new(scenario, ensemble, aois).with_step(step);
            if let Some(frame) = body_fixed_frame {
                analysis = analysis.with_body_fixed_frame(frame);
            }
            analysis.compute()
        });

        Ok(PyAccessResults {
            results: results.map_err(PyAccessError)?,
        })
    }

    fn __repr__(&self) -> String {
        let sc_count = self.scenario.spacecraft().len();
        let aoi_count = self.aois.len();
        let aoi_label = if aoi_count == 1 { "AOI" } else { "AOIs" };
        format!("SarAccessAnalysis({sc_count} spacecraft, {aoi_count} {aoi_label})")
    }
}
