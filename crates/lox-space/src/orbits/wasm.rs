// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
//                         2026 Hadrien Develay <hadrien.develay@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use glam::DVec3;
use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{prelude::*, JsCast};

use crate::bodies::wasm::{JsOrigin, JsUndefinedOriginPropertyError};
use crate::bodies::DynOrigin;
use crate::earth::wasm::ut1::{JsEopProvider, JsEopProviderError};
use crate::ephem::Ephemeris;
use crate::ephem::wasm::{JsDafSpkError, JsSpk};
use crate::frames::DynFrame;
use crate::frames::wasm::{JsDynRotationError, JsFrame};
use crate::math::roots::{Brent, RootFinderError};
use crate::orbits::analysis::{DynPass, ElevationMask, ElevationMaskError, Pass, VisibilityError, visibility_combined};
use crate::orbits::elements::{DynKeplerian, Keplerian};
use crate::orbits::events::{Event, FindEventError, Window};
use crate::orbits::ground::{DynGroundLocation, DynGroundPropagator, GroundPropagatorError, Observables};
use crate::orbits::propagators::Propagator;
use crate::orbits::propagators::semi_analytical::{DynVallado, Vallado, ValladoError};
use crate::orbits::propagators::sgp4::{Sgp4, Sgp4Error};
use crate::orbits::states::DynState;
use crate::orbits::trajectories::{DynTrajectory, Trajectory, TrajectoryError};
use crate::orbits::states::State;
use crate::time::DynTime;
use crate::time::deltas::TimeDelta;
use crate::time::offsets::DefaultOffsetProvider;
use crate::time::time_scales::{DynTimeScale, Tai};
use crate::time::wasm::deltas::JsTimeDelta;
use crate::time::wasm::time::{JsTime, JsTimes};
use crate::wasm::js_error_with_name;
use std::error::Error;
use std::fmt;
use lox_bodies::Origin;

use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use sgp4::Elements;

pub struct JsFindEventError(FindEventError);

impl From<JsFindEventError> for JsValue {
	fn from(err: JsFindEventError) -> Self {
		js_error_with_name(err.0, "FindEventError")
	}
}

pub struct JsRootFinderError(RootFinderError);

impl From<JsRootFinderError> for JsValue {
	fn from(err: JsRootFinderError) -> Self {
		js_error_with_name(err.0, "RootFinderError")
	}
}

pub struct JsVisibilityError(VisibilityError);

impl From<JsVisibilityError> for JsValue {
	fn from(err: JsVisibilityError) -> Self {
		js_error_with_name(err.0, "VisibilityError")
	}
}

pub struct JsTrajectoryError(TrajectoryError);

impl From<JsTrajectoryError> for JsValue {
	fn from(err: JsTrajectoryError) -> Self {
		js_error_with_name(err.0, "TrajectoryError")
	}
}

pub struct JsValladoError(ValladoError);

impl From<JsValladoError> for JsValue {
	fn from(err: JsValladoError) -> Self {
		js_error_with_name(err.0, "ValladoError")
	}
}

pub struct JsGroundPropagatorError(GroundPropagatorError);

impl From<JsGroundPropagatorError> for JsValue {
	fn from(err: JsGroundPropagatorError) -> Self {
		js_error_with_name(err.0, "GroundPropagatorError")
	}
}

pub struct JsSgp4Error(Sgp4Error);

impl From<JsSgp4Error> for JsValue {
	fn from(err: JsSgp4Error) -> Self {
		js_error_with_name(err.0, "Sgp4Error")
	}
}

pub struct JsElevationMaskError(ElevationMaskError);

impl From<JsElevationMaskError> for JsValue {
	fn from(err: JsElevationMaskError) -> Self {
		js_error_with_name(err.0, "ElevationMaskError")
	}
}

#[derive(Debug)]
struct CallbackError(String);

impl fmt::Display for CallbackError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl Error for CallbackError {}

fn callback_error_box(e: JsValue) -> Box<dyn Error + Send + Sync> {
	Box::new(CallbackError(
		e.as_string().unwrap_or_else(|| format!("{:?}", e)),
	))
}

fn callback_error_msg(msg: &str) -> Box<dyn Error + Send + Sync> {
	Box::new(CallbackError(msg.to_string()))
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
/// Raises:
///     FindEventError: If an error occurs during event finding.
#[wasm_bindgen(js_name="findEvents")]
pub fn find_events(func: &Function, start: JsTime, times: Vec<f64>) -> Result<Vec<JsEvent>, JsValue> {
	let root_finder = Brent::default();
	let events = crate::orbits::events::find_events(
		|t| {
			func.call1(&JsValue::NULL, &JsValue::from_f64(t))
				.map_err(callback_error_box)?
				.as_f64()
				.ok_or_else(|| callback_error_msg("callback must return number"))
		},
		start.inner(),
		&times,
		root_finder,
	)
	.map_err(JsFindEventError)?
	.into_iter()
	.map(JsEvent)
	.collect();
	Ok(events)
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
/// Raises:
///    RootFinderError: If an error occurs during root finding.
#[wasm_bindgen(js_name="findWindows")]
pub fn find_windows(func: &Function, start: JsTime, end: JsTime, times: Vec<f64>) -> Result<Vec<JsWindow>, JsValue> {
	let root_finder = Brent::default();
	let windows = crate::orbits::events::find_windows(
		|t| {
			func.call1(&JsValue::NULL, &JsValue::from_f64(t))
				.map_err(callback_error_box)?
				.as_f64()
				.ok_or_else(|| callback_error_msg("callback must return number"))
		},
		start.inner(),
		end.inner(),
		&times,
		root_finder,
	)
	.map_err(JsRootFinderError)?
	.into_iter()
	.map(JsWindow)
	.collect();
	Ok(windows)
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
#[wasm_bindgen(js_name = "State")]
#[derive(Clone, Debug)]
pub struct JsState(DynState);

fn vec3_from_slice(values: &[f64]) -> Result<DVec3, JsValue> {
	if values.len() != 3 {
		return Err(js_error_with_name("expected length 3 array", "ValueError"));
	}
	Ok(DVec3::new(values[0], values[1], values[2]))
}

fn array_from_vec3(v: DVec3) -> Array {
	Array::of3(
		&JsValue::from_f64(v.x),
		&JsValue::from_f64(v.y),
		&JsValue::from_f64(v.z),
	)
}

#[wasm_bindgen(js_class = "State")]
impl JsState {
	#[wasm_bindgen(constructor)]
	pub fn new(
		time: JsTime,
		position: Vec<f64>,
		velocity: Vec<f64>,
		origin: Option<JsOrigin>,
		frame: Option<JsFrame>,
	) -> Result<JsState, JsValue> {
		let origin = origin.map(|o| o.inner()).unwrap_or_else(DynOrigin::default);
		let frame = frame.map(|f| f.inner()).unwrap_or_else(DynFrame::default);
		let position = vec3_from_slice(&position)?;
		let velocity = vec3_from_slice(&velocity)?;
		Ok(JsState(State::new(time.inner(), position, velocity, origin, frame)))
	}

	/// Return the epoch of this state.
	#[wasm_bindgen(getter)]
	pub fn time(&self) -> JsTime {
		JsTime::from_inner(self.0.time())
	}

	/// Return the central body (origin) of this state.
	#[wasm_bindgen(getter)]
	pub fn origin(&self) -> JsOrigin {
		JsOrigin::from_inner(self.0.origin())
	}

	/// Return the reference frame of this state.
	#[wasm_bindgen(getter, js_name = "referenceFrame")]
	pub fn reference_frame(&self) -> JsFrame {
		JsFrame::from_inner(self.0.reference_frame())
	}

	/// Return the position vector as an array [x, y, z] in km.
	#[wasm_bindgen(getter)]
	pub fn position(&self) -> Array {
		array_from_vec3(self.0.position())
	}

	/// Return the velocity vector as an array [vx, vy, vz] in km/s.
	#[wasm_bindgen(getter)]
	pub fn velocity(&self) -> Array {
		array_from_vec3(self.0.velocity())
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
    ///     DynRotationError: If the transformation fails.
	#[wasm_bindgen(js_name="toFrame")]
	pub fn to_frame(&self, frame: JsFrame, provider: Option<JsEopProvider>) -> Result<JsState, JsValue> {
		let provider = provider.map(|p| p.inner());
		let origin = self.0.reference_frame();
		let target = frame.inner();
		let time = self.0.time();
		let rot = match provider.as_ref() {
			Some(provider) => provider
				.try_rotation(origin, target, time)
				.map_err(JsDynRotationError)?,
			None => DefaultRotationProvider
				.try_rotation(origin, target, time)
				.map_err(JsDynRotationError)?,
		};
		let (r1, v1) = rot.rotate_state(self.0.position(), self.0.velocity());
		Ok(JsState(State::new(
			self.0.time(),
			r1,
			v1,
			self.0.origin(),
			frame.inner(),
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
    ///     DafSPKError: If the transformation fails.
	#[wasm_bindgen(js_name="toOrigin")]
	pub fn to_origin(&self, target: JsOrigin, ephemeris: &JsSpk) -> Result<JsState, JsValue> {
		let frame = self.reference_frame();
		let s = if frame.inner() != DynFrame::Icrf {
			self.to_frame(JsFrame::from_inner(DynFrame::Icrf), None)?
		} else {
			self.clone()
		};
		let spk = ephemeris.inner();
		let mut s1 = JsState(
			s.0
				.to_origin_dynamic(target.inner(), spk)
				.map_err(JsDafSpkError)?,
		);
		if frame.inner() != DynFrame::Icrf {
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
	#[wasm_bindgen(js_name="toKeplerian")]
	pub fn to_keplerian(&self) -> Result<JsKeplerian, JsValue> {
		if self.0.reference_frame() != DynFrame::Icrf {
			return Err(js_error_with_name(
				"only inertial frames are supported for conversion to Keplerian elements",
				"ValueError",
			));
		}
		Ok(JsKeplerian(
			self.0
				.try_to_keplerian()
				.map_err(JsUndefinedOriginPropertyError)?,
		))
	}

	/// Compute the rotation matrix from inertial to LVLH (Local Vertical Local Horizontal) frame.
    ///
    /// Returns:
    ///     3x3 rotation matrix as a numpy array.
    ///
    /// Raises:
    ///     ValueError: If the state is not in an inertial frame.
	#[wasm_bindgen(js_name="rotationLvlh")]
	pub fn rotation_lvlh(&self) -> Result<Array, JsValue> {
		if self.0.reference_frame() != DynFrame::Icrf {
			return Err(js_error_with_name(
				"only inertial frames are supported for the LVLH rotation matrix",
				"ValueError",
			));
		}
		let rot = self.0.try_rotation_lvlh().map_err(|e| js_error_with_name(e, "ValueError"))?;
		let rot: Vec<Vec<f64>> = rot.to_cols_array_2d().iter().map(|v| v.to_vec()).collect();
		Ok(to_js_2d(&rot))
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
	#[wasm_bindgen(js_name="toGroundLocation")]
	pub fn to_ground_location(&self) -> Result<JsGroundLocation, JsValue> {
		Ok(JsGroundLocation(
			self.0
				.to_dyn_ground_location()
				.map_err(|err| js_error_with_name(err, "ValueError"))?,
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
#[wasm_bindgen(js_name = "Keplerian")]
pub struct JsKeplerian(DynKeplerian);

#[wasm_bindgen(js_class = "Keplerian")]
impl JsKeplerian {
	#[wasm_bindgen(constructor)]
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		time: JsTime,
		semi_major_axis: f64,
		eccentricity: f64,
		inclination: f64,
		longitude_of_ascending_node: f64,
		argument_of_periapsis: f64,
		true_anomaly: f64,
		origin: Option<JsOrigin>,
	) -> Result<JsKeplerian, JsValue> {
		let origin = origin.map(|o| o.inner()).unwrap_or_else(DynOrigin::default);
		Ok(JsKeplerian(
			Keplerian::with_dynamic(
				time.inner(),
				origin,
				semi_major_axis,
				eccentricity,
				inclination,
				longitude_of_ascending_node,
				argument_of_periapsis,
				true_anomaly,
			)
			.map_err(|err| js_error_with_name(err, "ValueError"))?,
		))
	}
	/// Return the epoch of these elements.
	#[wasm_bindgen(getter)]
	pub fn time(&self) -> JsTime {
		JsTime::from_inner(self.0.time())
	}

	/// Return the central body (origin) of this orbit.
	#[wasm_bindgen(getter)]
	pub fn origin(&self) -> JsOrigin {
		JsOrigin::from_inner(self.0.origin())
	}

	/// Return the semi-major axis in km.
	#[wasm_bindgen(getter, js_name = "semiMajorAxis")]
	pub fn semi_major_axis(&self) -> f64 {
		self.0.semi_major_axis()
	}

	/// Return the orbital eccentricity.
	#[wasm_bindgen(getter)]
	pub fn eccentricity(&self) -> f64 {
		self.0.eccentricity()
	}

	/// Return the inclination in radians.
	#[wasm_bindgen(getter)]
	pub fn inclination(&self) -> f64 {
		self.0.inclination()
	}

	/// Return the longitude of the ascending node (RAAN) in radians.
	#[wasm_bindgen(getter, js_name = "longitudeOfAscendingNode")]
	pub fn longitude_of_ascending_node(&self) -> f64 {
		self.0.longitude_of_ascending_node()
	}

	/// Return the argument of periapsis in radians.
	#[wasm_bindgen(getter, js_name = "argumentOfPeriapsis")]
	pub fn argument_of_periapsis(&self) -> f64 {
		self.0.argument_of_periapsis()
	}

	/// Return the true anomaly in radians.
	#[wasm_bindgen(getter, js_name = "trueAnomaly")]
	pub fn true_anomaly(&self) -> f64 {
		self.0.true_anomaly()
	}

	    /// Convert these Keplerian elements to a Cartesian state.
    ///
    /// Returns:
    ///     State with position and velocity vectors.
	#[wasm_bindgen(js_name = "toCartesian")]
	pub fn to_cartesian(&self) -> Result<JsState, JsValue> {
		Ok(JsState(self.0.to_cartesian()))
	}

	/// Return the orbital period.
    ///
    /// Returns:
    ///     TimeDelta representing one complete orbit.
	#[wasm_bindgen(js_name = "orbitalPeriod")]
	pub fn orbital_period(&self) -> JsTimeDelta {
		JsTimeDelta::from_inner(self.0.orbital_period())
	}
}

/// A time-series of orbital states with interpolation support.
///
/// Trajectories store a sequence of States and provide interpolation to
/// compute states at arbitrary times between the stored samples.
///
/// Args:
///     states: List of State objects in chronological order.
#[wasm_bindgen(js_name = "Trajectory")]
#[derive(Clone, Debug)]
pub struct JsTrajectory(DynTrajectory);

#[wasm_bindgen(js_class = "Trajectory")]
impl JsTrajectory {
	#[wasm_bindgen(constructor)]
	pub fn new(states: Vec<JsState>) -> Result<Self, JsValue> {
		let states: Vec<DynState> = states.into_iter().map(|s| s.0).collect();
		Ok(JsTrajectory(
			Trajectory::new(&states).map_err(JsTrajectoryError)?,
		))
	}

	/// Return the central body (origin) of this trajectory.
	#[wasm_bindgen(getter)]
	pub fn origin(&self) -> JsOrigin {
		JsOrigin::from_inner(self.0.origin())
	}

	/// Return the reference frame of this trajectory.
	#[wasm_bindgen(getter, js_name = "referenceFrame")]
	pub fn reference_frame(&self) -> JsFrame {
		JsFrame::from_inner(self.0.reference_frame())
	}

	/// Export trajectory to an array.
    ///
    /// Returns:
    ///     2D array with columns [t, x, y, z, vx, vy, vz].
	#[wasm_bindgen(js_name = "toArray")]
	pub fn to_array(&self) -> Result<Array, JsValue> {
		Ok(to_js_2d(&self.0.to_vec()))
	}

	/// Return the list of states in this trajectory.
	#[wasm_bindgen(getter)]
	pub fn states(&self) -> Vec<JsState> {
		self.0.states().into_iter().map(JsState).collect()
	}

	/// Find events where a function crosses zero.
    ///
    /// Args:
    ///     func: Function that takes a State and returns a float.
    ///           Events are detected where the function crosses zero.
    ///
    /// Returns:
    ///     List of Event objects.
	#[wasm_bindgen(js_name = "findEvents")]
	pub fn find_events(&self, func: &Function) -> Result<Vec<JsEvent>, JsValue> {
		let res = self.0.find_events(|s| {
			func.call1(&JsValue::NULL, &JsValue::from(JsState(s)))
				.map_err(callback_error_box)?
				.as_f64()
				.ok_or_else(|| callback_error_msg("callback must return number"))
		});
		let events = res.map_err(JsFindEventError)?;
		Ok(events.into_iter().map(JsEvent).collect())
	}

	/// Find time windows where a function is positive.
    ///
    /// Args:
    ///     func: Function that takes a State and returns a float.
    ///           Windows are periods where the function is positive.
    ///
    /// Returns:
    ///     List of Window objects.
	#[wasm_bindgen(js_name = "findWindows")]
	pub fn find_windows(&self, func: &Function) -> Result<Vec<JsWindow>, JsValue> {
		let res = self.0.find_windows(|s| {
			func.call1(&JsValue::NULL, &JsValue::from(JsState(s)))
				.map_err(callback_error_box)?
				.as_f64()
				.ok_or_else(|| callback_error_msg("callback must return number"))
		});
		let windows = res.map_err(JsRootFinderError)?;
		Ok(windows.into_iter().map(JsWindow).collect())
	}


    /// Interpolate the trajectory at a specific time.
    ///
    /// Args:
    ///     delta: TimeDelta relative to trajectory start
    ///
    /// Returns:
    ///     Interpolated State at the requested time.
    ///
    /// Raises:
    ///     ValueError: If the time argument is invalid.
    #[wasm_bindgen(js_name = "interpolateDelta")]
    pub fn interpolate_delta(&self, delta: &JsTimeDelta) -> JsState {
        JsState(self.0.interpolate(delta.inner()))
    }


    /// Interpolate the trajectory at a specific time.
    ///
    /// Args:
    ///     time: Time (absolute)
    ///
    /// Returns:
    ///     Interpolated State at the requested time.
    ///
    /// Raises:
    ///     ValueError: If the time argument is invalid.
    #[wasm_bindgen(js_name = "interpolateAt")]
    pub fn interpolate_at(&self, time: &JsTime) -> JsState {
        JsState(self.0.interpolate_at(time.inner()))
    }

	/// Transform all states in the trajectory to a different reference frame.
    ///
    /// Args:
    ///     frame: Target reference frame.
    ///     provider: EOP provider (required for ITRF transformations).
    ///
    /// Returns:
    ///     A new Trajectory in the target frame.
	#[wasm_bindgen(js_name = "toFrame")]
	pub fn to_frame(&self, frame: JsFrame, provider: Option<JsEopProvider>) -> Result<Self, JsValue> {
		let mut states: Vec<DynState> = Vec::with_capacity(self.0.states().len());
		for s in self.0.states() {
			states.push(JsState(s).to_frame(frame.clone(), provider.clone())?.0);
		}
		Ok(JsTrajectory(
			Trajectory::new(&states).map_err(JsTrajectoryError)?,
		))
	}

	/// Transform all states in the trajectory to a different central body.
    ///
    /// Args:
    ///     target: Target central body (origin).
    ///     ephemeris: SPK ephemeris data.
    ///
    /// Returns:
    ///     A new Trajectory relative to the target origin.
	#[wasm_bindgen(js_name = "toOrigin")]
	pub fn to_origin(&self, target: JsOrigin, ephemeris: &JsSpk) -> Result<Self, JsValue> {
		let mut states: Vec<JsState> = Vec::with_capacity(self.0.states().len());
		for s in self.0.states() {
			states.push(JsState(s).to_origin(target.clone(), ephemeris)?);
		}
		let states: Vec<DynState> = states.into_iter().map(|s| s.0).collect();
		Ok(Self(Trajectory::new(&states).map_err(JsTrajectoryError)?))
	}
}

/// Represents a detected event (zero-crossing of a function).
///
/// Events are detected when a monitored function crosses zero during
/// trajectory analysis. The crossing direction indicates whether the
/// function went from negative to positive ("up") or positive to negative ("down").
#[wasm_bindgen(js_name = "Event")]
#[derive(Clone, Debug)]
pub struct JsEvent(Event<DynTimeScale>);

#[wasm_bindgen(js_class = "Event")]
impl JsEvent {
	/// Return the time of this event.
	#[wasm_bindgen(getter)]
	pub fn time(&self) -> JsTime {
		JsTime::from_inner(self.0.time())
	}

	/// Return the crossing direction ("up" or "down").
	#[wasm_bindgen(getter)]
	pub fn crossing(&self) -> String {
		self.0.crossing().to_string()
	}

	#[wasm_bindgen(js_name = "toString")]
	pub fn to_string(&self) -> String {
		format!("Event - {}crossing at {}", self.crossing(), self.time().to_string())
	}

	fn debug(&self) -> String {
        format!("Event({}, {})", self.time().to_string(), self.crossing())
    }
}

/// Represents a time window (interval between two times).
///
/// Windows are used to represent periods when certain conditions are met,
/// such as visibility windows between a ground station and spacecraft.
#[wasm_bindgen(js_name = "Window")]
#[derive(Clone, Debug)]
pub struct JsWindow(Window<DynTimeScale>);

#[wasm_bindgen(js_class = "Window")]
impl JsWindow {
	fn debug(&self) -> String {
        format!(
            "Window({}, {})",
            self.start().to_string(),
            self.end().to_string()
        )
    }

	/// Return the start time of this window.
	#[wasm_bindgen(getter)]
	pub fn start(&self) -> JsTime {
		JsTime::from_inner(self.0.start())
	}

	/// Return the end time of this window.
	#[wasm_bindgen(getter)]
	pub fn end(&self) -> JsTime {
		JsTime::from_inner(self.0.end())
	}

	/// Return the duration of this window.
	#[wasm_bindgen(getter)]
	pub fn duration(&self) -> JsTimeDelta {
		JsTimeDelta::from_inner(self.0.duration())
	}
}

impl JsWindow {
	pub fn inner(&self) -> Window<DynTimeScale> {
		self.0.clone()
	}

	pub fn from_inner(window: Window<DynTimeScale>) -> Self {
		Self(window)
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
#[wasm_bindgen(js_name = "Vallado")]
#[derive(Clone, Debug)]
pub struct JsVallado(DynVallado);

#[wasm_bindgen(js_class = "Vallado")]
impl JsVallado {
	#[wasm_bindgen(constructor)]
	pub fn new(initial_state: JsState, max_iter: Option<i32>) -> Result<Self, JsValue> {
		let mut vallado = Vallado::with_dynamic(initial_state.0).map_err(|_| {
			js_error_with_name(
				"only inertial frames are supported for the Vallado propagator",
				"ValueError",
			)
		})?;
		if let Some(max_iter) = max_iter {
			vallado.with_max_iter(max_iter);
		}
		Ok(JsVallado(vallado))
	}

	/// Propagate the orbit to one or more times.
    ///
    /// Args:
    ///     time: Single Time
    ///
    /// Returns:
    ///     State
    ///
    /// Raises:
    ///     ValladoError: If propagation fails.
	#[wasm_bindgen(js_name = "propagateAt")]
	pub fn propagate_at(&self, time: JsTime) -> Result<JsState, JsValue> {
		let state = self.0.propagate(time.inner()).map_err(JsValladoError)?;
		Ok(JsState(state))
	}


	/// Propagate the orbit to one or more times.
    ///
    /// Args:
    ///     times: List of Times
    ///
    /// Returns:
    ///     Trajectory
    ///
    /// Raises:
    ///     ValladoError: If propagation fails.
    pub fn propagate(&self, times: &JsTimes,) -> Result<JsTrajectory, JsValue> {
        let times: Vec<DynTime> = times.vec_inner();
        let traj = self
            .0
            .propagate_all(times.into_iter())
            .map_err(JsValladoError)?;
        Ok(JsTrajectory(traj))
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
#[wasm_bindgen(js_name = "GroundLocation")]
#[derive(Clone, Debug)]
pub struct JsGroundLocation(DynGroundLocation);

#[wasm_bindgen(js_class = "GroundLocation")]
impl JsGroundLocation {
	#[wasm_bindgen(constructor)]
	pub fn new(origin: &JsOrigin, longitude: f64, latitude: f64, altitude: f64) -> Result<Self, JsValue> {
		let origin = origin.inner().clone();
		Ok(JsGroundLocation(
			DynGroundLocation::with_dynamic(longitude, latitude, altitude, origin)
				.map_err(|e| js_error_with_name(e, "ValueError"))?,
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
	pub fn observables(
		&self,
		state: JsState,
		provider: Option<JsEopProvider>,
		frame: Option<JsFrame>,
	) -> Result<JsObservables, JsValue> {
		let frame = frame.unwrap_or(JsFrame::from_inner(DynFrame::Iau(state.0.origin())));
		let state = state.to_frame(frame, provider)?;
		let rot = self.0.rotation_to_topocentric();
		let position = rot * (state.0.position() - self.0.body_fixed_position());
		let velocity = rot * state.0.velocity();
		let range = position.length();
		let range_rate = position.dot(velocity) / range;
		let elevation = (position.z / range).asin();
		let azimuth = position.y.atan2(-position.x);
		Ok(JsObservables(Observables::new(
			azimuth, elevation, range, range_rate,
		)))
	}

	/// Return the rotation matrix from body-fixed to topocentric frame.
    ///
    /// Returns:
    ///     3x3 rotation matrix as an array.
	#[wasm_bindgen(js_name = "rotationToTopocentric")]
	pub fn rotation_to_topocentric(&self) -> Result<Array, JsValue> {
		let rot = self.0.rotation_to_topocentric();
		let rot: Vec<Vec<f64>> = rot.to_cols_array_2d().iter().map(|v| v.to_vec()).collect();
		Ok(to_js_2d(&rot))
	}

	/// Return the geodetic longitude in radians.
	#[wasm_bindgen(getter)]
	pub fn longitude(&self) -> f64 {
		self.0.longitude()
	}

	/// Return the geodetic latitude in radians.
	#[wasm_bindgen(getter)]
	pub fn latitude(&self) -> f64 {
		self.0.latitude()
	}

	/// Return the altitude above the reference ellipsoid in km.
	#[wasm_bindgen(getter)]
	pub fn altitude(&self) -> f64 {
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
#[wasm_bindgen(js_name = "GroundPropagator")]
pub struct JsGroundPropagator(DynGroundPropagator);

#[wasm_bindgen(js_class = "GroundPropagator")]
impl JsGroundPropagator {
	#[wasm_bindgen(constructor)]
	pub fn new(location: JsGroundLocation) -> Self {
		JsGroundPropagator(DynGroundPropagator::with_dynamic(location.0))
	}

	/// Propagate the ground station to one or more times.
    ///
    /// Args:
    ///     time: a single Time
	///
    /// Returns:
    ///     State
    ///
    /// Raises:
    ///     GroundPropagationError: If propagation fails.
	#[wasm_bindgen(js_name = "propagateAt")]
	pub fn propagate_at(&self, time: JsTime) -> Result<JsState, JsValue> {
		let state = self
				.0
				.propagate_dyn(time.inner())
				.map_err(JsGroundPropagatorError)?;
		Ok(JsState(state).into())
	}

	/// Propagate the ground station to one or more times.
    ///
    /// Args:
    ///     times: list of times
    ///
    /// Returns:
    ///     State Trajectory
    ///
    /// Raises:
    ///     GroundPropagationError: If propagation fails.
    pub fn propagate(&self, times: &JsTimes,) -> Result<JsTrajectory, JsValue> {
		let times: Vec<DynTime> = times.vec_inner();
        let mut states: Vec<DynState> = Vec::with_capacity(times.len());
        for t in times {
            let state = self
                .0
                .propagate_dyn(t)
                .map_err(JsGroundPropagatorError)?;
            states.push(state);
        }
        let traj = Trajectory::new(&states).map_err(JsTrajectoryError)?;
        Ok(JsTrajectory(traj))
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
#[wasm_bindgen(js_name = "SGP4")]
pub struct JsSgp4(Sgp4);

#[wasm_bindgen(js_class = "SGP4")]
impl JsSgp4 {
	#[wasm_bindgen(constructor)]
	pub fn new(tle: &str) -> Result<Self, JsValue> {
		let lines: Vec<&str> = tle.trim().split('\n').collect();
		let elements = if lines.len() == 3 {
			Elements::from_tle(
				Some(lines[0].to_string()),
				lines[1].as_bytes(),
				lines[2].as_bytes(),
			)
			.map_err(|err| js_error_with_name(err, "ValueError"))?
		} else if lines.len() == 2 {
			Elements::from_tle(None, lines[0].as_bytes(), lines[1].as_bytes())
				.map_err(|err| js_error_with_name(err, "ValueError"))?
		} else {
			return Err(js_error_with_name("invalid TLE", "ValueError"));
		};
		Ok(JsSgp4(
			Sgp4::new(elements).map_err(|err| js_error_with_name(err, "ValueError"))?,
		))
	}

	/// Return the TLE epoch time.
	#[wasm_bindgen(getter)]
	pub fn time(&self) -> JsTime {
		JsTime::from_inner(
			self.0
				.time()
				.try_to_scale(DynTimeScale::Tai, &DefaultOffsetProvider)
				.unwrap(),
		)
	}

	/// Propagate the orbit to one or more times.
    ///
    /// Args:
    ///     time: Single Time or list of Times.
    ///     provider: EOP provider (optional, for UT1 time conversions).
    ///
    /// Returns:
    ///     State (if single time
    ///
    /// Raises:
    ///     Sgp4Error: If propagation fails.
	///
	/// XXX: This is not covered by tests and the provider will have problems when it is optional
	pub fn propagate_at(&self, time: JsTime, provider: Option<JsEopProvider>) -> Result<JsState, JsValue> {
		let provider = provider.map(|p| p.inner());
		let (time, dyntime) = match provider.as_ref() {
			Some(provider) => (
				time
					.inner()
					.try_to_scale(Tai, provider)
					.map_err(JsEopProviderError)?,
				time
					.inner()
					.try_to_scale(DynTimeScale::Tai, provider)
					.map_err(JsEopProviderError)?,
			),
			None => (time.inner().to_scale(Tai), time.inner().to_scale(DynTimeScale::Tai)),
		};
		let s1 = self.0.propagate(time).map_err(JsSgp4Error)?;
		Ok(JsState(State::new(
				dyntime,
				s1.position(),
				s1.velocity(),
				DynOrigin::default(),
				DynFrame::default(),
			))
			.into())
	}

	/// Propagate the orbit to one or more times.
    ///
    /// Args:
    ///     times: list of Times.
    ///     provider: EOP provider (optional, for UT1 time conversions).
    ///
    /// Returns:
    ///     Trajectory (if list of times).
    ///
    /// Raises:
    ///     Sgp4Error: If propagation fails.
	///
	/// XXX: This is not covered by tests and the provider will have problems when it is optional
	pub fn propagate(&self, times: &JsTimes, provider: Option<JsEopProvider>) -> Result<JsTrajectory, JsValue> {
		let provider = provider.map(|p| p.inner());
		let dyn_times: Vec<DynTime> = times.vec_inner();
		let mut states = Vec::with_capacity(dyn_times.len());

		for time in dyn_times {
			let (tai_time, dyn_time) = match provider.as_ref() {
				Some(p) => (
					time.try_to_scale(Tai, p).map_err(JsEopProviderError)?,
					time.try_to_scale(DynTimeScale::Tai, p).map_err(JsEopProviderError)?,
				),
				None => (
					time.to_scale(Tai),
					time.to_scale(DynTimeScale::Tai),
				),
			};

			let pv = self.0.propagate(tai_time).map_err(JsSgp4Error)?;
			states.push(State::new(dyn_time, pv.position(), pv.velocity(), DynOrigin::default(), DynFrame::default()));
		}

		Ok(JsTrajectory(Trajectory::new(&states).map_err(JsTrajectoryError)?))
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
///     VisibilityError: If visibility computation fails.
#[wasm_bindgen]
pub fn visibility(
	times: &JsTimes,
	gs: JsGroundLocation,
	mask: &JsElevationMask,
	sc: &JsTrajectory,
	ephemeris: &JsSpk,
	bodies: Option<Vec<JsOrigin>>,
) -> Result<Vec<JsPass>, JsValue> {
	if gs.0.origin().name() != sc.0.origin().name() {
		return Err(js_error_with_name(
			"ground station and spacecraft must have the same origin",
			"ValueError",
		));
	}
	let times: Vec<DynTime> = times.vec_inner();
	let mask = &mask.0;
	let ephemeris = ephemeris.inner();
	let bodies: Vec<DynOrigin> = bodies.unwrap_or_default().into_iter().map(|b| b.inner()).collect();
	Ok(
		visibility_combined(&times, &gs.0, mask, &bodies, &sc.0, ephemeris)
			.map_err(JsVisibilityError)?
			.into_iter()
			.map(JsPass)
			.collect(),
	)
}

/// Collection of named trajectories for batch visibility analysis.
///
/// Ensembles allow computing visibility for multiple spacecraft against
/// multiple ground stations efficiently using `visibility_all`.
///
/// Args:
///     names: spacecraft names
///     trajectories: spacecraft Trajectories
///
/// Note: The order must be the same for names and trajectories.
#[wasm_bindgen(js_name = "Ensemble")]
pub struct JsEnsemble(HashMap<String, DynTrajectory>);

#[wasm_bindgen(js_class = "Ensemble")]
impl JsEnsemble {
    #[wasm_bindgen(constructor)]
    pub fn new(names: Vec<String>, trajectories: Vec<JsTrajectory>) -> Result<Self, JsValue> {
        if names.len() != trajectories.len() {
            return Err(js_error_with_name("names and trajectories length mismatch", "TypeError"));
        }
        let mut map: HashMap<String, DynTrajectory> = HashMap::with_capacity(names.len());
        for (name, traj) in names.into_iter().zip(trajectories.into_iter()) {
            map.insert(name, traj.0);
        }
        Ok(Self(map))
    }
}

/// Compute visibility for multiple spacecraft and ground stations.
///
/// This function efficiently computes visibility passes for all combinations
/// of spacecraft and ground stations.
///
/// Args:
///     times: List of Time objects defining the analysis period.
///     ground_stations: list of GroundSTation
///     spacecraft: Ensemble of spacecraft trajectories.
///     ephemeris: SPK ephemeris data.
///     bodies: Optional list of bodies for occlusion testing.
///
/// Returns:
///     Nested object: {spacecraft_name: {station_name: [passes]}}.
#[wasm_bindgen(js_name = "visibilityAll")]
pub fn visibility_all(
    times: &JsTimes,
    ground_stations: Vec<JsGroundStation>,
    spacecraft: &JsEnsemble,
    ephemeris: &JsSpk,
    bodies: Option<Vec<JsOrigin>>,
) -> Result<Object, JsValue> {
    let times: Vec<DynTime> = times.vec_inner();
    let bodies: Vec<DynOrigin> = bodies.unwrap_or_default().into_iter().map(|b| b.inner()).collect();
    let spacecraft = &spacecraft.0;
    let ephemeris = ephemeris.inner();

    let mut gs_map: HashMap<String, (JsGroundLocation, JsElevationMask)> = HashMap::new();
    for gs in ground_stations {
        gs_map.insert(gs.name.clone(), (gs.location, gs.mask));
    }

    let result = Object::new();
    for (sc_name, sc_trajectory) in spacecraft {
        let inner = Object::new();
        for (gs_name, (gs_location, gs_mask)) in gs_map.iter() {
            let passes = visibility_combined(
                &times,
                &gs_location.0,
                &gs_mask.0,
                &bodies,
                sc_trajectory,
                ephemeris,
            )
            .map_err(JsVisibilityError)?
            .into_iter()
            .map(JsPass)
            .map(JsValue::from)
            .collect::<Array>();
            Reflect::set(&inner, &JsValue::from_str(gs_name), &passes)?;
        }
        Reflect::set(&result, &JsValue::from_str(sc_name), &inner)?;
    }

    Ok(result)
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
#[wasm_bindgen(js_name = "ElevationMask")]
#[derive(Debug, Clone, PartialEq)]
pub struct JsElevationMask(ElevationMask);

#[wasm_bindgen(js_class = "ElevationMask")]
impl JsElevationMask {
	#[wasm_bindgen(constructor)]
	pub fn new(azimuth: Option<Vec<f64>>, elevation: Option<Vec<f64>>, min_elevation: Option<f64>) -> Result<Self, JsValue> {
		if let Some(min_elevation) = min_elevation {
			return Ok(JsElevationMask(ElevationMask::with_fixed_elevation(min_elevation)));
		}
		if let (Some(az), Some(el)) = (azimuth, elevation) {
			return Ok(JsElevationMask(
				ElevationMask::new(az, el).map_err(JsElevationMaskError)?,
			));
		}
		Err(js_error_with_name(
			"invalid argument combination, either `min_elevation` or `azimuth` and `elevation` arrays need to be present",
			"ValueError",
		))
	}

	/// Create a fixed elevation mask with constant minimum elevation.
    ///
    /// Args:
    ///     min_elevation: Minimum elevation angle in radians.
    ///
    /// Returns:
    ///     ElevationMask with fixed minimum elevation.
	pub fn fixed(min_elevation: f64) -> Self {
		JsElevationMask(ElevationMask::with_fixed_elevation(min_elevation))
	}

	/// Create a variable elevation mask from azimuth-dependent data.
    ///
    /// Args:
    ///     azimuth: Array of azimuth angles in radians.
    ///     elevation: Array of minimum elevations in radians.
    ///
    /// Returns:
    ///     ElevationMask with variable minimum elevation.
	pub fn variable(azimuth: Vec<f64>, elevation: Vec<f64>) -> Result<Self, JsValue> {
		Ok(JsElevationMask(
			ElevationMask::new(azimuth, elevation).map_err(JsElevationMaskError)?,
		))
	}

	/// Return the azimuth array (for variable masks only).
	pub fn azimuth(&self) -> Option<Vec<f64>> {
		match &self.0 {
			ElevationMask::Fixed(_) => None,
			ElevationMask::Variable(series) => Some(series.x().to_vec()),
		}
	}

	/// Return the elevation array (for variable masks only).
	pub fn elevation(&self) -> Option<Vec<f64>> {
		match &self.0 {
			ElevationMask::Fixed(_) => None,
			ElevationMask::Variable(series) => Some(series.y().to_vec()),
		}
	}

	/// Return the fixed elevation value (for fixed masks only).
	#[wasm_bindgen(js_name = "fixedElevation")]
	pub fn fixed_elevation(&self) -> Option<f64> {
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
	#[wasm_bindgen(js_name = "minElevation")]
	pub fn min_elevation(&self, azimuth: f64) -> f64 {
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
#[wasm_bindgen(js_name = "Observables")]
#[derive(Clone, Debug)]
pub struct JsObservables(Observables);

#[wasm_bindgen(js_class = "Observables")]
impl JsObservables {
	#[wasm_bindgen(constructor)]
	pub fn new(azimuth: f64, elevation: f64, range: f64, range_rate: f64) -> Self {
		JsObservables(Observables::new(azimuth, elevation, range, range_rate))
	}

	/// Return the azimuth angle in radians.
	#[wasm_bindgen(getter)]
	pub fn azimuth(&self) -> f64 {
		self.0.azimuth()
	}

	/// Return the elevation angle in radians.
	#[wasm_bindgen(getter)]
	pub fn elevation(&self) -> f64 {
		self.0.elevation()
	}

	/// Return the range (distance) in km.
	#[wasm_bindgen(getter)]
	pub fn range(&self) -> f64 {
		self.0.range()
	}

	/// Return the range rate in km/s.
	#[wasm_bindgen(getter, js_name = "rangeRate")]
	pub fn range_rate(&self) -> f64 {
		self.0.range_rate()
	}
}

impl JsObservables {
    pub fn inner(&self) -> Observables {
        self.0.clone()
    }

    pub fn from_inner(provider: Observables) -> Self {
        Self(provider)
    }
}

/// Represents a visibility pass between a ground station and spacecraft.
///
/// A Pass contains the visibility window (start and end times) along with
/// observables computed at regular intervals throughout the pass.
#[wasm_bindgen(js_name = "Pass")]
#[derive(Debug, Clone)]
pub struct JsPass(DynPass);

#[wasm_bindgen(js_class = "Pass")]
impl JsPass {
	#[wasm_bindgen(constructor)]
	pub fn new(window: JsWindow, times: &JsTimes, observables: Vec<JsObservables>) -> Result<Self, JsValue> {
		let times: Vec<DynTime> = times.vec_inner();
		let observables: Vec<Observables> = observables.into_iter().map(|o| o.0).collect();

		let pass = Pass::new(window.0, times, observables)
			.map_err(|e| js_error_with_name(e, "ValueError"))?;

		Ok(JsPass(pass))
	}

	/// Return the visibility window for this pass.
	#[wasm_bindgen(getter)]
	pub fn window(&self) -> JsWindow {
		JsWindow::from_inner(*self.0.window())
	}

	/// Return the time samples during this pass.
	#[wasm_bindgen(getter)]
	pub fn times(&self) -> JsTimes {
		JsTimes::from_inner(self.0.times().iter().map(|&t| JsTime::from_inner(t)).collect())
	}

	/// Return the observables at each time sample.
	#[wasm_bindgen(getter)]
	pub fn observables(&self) -> Vec<JsObservables> {
		self.0
			.observables()
			.iter()
			.map(|o| JsObservables::from_inner(o.clone()))
			.collect()
	}


    /// Interpolate observables at a specific time within the pass.
    ///
    /// Args:
    ///     time: Time to interpolate at.
    ///
    /// Returns:
    ///     Interpolated Observables, or None if time is outside the pass.
	pub fn interpolate(&self, time: JsTime) -> Option<JsObservables> {
		self.0.interpolate(time.inner()).map(JsObservables)
	}

	fn debug(&self) -> String {
        let window = self.0.window();
        format!(
            "Pass(window=Window({}, {}), {} observables)",
            window.start(),
            window.end(),
            self.0.observables().len()
        )
    }
}

fn to_js_2d<T: AsRef<[f64]>>(values: &[T]) -> Array {
	let outer = Array::new();
	for row in values {
		let row = row.as_ref();
		let arr = Array::new();
		for v in row {
			arr.push(&JsValue::from_f64(*v));
		}
		outer.push(&arr);
	}
	outer
}

#[wasm_bindgen(js_name = "GroundStation")]
#[derive(Clone, Debug)]
pub struct JsGroundStation {
    name: String,
    location: JsGroundLocation,
    mask: JsElevationMask,
}

#[wasm_bindgen(js_class = "GroundStation")]
impl JsGroundStation {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, location: JsGroundLocation, mask: JsElevationMask) -> Self {
        Self { name, location, mask }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn location(&self) -> JsGroundLocation {
        self.location.clone()
    }

    pub fn mask(&self) -> JsElevationMask {
        self.mask.clone()
    }
}
