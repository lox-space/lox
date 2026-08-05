// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use lox_analysis::imaging::AccessAnalysis;
use lox_analysis::pipeline::{
    AnalysisError, Eclipse, NoEphemeris, NoEphemerisError, Parallelism, Window,
};
use lox_analysis::power::{PowerBudgetAnalysis, SpacecraftPower};
use lox_analysis::visibility::{InterSatelliteAnalysis, VisibilityAnalysis};
use lox_bodies::CoordinateOrigin;
use lox_core::coords::Cartesian;
use lox_ephem::Ephemeris;
use lox_ephem::spk::parser::DafSpkError;
use lox_time::Time;
use lox_time::time_scales::Tdb;
use pyo3::create_exception;

create_exception!(
    lox_space,
    PyAnalysisError,
    pyo3::exceptions::PyException,
    "Base class for analysis failures."
);
create_exception!(
    lox_space,
    PyRotationFailed,
    PyAnalysisError,
    "A frame rotation failed."
);
create_exception!(
    lox_space,
    PyEphemerisFailed,
    PyAnalysisError,
    "An ephemeris lookup failed."
);
create_exception!(
    lox_space,
    PyDetectionFailed,
    PyAnalysisError,
    "Root-finding or event detection failed."
);

use crate::analysis::assets::{AssetId, GroundStation, Scenario, Spacecraft};
use crate::analysis::events::{Event, ZeroCrossing};
use crate::analysis::imaging::{AccessError, Aoi, AoiId, LookSide, OpticalPayload, SarPayload};
use crate::analysis::sun::{AnalyticalSunEphemeris, AnalyticalSunEphemerisError};
use crate::analysis::visibility::{ElevationMask, ElevationMaskError, Pass};
use crate::bodies::Origin;
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
use lox_frames::Frame;
use lox_frames::providers::DefaultRotationProvider;
use lox_orbits::orbits::Ensemble;
use lox_orbits::propagators::OrbitSource;
use lox_time::intervals::TimeInterval;
use lox_units::{Angle, Distance, Velocity};

use numpy::{PyArray1, PyArrayMethods};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;

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
pub struct PyEvent(pub Event);

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
pub struct PyScenario(pub Scenario);

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
        let mut scenario = Scenario::new(start.0, end.0, Origin::Earth, Frame::Icrf);
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
        PyTime(self.0.interval().start().into_dynamic())
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
        PyTime(self.0.interval().end().into_dynamic())
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
pub struct PyEnsemble(pub Ensemble<AssetId, Origin, Frame>);

#[pymethods]
impl PyEnsemble {
    /// Return the trajectory for a given spacecraft id.
    fn get(&self, id: &str) -> Option<PyTrajectory> {
        self.0
            .get(&AssetId::new(id))
            .map(|t| PyTrajectory(t.clone().into_dynamic()))
    }

    fn __len__(&self) -> usize {
        self.0.len()
    }

    fn __repr__(&self) -> String {
        format!("Ensemble({} trajectories)", self.0.len())
    }
}

// ---------------------------------------------------------------------------
// Shared analysis-binding helpers
// ---------------------------------------------------------------------------

/// Either a real SPK borrowed from Python, or nothing.
///
/// The Rust analyses take an ephemeris unconditionally but only consult it for
/// occulting bodies, while the Python constructors make it optional. One handle
/// type keeps the two arms from duplicating every call site — `Ephemeris` has
/// generic methods, so it cannot be used as `dyn`.
pub enum EphemerisHandle<'a> {
    /// A borrowed SPK kernel.
    Spk(&'a Spk),
    /// The analytical Sun model — accurate enough for eclipse geometry in an
    /// Earth-centred scenario, and it needs no kernel.
    AnalyticalSun(AnalyticalSunEphemeris),
    /// No ephemeris; fails if anything actually looks a body up.
    Missing(NoEphemeris),
}

/// Failure from either arm of an [`EphemerisHandle`].
///
/// Written out rather than derived: `thiserror` is not a dependency of this
/// crate and one two-variant passthrough does not justify adding it.
#[derive(Debug)]
pub enum EphemerisHandleError {
    /// The SPK lookup failed.
    Spk(DafSpkError),
    /// The analytical Sun model was asked for something it cannot provide.
    AnalyticalSun(AnalyticalSunEphemerisError),
    /// No ephemeris was supplied.
    Missing(NoEphemerisError),
}

impl std::fmt::Display for EphemerisHandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spk(e) => e.fmt(f),
            Self::AnalyticalSun(e) => e.fmt(f),
            Self::Missing(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for EphemerisHandleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spk(e) => Some(e),
            Self::AnalyticalSun(e) => Some(e),
            Self::Missing(e) => Some(e),
        }
    }
}

impl From<DafSpkError> for EphemerisHandleError {
    fn from(e: DafSpkError) -> Self {
        Self::Spk(e)
    }
}

impl From<NoEphemerisError> for EphemerisHandleError {
    fn from(e: NoEphemerisError) -> Self {
        Self::Missing(e)
    }
}

impl From<AnalyticalSunEphemerisError> for EphemerisHandleError {
    fn from(e: AnalyticalSunEphemerisError) -> Self {
        Self::AnalyticalSun(e)
    }
}

impl Ephemeris for EphemerisHandle<'_> {
    type Error = EphemerisHandleError;

    fn state<O1: CoordinateOrigin, O2: CoordinateOrigin>(
        &self,
        time: Time<Tdb>,
        origin: O1,
        target: O2,
    ) -> Result<Cartesian, Self::Error> {
        match self {
            Self::Spk(spk) => Ok(spk.state(time, origin, target)?),
            Self::AnalyticalSun(sun) => Ok(sun.state(time, origin, target)?),
            Self::Missing(none) => Ok(none.state(time, origin, target)?),
        }
    }
}

/// Borrows the SPK out of its Python object, or yields the empty handle.
///
/// `PySpk` is frozen, so the borrow needs no GIL token and stays valid across
/// `py.detach` for as long as the owning `Py` lives — which it does, on `self`.
fn bind_ephemeris<'a>(_py: Python<'_>, ephemeris: Option<&'a Py<PySpk>>) -> EphemerisHandle<'a> {
    match ephemeris {
        Some(spk) => EphemerisHandle::Spk(&spk.get().0),
        None => EphemerisHandle::Missing(NoEphemeris),
    }
}

/// Like [`bind_ephemeris`], but falls back to the analytical Sun rather than to
/// nothing — for analyses whose only ephemeris need *is* the Sun.
fn bind_sun_ephemeris<'a>(ephemeris: Option<&'a Py<PySpk>>) -> EphemerisHandle<'a> {
    match ephemeris {
        Some(spk) => EphemerisHandle::Spk(&spk.get().0),
        None => EphemerisHandle::AnalyticalSun(AnalyticalSunEphemeris),
    }
}

/// Converts the occulting-body list, rejecting occulters without an ephemeris.
fn parse_occulting_bodies(
    bodies: Option<Vec<Bound<'_, PyAny>>>,
    has_ephemeris: bool,
) -> PyResult<Vec<Origin>> {
    let bodies: Vec<Origin> = bodies
        .unwrap_or_default()
        .iter()
        .map(|b| Ok(PyOrigin::try_from(b)?.0))
        .collect::<PyResult<_>>()?;
    if !bodies.is_empty() && !has_ephemeris {
        return Err(PyValueError::new_err(
            "ephemeris is required when occulting_bodies is set",
        ));
    }
    Ok(bodies)
}

/// Resolves the ensemble, propagating the scenario when none was supplied.
fn resolve_ensemble(
    scenario: &Scenario,
    ensemble: &Option<Ensemble<AssetId, Origin, Frame>>,
) -> PyResult<Ensemble<AssetId, Origin, Frame>> {
    match ensemble {
        Some(e) => Ok(e.clone()),
        None => scenario
            .propagate(&DefaultRotationProvider)
            .map_err(|e| PyValueError::new_err(e.to_string())),
    }
}

/// Turns the `parallel`/`workers` keyword pair into a [`Parallelism`].
fn parallelism(parallel: bool, workers: Option<usize>) -> Parallelism {
    if parallel {
        Parallelism::Rayon(workers)
    } else {
        Parallelism::Sequential
    }
}

/// Renders an [`AnalysisError`] and its `source()` chain into one message.
///
/// Typed recovery is an in-process guarantee; across the Python boundary the
/// chain is flattened, so the message has to carry what the types would have.
fn render_error(error: &AnalysisError) -> String {
    let mut message = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

/// Maps an [`AnalysisError`] onto its Python exception type.
fn analysis_error(error: AnalysisError) -> PyErr {
    let message = render_error(&error);
    match error {
        AnalysisError::Rotation(_) => PyRotationFailed::new_err(message),
        AnalysisError::Ephemeris(_) => PyEphemerisFailed::new_err(message),
        AnalysisError::Detect(_) => PyDetectionFailed::new_err(message),
        AnalysisError::Stage(_) => PyAnalysisError::new_err(message),
    }
}

/// Splits per-target results into the successes and errors a `*Run` exposes.
///
/// A target that produced no items appears in the item map with an empty list,
/// not in `errors` — "found nothing" and "failed" are different outcomes, and
/// collapsing them is how a caller ends up treating an ephemeris gap as a
/// coverage hole.
#[allow(clippy::type_complexity)]
fn split_pairs<T>(
    results: Vec<((AssetId, AssetId), Result<Vec<T>, AnalysisError>)>,
) -> (
    HashMap<(String, String), Vec<T>>,
    HashMap<(String, String), String>,
) {
    let mut items = HashMap::new();
    let mut errors = HashMap::new();
    for ((a, b), result) in results {
        let key = (a.as_str().to_string(), b.as_str().to_string());
        match result {
            Ok(v) => {
                items.insert(key, v);
            }
            Err(e) => {
                errors.insert(key, render_error(&e));
            }
        }
    }
    (items, errors)
}

/// Ground-station-to-spacecraft visibility, yielding passes with observables.
///
/// Args:
///     scenario: Scenario containing spacecraft, ground stations, and time
///         interval.
///     ensemble: Optional pre-computed Ensemble. If omitted, the scenario is
///         propagated automatically.
///     ephemeris: SPK ephemeris. Required only when ``occulting_bodies`` is set.
///     occulting_bodies: Additional bodies to check for line-of-sight
///         occultation, beyond the trajectories' own origin.
///     step: Sampling step for detection and observables (default: 60 s).
///     min_pass_duration: Discards passes shorter than this, and coarsens the
///         scan as far as that allows.
///     min_range: Discards geometry closer than this.
///     max_range: Discards geometry farther than this.
#[pyclass(name = "VisibilityAnalysis", module = "lox_space", frozen)]
pub struct PyVisibilityAnalysis {
    scenario: Scenario,
    ensemble: Option<Ensemble<AssetId, Origin, Frame>>,
    ephemeris: Option<Py<PySpk>>,
    occulting_bodies: Vec<Origin>,
    step: TimeDelta,
    min_pass_duration: Option<TimeDelta>,
    min_range: Option<Distance>,
    max_range: Option<Distance>,
}

#[pymethods]
impl PyVisibilityAnalysis {
    #[new]
    #[pyo3(signature = (scenario, ensemble=None, ephemeris=None, occulting_bodies=None, step=None, min_pass_duration=None, min_range=None, max_range=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        scenario: PyScenario,
        ensemble: Option<PyEnsemble>,
        ephemeris: Option<Py<PySpk>>,
        occulting_bodies: Option<Vec<Bound<'_, PyAny>>>,
        step: Option<PyTimeDelta>,
        min_pass_duration: Option<PyTimeDelta>,
        min_range: Option<PyDistance>,
        max_range: Option<PyDistance>,
    ) -> PyResult<Self> {
        let occulting_bodies = parse_occulting_bodies(occulting_bodies, ephemeris.is_some())?;
        Ok(Self {
            scenario: scenario.0,
            ensemble: ensemble.map(|e| e.0),
            ephemeris,
            occulting_bodies,
            step: step
                .map(|s| s.0)
                .unwrap_or_else(|| TimeDelta::from_seconds_f64(60.0)),
            min_pass_duration: min_pass_duration.map(|d| d.0),
            min_range: min_range.map(|d| d.0),
            max_range: max_range.map(|d| d.0),
        })
    }

    /// Computes the passes for one (ground station, spacecraft) pair.
    ///
    /// Args:
    ///     ground_station: The station.
    ///     spacecraft: The spacecraft; must have a trajectory in the ensemble.
    ///     interval: Optional interval; defaults to the scenario's.
    ///
    /// Returns:
    ///     list[Pass]
    ///
    /// Raises:
    ///     AnalysisError: if detection fails or the spacecraft has no trajectory.
    #[pyo3(signature = (ground_station, spacecraft, interval=None))]
    fn single(
        &self,
        py: Python<'_>,
        ground_station: PyGroundStation,
        spacecraft: PySpacecraft,
        interval: Option<PyInterval>,
    ) -> PyResult<Vec<PyPass>> {
        let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
        let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
        let eph = bind_ephemeris(py, self.ephemeris.as_ref());
        let passes = py
            .detach(|| {
                self.build(&ensemble, &eph)
                    .single(&ground_station.0, &spacecraft.0, interval)
            })
            .map_err(analysis_error)?;
        Ok(passes.into_iter().map(PyPass).collect())
    }

    /// Computes the contact windows for one pair, without observables.
    ///
    /// Cheaper than :meth:`single` by roughly a third — nothing samples azimuth,
    /// elevation, range, or range rate. Use it when only the timing matters.
    ///
    /// Returns:
    ///     list[Window]
    #[pyo3(signature = (ground_station, spacecraft, interval=None))]
    fn windows(
        &self,
        py: Python<'_>,
        ground_station: PyGroundStation,
        spacecraft: PySpacecraft,
        interval: Option<PyInterval>,
    ) -> PyResult<Vec<PyWindow>> {
        let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
        let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
        let eph = bind_ephemeris(py, self.ephemeris.as_ref());
        let windows: Result<Vec<_>, _> = py.detach(|| {
            self.build(&ensemble, &eph)
                .windows(&ground_station.0, &spacecraft.0, interval)
                .collect()
        });
        Ok(windows
            .map_err(analysis_error)?
            .into_iter()
            .map(PyWindow)
            .collect())
    }

    /// Computes passes for every (ground station, spacecraft) pair.
    ///
    /// Args:
    ///     interval: Optional interval; defaults to the scenario's.
    ///     parallel: Fan out across threads (default: True).
    ///     workers: Thread count. ``None`` uses the global pool; a number builds
    ///         a pool local to this call, so it cannot disturb concurrent work.
    ///
    /// Returns:
    ///     VisibilityRun with ``.passes`` and ``.errors``.
    #[pyo3(signature = (interval=None, parallel=true, workers=None))]
    fn run(
        &self,
        py: Python<'_>,
        interval: Option<PyInterval>,
        parallel: bool,
        workers: Option<usize>,
    ) -> PyResult<PyVisibilityRun> {
        let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
        let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
        let mode = parallelism(parallel, workers);
        let eph = bind_ephemeris(py, self.ephemeris.as_ref());
        let results = py.detach(|| self.build(&ensemble, &eph).run(interval, mode));
        let (passes, errors) = split_pairs(results);
        Ok(PyVisibilityRun {
            passes: passes
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(PyPass).collect()))
                .collect(),
            errors,
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "VisibilityAnalysis({} ground stations, {} spacecraft)",
            self.scenario.ground_stations().len(),
            self.scenario.spacecraft().len(),
        )
    }
}

impl PyVisibilityAnalysis {
    fn build<'a>(
        &'a self,
        ensemble: &'a Ensemble<AssetId, Origin, Frame>,
        eph: &'a EphemerisHandle<'a>,
    ) -> VisibilityAnalysis<'a, Origin, Frame, EphemerisHandle<'a>> {
        let mut analysis = VisibilityAnalysis::new(&self.scenario, ensemble, eph)
            .with_step(self.step)
            .with_occulting_bodies(self.occulting_bodies.clone())
            .with_range_limits(self.min_range, self.max_range);
        if let Some(d) = self.min_pass_duration {
            analysis = analysis.with_min_pass_duration(d);
        }
        analysis
    }
}

/// Result of :meth:`VisibilityAnalysis.run`.
///
/// A dumb container, deliberately: the aggregate ``VisibilityResults`` it
/// replaces carried computed behaviour, which meant the analysis's knobs had to
/// be remembered inside the result to answer later questions.
#[pyclass(name = "VisibilityRun", module = "lox_space", frozen)]
pub struct PyVisibilityRun {
    passes: HashMap<(String, String), Vec<PyPass>>,
    errors: HashMap<(String, String), String>,
}

#[pymethods]
impl PyVisibilityRun {
    /// Passes per (ground station, spacecraft) pair. A pair with no passes maps
    /// to an empty list rather than being absent.
    #[getter]
    fn passes(&self) -> HashMap<(String, String), Vec<PyPass>> {
        self.passes.clone()
    }

    /// Error message per pair that failed. Absent pairs succeeded.
    #[getter]
    fn errors(&self) -> HashMap<(String, String), String> {
        self.errors.clone()
    }

    fn __repr__(&self) -> String {
        let total: usize = self.passes.values().map(Vec::len).sum();
        format!(
            "VisibilityRun({} pairs, {total} passes, {} errors)",
            self.passes.len(),
            self.errors.len()
        )
    }
}

/// Spacecraft-to-spacecraft contacts, yielding windows.
///
/// A separate class from ``VisibilityAnalysis`` because the item type differs:
/// sat-to-sat contacts have no ground-station observables, so they cannot be
/// passes. Replaces the old ``inter_satellite=True`` flag.
///
/// Args:
///     scenario: Scenario containing the spacecraft and time interval.
///     ensemble: Optional pre-computed Ensemble.
///     ephemeris: SPK ephemeris. Required only when ``occulting_bodies`` is set.
///     occulting_bodies: Bodies to check beyond the scenario's central body,
///         which is always checked.
///     step: Sampling step for detection (default: 60 s).
///     min_duration: Discards windows shorter than this.
///     min_range: Discards contacts closer than this.
///     max_range: Discards contacts farther than this.
#[pyclass(name = "InterSatelliteAnalysis", module = "lox_space", frozen)]
pub struct PyInterSatelliteAnalysis {
    scenario: Scenario,
    ensemble: Option<Ensemble<AssetId, Origin, Frame>>,
    ephemeris: Option<Py<PySpk>>,
    occulting_bodies: Vec<Origin>,
    step: TimeDelta,
    min_duration: Option<TimeDelta>,
    min_range: Option<Distance>,
    max_range: Option<Distance>,
}

#[pymethods]
impl PyInterSatelliteAnalysis {
    #[new]
    #[pyo3(signature = (scenario, ensemble=None, ephemeris=None, occulting_bodies=None, step=None, min_duration=None, min_range=None, max_range=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        scenario: PyScenario,
        ensemble: Option<PyEnsemble>,
        ephemeris: Option<Py<PySpk>>,
        occulting_bodies: Option<Vec<Bound<'_, PyAny>>>,
        step: Option<PyTimeDelta>,
        min_duration: Option<PyTimeDelta>,
        min_range: Option<PyDistance>,
        max_range: Option<PyDistance>,
    ) -> PyResult<Self> {
        let occulting_bodies = parse_occulting_bodies(occulting_bodies, ephemeris.is_some())?;
        Ok(Self {
            scenario: scenario.0,
            ensemble: ensemble.map(|e| e.0),
            ephemeris,
            occulting_bodies,
            step: step
                .map(|s| s.0)
                .unwrap_or_else(|| TimeDelta::from_seconds_f64(60.0)),
            min_duration: min_duration.map(|d| d.0),
            min_range: min_range.map(|d| d.0),
            max_range: max_range.map(|d| d.0),
        })
    }

    /// Computes the contact windows for one spacecraft pair.
    #[pyo3(signature = (first, second, interval=None))]
    fn single(
        &self,
        py: Python<'_>,
        first: PySpacecraft,
        second: PySpacecraft,
        interval: Option<PyInterval>,
    ) -> PyResult<Vec<PyWindow>> {
        let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
        let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
        let eph = bind_ephemeris(py, self.ephemeris.as_ref());
        let windows = py
            .detach(|| {
                self.build(&ensemble, &eph)
                    .single(&first.0, &second.0, interval)
            })
            .map_err(analysis_error)?;
        Ok(windows.into_iter().map(PyWindow).collect())
    }

    /// Computes contact windows for every unordered spacecraft pair.
    ///
    /// Returns:
    ///     InterSatelliteRun with ``.windows`` and ``.errors``.
    #[pyo3(signature = (interval=None, parallel=true, workers=None))]
    fn run(
        &self,
        py: Python<'_>,
        interval: Option<PyInterval>,
        parallel: bool,
        workers: Option<usize>,
    ) -> PyResult<PyInterSatelliteRun> {
        let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
        let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
        let mode = parallelism(parallel, workers);
        let eph = bind_ephemeris(py, self.ephemeris.as_ref());
        let results = py.detach(|| self.build(&ensemble, &eph).run(interval, mode));
        let (windows, errors) = split_pairs(results);
        Ok(PyInterSatelliteRun {
            windows: windows
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(PyWindow).collect()))
                .collect(),
            errors,
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "InterSatelliteAnalysis({} spacecraft)",
            self.scenario.spacecraft().len()
        )
    }
}

impl PyInterSatelliteAnalysis {
    fn build<'a>(
        &'a self,
        ensemble: &'a Ensemble<AssetId, Origin, Frame>,
        eph: &'a EphemerisHandle<'a>,
    ) -> InterSatelliteAnalysis<'a, Origin, Frame, EphemerisHandle<'a>> {
        let mut analysis = InterSatelliteAnalysis::new(&self.scenario, ensemble, eph)
            .with_step(self.step)
            .with_occulting_bodies(self.occulting_bodies.clone())
            .with_range_limits(self.min_range, self.max_range);
        if let Some(d) = self.min_duration {
            analysis = analysis.with_min_duration(d);
        }
        analysis
    }
}

/// Result of :meth:`InterSatelliteAnalysis.run`.
#[pyclass(name = "InterSatelliteRun", module = "lox_space", frozen)]
pub struct PyInterSatelliteRun {
    windows: HashMap<(String, String), Vec<PyWindow>>,
    errors: HashMap<(String, String), String>,
}

#[pymethods]
impl PyInterSatelliteRun {
    /// Contact windows per spacecraft pair.
    #[getter]
    fn windows(&self) -> HashMap<(String, String), Vec<PyWindow>> {
        self.windows.clone()
    }

    /// Error message per pair that failed.
    #[getter]
    fn errors(&self) -> HashMap<(String, String), String> {
        self.errors.clone()
    }

    fn __repr__(&self) -> String {
        let total: usize = self.windows.values().map(Vec::len).sum();
        format!(
            "InterSatelliteRun({} pairs, {total} windows, {} errors)",
            self.windows.len(),
            self.errors.len()
        )
    }
}

/// A contact window: an interval with no observables attached.
#[pyclass(name = "Window", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyWindow(pub Window);

#[pymethods]
impl PyWindow {
    /// The contact interval.
    #[getter]
    fn interval(&self) -> PyInterval {
        PyInterval(self.0.0)
    }

    /// Window start.
    #[getter]
    fn start(&self) -> PyTime {
        PyTime(self.0.0.start())
    }

    /// Window end.
    #[getter]
    fn end(&self) -> PyTime {
        PyTime(self.0.0.end())
    }

    /// Window duration.
    #[getter]
    fn duration(&self) -> PyTimeDelta {
        PyTimeDelta(self.0.0.duration())
    }

    fn __repr__(&self) -> String {
        format!("Window({}, {})", self.0.0.start(), self.0.0.end())
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
pub struct PyPass(pub Pass);

#[pymethods]
impl PyPass {
    #[new]
    fn new(
        interval: PyInterval,
        times: Vec<PyTime>,
        observables: Vec<PyObservables>,
    ) -> PyResult<Self> {
        let times: Vec<crate::time::Time> = times.into_iter().map(|t| t.0).collect();
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
// PowerBudgetAnalysis Python bindings
// ---------------------------------------------------------------------------

/// Eclipse intervals plus the continuous beta-angle and solar-flux channels.
///
/// Args:
///     scenario: Scenario containing the spacecraft and time interval.
///     ephemeris: Optional SPK ephemeris for the Sun. When omitted, an
///         analytical model is used, which is valid for Earth-centred scenarios.
///     ensemble: Optional pre-computed Ensemble.
///     step: Sampling step for detection and for the continuous channels
///         (default: 60 s).
#[pyclass(name = "PowerBudgetAnalysis", module = "lox_space", frozen)]
pub struct PyPowerBudgetAnalysis {
    scenario: Scenario,
    ensemble: Option<Ensemble<AssetId, Origin, Frame>>,
    ephemeris: Option<Py<PySpk>>,
    step: TimeDelta,
}

#[pymethods]
impl PyPowerBudgetAnalysis {
    #[new]
    #[pyo3(signature = (scenario, ephemeris=None, ensemble=None, step=None))]
    fn new(
        scenario: PyScenario,
        ephemeris: Option<Py<PySpk>>,
        ensemble: Option<PyEnsemble>,
        step: Option<PyTimeDelta>,
    ) -> Self {
        Self {
            scenario: scenario.0,
            ensemble: ensemble.map(|e| e.0),
            ephemeris,
            step: step
                .map(|s| s.0)
                .unwrap_or_else(|| TimeDelta::from_seconds_f64(60.0)),
        }
    }

    /// Computes one spacecraft's eclipse intervals.
    ///
    /// Returns:
    ///     list[Eclipse]
    #[pyo3(signature = (spacecraft, interval=None))]
    fn eclipses(
        &self,
        py: Python<'_>,
        spacecraft: PySpacecraft,
        interval: Option<PyInterval>,
    ) -> PyResult<Vec<PyEclipse>> {
        let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
        let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
        let eph = bind_sun_ephemeris(self.ephemeris.as_ref());
        let eclipses = py
            .detach(|| {
                self.build(&ensemble, &eph)
                    .eclipses(&spacecraft.0, interval)
            })
            .map_err(analysis_error)?;
        Ok(eclipses.into_iter().map(PyEclipse).collect())
    }

    /// Samples one spacecraft's beta angle over the interval.
    ///
    /// Not an event stream: the beta angle has a value at every instant, so it
    /// is sampled directly rather than detected.
    ///
    /// Returns:
    ///     TimeSeries of radians.
    #[pyo3(signature = (spacecraft, interval=None))]
    fn beta_angle(
        &self,
        py: Python<'_>,
        spacecraft: PySpacecraft,
        interval: Option<PyInterval>,
    ) -> PyResult<PyTimeSeries> {
        let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
        let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
        let eph = bind_sun_ephemeris(self.ephemeris.as_ref());
        let series = py
            .detach(|| {
                self.build(&ensemble, &eph)
                    .beta_angle(&spacecraft.0, interval)
            })
            .map_err(analysis_error)?;
        Ok(PyTimeSeries(series))
    }

    /// Samples one spacecraft's solar flux over the interval.
    ///
    /// Returns:
    ///     TimeSeries of W/m².
    #[pyo3(signature = (spacecraft, interval=None))]
    fn solar_flux(
        &self,
        py: Python<'_>,
        spacecraft: PySpacecraft,
        interval: Option<PyInterval>,
    ) -> PyResult<PyTimeSeries> {
        let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
        let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
        let eph = bind_sun_ephemeris(self.ephemeris.as_ref());
        let series = py
            .detach(|| {
                self.build(&ensemble, &eph)
                    .solar_flux(&spacecraft.0, interval)
            })
            .map_err(analysis_error)?;
        Ok(PyTimeSeries(series))
    }

    /// Computes all three channels for every spacecraft in the scenario.
    ///
    /// Returns:
    ///     PowerBudgetRun with ``.spacecraft`` and ``.errors``.
    #[pyo3(signature = (interval=None, parallel=true, workers=None))]
    fn run(
        &self,
        py: Python<'_>,
        interval: Option<PyInterval>,
        parallel: bool,
        workers: Option<usize>,
    ) -> PyResult<PyPowerBudgetRun> {
        let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
        let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
        let mode = parallelism(parallel, workers);
        let eph = bind_sun_ephemeris(self.ephemeris.as_ref());
        let results = py.detach(|| self.build(&ensemble, &eph).run(interval, mode));

        let mut spacecraft = HashMap::new();
        let mut errors = HashMap::new();
        for (id, result) in results {
            let key = id.as_str().to_string();
            match result {
                Ok(power) => {
                    spacecraft.insert(key, PySpacecraftPower(power));
                }
                Err(e) => {
                    errors.insert(key, render_error(&e));
                }
            }
        }
        Ok(PyPowerBudgetRun { spacecraft, errors })
    }

    fn __repr__(&self) -> String {
        format!(
            "PowerBudgetAnalysis({} spacecraft)",
            self.scenario.spacecraft().len()
        )
    }
}

impl PyPowerBudgetAnalysis {
    fn build<'a>(
        &'a self,
        ensemble: &'a Ensemble<AssetId, Origin, Frame>,
        eph: &'a EphemerisHandle<'a>,
    ) -> PowerBudgetAnalysis<'a, Origin, Frame, EphemerisHandle<'a>> {
        PowerBudgetAnalysis::new(&self.scenario, ensemble, eph).with_step(self.step)
    }
}

/// One spacecraft's power-budget outputs.
#[pyclass(name = "SpacecraftPower", module = "lox_space", frozen)]
pub struct PySpacecraftPower(pub SpacecraftPower);

#[pymethods]
impl PySpacecraftPower {
    /// Eclipse intervals (umbra only; penumbra is not modelled).
    #[getter]
    fn eclipses(&self) -> Vec<PyEclipse> {
        self.0.eclipses.iter().copied().map(PyEclipse).collect()
    }

    /// Beta angle over the arc, in radians.
    #[getter]
    fn beta_angle(&self) -> PyTimeSeries {
        PyTimeSeries(self.0.beta.clone())
    }

    /// Solar flux over the arc, in W/m².
    #[getter]
    fn solar_flux(&self) -> PyTimeSeries {
        PyTimeSeries(self.0.flux.clone())
    }

    /// Fraction of ``interval`` spent in eclipse, in [0, 1].
    ///
    /// Takes the interval explicitly because the eclipse list alone cannot say
    /// whether "no eclipses" covers a week or a minute.
    fn eclipse_fraction(&self, interval: PyInterval) -> f64 {
        self.0.eclipse_fraction_over(interval.0)
    }

    /// Fraction of ``interval`` spent sunlit, ``1 - eclipse_fraction``.
    fn sunlit_fraction(&self, interval: PyInterval) -> f64 {
        self.0.sunlit_fraction_over(interval.0)
    }

    fn __repr__(&self) -> String {
        format!("SpacecraftPower({} eclipses)", self.0.eclipses.len())
    }
}

/// Result of :meth:`PowerBudgetAnalysis.run`.
#[pyclass(name = "PowerBudgetRun", module = "lox_space", frozen)]
pub struct PyPowerBudgetRun {
    spacecraft: HashMap<String, PySpacecraftPower>,
    errors: HashMap<String, String>,
}

#[pymethods]
impl PyPowerBudgetRun {
    /// Power outputs per spacecraft id.
    #[getter]
    fn spacecraft(&self, py: Python<'_>) -> PyResult<HashMap<String, Py<PySpacecraftPower>>> {
        self.spacecraft
            .iter()
            .map(|(k, v)| {
                Ok((
                    k.clone(),
                    Py::new(
                        py,
                        PySpacecraftPower(SpacecraftPower {
                            eclipses: v.0.eclipses.clone(),
                            beta: v.0.beta.clone(),
                            flux: v.0.flux.clone(),
                        }),
                    )?,
                ))
            })
            .collect()
    }

    /// Error message per spacecraft that failed.
    #[getter]
    fn errors(&self) -> HashMap<String, String> {
        self.errors.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "PowerBudgetRun({} spacecraft, {} errors)",
            self.spacecraft.len(),
            self.errors.len()
        )
    }
}

/// A single eclipse interval.
#[pyclass(name = "Eclipse", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone, Copy)]
pub struct PyEclipse(pub Eclipse);

#[pymethods]
impl PyEclipse {
    /// The eclipse interval.
    #[getter]
    fn interval(&self) -> PyInterval {
        PyInterval(self.0.0)
    }

    /// Eclipse entry.
    #[getter]
    fn start(&self) -> PyTime {
        PyTime(self.0.0.start())
    }

    /// Eclipse exit.
    #[getter]
    fn end(&self) -> PyTime {
        PyTime(self.0.0.end())
    }

    /// Eclipse duration.
    #[getter]
    fn duration(&self) -> PyTimeDelta {
        PyTimeDelta(self.0.0.duration())
    }

    fn __repr__(&self) -> String {
        format!("Eclipse({}, {})", self.0.0.start(), self.0.0.end())
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
            self.0.interval.start().into_dynamic(),
            self.0.interval.end().into_dynamic(),
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

// ---------------------------------------------------------------------------
// Access analyses (optical + SAR)
// ---------------------------------------------------------------------------

/// Result of an access analysis `run`.
///
/// Shared by the optical and SAR analyses: the item type is the same, only the
/// payload geometry that produced it differs.
#[pyclass(name = "AccessRun", module = "lox_space", frozen)]
pub struct PyAccessRun {
    windows: HashMap<(String, String), Vec<PyAccessWindow>>,
    errors: HashMap<(String, String), String>,
}

#[pymethods]
impl PyAccessRun {
    /// Access windows per (spacecraft, AOI) pair.
    #[getter]
    fn windows(&self) -> HashMap<(String, String), Vec<PyAccessWindow>> {
        self.windows.clone()
    }

    /// Error message per pair that failed.
    #[getter]
    fn errors(&self) -> HashMap<(String, String), String> {
        self.errors.clone()
    }

    fn __repr__(&self) -> String {
        let total: usize = self.windows.values().map(Vec::len).sum();
        format!(
            "AccessRun({} pairs, {total} windows, {} errors)",
            self.windows.len(),
            self.errors.len()
        )
    }
}

/// Generates the optical and SAR access bindings.
///
/// The two differ only in their payload type and their docs; a macro keeps the
/// GIL handling, ensemble resolution, and error routing in one place instead of
/// two copies that can drift.
macro_rules! access_analysis {
    ($py_name:literal, $ty:ident, $payload:ty, $doc:literal) => {
        #[doc = $doc]
        ///
        /// Args:
        ///     scenario: Scenario containing the spacecraft and time interval.
        ///     aois: List of ``(id, Aoi)`` tuples defining the areas of interest.
        ///     ensemble: Optional pre-computed Ensemble.
        ///     step: Sampling step for detection (default: 60 s).
        ///     body_fixed_frame: Body-fixed frame override; defaults to the IAU
        ///         frame of the scenario's origin.
        #[pyclass(name = $py_name, module = "lox_space", frozen)]
        pub struct $ty {
            scenario: Scenario,
            aois: Vec<(AoiId, Aoi)>,
            ensemble: Option<Ensemble<AssetId, Origin, Frame>>,
            step: TimeDelta,
            body_fixed_frame: Option<Frame>,
        }

        #[pymethods]
        impl $ty {
            #[new]
            #[pyo3(signature = (scenario, aois, ensemble=None, step=None, body_fixed_frame=None))]
            fn new(
                scenario: PyScenario,
                aois: Vec<(String, PyAoi)>,
                ensemble: Option<PyEnsemble>,
                step: Option<PyTimeDelta>,
                body_fixed_frame: Option<PyFrame>,
            ) -> Self {
                Self {
                    scenario: scenario.0,
                    aois: aois
                        .into_iter()
                        .map(|(id, aoi)| (AoiId::new(id), aoi.0))
                        .collect(),
                    ensemble: ensemble.map(|e| e.0),
                    step: step
                        .map(|s| s.0)
                        .unwrap_or_else(|| TimeDelta::from_seconds_f64(60.0)),
                    body_fixed_frame: body_fixed_frame.map(|f| f.0),
                }
            }

            /// Computes access windows for one (spacecraft, AOI) pair.
            ///
            /// Args:
            ///     spacecraft: The spacecraft; must carry the payload this
            ///         analysis reads.
            ///     aoi_id: Identifier of one of the configured AOIs.
            ///     interval: Optional interval; defaults to the scenario's.
            ///
            /// Returns:
            ///     list[AccessWindow]
            ///
            /// Raises:
            ///     AnalysisError: if the spacecraft carries no such payload, has
            ///         no trajectory, or detection fails.
            ///     KeyError: if ``aoi_id`` is not one of the configured AOIs.
            #[pyo3(signature = (spacecraft, aoi_id, interval=None))]
            fn single(
                &self,
                py: Python<'_>,
                spacecraft: PySpacecraft,
                aoi_id: &str,
                interval: Option<PyInterval>,
            ) -> PyResult<Vec<PyAccessWindow>> {
                let aoi = self
                    .aois
                    .iter()
                    .find(|(id, _)| id.as_str() == aoi_id)
                    .map(|(_, aoi)| aoi)
                    .ok_or_else(|| {
                        pyo3::exceptions::PyKeyError::new_err(format!(
                            "no AOI named {aoi_id:?} in this analysis"
                        ))
                    })?;
                let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
                let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
                let windows = py
                    .detach(|| self.build(&ensemble).single(&spacecraft.0, aoi, interval))
                    .map_err(analysis_error)?;
                Ok(windows.into_iter().map(PyAccessWindow).collect())
            }

            /// Computes access windows for every (spacecraft, AOI) pair.
            ///
            /// Spacecraft without the relevant payload are not targets at all,
            /// so they appear in neither ``.windows`` nor ``.errors``.
            ///
            /// Returns:
            ///     AccessRun with ``.windows`` and ``.errors``.
            #[pyo3(signature = (interval=None, parallel=true, workers=None))]
            fn run(
                &self,
                py: Python<'_>,
                interval: Option<PyInterval>,
                parallel: bool,
                workers: Option<usize>,
            ) -> PyResult<PyAccessRun> {
                let ensemble = resolve_ensemble(&self.scenario, &self.ensemble)?;
                let interval = interval.map_or(*self.scenario.interval(), |i| i.0);
                let mode = parallelism(parallel, workers);
                let results = py.detach(|| self.build(&ensemble).run(interval, mode));

                let mut windows = HashMap::new();
                let mut errors = HashMap::new();
                for ((sc, aoi), result) in results {
                    let key = (sc.as_str().to_string(), aoi.as_str().to_string());
                    match result {
                        Ok(v) => {
                            windows
                                .insert(key, v.into_iter().map(PyAccessWindow).collect::<Vec<_>>());
                        }
                        Err(e) => {
                            errors.insert(key, render_error(&e));
                        }
                    }
                }
                Ok(PyAccessRun { windows, errors })
            }

            fn __repr__(&self) -> String {
                let sc_count = self.scenario.spacecraft().len();
                let aoi_count = self.aois.len();
                let aoi_label = if aoi_count == 1 { "AOI" } else { "AOIs" };
                format!(
                    concat!($py_name, "({} spacecraft, {} {})"),
                    sc_count, aoi_count, aoi_label
                )
            }
        }

        impl $ty {
            fn build<'a>(
                &'a self,
                ensemble: &'a Ensemble<AssetId, Origin, Frame>,
            ) -> AccessAnalysis<'a, $payload, Origin, Frame> {
                let mut analysis = AccessAnalysis::new(&self.scenario, ensemble, self.aois.clone())
                    .with_step(self.step);
                if let Some(frame) = self.body_fixed_frame {
                    analysis = analysis.with_body_fixed_frame(frame);
                }
                analysis
            }
        }
    };
}

access_analysis!(
    "OpticalAccessAnalysis",
    PyOpticalAccessAnalysis,
    OpticalPayload,
    "AOI optical access: imaging windows for spacecraft carrying an optical payload."
);

access_analysis!(
    "SarAccessAnalysis",
    PySarAccessAnalysis,
    SarPayload,
    "AOI SAR access: imaging windows for spacecraft carrying a SAR payload."
);
