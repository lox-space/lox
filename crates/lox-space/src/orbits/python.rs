// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::analysis::python::PyObservables;
use crate::bodies::TryPointMass;
use crate::bodies::python::{PyOrigin, PyUndefinedOriginPropertyError};
use crate::earth::python::ut1::{PyEopProvider, PyEopProviderError};
use crate::ephem::python::{PyDafSpkError, PySpk};
use crate::frames::DynFrame;
use crate::frames::python::{PyDynRotationError, PyFrame};
use crate::orbits::events::{DetectError, Event, ZeroCrossing};
use crate::orbits::ground::{
    DynGroundLocation, DynGroundPropagator, GroundPropagatorError, Observables,
};
use crate::orbits::orbits::{
    CartesianOrbit, DynCartesianOrbit, DynTrajectory, TrajectorError, TrajectoryTransformationError,
};
use crate::orbits::propagators::Propagator;
use crate::orbits::propagators::numerical::{DynJ2Propagator, J2Error, J2Propagator};
use crate::orbits::propagators::semi_analytical::{DynVallado, Vallado, ValladoError};
use crate::orbits::propagators::sgp4::{Sgp4, Sgp4Error};
use crate::time::DynTime;
use crate::time::deltas::TimeDelta;
use crate::time::python::deltas::PyTimeDelta;
use crate::time::python::time::PyTime;
use crate::time::time_scales::{DynTimeScale, Tai};
use crate::units::python::{PyAngle, PyDistance, PyVelocity};
use glam::DVec3;
use lox_core::coords::{Cartesian, LonLatAlt};
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_time::intervals::{
    Interval, TimeInterval, complement_intervals, intersect_intervals, union_intervals,
};
use lox_units::{Angle, Distance, Velocity};
use numpy::{PyArray1, PyArray2, PyArrayMethods};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyString, PyType};
use sgp4::{Classification, Elements};
use std::f64::consts::PI;

/// Formats an f64 as a valid Python float literal (always includes a decimal point).
fn repr_f64(v: f64) -> String {
    let s = v.to_string();
    if v.is_finite() && !s.contains('.') {
        format!("{s}.0")
    } else {
        s
    }
}

struct PyTrajectoryTransformationError(TrajectoryTransformationError);

impl From<PyTrajectoryTransformationError> for PyErr {
    fn from(err: PyTrajectoryTransformationError) -> Self {
        // FIXME: wrong error type
        PyValueError::new_err(err.0.to_string())
    }
}

struct PyDetectError(DetectError);

impl From<PyDetectError> for PyErr {
    fn from(err: PyDetectError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// Concrete error for Python callback failures (avoids `Box<dyn Error>` sizing issues).
#[derive(Debug)]
struct PyCallbackError(String);

impl std::fmt::Display for PyCallbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PyCallbackError {}

struct PySsoError(lox_orbits::orbits::sso::SsoError);

impl From<PySsoError> for PyErr {
    fn from(err: PySsoError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// Convert a `DynTime` to TAI, using the EOP provider if available.
fn to_tai(
    time: DynTime,
    eop: Option<&lox_earth::eop::EopProvider>,
) -> PyResult<crate::time::Time<Tai>> {
    match eop {
        Some(eop) => time
            .try_to_scale(Tai, eop)
            .map_err(|e| PyEopProviderError(e).into()),
        None => Ok(time.to_scale(Tai)),
    }
}

/// Shared dispatch for the three-mode `propagate` pattern used by Vallado, J2,
/// and GroundPropagator.
///
/// `state_at` produces a single `DynCartesianOrbit` for a given `DynTime`.
/// `propagate_interval` produces a `DynTrajectory` for a given interval.
fn propagate_dispatch<'py>(
    py: Python<'py>,
    steps: &Bound<'py, PyAny>,
    end: Option<PyTime>,
    frame: Option<PyFrame>,
    provider: Option<&Bound<'_, PyEopProvider>>,
    state_at: impl Fn(DynTime) -> PyResult<DynCartesianOrbit>,
    propagate_interval: impl Fn(Interval<DynTime>) -> PyResult<DynTrajectory>,
) -> PyResult<Bound<'py, PyAny>> {
    if let Some(end) = end {
        let start = steps.extract::<PyTime>()?;
        let interval = Interval::new(start.0, end.0);
        let traj = PyTrajectory(propagate_interval(interval)?);
        return match frame {
            Some(frame) => Ok(Bound::new(py, traj.to_frame_inner(frame, provider)?)?.into_any()),
            None => Ok(Bound::new(py, traj)?.into_any()),
        };
    }
    if let Ok(time) = steps.extract::<PyTime>() {
        let state = PyCartesian(state_at(time.0)?);
        return match frame {
            Some(frame) => Ok(Bound::new(py, state.to_frame_inner(frame, provider)?)?.into_any()),
            None => Ok(Bound::new(py, state)?.into_any()),
        };
    }
    if let Ok(steps) = steps.extract::<Vec<PyTime>>() {
        let states: Result<Vec<_>, _> = steps.into_iter().map(|s| state_at(s.0)).collect();
        let traj = PyTrajectory(DynTrajectory::new(states?));
        return match frame {
            Some(frame) => Ok(Bound::new(py, traj.to_frame_inner(frame, provider)?)?.into_any()),
            None => Ok(Bound::new(py, traj)?.into_any()),
        };
    }
    Err(PyValueError::new_err("invalid time argument(s)"))
}

/// Find events where a function crosses zero.
///
/// Detects zero-crossings of a user-defined function of time.
///
/// Args:
///     func: Function that takes a Time and returns a float.
///     start: Start time of the analysis period.
///     end: End time of the analysis period.
///     step: Step size for sampling the function.
///
/// Returns:
///     List of Event objects at the detected zero-crossings.
#[pyfunction]
pub fn find_events(
    func: &Bound<'_, PyAny>,
    start: PyTime,
    end: PyTime,
    step: PyTimeDelta,
) -> PyResult<Vec<PyEvent>> {
    let interval = TimeInterval::new(start.0, end.0);
    let events = crate::orbits::events::try_find_events(
        |t| {
            let py_time = PyTime(t);
            func.call((py_time,), None)
                .and_then(|obj| obj.extract::<f64>())
                .map_err(|e| PyCallbackError(e.to_string()))
        },
        interval,
        step.0,
    )
    .map_err(PyDetectError)?;
    Ok(events.into_iter().map(PyEvent).collect())
}

/// Find time windows where a function is positive.
///
/// Finds all intervals where a user-defined function is positive.
/// Windows are bounded by zero-crossings of the function.
///
/// Args:
///     func: Function that takes a Time and returns a float.
///     start: Start time of the analysis period.
///     end: End time of the analysis period.
///     step: Step size for sampling the function.
///
/// Returns:
///     List of Interval objects for intervals where the function is positive.
#[pyfunction]
pub fn find_windows(
    func: &Bound<'_, PyAny>,
    start: PyTime,
    end: PyTime,
    step: PyTimeDelta,
) -> PyResult<Vec<PyInterval>> {
    let interval = TimeInterval::new(start.0, end.0);
    let windows = crate::orbits::events::try_find_windows(
        |t| {
            let py_time = PyTime(t);
            func.call((py_time,), None)
                .and_then(|obj| obj.extract::<f64>())
                .map_err(|e| PyCallbackError(e.to_string()))
        },
        interval,
        step.0,
    )
    .map_err(PyDetectError)?;
    Ok(windows.into_iter().map(PyInterval).collect())
}

/// Intersect two sorted lists of intervals.
///
/// Args:
///     a: First list of intervals.
///     b: Second list of intervals.
///
/// Returns:
///     List of intervals representing the intersection.
#[pyfunction]
#[pyo3(name = "intersect_intervals")]
pub fn py_intersect_intervals(a: Vec<PyInterval>, b: Vec<PyInterval>) -> Vec<PyInterval> {
    let a: Vec<_> = a.into_iter().map(|i| i.0).collect();
    let b: Vec<_> = b.into_iter().map(|i| i.0).collect();
    intersect_intervals(&a, &b)
        .into_iter()
        .map(PyInterval)
        .collect()
}

/// Compute the union of two sorted lists of intervals.
///
/// Args:
///     a: First list of intervals.
///     b: Second list of intervals.
///
/// Returns:
///     List of merged intervals representing the union.
#[pyfunction]
#[pyo3(name = "union_intervals")]
pub fn py_union_intervals(a: Vec<PyInterval>, b: Vec<PyInterval>) -> Vec<PyInterval> {
    let a: Vec<_> = a.into_iter().map(|i| i.0).collect();
    let b: Vec<_> = b.into_iter().map(|i| i.0).collect();
    union_intervals(&a, &b)
        .into_iter()
        .map(PyInterval)
        .collect()
}

/// Compute the complement of intervals within a bounding interval.
///
/// Args:
///     intervals: List of intervals to complement.
///     bound: Bounding interval.
///
/// Returns:
///     List of gap intervals within the bound.
#[pyfunction]
#[pyo3(name = "complement_intervals")]
pub fn py_complement_intervals(intervals: Vec<PyInterval>, bound: PyInterval) -> Vec<PyInterval> {
    let intervals: Vec<_> = intervals.into_iter().map(|i| i.0).collect();
    complement_intervals(&intervals, bound.0)
        .into_iter()
        .map(PyInterval)
        .collect()
}

/// Represents an orbital state (position and velocity) at a specific time.
///
/// A `Cartesian` captures the complete kinematic state of an object in space,
/// including its position, velocity, time, central body (origin), and
/// reference frame.
///
/// Args:
///     time: The epoch of this state.
///     position: Position vector as array-like [x, y, z] in meters.
///     velocity: Velocity vector as array-like [vx, vy, vz] in m/s.
///     x, y, z: Individual position components as Distance (alternative to position).
///     vx, vy, vz: Individual velocity components as Velocity (alternative to velocity).
///     origin: Central body (default: Earth).
///     frame: Reference frame (default: ICRF).
#[pyclass(name = "Cartesian", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyCartesian(pub DynCartesianOrbit);

#[pymethods]
impl PyCartesian {
    #[new]
    #[pyo3(signature = (time, position=None, velocity=None, *, x=None, y=None, z=None, vx=None, vy=None, vz=None, origin=None, frame=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        time: PyTime,
        position: Option<Vec<f64>>,
        velocity: Option<Vec<f64>>,
        x: Option<PyDistance>,
        y: Option<PyDistance>,
        z: Option<PyDistance>,
        vx: Option<PyVelocity>,
        vy: Option<PyVelocity>,
        vz: Option<PyVelocity>,
        origin: Option<&Bound<'_, PyAny>>,
        frame: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let origin = origin
            .map(PyOrigin::try_from)
            .transpose()?
            .unwrap_or_default();
        let frame = frame
            .map(PyFrame::try_from)
            .transpose()?
            .unwrap_or_default();

        let pos = if let Some(arr) = position {
            if arr.len() != 3 {
                return Err(PyValueError::new_err(
                    "position array must have exactly 3 elements",
                ));
            }
            DVec3::new(arr[0], arr[1], arr[2])
        } else if let (Some(x), Some(y), Some(z)) = (x, y, z) {
            DVec3::new(x.0.to_meters(), y.0.to_meters(), z.0.to_meters())
        } else {
            return Err(PyValueError::new_err(
                "either 'position' array or 'x', 'y', 'z' keyword arguments are required",
            ));
        };

        let vel = if let Some(arr) = velocity {
            if arr.len() != 3 {
                return Err(PyValueError::new_err(
                    "velocity array must have exactly 3 elements",
                ));
            }
            DVec3::new(arr[0], arr[1], arr[2])
        } else if let (Some(vx), Some(vy), Some(vz)) = (vx, vy, vz) {
            DVec3::new(
                vx.0.to_meters_per_second(),
                vy.0.to_meters_per_second(),
                vz.0.to_meters_per_second(),
            )
        } else {
            return Err(PyValueError::new_err(
                "either 'velocity' array or 'vx', 'vy', 'vz' keyword arguments are required",
            ));
        };

        Ok(PyCartesian(CartesianOrbit::new(
            Cartesian::from_vecs(pos, vel),
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

    /// Return the position vector as a numpy array in meters.
    fn position<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let pos = self.0.position();
        PyArray1::from_slice(py, &[pos.x, pos.y, pos.z])
    }

    /// Return the velocity vector as a numpy array in m/s.
    fn velocity<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let vel = self.0.velocity();
        PyArray1::from_slice(py, &[vel.x, vel.y, vel.z])
    }

    /// Return the x component of the position.
    #[getter]
    fn x(&self) -> PyDistance {
        PyDistance(Distance::meters(self.0.position().x))
    }

    /// Return the y component of the position.
    #[getter]
    fn y(&self) -> PyDistance {
        PyDistance(Distance::meters(self.0.position().y))
    }

    /// Return the z component of the position.
    #[getter]
    fn z(&self) -> PyDistance {
        PyDistance(Distance::meters(self.0.position().z))
    }

    /// Return the x component of the velocity.
    #[getter]
    fn vx(&self) -> PyVelocity {
        PyVelocity(Velocity::meters_per_second(self.0.velocity().x))
    }

    /// Return the y component of the velocity.
    #[getter]
    fn vy(&self) -> PyVelocity {
        PyVelocity(Velocity::meters_per_second(self.0.velocity().y))
    }

    /// Return the z component of the velocity.
    #[getter]
    fn vz(&self) -> PyVelocity {
        PyVelocity(Velocity::meters_per_second(self.0.velocity().z))
    }

    /// Transform this state to a different reference frame.
    ///
    /// Args:
    ///     frame: Target reference frame.
    ///     provider: EOP provider (required for ITRF transformations).
    ///
    /// Returns:
    ///     A new Cartesian in the target frame.
    ///
    /// Raises:
    ///     FrameTransformationError: If the transformation fails.
    #[pyo3(signature = (frame, provider=None))]
    fn to_frame(
        &self,
        frame: &Bound<'_, PyAny>,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Self> {
        let frame: PyFrame = frame.try_into()?;
        self.to_frame_inner(frame, provider)
    }

    /// Transform this state to a different central body.
    ///
    /// Args:
    ///     target: Target central body (origin).
    ///     ephemeris: SPK ephemeris data for computing body positions.
    ///
    /// Returns:
    ///     A new Cartesian relative to the target origin.
    ///
    /// Raises:
    ///     ValueError: If the transformation fails.
    fn to_origin(&self, target: &Bound<'_, PyAny>, ephemeris: &Bound<'_, PySpk>) -> PyResult<Self> {
        let target: PyOrigin = target.try_into()?;
        self.to_origin_inner(target, ephemeris)
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
                .try_to_ground_location()
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
        ))
    }

    fn __repr__(&self) -> String {
        let pos = self.0.position();
        let vel = self.0.velocity();
        format!(
            "Cartesian({}, [{}, {}, {}], [{}, {}, {}], origin={}, frame={})",
            self.time().__repr__(),
            repr_f64(pos.x),
            repr_f64(pos.y),
            repr_f64(pos.z),
            repr_f64(vel.x),
            repr_f64(vel.y),
            repr_f64(vel.z),
            self.origin().__repr__(),
            self.reference_frame().__repr__(),
        )
    }
}

impl PyCartesian {
    pub(crate) fn to_frame_inner(
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
        Ok(PyCartesian(CartesianOrbit::new(
            Cartesian::from_vecs(r1, v1),
            self.0.time(),
            self.0.origin(),
            frame.0,
        )))
    }

    pub(crate) fn to_origin_inner(
        &self,
        target: PyOrigin,
        ephemeris: &Bound<'_, PySpk>,
    ) -> PyResult<Self> {
        let frame = self.reference_frame();
        let s = if frame.0 != DynFrame::Icrf {
            self.to_frame_inner(PyFrame(DynFrame::Icrf), None)?
        } else {
            self.clone()
        };
        let spk = &ephemeris.borrow().0;
        let mut s1 = Self(s.0.try_to_origin(target.0, spk).map_err(PyDafSpkError)?);
        if frame.0 != DynFrame::Icrf {
            s1 = s1.to_frame_inner(frame, None)?
        }
        Ok(s1)
    }
}

/// Represents an orbit using Keplerian (classical) orbital elements.
///
/// Keplerian elements describe an orbit using six parameters that define
/// its shape, orientation, and position along the orbit.
///
/// The orbital shape can be specified in three ways:
/// - ``semi_major_axis`` + ``eccentricity``
/// - ``periapsis_radius`` + ``apoapsis_radius`` (keyword-only)
/// - ``periapsis_altitude`` + ``apoapsis_altitude`` (keyword-only)
///
/// Args:
///     time: Epoch of the elements.
///     semi_major_axis: Semi-major axis as Distance.
///     eccentricity: Orbital eccentricity (0 = circular, <1 = elliptical).
///     inclination: Inclination as Angle (default 0).
///     longitude_of_ascending_node: RAAN as Angle (default 0).
///     argument_of_periapsis: Argument of periapsis as Angle (default 0).
///     true_anomaly: True anomaly as Angle (default 0).
///     origin: Central body (default: Earth).
///     periapsis_radius: Periapsis radius as Distance (keyword-only).
///     apoapsis_radius: Apoapsis radius as Distance (keyword-only).
///     periapsis_altitude: Periapsis altitude as Distance (keyword-only).
///     apoapsis_altitude: Apoapsis altitude as Distance (keyword-only).
///     mean_anomaly: Mean anomaly as Angle (keyword-only, mutually exclusive with true_anomaly).
#[pyclass(name = "Keplerian", module = "lox_space", frozen)]
pub struct PyKeplerian(pub crate::orbits::orbits::DynKeplerianOrbit);

#[pymethods]
impl PyKeplerian {
    #[new]
    #[pyo3(signature = (
        time,
        semi_major_axis=None,
        eccentricity=None,
        inclination=None,
        longitude_of_ascending_node=None,
        argument_of_periapsis=None,
        true_anomaly=None,
        origin=None,
        *,
        periapsis_radius=None,
        apoapsis_radius=None,
        periapsis_altitude=None,
        apoapsis_altitude=None,
        mean_anomaly=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        time: PyTime,
        semi_major_axis: Option<PyDistance>,
        eccentricity: Option<f64>,
        inclination: Option<PyAngle>,
        longitude_of_ascending_node: Option<PyAngle>,
        argument_of_periapsis: Option<PyAngle>,
        true_anomaly: Option<PyAngle>,
        origin: Option<&Bound<'_, PyAny>>,
        periapsis_radius: Option<PyDistance>,
        apoapsis_radius: Option<PyDistance>,
        periapsis_altitude: Option<PyDistance>,
        apoapsis_altitude: Option<PyDistance>,
        mean_anomaly: Option<PyAngle>,
    ) -> PyResult<Self> {
        use lox_orbits::orbits::builders::KeplerianOrbitBuilder;

        let origin = origin
            .map(PyOrigin::try_from)
            .transpose()?
            .map(|o| o.0)
            .unwrap_or_default();
        origin
            .try_gravitational_parameter()
            .map_err(PyUndefinedOriginPropertyError)?;

        let tai = to_tai(time.0, None)?;
        let mut builder = KeplerianOrbitBuilder::new()
            .with_time(tai)
            .with_origin(origin);

        match (
            semi_major_axis,
            eccentricity,
            periapsis_radius,
            apoapsis_radius,
            periapsis_altitude,
            apoapsis_altitude,
        ) {
            (Some(sma), Some(ecc), None, None, None, None) => {
                builder = builder.with_semi_major_axis(Distance::meters(sma.0.to_meters()), ecc);
            }
            (None, None, Some(rp), Some(ra), None, None) => {
                builder = builder.with_radii(
                    Distance::meters(rp.0.to_meters()),
                    Distance::meters(ra.0.to_meters()),
                );
            }
            (None, None, None, None, Some(alt_p), Some(alt_a)) => {
                builder = builder.with_altitudes(
                    Distance::meters(alt_p.0.to_meters()),
                    Distance::meters(alt_a.0.to_meters()),
                );
            }
            (None, None, None, None, None, None) => {
                return Err(PyValueError::new_err(
                    "orbital shape must be specified via one of: \
                     (semi_major_axis, eccentricity), \
                     (periapsis_radius, apoapsis_radius), or \
                     (periapsis_altitude, apoapsis_altitude)",
                ));
            }
            _ => {
                return Err(PyValueError::new_err(
                    "orbital shape must be specified via exactly one of: \
                     (semi_major_axis, eccentricity), \
                     (periapsis_radius, apoapsis_radius), or \
                     (periapsis_altitude, apoapsis_altitude)",
                ));
            }
        }

        if let Some(inc) = inclination {
            builder = builder.with_inclination(Angle::radians(inc.0.to_radians()));
        }
        if let Some(raan) = longitude_of_ascending_node {
            builder = builder.with_longitude_of_ascending_node(Angle::radians(raan.0.to_radians()));
        }
        if let Some(aop) = argument_of_periapsis {
            builder = builder.with_argument_of_periapsis(Angle::radians(aop.0.to_radians()));
        }
        if let Some(ta) = true_anomaly {
            builder = builder.with_true_anomaly(Angle::radians(ta.0.to_radians()));
        }
        if let Some(ma) = mean_anomaly {
            builder = builder.with_mean_anomaly(Angle::radians(ma.0.to_radians()));
        }

        let orbit = builder
            .build()
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(PyKeplerian(orbit.into_dyn()))
    }

    /// Construct a circular orbit.
    ///
    /// Exactly one of ``semi_major_axis`` or ``altitude`` must be provided.
    /// Eccentricity is always 0 and argument of periapsis is always 0.
    ///
    /// Args:
    ///     time: Epoch of the orbit.
    ///     semi_major_axis: Semi-major axis (mutually exclusive with altitude).
    ///     altitude: Orbital altitude (mutually exclusive with semi_major_axis).
    ///     inclination: Inclination (default 0).
    ///     longitude_of_ascending_node: RAAN (default 0).
    ///     true_anomaly: True anomaly (default 0).
    ///     origin: Central body (default: Earth).
    #[classmethod]
    #[pyo3(signature = (
        time,
        *,
        semi_major_axis=None,
        altitude=None,
        inclination=None,
        longitude_of_ascending_node=None,
        true_anomaly=None,
        origin=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn circular(
        _cls: &Bound<'_, PyType>,
        time: PyTime,
        semi_major_axis: Option<PyDistance>,
        altitude: Option<PyDistance>,
        inclination: Option<PyAngle>,
        longitude_of_ascending_node: Option<PyAngle>,
        true_anomaly: Option<PyAngle>,
        origin: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        use lox_orbits::orbits::builders::CircularBuilder;

        let origin = origin
            .map(PyOrigin::try_from)
            .transpose()?
            .map(|o| o.0)
            .unwrap_or_default();
        origin
            .try_gravitational_parameter()
            .map_err(PyUndefinedOriginPropertyError)?;

        let tai = to_tai(time.0, None)?;

        let mut builder = CircularBuilder::new().with_time(tai).with_origin(origin);

        match (semi_major_axis, altitude) {
            (Some(sma), None) => {
                builder = builder.with_semi_major_axis(Distance::meters(sma.0.to_meters()));
            }
            (None, Some(alt)) => {
                builder = builder.with_altitude(Distance::meters(alt.0.to_meters()));
            }
            _ => {
                return Err(PyValueError::new_err(
                    "exactly one of `semi_major_axis` or `altitude` must be specified",
                ));
            }
        }

        if let Some(inc) = inclination {
            builder = builder.with_inclination(Angle::radians(inc.0.to_radians()));
        }
        if let Some(raan) = longitude_of_ascending_node {
            builder = builder.with_longitude_of_ascending_node(Angle::radians(raan.0.to_radians()));
        }
        if let Some(ta) = true_anomaly {
            builder = builder.with_true_anomaly(Angle::radians(ta.0.to_radians()));
        }

        let orbit = builder
            .build()
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(PyKeplerian(orbit.into_dyn()))
    }

    /// Construct a Sun-synchronous orbit.
    ///
    /// Exactly one of ``altitude``, ``semi_major_axis``, or ``inclination``
    /// must be provided. The remaining orbital elements are derived from the
    /// SSO constraint.
    ///
    /// Args:
    ///     time: Epoch of the orbit.
    ///     altitude: Orbital altitude (mutually exclusive with semi_major_axis/inclination).
    ///     semi_major_axis: Semi-major axis (mutually exclusive with altitude/inclination).
    ///     inclination: Inclination (mutually exclusive with altitude/semi_major_axis).
    ///     eccentricity: Eccentricity (default 0.0).
    ///     ltan: Local time of ascending node as ``(hours, minutes)`` tuple.
    ///     ltdn: Local time of descending node as ``(hours, minutes)`` tuple.
    ///     argument_of_periapsis: Argument of periapsis (default 0.0).
    ///     true_anomaly: True anomaly (default 0.0).
    ///     provider: EOP provider for time scale conversions.
    #[classmethod]
    #[pyo3(signature = (
        time,
        *,
        altitude=None,
        semi_major_axis=None,
        inclination=None,
        eccentricity=0.0,
        ltan=None,
        ltdn=None,
        argument_of_periapsis=None,
        true_anomaly=None,
        provider=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn sso(
        _cls: &Bound<'_, PyType>,
        time: PyTime,
        altitude: Option<PyDistance>,
        semi_major_axis: Option<PyDistance>,
        inclination: Option<PyAngle>,
        eccentricity: f64,
        ltan: Option<(u8, u8)>,
        ltdn: Option<(u8, u8)>,
        argument_of_periapsis: Option<PyAngle>,
        true_anomaly: Option<PyAngle>,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Self> {
        use lox_orbits::orbits::sso::SsoBuilder;

        let tai = to_tai(time.0, provider.map(|p| &p.get().0))?;

        // Pre-extract values as Copy types for use in both match arms.
        let alt = altitude.map(|a| Distance::meters(a.0.to_meters()));
        let sma = semi_major_axis.map(|s| Distance::meters(s.0.to_meters()));
        let inc = inclination.map(|i| Angle::radians(i.0.to_radians()));
        let aop = argument_of_periapsis.map(|a| Angle::radians(a.0.to_radians()));
        let ta = true_anomaly.map(|a| Angle::radians(a.0.to_radians()));

        macro_rules! configure_and_build {
            ($builder:expr) => {{
                let mut builder = $builder;
                match (alt, sma, inc) {
                    (Some(a), None, None) => builder = builder.with_altitude(a),
                    (None, Some(s), None) => builder = builder.with_semi_major_axis(s),
                    (None, None, Some(i)) => builder = builder.with_inclination(i),
                    _ => {
                        return Err(PyValueError::new_err(
                            "exactly one of `altitude`, `semi_major_axis`, \
                             or `inclination` must be specified",
                        ));
                    }
                }
                builder = builder.with_eccentricity(eccentricity);
                match (ltan, ltdn) {
                    (Some((h, m)), None) => builder = builder.with_ltan(h, m),
                    (None, Some((h, m))) => builder = builder.with_ltdn(h, m),
                    (None, None) => {}
                    _ => {
                        return Err(PyValueError::new_err(
                            "at most one of `ltan` or `ltdn` can be specified",
                        ));
                    }
                }
                if let Some(aop) = aop {
                    builder = builder.with_argument_of_periapsis(aop);
                }
                if let Some(ta) = ta {
                    builder = builder.with_true_anomaly(ta);
                }
                builder.build().map_err(PySsoError)?
            }};
        }

        let orbit = match provider {
            Some(p) => {
                configure_and_build!(
                    SsoBuilder::default()
                        .with_provider(&p.get().0)
                        .with_time(tai)
                )
            }
            None => configure_and_build!(SsoBuilder::default().with_time(tai)),
        };

        Ok(PyKeplerian(orbit.into_dyn()))
    }

    /// Return the epoch of these elements.
    fn time(&self) -> PyTime {
        PyTime(self.0.time())
    }

    /// Return the central body (origin) of this orbit.
    fn origin(&self) -> PyOrigin {
        PyOrigin(self.0.origin())
    }

    /// Return the semi-major axis.
    fn semi_major_axis(&self) -> PyDistance {
        PyDistance(self.0.semi_major_axis())
    }

    /// Return the orbital eccentricity.
    fn eccentricity(&self) -> f64 {
        self.0.eccentricity().as_f64()
    }

    /// Return the inclination.
    fn inclination(&self) -> PyAngle {
        PyAngle(Angle::radians(self.0.inclination().as_f64()))
    }

    /// Return the longitude of the ascending node (RAAN).
    fn longitude_of_ascending_node(&self) -> PyAngle {
        PyAngle(Angle::radians(
            self.0.longitude_of_ascending_node().as_f64(),
        ))
    }

    /// Return the argument of periapsis.
    fn argument_of_periapsis(&self) -> PyAngle {
        PyAngle(Angle::radians(self.0.argument_of_periapsis().as_f64()))
    }

    /// Return the true anomaly.
    fn true_anomaly(&self) -> PyAngle {
        PyAngle(Angle::radians(self.0.true_anomaly().as_f64()))
    }

    /// Convert these Keplerian elements to a Cartesian state.
    ///
    /// Returns:
    ///     Cartesian with position and velocity vectors.
    fn to_cartesian(&self) -> PyResult<PyCartesian> {
        Ok(PyCartesian(
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

    fn __repr__(&self) -> String {
        format!(
            "Keplerian({}, {}, {}, {}, {}, {}, {}, origin={})",
            self.time().__repr__(),
            self.semi_major_axis().__repr__(),
            repr_f64(self.eccentricity()),
            self.inclination().__repr__(),
            self.longitude_of_ascending_node().__repr__(),
            self.argument_of_periapsis().__repr__(),
            self.true_anomaly().__repr__(),
            self.origin().__repr__(),
        )
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
        let states: Vec<PyCartesian> = states.extract()?;
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
        origin: Option<&Bound<'_, PyAny>>,
        frame: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let origin = origin
            .map(PyOrigin::try_from)
            .transpose()?
            .unwrap_or_default();
        let frame = frame
            .map(PyFrame::try_from)
            .transpose()?
            .unwrap_or_default();
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
        let data: Vec<Vec<f64>> = self.0.to_vec();
        Ok(PyArray2::from_vec2(py, &data)?)
    }

    /// Return the list of states in this trajectory.
    fn states(&self) -> Vec<PyCartesian> {
        self.0.states().into_iter().map(PyCartesian).collect()
    }

    /// Find events where a function crosses zero.
    ///
    /// Args:
    ///     func: Function that takes a State and returns a float.
    ///           Events are detected where the function crosses zero.
    ///     step: Step size for sampling the function.
    ///
    /// Returns:
    ///     List of Event objects.
    fn find_events(&self, func: &Bound<'_, PyAny>, step: PyTimeDelta) -> PyResult<Vec<PyEvent>> {
        let traj = &self.0;
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());
        let events = crate::orbits::events::try_find_events(
            |t| {
                let state = PyCartesian(traj.interpolate_at(t));
                func.call((state,), None)
                    .and_then(|obj| obj.extract::<f64>())
                    .map_err(|e| PyCallbackError(e.to_string()))
            },
            interval,
            step.0,
        )
        .map_err(PyDetectError)?;
        Ok(events.into_iter().map(PyEvent).collect())
    }

    /// Find time windows where a function is positive.
    ///
    /// Args:
    ///     func: Function that takes a State and returns a float.
    ///           Windows are periods where the function is positive.
    ///     step: Step size for sampling the function.
    ///
    /// Returns:
    ///     List of Interval objects.
    fn find_windows(
        &self,
        func: &Bound<'_, PyAny>,
        step: PyTimeDelta,
    ) -> PyResult<Vec<PyInterval>> {
        let traj = &self.0;
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());
        let windows = crate::orbits::events::try_find_windows(
            |t| {
                let state = PyCartesian(traj.interpolate_at(t));
                func.call((state,), None)
                    .and_then(|obj| obj.extract::<f64>())
                    .map_err(|e| PyCallbackError(e.to_string()))
            },
            interval,
            step.0,
        )
        .map_err(PyDetectError)?;
        Ok(windows.into_iter().map(PyInterval).collect())
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
    fn interpolate(&self, time: &Bound<'_, PyAny>) -> PyResult<PyCartesian> {
        if let Ok(delta) = time.extract::<PyTimeDelta>() {
            return Ok(PyCartesian(self.0.interpolate(delta.0)));
        }
        if let Ok(time) = time.extract::<PyTime>() {
            return Ok(PyCartesian(self.0.interpolate_at(time.0)));
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
        frame: &Bound<'_, PyAny>,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Self> {
        let frame: PyFrame = frame.try_into()?;
        self.to_frame_inner(frame, provider)
    }

    /// Transform all states in the trajectory to a different central body.
    ///
    /// Args:
    ///     target: Target central body (origin).
    ///     ephemeris: SPK ephemeris data.
    ///
    /// Returns:
    ///     A new Trajectory relative to the target origin.
    fn to_origin(&self, target: &Bound<'_, PyAny>, ephemeris: &Bound<'_, PySpk>) -> PyResult<Self> {
        let target: PyOrigin = target.try_into()?;
        let mut states: Vec<DynCartesianOrbit> = Vec::with_capacity(self.states().len());
        for s in self.states() {
            states.push(s.to_origin_inner(target.clone(), ephemeris)?.0);
        }
        Ok(Self(DynTrajectory::new(states)))
    }

    pub fn __repr__(&self) -> String {
        let n = self.0.states().len();
        format!(
            "Trajectory({n} states, origin={}, frame={})",
            self.origin().__repr__(),
            self.reference_frame().__repr__(),
        )
    }
}

impl PyTrajectory {
    pub(crate) fn to_frame_inner(
        &self,
        frame: PyFrame,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Self> {
        let mut states: Vec<DynCartesianOrbit> = Vec::with_capacity(self.0.states().len());
        for s in self.0.states() {
            states.push(PyCartesian(s).to_frame_inner(frame.clone(), provider)?.0);
        }
        Ok(PyTrajectory(DynTrajectory::new(states)))
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
#[pyclass(name = "Event", module = "lox_space", frozen)]
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

/// Represents a time window (interval between two times).
///
/// Intervals are used to represent periods when certain conditions are met,
/// such as visibility intervals between a ground station and spacecraft.
///
/// Args:
///     start: The start time of the interval.
///     end: The end time of the interval.
#[pyclass(name = "Interval", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyInterval(pub TimeInterval<DynTimeScale>);

#[pymethods]
impl PyInterval {
    #[new]
    fn new(start: PyTime, end: PyTime) -> Self {
        PyInterval(TimeInterval::new(start.0, end.0))
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Interval({}, {})",
            self.start().__repr__(),
            self.end().__repr__(),
        )
    }

    /// Return the start time of this interval.
    fn start(&self) -> PyTime {
        PyTime(self.0.start())
    }

    /// Return the end time of this interval.
    fn end(&self) -> PyTime {
        PyTime(self.0.end())
    }

    /// Return the duration of this interval.
    fn duration(&self) -> PyTimeDelta {
        PyTimeDelta(self.0.duration())
    }

    /// Return whether this interval is empty (start >= end).
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return whether this interval contains the given time.
    fn contains_time(&self, time: PyTime) -> bool {
        self.0.contains_time(time.0)
    }

    /// Return whether this interval fully contains another interval.
    fn contains(&self, other: &PyInterval) -> bool {
        self.0.contains(&other.0)
    }

    /// Return the intersection of this interval with another.
    fn intersect(&self, other: PyInterval) -> PyInterval {
        PyInterval(self.0.intersect(other.0))
    }

    /// Return whether this interval overlaps with another.
    fn overlaps(&self, other: PyInterval) -> bool {
        self.0.overlaps(other.0)
    }

    /// Return a list of times spaced by the given step within this interval.
    ///
    /// Args:
    ///     step: The step size (must be non-zero).
    ///
    /// Returns:
    ///     List of Time objects.
    ///
    /// Raises:
    ///     ValueError: If step is zero.
    fn step_by(&self, step: PyTimeDelta) -> PyResult<Vec<PyTime>> {
        if step.0.is_zero() {
            return Err(PyValueError::new_err("step must be non-zero"));
        }
        Ok(self.0.step_by(step.0).map(PyTime).collect())
    }

    /// Return a list of n evenly-spaced times within this interval.
    ///
    /// Args:
    ///     n: Number of points (must be >= 2).
    ///
    /// Returns:
    ///     List of Time objects.
    ///
    /// Raises:
    ///     ValueError: If n < 2.
    fn linspace(&self, n: usize) -> PyResult<Vec<PyTime>> {
        if n < 2 {
            return Err(PyValueError::new_err("n must be >= 2"));
        }
        Ok(self.0.linspace(n).into_iter().map(PyTime).collect())
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
    fn new(initial_state: PyCartesian, max_iter: Option<i32>) -> PyResult<Self> {
        let mut vallado = Vallado::try_new(initial_state.0).map_err(PyValladoError)?;
        if let Some(max_iter) = max_iter {
            vallado = vallado.with_max_iter(max_iter);
        }
        Ok(PyVallado(vallado))
    }

    /// Propagate the orbit.
    ///
    /// Supports three calling modes:
    ///
    /// - Single time: ``propagate(time)`` → State
    /// - Two times: ``propagate(start, end)`` → Trajectory (propagator-chosen steps)
    /// - List of times: ``propagate([t1, t2, ...])`` → Trajectory (caller-chosen steps)
    ///
    /// Args:
    ///     steps: Single Time, list of Times, or start Time (when ``end`` is given).
    ///     end: End time (optional, for interval propagation).
    ///     frame: Target reference frame (optional).
    ///     provider: EOP provider for frame transformation (optional).
    ///
    /// Returns:
    ///     State or Trajectory, optionally transformed to the target frame.
    ///
    /// Raises:
    ///     ValueError: If propagation or frame transformation fails.
    #[pyo3(signature = (steps, end=None, frame=None, provider=None))]
    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
        end: Option<PyTime>,
        frame: Option<&Bound<'_, PyAny>>,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let frame = frame.map(PyFrame::try_from).transpose()?;
        propagate_dispatch(
            py,
            steps,
            end,
            frame,
            provider,
            |t| Ok(self.0.state_at(t).map_err(PyValladoError)?),
            |i| Ok(self.0.propagate(i).map_err(PyValladoError)?),
        )
    }

    fn __repr__(&self) -> String {
        let state = PyCartesian(*self.0.initial_state());
        let max_iter = self.0.max_iter();
        format!("Vallado({}, max_iter={})", state.__repr__(), max_iter,)
    }
}

pub struct PyJ2Error(pub J2Error);

impl From<PyJ2Error> for PyErr {
    fn from(err: PyJ2Error) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// Numerical J2 orbit propagator using Dormand-Prince 8(5,3) integration.
///
/// This propagator accounts for the J2 zonal harmonic perturbation, which
/// models the oblateness of the central body. It uses an adaptive Runge-Kutta
/// integrator (DOP853).
///
/// Args:
///     initial_state: Initial orbital state.
///     rtol: Relative tolerance (default: 1e-8).
///     atol: Absolute tolerance (default: 1e-6).
///     h_max: Maximum step size in seconds (default: auto from orbital timescale).
///     h_min: Minimum step size in seconds (default: 1e-6).
///     max_steps: Maximum number of integration steps (default: 100000).
#[pyclass(name = "J2", module = "lox_space", frozen)]
#[derive(Clone)]
pub struct PyJ2Propagator(pub DynJ2Propagator);

#[pymethods]
impl PyJ2Propagator {
    #[new]
    #[pyo3(signature = (initial_state, rtol=None, atol=None, h_max=None, h_min=None, max_steps=None))]
    fn new(
        initial_state: PyCartesian,
        rtol: Option<f64>,
        atol: Option<f64>,
        h_max: Option<f64>,
        h_min: Option<f64>,
        max_steps: Option<usize>,
    ) -> PyResult<Self> {
        let mut propagator =
            J2Propagator::try_new(initial_state.0).map_err(PyUndefinedOriginPropertyError)?;
        if let Some(rtol) = rtol {
            propagator = propagator.with_rtol(rtol);
        }
        if let Some(atol) = atol {
            propagator = propagator.with_atol(atol);
        }
        if let Some(h_max) = h_max {
            propagator = propagator.with_h_max(h_max);
        }
        if let Some(h_min) = h_min {
            propagator = propagator.with_h_min(h_min);
        }
        if let Some(max_steps) = max_steps {
            propagator = propagator.with_max_steps(max_steps);
        }
        Ok(PyJ2Propagator(propagator))
    }

    /// Propagate the orbit.
    ///
    /// Supports three calling modes:
    ///
    /// - Single time: ``propagate(time)`` → State
    /// - Two times: ``propagate(start, end)`` → Trajectory (adaptive ODE steps)
    /// - List of times: ``propagate([t1, t2, ...])`` → Trajectory (caller-chosen steps)
    ///
    /// Args:
    ///     steps: Single Time, list of Times, or start Time (when ``end`` is given).
    ///     end: End time (optional, for interval propagation).
    ///     frame: Target reference frame (optional).
    ///     provider: EOP provider for frame transformation (optional).
    ///
    /// Returns:
    ///     State or Trajectory, optionally transformed to the target frame.
    ///
    /// Raises:
    ///     ValueError: If propagation or frame transformation fails.
    #[pyo3(signature = (steps, end=None, frame=None, provider=None))]
    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
        end: Option<PyTime>,
        frame: Option<&Bound<'_, PyAny>>,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let frame = frame.map(PyFrame::try_from).transpose()?;
        propagate_dispatch(
            py,
            steps,
            end,
            frame,
            provider,
            |t| Ok(self.0.state_at(t).map_err(PyJ2Error)?),
            |i| Ok(self.0.propagate(i).map_err(PyJ2Error)?),
        )
    }

    fn __repr__(&self) -> String {
        let state = PyCartesian(*self.0.initial_state());
        format!("J2({})", state.__repr__())
    }
}

/// Represents a location on the surface of a celestial body.
///
/// Ground locations are specified using geodetic coordinates (longitude, latitude,
/// altitude) relative to a central body's reference ellipsoid.
///
/// Args:
///     origin: The central body (e.g., Earth, Moon).
///     longitude: Geodetic longitude as Angle.
///     latitude: Geodetic latitude as Angle.
///     altitude: Altitude above the reference ellipsoid as Distance.
#[pyclass(name = "GroundLocation", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyGroundLocation(pub DynGroundLocation);

#[pymethods]
impl PyGroundLocation {
    #[new]
    fn new(
        origin: &Bound<'_, PyAny>,
        longitude: PyAngle,
        latitude: PyAngle,
        altitude: PyDistance,
    ) -> PyResult<Self> {
        let origin: PyOrigin = origin.try_into()?;
        let coordinates = LonLatAlt::builder()
            .longitude(longitude.0)
            .latitude(latitude.0)
            .altitude(altitude.0)
            .build()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PyGroundLocation(
            DynGroundLocation::try_new(coordinates, origin.0).map_err(PyValueError::new_err)?,
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
        state: PyCartesian,
        provider: Option<&Bound<'_, PyEopProvider>>,
        frame: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyObservables> {
        let frame = frame
            .map(PyFrame::try_from)
            .transpose()?
            .unwrap_or(PyFrame(DynFrame::Iau(state.0.origin())));
        let state = state.to_frame_inner(frame, provider)?;
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

    /// Return the geodetic longitude.
    fn longitude(&self) -> PyAngle {
        PyAngle(Angle::radians(self.0.longitude()))
    }

    /// Return the geodetic latitude.
    fn latitude(&self) -> PyAngle {
        PyAngle(Angle::radians(self.0.latitude()))
    }

    /// Return the altitude above the reference ellipsoid.
    fn altitude(&self) -> PyDistance {
        PyDistance(Distance::kilometers(self.0.altitude()))
    }

    /// Return the central body (origin).
    fn origin(&self) -> PyOrigin {
        PyOrigin(self.0.origin())
    }

    pub fn __repr__(&self) -> String {
        format!(
            "GroundLocation({}, {}, {}, {})",
            PyOrigin(self.0.origin()).__repr__(),
            self.longitude().__repr__(),
            self.latitude().__repr__(),
            self.altitude().__repr__(),
        )
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
    fn new(location: PyGroundLocation) -> PyResult<Self> {
        Ok(PyGroundPropagator(
            DynGroundPropagator::try_new(location.0).map_err(PyValueError::new_err)?,
        ))
    }

    /// Propagate the ground station.
    ///
    /// Supports three calling modes:
    ///
    /// - Single time: ``propagate(time)`` → State
    /// - Two times: ``propagate(start, end)`` → Trajectory (fixed step)
    /// - List of times: ``propagate([t1, t2, ...])`` → Trajectory (caller-chosen steps)
    ///
    /// Args:
    ///     steps: Single Time, list of Times, or start Time (when ``end`` is given).
    ///     end: End time (optional, for interval propagation).
    ///     frame: Target reference frame (optional).
    ///     provider: EOP provider for frame transformation (optional).
    ///
    /// Returns:
    ///     State or Trajectory, optionally transformed to the target frame.
    ///
    /// Raises:
    ///     ValueError: If propagation or frame transformation fails.
    #[pyo3(signature = (steps, end=None, frame=None, provider=None))]
    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
        end: Option<PyTime>,
        frame: Option<&Bound<'_, PyAny>>,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let frame = frame.map(PyFrame::try_from).transpose()?;
        propagate_dispatch(
            py,
            steps,
            end,
            frame,
            provider,
            |t| Ok(self.0.state_at(t)),
            |i| Ok(self.0.propagate(i).map_err(PyGroundPropagatorError)?),
        )
    }

    fn __repr__(&self) -> String {
        let loc = PyGroundLocation(self.0.location().clone());
        format!("GroundPropagator({})", loc.__repr__())
    }
}

/// Two-Line Element set (TLE) for satellite orbit data.
///
/// Parses and exposes the orbital elements from a NORAD Two-Line Element set.
///
/// Args:
///     tle: TLE as a string (2 or 3 lines) or a list of 2–3 strings.
#[pyclass(name = "TLE", module = "lox_space", frozen)]
#[derive(Clone)]
pub struct PyTle {
    elements: Elements,
    raw: String,
}

impl PyTle {
    fn from_string(tle: &str) -> PyResult<Self> {
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
        Ok(PyTle {
            elements,
            raw: tle.trim().to_string(),
        })
    }
}

#[pymethods]
impl PyTle {
    #[new]
    fn new(tle: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(s) = tle.cast::<PyString>() {
            Self::from_string(&s.to_string())
        } else if let Ok(list) = tle.cast::<PyList>() {
            let lines: Vec<String> = list.extract()?;
            Self::from_string(&lines.join("\n"))
        } else {
            Err(PyValueError::new_err(
                "expected a TLE string or list of strings",
            ))
        }
    }

    /// Satellite name, if present (line 0 of a 3-line TLE).
    fn object_name(&self) -> Option<String> {
        self.elements.object_name.clone()
    }

    /// International designator (e.g. "98067A").
    fn international_designator(&self) -> Option<String> {
        self.elements.international_designator.clone()
    }

    /// NORAD catalog number.
    fn norad_id(&self) -> u64 {
        self.elements.norad_id
    }

    /// Classification: "U" (unclassified), "C" (classified), or "S" (secret).
    fn classification(&self) -> &str {
        match self.elements.classification {
            Classification::Unclassified => "U",
            Classification::Classified => "C",
            Classification::Secret => "S",
        }
    }

    /// TLE epoch as a Time (TAI scale).
    fn epoch(&self) -> PyTime {
        let tai: lox_time::time::Time<Tai> = self.elements.datetime.and_utc().into();
        PyTime(tai.into_dyn())
    }

    /// Orbital inclination.
    fn inclination(&self) -> PyAngle {
        PyAngle::new(self.elements.inclination * PI / 180.0)
    }

    /// Right ascension of the ascending node (RAAN).
    fn right_ascension(&self) -> PyAngle {
        PyAngle::new(self.elements.right_ascension * PI / 180.0)
    }

    /// Orbital eccentricity (dimensionless).
    fn eccentricity(&self) -> f64 {
        self.elements.eccentricity
    }

    /// Argument of perigee.
    fn argument_of_perigee(&self) -> PyAngle {
        PyAngle::new(self.elements.argument_of_perigee * PI / 180.0)
    }

    /// Mean anomaly.
    fn mean_anomaly(&self) -> PyAngle {
        PyAngle::new(self.elements.mean_anomaly * PI / 180.0)
    }

    /// Mean motion in revolutions per day (Kozai convention).
    fn mean_motion(&self) -> f64 {
        self.elements.mean_motion
    }

    /// First derivative of mean motion (rev/day²).
    fn mean_motion_dot(&self) -> f64 {
        self.elements.mean_motion_dot
    }

    /// Second derivative of mean motion (rev/day³).
    fn mean_motion_ddot(&self) -> f64 {
        self.elements.mean_motion_ddot
    }

    /// BSTAR drag term (earth radii⁻¹).
    fn drag_term(&self) -> f64 {
        self.elements.drag_term
    }

    /// Element set number.
    fn element_set_number(&self) -> u64 {
        self.elements.element_set_number
    }

    /// Revolution number at epoch.
    fn revolution_number(&self) -> u64 {
        self.elements.revolution_number
    }

    /// Ephemeris type (always 0 in distributed data).
    fn ephemeris_type(&self) -> u8 {
        self.elements.ephemeris_type
    }

    fn __repr__(&self) -> String {
        let escaped = self.raw.replace('\\', "\\\\").replace('\n', "\\n");
        format!("TLE(\"{}\")", escaped)
    }

    fn __str__(&self) -> &str {
        &self.raw
    }

    fn __getnewargs__(&self) -> (String,) {
        (self.raw.clone(),)
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
///     tle: TLE object, string (2 or 3 lines), or list of 2–3 strings.
#[pyclass(name = "SGP4", module = "lox_space", frozen)]
#[derive(Clone)]
pub struct PySgp4 {
    pub inner: Sgp4,
    tle: PyTle,
}

#[pymethods]
impl PySgp4 {
    #[new]
    fn new(tle: &Bound<'_, PyAny>) -> PyResult<Self> {
        let tle = if let Ok(t) = tle.extract::<PyTle>() {
            t
        } else {
            PyTle::new(tle)?
        };
        let sgp4 = Sgp4::new(tle.elements.clone())
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(PySgp4 { inner: sgp4, tle })
    }

    /// Return the parsed TLE.
    fn tle(&self) -> PyTle {
        self.tle.clone()
    }

    /// Return the TLE epoch time.
    fn time(&self) -> PyTime {
        self.tle.epoch()
    }

    /// Propagate the orbit.
    ///
    /// Supports three calling modes:
    ///
    /// - Single time: ``propagate(time)`` → State
    /// - Two times: ``propagate(start, end)`` → Trajectory (propagator-chosen steps)
    /// - List of times: ``propagate([t1, t2, ...])`` → Trajectory (caller-chosen steps)
    ///
    /// Args:
    ///     steps: Single Time, list of Times, or start Time (when ``end`` is given).
    ///     end: End time (optional, for interval propagation).
    ///     frame: Target reference frame (optional).
    ///     provider: EOP provider (optional, for UT1 time conversions and frame transformation).
    ///
    /// Returns:
    ///     State or Trajectory, optionally transformed to the target frame.
    ///
    /// Raises:
    ///     ValueError: If propagation or frame transformation fails.
    #[pyo3(signature = (steps, end=None, frame=None, provider=None))]
    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
        end: Option<PyTime>,
        frame: Option<&Bound<'_, PyAny>>,
        provider: Option<&Bound<'_, PyEopProvider>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let frame = frame.map(PyFrame::try_from).transpose()?;
        let eop = provider.map(|p| &p.get().0);
        propagate_dispatch(
            py,
            steps,
            end,
            frame,
            provider,
            |t| {
                let tai = to_tai(t, eop)?;
                Ok(self.inner.state_at(tai).map_err(PySgp4Error)?.into_dyn())
            },
            |i| {
                let interval = Interval::new(to_tai(i.start(), eop)?, to_tai(i.end(), eop)?);
                Ok(self
                    .inner
                    .propagate(interval)
                    .map_err(PySgp4Error)?
                    .into_dyn())
            },
        )
    }

    fn __repr__(&self) -> String {
        let escaped = self.tle.raw.replace('\\', "\\\\").replace('\n', "\\n");
        format!("SGP4(\"{}\")", escaped)
    }
}
