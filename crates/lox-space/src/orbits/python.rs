// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use crate::bodies::python::{PyOrigin, PyUndefinedOriginPropertyError};
use crate::bodies::{DynOrigin, Origin, TryPointMass};
use crate::earth::python::ut1::{PyEopProvider, PyEopProviderError};
use crate::ephem::Ephemeris;
use crate::ephem::python::{PyDafSpkError, PySpk};
use crate::frames::DynFrame;
use crate::frames::python::{PyDynRotationError, PyFrame};
use crate::math::roots::{Brent, RootFinderError};
use crate::orbits::analysis::{
    DynPass, ElevationMask, ElevationMaskError, Pass, VisibilityError, visibility_combined,
};
use crate::orbits::events::{Event, FindEventError, Window};
use crate::orbits::ground::{
    DynGroundLocation, DynGroundPropagator, GroundPropagatorError, Observables,
};
use crate::orbits::orbits::{
    CartesianOrbit, DynCartesianOrbit, DynTrajectory, TrajectorError, TrajectoryTransformationError,
};
use crate::orbits::propagators::Propagator;
use crate::orbits::propagators::semi_analytical::{DynVallado, Vallado, ValladoError};
use crate::orbits::propagators::sgp4::{Sgp4, Sgp4Error};
use crate::time::DynTime;
use crate::time::deltas::TimeDelta;
use crate::time::offsets::DefaultOffsetProvider;
use crate::time::python::deltas::PyTimeDelta;
use crate::time::python::time::PyTime;
use crate::time::time_scales::{DynTimeScale, Tai};
use lox_core::anomalies::TrueAnomaly;
use lox_core::coords::Cartesian;
use lox_core::elements::{
    ArgumentOfPeriapsis, Eccentricity, Inclination, Keplerian, LongitudeOfAscendingNode,
};
use lox_units::{Angle, Distance};

use glam::DVec3;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use numpy::{PyArray1, PyArray2, PyArrayMethods};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyType};
use rayon::prelude::*;
use sgp4::Elements;

struct PyTrajectoryTransformationError(TrajectoryTransformationError);

impl From<PyTrajectoryTransformationError> for PyErr {
    fn from(err: PyTrajectoryTransformationError) -> Self {
        // FIXME: wrong error type
        PyValueError::new_err(err.0.to_string())
    }
}

struct PyFindEventError(FindEventError);

impl From<PyFindEventError> for PyErr {
    fn from(err: PyFindEventError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

struct PyRootFinderError(RootFinderError);

impl From<PyRootFinderError> for PyErr {
    fn from(err: PyRootFinderError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

struct PyVisibilityError(VisibilityError);

impl From<PyVisibilityError> for PyErr {
    fn from(err: PyVisibilityError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// Find events where a function crosses zero.
///
/// This function detects zero-crossings of a user-defined function over a
/// time span. Events can be found for any scalar function of time.
///
/// Args:
///     func: Function that takes a float (seconds from start) and returns a float.
///     start: Reference time (epoch).
///     times: Array of time offsets in seconds from start.
///
/// Returns:
///     List of Event objects at the detected zero-crossings.
#[pyfunction]
pub fn find_events(
    func: &Bound<'_, PyAny>,
    start: PyTime,
    times: Vec<f64>,
) -> PyResult<Vec<PyEvent>> {
    let root_finder = Brent::default();
    Ok(crate::orbits::events::find_events(
        |t| {
            func.call((t,), None)
                .and_then(|obj| obj.extract::<f64>())
                .map_err(Into::into)
        },
        start.0,
        &times,
        root_finder,
    )
    .map_err(PyFindEventError)?
    .into_iter()
    .map(PyEvent)
    .collect())
}

/// Find time windows where a function is positive.
///
/// This function finds all intervals where a user-defined function is
/// positive. Windows are bounded by zero-crossings of the function.
///
/// Args:
///     func: Function that takes a float (seconds from start) and returns a float.
///     start: Start time of the analysis period.
///     end: End time of the analysis period.
///     times: Array of time offsets in seconds from start.
///
/// Returns:
///     List of Window objects for intervals where the function is positive.
#[pyfunction]
pub fn find_windows(
    _py: Python<'_>,
    func: &Bound<'_, PyAny>,
    start: PyTime,
    end: PyTime,
    times: Vec<f64>,
) -> PyResult<Vec<PyWindow>> {
    let root_finder = Brent::default();
    let res = crate::orbits::events::find_windows(
        |t| {
            func.call((t,), None)
                .and_then(|obj| obj.extract::<f64>())
                .map_err(Into::into)
        },
        start.0,
        end.0,
        &times,
        root_finder,
    );
    let windows = res.map_err(PyRootFinderError)?;
    Ok(windows.into_iter().map(PyWindow).collect())
}

/// Represents an orbital state (position and velocity) at a specific time.
///
/// A `State` captures the complete kinematic state of an object in space,
/// including its position, velocity, time, central body (origin), and
/// reference frame.
///
/// Args:
///     time: The epoch of this state.
///     position: Position vector (x, y, z) in km.
///     velocity: Velocity vector (vx, vy, vz) in km/s.
///     origin: Central body (default: Earth).
///     frame: Reference frame (default: ICRF).
#[pyclass(name = "State", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyState(pub DynCartesianOrbit);

#[pymethods]
impl PyState {
    #[new]
    #[pyo3(signature = (time, position, velocity, origin=None, frame=None))]
    fn new(
        time: PyTime,
        position: (f64, f64, f64),
        velocity: (f64, f64, f64),
        origin: Option<PyOrigin>,
        frame: Option<PyFrame>,
    ) -> PyResult<Self> {
        let origin = origin.unwrap_or_default();
        let frame = frame.unwrap_or_default();

        Ok(PyState(CartesianOrbit::new(
            Cartesian::from_vecs(
                DVec3::new(position.0, position.1, position.2),
                DVec3::new(velocity.0, velocity.1, velocity.2),
            ),
            time.0,
            origin.0,
            frame.0,
        )))
    }

    /// Return the epoch of this state.
    fn time(&self) -> PyTime {
        PyTime(self.0.time())
    }

    /// Return the central body (origin) of this state.
    fn origin(&self) -> PyOrigin {
        PyOrigin(self.0.origin())
    }

    /// Return the reference frame of this state.
    fn reference_frame(&self) -> PyFrame {
        PyFrame(self.0.reference_frame())
    }

    /// Return the position vector as a numpy array [x, y, z] in km.
    fn position<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let pos = self.0.position().to_array();
        PyArray1::from_slice(py, &pos)
    }

    /// Return the velocity vector as a numpy array [vx, vy, vz] in km/s.
    fn velocity<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let vel = self.0.velocity().to_array();
        PyArray1::from_slice(py, &vel)
    }

    /// Transform this state to a different reference frame.
    ///
    /// Args:
    ///     frame: Target reference frame.
    ///     provider: EOP provider (required for ITRF transformations).
    ///
    /// Returns:
    ///     A new State in the target frame.
    ///
    /// Raises:
    ///     FrameTransformationError: If the transformation fails.
    #[pyo3(signature = (frame, provider=None))]
    fn to_frame(
        &self,
        frame: PyFrame,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Self> {
        let provider = provider.map(|p| &p.get().0);
        let origin = self.0.reference_frame();
        let target = frame.0;
        let time = self.0.time();
        let rot = match provider {
            Some(provider) => provider
                .try_rotation(origin, target, time)
                .map_err(PyDynRotationError),
            None => DefaultRotationProvider
                .try_rotation(origin, target, time)
                .map_err(PyDynRotationError),
        }?;
        let (r1, v1) = rot.rotate_state(self.0.position(), self.0.velocity());
        Ok(PyState(CartesianOrbit::new(
            Cartesian::from_vecs(r1, v1),
            self.0.time(),
            self.0.origin(),
            frame.0,
        )))
    }

    /// Transform this state to a different central body.
    ///
    /// Args:
    ///     target: Target central body (origin).
    ///     ephemeris: SPK ephemeris data for computing body positions.
    ///
    /// Returns:
    ///     A new State relative to the target origin.
    ///
    /// Raises:
    ///     ValueError: If the transformation fails.
    fn to_origin(&self, target: PyOrigin, ephemeris: &Bound<'_, PySpk>) -> PyResult<Self> {
        let frame = self.reference_frame();
        let s = if frame.0 != DynFrame::Icrf {
            self.to_frame(PyFrame(DynFrame::Icrf), None)?
        } else {
            self.clone()
        };
        let spk = &ephemeris.borrow().0;
        let mut s1 = Self(
            s.0.to_origin_dynamic(target.0, spk)
                .map_err(PyDafSpkError)?,
        );
        if frame.0 != DynFrame::Icrf {
            s1 = s1.to_frame(frame, None)?
        }
        Ok(s1)
    }

    /// Convert this Cartesian state to Keplerian orbital elements.
    ///
    /// Returns:
    ///     Keplerian elements representing this orbit.
    ///
    /// Raises:
    ///     ValueError: If the state is not in an inertial frame.
    ///     UndefinedOriginPropertyError: If the origin has no gravitational parameter.
    fn to_keplerian(&self) -> PyResult<PyKeplerian> {
        if self.0.reference_frame() != DynFrame::Icrf {
            return Err(PyValueError::new_err(
                "only inertial frames are supported for conversion to Keplerian elements",
            ));
        }
        Ok(PyKeplerian(
            self.0
                .try_to_keplerian()
                .map_err(PyUndefinedOriginPropertyError)?,
        ))
    }

    /// Compute the rotation matrix from inertial to LVLH (Local Vertical Local Horizontal) frame.
    ///
    /// Returns:
    ///     3x3 rotation matrix as a numpy array.
    ///
    /// Raises:
    ///     ValueError: If the state is not in an inertial frame.
    fn rotation_lvlh<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        if self.0.reference_frame() != DynFrame::Icrf {
            return Err(PyValueError::new_err(
                "only inertial frames are supported for the LVLH rotation matrix",
            ));
        }
        let rot = self.0.try_rotation_lvlh().map_err(PyValueError::new_err)?;
        let rot: Vec<Vec<f64>> = rot.to_cols_array_2d().iter().map(|v| v.to_vec()).collect();
        Ok(PyArray2::from_vec2(py, &rot)?)
    }

    /// Convert this state to a ground location.
    ///
    /// This is useful for converting a state in a body-fixed frame to geodetic coordinates.
    ///
    /// Returns:
    ///     GroundLocation with longitude, latitude, and altitude.
    ///
    /// Raises:
    ///     ValueError: If conversion fails.
    fn to_ground_location(&self) -> PyResult<PyGroundLocation> {
        Ok(PyGroundLocation(
            self.0
                .to_dyn_ground_location()
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
        ))
    }
}

/// Represents an orbit using Keplerian (classical) orbital elements.
///
/// Keplerian elements describe an orbit using six parameters that define
/// its shape, orientation, and position along the orbit.
///
/// Args:
///     time: Epoch of the elements.
///     semi_major_axis: Semi-major axis in km.
///     eccentricity: Orbital eccentricity (0 = circular, <1 = elliptical).
///     inclination: Inclination in radians.
///     longitude_of_ascending_node: RAAN in radians.
///     argument_of_periapsis: Argument of periapsis in radians.
///     true_anomaly: True anomaly in radians.
///     origin: Central body (default: Earth).
#[pyclass(name = "Keplerian", module = "lox_space", frozen)]
pub struct PyKeplerian(pub crate::orbits::orbits::DynKeplerianOrbit);

#[pymethods]
impl PyKeplerian {
    #[new]
    #[pyo3(signature = (
        time,
        semi_major_axis,
        eccentricity,
        inclination,
        longitude_of_ascending_node,
        argument_of_periapsis,
        true_anomaly,
        origin=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        time: PyTime,
        semi_major_axis: f64,
        eccentricity: f64,
        inclination: f64,
        longitude_of_ascending_node: f64,
        argument_of_periapsis: f64,
        true_anomaly: f64,
        origin: Option<PyOrigin>,
    ) -> PyResult<Self> {
        let origin = origin.map(|origin| origin.0).unwrap_or_default();
        origin
            .try_gravitational_parameter()
            .map_err(PyUndefinedOriginPropertyError)?;
        let keplerian = Keplerian::new(
            Distance::kilometers(semi_major_axis),
            Eccentricity::try_new(eccentricity)
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
            Inclination::try_new(Angle::radians(inclination))
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
            LongitudeOfAscendingNode::try_new(Angle::radians(longitude_of_ascending_node))
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
            ArgumentOfPeriapsis::try_new(Angle::radians(argument_of_periapsis))
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
            TrueAnomaly::new(Angle::radians(true_anomaly)),
        );
        let orbit = crate::orbits::orbits::KeplerianOrbit::try_from_keplerian(
            keplerian,
            time.0,
            origin,
            DynFrame::Icrf,
        )
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(PyKeplerian(orbit))
    }

    /// Return the epoch of these elements.
    fn time(&self) -> PyTime {
        PyTime(self.0.time())
    }

    /// Return the central body (origin) of this orbit.
    fn origin(&self) -> PyOrigin {
        PyOrigin(self.0.origin())
    }

    /// Return the semi-major axis in km.
    fn semi_major_axis(&self) -> f64 {
        self.0.semi_major_axis().to_kilometers()
    }

    /// Return the orbital eccentricity.
    fn eccentricity(&self) -> f64 {
        self.0.eccentricity().as_f64()
    }

    /// Return the inclination in radians.
    fn inclination(&self) -> f64 {
        self.0.inclination().as_f64()
    }

    /// Return the longitude of the ascending node (RAAN) in radians.
    fn longitude_of_ascending_node(&self) -> f64 {
        self.0.longitude_of_ascending_node().as_f64()
    }

    /// Return the argument of periapsis in radians.
    fn argument_of_periapsis(&self) -> f64 {
        self.0.argument_of_periapsis().as_f64()
    }

    /// Return the true anomaly in radians.
    fn true_anomaly(&self) -> f64 {
        self.0.true_anomaly().as_f64()
    }

    /// Convert these Keplerian elements to a Cartesian state.
    ///
    /// Returns:
    ///     State with position and velocity vectors.
    fn to_cartesian(&self) -> PyResult<PyState> {
        Ok(PyState(
            self.0
                .try_to_cartesian()
                .map_err(PyUndefinedOriginPropertyError)?,
        ))
    }

    /// Return the orbital period.
    ///
    /// Returns:
    ///     TimeDelta representing one complete orbit.
    fn orbital_period(&self) -> PyResult<PyTimeDelta> {
        self.0
            .orbital_period()
            .map(PyTimeDelta)
            .ok_or_else(|| PyValueError::new_err("orbital period is not defined for this orbit"))
    }
}

/// A time-series of orbital states with interpolation support.
///
/// Trajectories store a sequence of States and provide interpolation to
/// compute states at arbitrary times between the stored samples.
///
/// Args:
///     states: List of State objects in chronological order.
#[pyclass(name = "Trajectory", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyTrajectory(pub DynTrajectory);

pub struct PyTrajectorError(pub TrajectorError);

impl From<PyTrajectorError> for PyErr {
    fn from(err: PyTrajectorError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

#[pymethods]
impl PyTrajectory {
    #[new]
    fn new(states: &Bound<'_, PyList>) -> PyResult<Self> {
        let states: Vec<PyState> = states.extract()?;
        let states: Vec<DynCartesianOrbit> = states.into_iter().map(|s| s.0).collect();
        Ok(PyTrajectory(
            DynTrajectory::try_new(states).map_err(PyTrajectorError)?,
        ))
    }

    /// Create a Trajectory from a numpy array.
    ///
    /// Args:
    ///     start_time: Reference epoch for the trajectory.
    ///     array: 2D numpy array with columns [t, x, y, z, vx, vy, vz] where t is seconds from start_time.
    ///     origin: Central body (default: Earth).
    ///     frame: Reference frame (default: ICRF).
    ///
    /// Returns:
    ///     A new Trajectory.
    ///
    /// Raises:
    ///     ValueError: If the array has invalid shape.
    #[classmethod]
    #[pyo3(signature = (start_time, array, origin=None, frame=None))]
    fn from_numpy(
        _cls: &Bound<'_, PyType>,
        start_time: PyTime,
        array: &Bound<'_, PyArray2<f64>>,
        origin: Option<PyOrigin>,
        frame: Option<PyFrame>,
    ) -> PyResult<Self> {
        let origin = origin.unwrap_or_default();
        let frame = frame.unwrap_or_default();
        let array = array.to_owned_array();
        if array.ncols() != 7 {
            return Err(PyValueError::new_err("invalid shape"));
        }
        let mut states: Vec<DynCartesianOrbit> = Vec::with_capacity(array.nrows());
        for row in array.rows() {
            let time = start_time.0 + TimeDelta::from_seconds_f64(row[0]);
            let position = DVec3::new(row[1], row[2], row[3]);
            let velocity = DVec3::new(row[4], row[5], row[6]);
            states.push(CartesianOrbit::new(
                Cartesian::from_vecs(position, velocity),
                time,
                origin.0,
                frame.0,
            ));
        }
        Ok(PyTrajectory(
            DynTrajectory::try_new(states).map_err(PyTrajectorError)?,
        ))
    }

    /// Return the central body (origin) of this trajectory.
    fn origin(&self) -> PyOrigin {
        PyOrigin(self.0.origin())
    }

    /// Return the reference frame of this trajectory.
    fn reference_frame(&self) -> PyFrame {
        PyFrame(self.0.reference_frame())
    }

    /// Export trajectory to a numpy array.
    ///
    /// Returns:
    ///     2D numpy array with columns [t, x, y, z, vx, vy, vz].
    fn to_numpy<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        Ok(PyArray2::from_vec2(py, &self.0.to_vec())?)
    }

    /// Return the list of states in this trajectory.
    fn states(&self) -> Vec<PyState> {
        self.0.states().into_iter().map(PyState).collect()
    }

    /// Find events where a function crosses zero.
    ///
    /// Args:
    ///     func: Function that takes a State and returns a float.
    ///           Events are detected where the function crosses zero.
    ///
    /// Returns:
    ///     List of Event objects.
    fn find_events(&self, func: &Bound<'_, PyAny>) -> PyResult<Vec<PyEvent>> {
        let res = self.0.find_events(|s| {
            // Python callable gets PyState and must return float; propagate exceptions
            func.call((PyState(s),), None)
                .and_then(|obj| obj.extract::<f64>())
        });
        let events = res.map_err(PyFindEventError)?;
        Ok(events.into_iter().map(PyEvent).collect())
    }

    /// Find time windows where a function is positive.
    ///
    /// Args:
    ///     func: Function that takes a State and returns a float.
    ///           Windows are periods where the function is positive.
    ///
    /// Returns:
    ///     List of Window objects.
    fn find_windows(&self, _py: Python<'_>, func: &Bound<'_, PyAny>) -> PyResult<Vec<PyWindow>> {
        let res = self.0.find_windows(|s| {
            func.call((PyState(s),), None)
                .and_then(|obj| obj.extract::<f64>())
        });
        let windows = res.map_err(PyRootFinderError)?;
        Ok(windows.into_iter().map(PyWindow).collect())
    }

    /// Interpolate the trajectory at a specific time.
    ///
    /// Args:
    ///     time: Either a Time (absolute) or TimeDelta (relative to trajectory start).
    ///
    /// Returns:
    ///     Interpolated State at the requested time.
    ///
    /// Raises:
    ///     ValueError: If the time argument is invalid.
    fn interpolate(&self, time: &Bound<'_, PyAny>) -> PyResult<PyState> {
        if let Ok(delta) = time.extract::<PyTimeDelta>() {
            return Ok(PyState(self.0.interpolate(delta.0)));
        }
        if let Ok(time) = time.extract::<PyTime>() {
            return Ok(PyState(self.0.interpolate_at(time.0)));
        }
        Err(PyValueError::new_err("invalid time argument"))
    }

    /// Transform all states in the trajectory to a different reference frame.
    ///
    /// Args:
    ///     frame: Target reference frame.
    ///     provider: EOP provider (required for ITRF transformations).
    ///
    /// Returns:
    ///     A new Trajectory in the target frame.
    #[pyo3(signature = (frame, provider=None))]
    fn to_frame(
        &self,
        frame: PyFrame,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Self> {
        let mut states: Vec<DynCartesianOrbit> = Vec::with_capacity(self.0.states().len());
        for s in self.0.states() {
            states.push(PyState(s).to_frame(frame.clone(), provider)?.0);
        }
        Ok(PyTrajectory(DynTrajectory::new(states)))
    }

    /// Transform all states in the trajectory to a different central body.
    ///
    /// Args:
    ///     target: Target central body (origin).
    ///     ephemeris: SPK ephemeris data.
    ///
    /// Returns:
    ///     A new Trajectory relative to the target origin.
    fn to_origin(&self, target: PyOrigin, ephemeris: &Bound<'_, PySpk>) -> PyResult<Self> {
        let mut states: Vec<DynCartesianOrbit> = Vec::with_capacity(self.states().len());
        for s in self.states() {
            states.push(s.to_origin(target.clone(), ephemeris)?.0);
        }
        Ok(Self(DynTrajectory::new(states)))
    }
}

/// Represents a detected event (zero-crossing of a function).
///
/// Events are detected when a monitored function crosses zero during
/// trajectory analysis. The crossing direction indicates whether the
/// function went from negative to positive ("up") or positive to negative ("down").
#[pyclass(name = "Event", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyEvent(pub Event<DynTimeScale>);

#[pymethods]
impl PyEvent {
    fn __repr__(&self) -> String {
        format!("Event({}, {})", self.time().__str__(), self.crossing())
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

/// Represents a time window (interval between two times).
///
/// Windows are used to represent periods when certain conditions are met,
/// such as visibility windows between a ground station and spacecraft.
#[pyclass(name = "Window", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyWindow(pub Window<DynTimeScale>);

#[pymethods]
impl PyWindow {
    fn __repr__(&self) -> String {
        format!(
            "Window({}, {})",
            self.start().__str__(),
            self.end().__str__()
        )
    }

    /// Return the start time of this window.
    fn start(&self) -> PyTime {
        PyTime(self.0.start())
    }

    /// Return the end time of this window.
    fn end(&self) -> PyTime {
        PyTime(self.0.end())
    }

    /// Return the duration of this window.
    fn duration(&self) -> PyTimeDelta {
        PyTimeDelta(self.0.duration())
    }
}

/// Semi-analytical Keplerian orbit propagator using Vallado's method.
///
/// This propagator uses Kepler's equation and handles elliptical, parabolic,
/// and hyperbolic orbits. It's suitable for two-body propagation without
/// perturbations.
///
/// Args:
///     initial_state: Initial orbital state (must be in an inertial frame).
///     max_iter: Maximum iterations for Kepler's equation solver (default: 50).
#[pyclass(name = "Vallado", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyVallado(pub DynVallado);

pub struct PyValladoError(pub ValladoError);

impl From<PyValladoError> for PyErr {
    fn from(err: PyValladoError) -> Self {
        // TODO: Use better error type
        PyValueError::new_err(err.0.to_string())
    }
}

#[pymethods]
impl PyVallado {
    #[new]
    #[pyo3(signature =(initial_state, max_iter=None))]
    fn new(initial_state: PyState, max_iter: Option<i32>) -> PyResult<Self> {
        let mut vallado = Vallado::with_dynamic(initial_state.0).map_err(|_| {
            PyValueError::new_err("only inertial frames are supported for the Vallado propagator")
        })?;
        if let Some(max_iter) = max_iter {
            vallado.with_max_iter(max_iter);
        }
        Ok(PyVallado(vallado))
    }

    /// Propagate the orbit to one or more times.
    ///
    /// Args:
    ///     steps: Single Time or list of Times.
    ///
    /// Returns:
    ///     State (if single time) or Trajectory (if list of times).
    ///
    /// Raises:
    ///     ValueError: If propagation fails.
    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(time) = steps.extract::<PyTime>() {
            return Ok(Bound::new(
                py,
                PyState(self.0.propagate(time.0).map_err(PyValladoError)?),
            )?
            .into_any());
        }
        if let Ok(steps) = steps.extract::<Vec<PyTime>>() {
            let steps = steps.into_iter().map(|s| s.0);
            return Ok(Bound::new(
                py,
                PyTrajectory(self.0.propagate_all(steps).map_err(PyValladoError)?),
            )?
            .into_any());
        }
        Err(PyValueError::new_err("invalid time delta(s)"))
    }
}

/// Represents a location on the surface of a celestial body.
///
/// Ground locations are specified using geodetic coordinates (longitude, latitude,
/// altitude) relative to a central body's reference ellipsoid.
///
/// Args:
///     origin: The central body (e.g., Earth, Moon).
///     longitude: Geodetic longitude in radians.
///     latitude: Geodetic latitude in radians.
///     altitude: Altitude above the reference ellipsoid in km.
#[pyclass(name = "GroundLocation", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyGroundLocation(pub DynGroundLocation);

#[pymethods]
impl PyGroundLocation {
    #[new]
    fn new(origin: PyOrigin, longitude: f64, latitude: f64, altitude: f64) -> PyResult<Self> {
        Ok(PyGroundLocation(
            DynGroundLocation::with_dynamic(longitude, latitude, altitude, origin.0)
                .map_err(PyValueError::new_err)?,
        ))
    }

    /// Compute observables (azimuth, elevation, range, range rate) to a target.
    ///
    /// Args:
    ///     state: Target state (e.g., spacecraft position).
    ///     provider: EOP provider (for accurate Earth rotation).
    ///     frame: Body-fixed frame (default: IAU frame of origin).
    ///
    /// Returns:
    ///     Observables with azimuth, elevation, range, and range rate.
    #[pyo3(signature = (state, provider=None, frame=None))]
    fn observables(
        &self,
        state: PyState,
        provider: Option<&Bound<'_, PyEopProvider>>,
        frame: Option<PyFrame>,
    ) -> PyResult<PyObservables> {
        let frame = frame.unwrap_or(PyFrame(DynFrame::Iau(state.0.origin())));
        let state = state.to_frame(frame, provider)?;
        let rot = self.0.rotation_to_topocentric();
        let position = rot * (state.0.position() - self.0.body_fixed_position());
        let velocity = rot * state.0.velocity();
        let range = position.length();
        let range_rate = position.dot(velocity) / range;
        let elevation = (position.z / range).asin();
        let azimuth = position.y.atan2(-position.x);
        Ok(PyObservables(Observables::new(
            azimuth, elevation, range, range_rate,
        )))
    }

    /// Return the rotation matrix from body-fixed to topocentric frame.
    ///
    /// Returns:
    ///     3x3 rotation matrix as a numpy array.
    fn rotation_to_topocentric<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let rot = self.0.rotation_to_topocentric();
        let rot: Vec<Vec<f64>> = rot.to_cols_array_2d().iter().map(|v| v.to_vec()).collect();
        Ok(PyArray2::from_vec2(py, &rot)?)
    }

    /// Return the geodetic longitude in radians.
    fn longitude(&self) -> f64 {
        self.0.longitude()
    }

    /// Return the geodetic latitude in radians.
    fn latitude(&self) -> f64 {
        self.0.latitude()
    }

    /// Return the altitude above the reference ellipsoid in km.
    fn altitude(&self) -> f64 {
        self.0.altitude()
    }
}

/// Propagator for ground station positions.
///
/// Computes the position of a ground station at arbitrary times by
/// rotating the body-fixed position according to the body's rotation.
///
/// Args:
///     location: The ground location to propagate.
#[pyclass(name = "GroundPropagator", module = "lox_space", frozen)]
pub struct PyGroundPropagator(DynGroundPropagator);

pub struct PyGroundPropagatorError(pub GroundPropagatorError);

impl From<PyGroundPropagatorError> for PyErr {
    fn from(err: PyGroundPropagatorError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

#[pymethods]
impl PyGroundPropagator {
    #[new]
    fn new(location: PyGroundLocation) -> Self {
        PyGroundPropagator(DynGroundPropagator::with_dynamic(location.0))
    }

    /// Propagate the ground station to one or more times.
    ///
    /// Args:
    ///     steps: Single Time or list of Times.
    ///
    /// Returns:
    ///     State (if single time) or Trajectory (if list of times).
    ///
    /// Raises:
    ///     ValueError: If propagation fails.
    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(time) = steps.extract::<PyTime>() {
            return Ok(Bound::new(
                py,
                PyState(
                    self.0
                        .propagate_dyn(time.0)
                        .map_err(PyGroundPropagatorError)?,
                ),
            )?
            .into_any());
        }
        if let Ok(steps) = steps.extract::<Vec<PyTime>>() {
            let steps = steps.into_iter().map(|s| s.0);
            return Ok(Bound::new(
                py,
                PyTrajectory(
                    self.0
                        .propagate_all_dyn(steps)
                        .map_err(PyGroundPropagatorError)?,
                ),
            )?
            .into_any());
        }
        Err(PyValueError::new_err("invalid time delta(s)"))
    }
}

pub struct PySgp4Error(pub Sgp4Error);

impl From<PySgp4Error> for PyErr {
    fn from(err: PySgp4Error) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// SGP4 (Simplified General Perturbations 4) orbit propagator.
///
/// SGP4 is the standard propagator for objects tracked by NORAD/Space-Track.
/// It uses Two-Line Element (TLE) data and models atmospheric drag, solar
/// radiation pressure, and gravitational perturbations.
///
/// Args:
///     tle: Two-Line Element set (2 or 3 lines).
#[pyclass(name = "SGP4", module = "lox_space", frozen)]
pub struct PySgp4(pub Sgp4);

#[pymethods]
impl PySgp4 {
    #[new]
    fn new(tle: &str) -> PyResult<Self> {
        let lines: Vec<&str> = tle.trim().split('\n').collect();
        let elements = if lines.len() == 3 {
            Elements::from_tle(
                Some(lines[0].to_string()),
                lines[1].as_bytes(),
                lines[2].as_bytes(),
            )
            .map_err(|err| PyValueError::new_err(err.to_string()))?
        } else if lines.len() == 2 {
            Elements::from_tle(None, lines[0].as_bytes(), lines[1].as_bytes())
                .map_err(|err| PyValueError::new_err(err.to_string()))?
        } else {
            return Err(PyValueError::new_err("invalid TLE"));
        };
        Ok(PySgp4(
            Sgp4::new(elements).map_err(|err| PyValueError::new_err(err.to_string()))?,
        ))
    }

    /// Return the TLE epoch time.
    fn time(&self) -> PyTime {
        PyTime(
            self.0
                .time()
                .try_to_scale(DynTimeScale::Tai, &DefaultOffsetProvider)
                .unwrap(),
        )
    }

    /// Propagate the orbit to one or more times.
    ///
    /// Args:
    ///     steps: Single Time or list of Times.
    ///     provider: EOP provider (optional, for UT1 time conversions).
    ///
    /// Returns:
    ///     State (if single time) or Trajectory (if list of times).
    ///
    /// Raises:
    ///     ValueError: If propagation fails.
    #[pyo3(signature = (steps, provider=None))]
    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let provider = provider.map(|p| &p.get().0);
        if let Ok(pytime) = steps.extract::<PyTime>() {
            let (time, dyntime) = match provider {
                Some(provider) => (
                    pytime
                        .0
                        .try_to_scale(Tai, provider)
                        .map_err(PyEopProviderError)?,
                    pytime
                        .0
                        .try_to_scale(DynTimeScale::Tai, provider)
                        .map_err(PyEopProviderError)?,
                ),
                None => (pytime.0.to_scale(Tai), pytime.0.to_scale(DynTimeScale::Tai)),
            };
            let s1 = self.0.propagate(time).map_err(PySgp4Error)?;
            return Ok(Bound::new(
                py,
                PyState(CartesianOrbit::new(
                    Cartesian::from_vecs(s1.position(), s1.velocity()),
                    dyntime,
                    DynOrigin::default(),
                    DynFrame::default(),
                )),
            )?
            .into_any());
        }
        if let Ok(pysteps) = steps.extract::<Vec<PyTime>>() {
            let mut states: Vec<DynCartesianOrbit> = Vec::with_capacity(pysteps.len());
            for step in pysteps {
                let (time, dyntime) = match provider {
                    Some(provider) => (
                        step.0
                            .try_to_scale(Tai, provider)
                            .map_err(PyEopProviderError)?,
                        step.0
                            .try_to_scale(DynTimeScale::Tai, provider)
                            .map_err(PyEopProviderError)?,
                    ),
                    None => (step.0.to_scale(Tai), step.0.to_scale(DynTimeScale::Tai)),
                };
                let s = self.0.propagate(time).map_err(PySgp4Error)?;
                let s = CartesianOrbit::new(
                    Cartesian::from_vecs(s.position(), s.velocity()),
                    dyntime,
                    DynOrigin::default(),
                    DynFrame::default(),
                );
                states.push(s);
            }
            return Ok(Bound::new(py, PyTrajectory(DynTrajectory::new(states)))?.into_any());
        }
        Err(PyValueError::new_err("invalid time delta(s)"))
    }
}

/// Compute visibility passes between a ground station and spacecraft.
///
/// This function finds all visibility windows where the spacecraft is above
/// the elevation mask as seen from the ground station.
///
/// Args:
///     times: List of Time objects defining the analysis period.
///     gs: Ground station location.
///     mask: Elevation mask defining minimum elevation constraints.
///     sc: Spacecraft trajectory.
///     ephemeris: SPK ephemeris data.
///     bodies: Optional list of bodies for occultation checking.
///
/// Returns:
///     List of Pass objects containing visibility windows and observables.
///
/// Raises:
///     ValueError: If ground station and spacecraft have different origins.
#[pyfunction]
#[pyo3(signature = (times, gs, mask, sc, ephemeris, bodies=None))]
pub fn visibility(
    times: &Bound<'_, PyList>,
    gs: PyGroundLocation,
    mask: &Bound<'_, PyElevationMask>,
    sc: &Bound<'_, PyTrajectory>,
    ephemeris: &Bound<'_, PySpk>,
    bodies: Option<Vec<PyOrigin>>,
) -> PyResult<Vec<PyPass>> {
    let sc = sc.get();
    if gs.0.origin().name() != sc.0.origin().name() {
        return Err(PyValueError::new_err(
            "ground station and spacecraft must have the same origin",
        ));
    }
    let times: Vec<DynTime> = times
        .extract::<Vec<PyTime>>()?
        .into_iter()
        .map(|s| s.0)
        .collect();
    let mask = &mask.borrow().0;
    let ephemeris = &ephemeris.get().0;
    let bodies: Vec<DynOrigin> = bodies
        .unwrap_or_default()
        .into_iter()
        .map(|b| b.0)
        .collect();
    Ok(
        crate::orbits::analysis::visibility_combined(
            &times, &gs.0, mask, &bodies, &sc.0, ephemeris,
        )
        .map_err(PyVisibilityError)?
        .into_iter()
        .map(PyPass)
        .collect(),
    )
}

/// Collection of named trajectories for batch visibility analysis.
///
/// Ensembles allow computing visibility for multiple spacecraft against
/// multiple ground stations efficiently using `visibility_all`.
///
/// Args:
///     ensemble: Dictionary mapping spacecraft names to their trajectories.
#[pyclass(name = "Ensemble", module = "lox_space", frozen)]
pub struct PyEnsemble(pub HashMap<String, DynTrajectory>);

#[pymethods]
impl PyEnsemble {
    #[new]
    pub fn new(ensemble: HashMap<String, PyTrajectory>) -> Self {
        Self(
            ensemble
                .into_iter()
                .map(|(name, trajectory)| (name, trajectory.0))
                .collect(),
        )
    }
}

/// Compute visibility for multiple spacecraft and ground stations.
///
/// This function efficiently computes visibility passes for all combinations
/// of spacecraft and ground stations, using parallel processing for large
/// workloads.
///
/// Args:
///     times: List of Time objects defining the analysis period.
///     ground_stations: Dictionary mapping station names to (location, mask) tuples.
///     spacecraft: Ensemble of spacecraft trajectories.
///     ephemeris: SPK ephemeris data.
///     bodies: Optional list of bodies for occultation checking.
///
/// Returns:
///     Nested dictionary: {spacecraft_name: {station_name: [passes]}}.
#[pyfunction]
#[pyo3(signature = (
    times,
    ground_stations,
    spacecraft,
    ephemeris,
    bodies=None,
))]
pub fn visibility_all(
    _py: Python<'_>,
    times: &Bound<'_, PyList>,
    ground_stations: HashMap<String, (PyGroundLocation, PyElevationMask)>,
    spacecraft: &Bound<'_, PyEnsemble>,
    ephemeris: &Bound<'_, PySpk>,
    bodies: Option<Vec<PyOrigin>>,
) -> PyResult<HashMap<String, HashMap<String, Vec<PyPass>>>> {
    let times: Vec<DynTime> = times
        .extract::<Vec<PyTime>>()?
        .into_iter()
        .map(|s| s.0)
        .collect();
    let bodies: Vec<DynOrigin> = bodies
        .unwrap_or_default()
        .into_iter()
        .map(|b| b.0)
        .collect();
    let spacecraft = &spacecraft.get().0;
    let ephemeris = &ephemeris.get().0;

    let _total_combinations = spacecraft.len() * ground_stations.len();

    // Adaptive strategy based on workload size
    if should_use_parallel(spacecraft.len(), ground_stations.len()) {
        visibility_all_parallel_optimized(
            _py,
            &times,
            &ground_stations,
            spacecraft,
            ephemeris,
            &bodies,
        )
    } else {
        visibility_all_sequential_optimized(
            _py,
            &times,
            &ground_stations,
            spacecraft,
            ephemeris,
            &bodies,
        )
    }
}

/// Determine if parallel processing should be used based on workload characteristics
fn should_use_parallel(spacecraft_count: usize, ground_station_count: usize) -> bool {
    let total_combinations = spacecraft_count * ground_station_count;

    // Use parallel processing if:
    // 1. We have enough work to justify overhead (>100 combinations)
    // 2. AND either enough spacecraft (>10) OR enough ground stations (>8)
    total_combinations > 100 && (spacecraft_count > 10 || ground_station_count > 8)
}

/// Sequential implementation optimized for small workloads
fn visibility_all_sequential_optimized<E>(
    _py: Python<'_>,
    times: &[DynTime],
    ground_stations: &HashMap<String, (PyGroundLocation, PyElevationMask)>,
    spacecraft: &HashMap<String, DynTrajectory>,
    ephemeris: &E,
    bodies: &[DynOrigin],
) -> PyResult<HashMap<String, HashMap<String, Vec<PyPass>>>>
where
    E: Ephemeris + Send + Sync,
{
    // Pre-allocate result with known capacity
    let mut result = HashMap::with_capacity(spacecraft.len());

    for (sc_name, sc_trajectory) in spacecraft {
        let mut gs_results = HashMap::with_capacity(ground_stations.len());

        for (gs_name, (gs_location, gs_mask)) in ground_stations {
            let passes = visibility_combined(
                times,
                &gs_location.0,
                &gs_mask.0,
                bodies,
                sc_trajectory,
                ephemeris,
            )
            .map_err(PyVisibilityError)?;

            gs_results.insert(gs_name.clone(), passes.into_iter().map(PyPass).collect());
        }

        result.insert(sc_name.clone(), gs_results);
    }

    Ok(result)
}

/// Parallel implementation optimized for large workloads
fn visibility_all_parallel_optimized<E>(
    py: Python<'_>,
    times: &[DynTime],
    ground_stations: &HashMap<String, (PyGroundLocation, PyElevationMask)>,
    spacecraft: &HashMap<String, DynTrajectory>,
    ephemeris: &E,
    bodies: &[DynOrigin],
) -> PyResult<HashMap<String, HashMap<String, Vec<PyPass>>>>
where
    E: Ephemeris + Send + Sync,
{
    // Create all combinations upfront for better work distribution
    let combinations: Vec<_> = spacecraft
        .iter()
        .flat_map(|(sc_name, sc_trajectory)| {
            ground_stations
                .iter()
                .map(move |(gs_name, (gs_location, gs_mask))| {
                    (sc_name, sc_trajectory, gs_name, gs_location, gs_mask)
                })
        })
        .collect();

    // Determine optimal chunk size based on number of combinations and available cores
    let num_threads = rayon::current_num_threads();
    let chunk_size = (combinations.len() / (num_threads * 2)).clamp(1, 50);

    // Process combinations in chunks with periodic GIL release
    let results: Result<Vec<_>, _> = py.detach(|| {
        combinations
            .par_chunks(chunk_size)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|(sc_name, sc_trajectory, gs_name, gs_location, gs_mask)| {
                        let passes = visibility_combined(
                            times,
                            &gs_location.0,
                            &gs_mask.0,
                            bodies,
                            sc_trajectory,
                            ephemeris,
                        )?;

                        let py_passes = passes.into_iter().map(PyPass).collect();
                        Ok(((*sc_name).clone(), (*gs_name).clone(), py_passes))
                    })
                    .collect::<Result<Vec<_>, VisibilityError>>()
            })
            .collect()
    });

    // Convert the flat results to nested hashmap structure
    let flat_results: Vec<_> = results
        .map_err(PyVisibilityError)?
        .into_iter()
        .flatten()
        .collect();
    let mut final_result: HashMap<String, HashMap<String, Vec<PyPass>>> = HashMap::new();

    for (sc_name, gs_name, passes) in flat_results {
        final_result
            .entry(sc_name)
            .or_default()
            .insert(gs_name, passes);
    }

    Ok(final_result)
}

pub struct PyElevationMaskError(pub ElevationMaskError);

impl From<PyElevationMaskError> for PyErr {
    fn from(err: PyElevationMaskError) -> Self {
        PyValueError::new_err(err.0.to_string())
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
        min_elevation: Option<f64>,
    ) -> PyResult<Self> {
        if let Some(min_elevation) = min_elevation {
            return Ok(PyElevationMask(ElevationMask::with_fixed_elevation(
                min_elevation,
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
    ///     min_elevation: Minimum elevation angle in radians.
    ///
    /// Returns:
    ///     ElevationMask with fixed minimum elevation.
    #[classmethod]
    fn fixed(_cls: &Bound<'_, PyType>, min_elevation: f64) -> Self {
        PyElevationMask(ElevationMask::with_fixed_elevation(min_elevation))
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

    fn __getnewargs__(&self) -> (Option<Vec<f64>>, Option<Vec<f64>>, Option<f64>) {
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
    fn fixed_elevation(&self) -> Option<f64> {
        match &self.0 {
            ElevationMask::Fixed(min_elevation) => Some(*min_elevation),
            ElevationMask::Variable(_) => None,
        }
    }

    /// Return the minimum elevation at the given azimuth.
    ///
    /// Args:
    ///     azimuth: Azimuth angle in radians.
    ///
    /// Returns:
    ///     Minimum elevation in radians.
    fn min_elevation(&self, azimuth: f64) -> f64 {
        self.0.min_elevation(azimuth)
    }
}

/// Observation data from a ground station to a target.
///
/// Observables contain the geometric relationship between a ground station
/// and a spacecraft, including angles and range information.
///
/// Args:
///     azimuth: Azimuth angle in radians (measured from north, clockwise).
///     elevation: Elevation angle in radians (above local horizon).
///     range: Distance to target in km.
///     range_rate: Rate of change of range in km/s.
#[pyclass(name = "Observables", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyObservables(pub Observables);

#[pymethods]
impl PyObservables {
    #[new]
    fn new(azimuth: f64, elevation: f64, range: f64, range_rate: f64) -> Self {
        PyObservables(Observables::new(azimuth, elevation, range, range_rate))
    }

    /// Return the azimuth angle in radians.
    fn azimuth(&self) -> f64 {
        self.0.azimuth()
    }

    /// Return the elevation angle in radians.
    fn elevation(&self) -> f64 {
        self.0.elevation()
    }

    /// Return the range (distance) in km.
    fn range(&self) -> f64 {
        self.0.range()
    }

    /// Return the range rate in km/s.
    fn range_rate(&self) -> f64 {
        self.0.range_rate()
    }
}

/// Represents a visibility pass between a ground station and spacecraft.
///
/// A Pass contains the visibility window (start and end times) along with
/// observables computed at regular intervals throughout the pass.
#[pyclass(name = "Pass", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyPass(pub DynPass);

#[pymethods]
impl PyPass {
    #[new]
    fn new(
        window: PyWindow,
        times: Vec<PyTime>,
        observables: Vec<PyObservables>,
    ) -> PyResult<Self> {
        let times: Vec<DynTime> = times.into_iter().map(|t| t.0).collect();
        let observables: Vec<Observables> = observables.into_iter().map(|o| o.0).collect();

        let pass = Pass::new(window.0, times, observables)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(PyPass(pass))
    }

    /// Return the visibility window for this pass.
    fn window(&self) -> PyWindow {
        PyWindow(*self.0.window())
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
        let window = self.0.window();
        format!(
            "Pass(window=Window({}, {}), {} observables)",
            window.start(),
            window.end(),
            self.0.observables().len()
        )
    }
}
