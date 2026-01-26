// SPDX-FileCopyrightText: 2026 Halvor Granskogen Bjørnstad <halvor.bjornstad@ksat.no>
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
use crate::time::wasm::time::JsTime;
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

	#[wasm_bindgen(getter)]
	pub fn time(&self) -> JsTime {
		JsTime::from_inner(self.0.time())
	}

	#[wasm_bindgen(getter)]
	pub fn origin(&self) -> JsOrigin {
		JsOrigin::from_inner(self.0.origin())
	}

	#[wasm_bindgen(js_name = "referenceFrame", getter)]
	pub fn reference_frame(&self) -> JsFrame {
		JsFrame::from_inner(self.0.reference_frame())
	}

	pub fn position(&self) -> Array {
		array_from_vec3(self.0.position())
	}

	pub fn velocity(&self) -> Array {
		array_from_vec3(self.0.velocity())
	}

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

	#[wasm_bindgen(js_name="toGroundLocation")]
	pub fn to_ground_location(&self) -> Result<JsGroundLocation, JsValue> {
		Ok(JsGroundLocation(
			self.0
				.to_dyn_ground_location()
				.map_err(|err| js_error_with_name(err, "ValueError"))?,
		))
	}
}

/// Represents an orbit using Keplerian elements.
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

	#[wasm_bindgen(getter)]
	pub fn time(&self) -> JsTime {
		JsTime::from_inner(self.0.time())
	}

	#[wasm_bindgen(getter)]
	pub fn origin(&self) -> JsOrigin {
		JsOrigin::from_inner(self.0.origin())
	}

	#[wasm_bindgen(getter, js_name = "semiMajorAxis")]
	pub fn semi_major_axis(&self) -> f64 {
		self.0.semi_major_axis()
	}

	#[wasm_bindgen(getter)]
	pub fn eccentricity(&self) -> f64 {
		self.0.eccentricity()
	}

	#[wasm_bindgen(getter)]
	pub fn inclination(&self) -> f64 {
		self.0.inclination()
	}

	#[wasm_bindgen(getter, js_name = "longitudeOfAscendingNode")]
	pub fn longitude_of_ascending_node(&self) -> f64 {
		self.0.longitude_of_ascending_node()
	}

	#[wasm_bindgen(getter, js_name = "argumentOfPeriapsis")]
	pub fn argument_of_periapsis(&self) -> f64 {
		self.0.argument_of_periapsis()
	}

	#[wasm_bindgen(getter, js_name = "trueAnomaly")]
	pub fn true_anomaly(&self) -> f64 {
		self.0.true_anomaly()
	}

	#[wasm_bindgen(js_name = "toCartesian")]
	pub fn to_cartesian(&self) -> Result<JsState, JsValue> {
		Ok(JsState(self.0.to_cartesian()))
	}

	#[wasm_bindgen(js_name = "orbitalPeriod")]
	pub fn orbital_period(&self) -> JsTimeDelta {
		JsTimeDelta::from_inner(self.0.orbital_period())
	}
}

/// A time-series of orbital states with interpolation support.
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

	#[wasm_bindgen(getter)]
	pub fn origin(&self) -> JsOrigin {
		JsOrigin::from_inner(self.0.origin())
	}

	#[wasm_bindgen(js_name = "referenceFrame", getter)]
	pub fn reference_frame(&self) -> JsFrame {
		JsFrame::from_inner(self.0.reference_frame())
	}

	#[wasm_bindgen(js_name = "toArray")]
	pub fn to_array(&self) -> Result<Array, JsValue> {
		Ok(to_js_2d(&self.0.to_vec()))
	}

	#[wasm_bindgen]
	pub fn states(&self) -> Vec<JsState> {
		self.0.states().into_iter().map(JsState).collect()
	}

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

    #[wasm_bindgen(js_name = "interpolateDelta")]
    pub fn interpolate_delta(&self, delta: &JsTimeDelta) -> JsState {
        JsState(self.0.interpolate(delta.inner()))
    }

    #[wasm_bindgen(js_name = "interpolateAt")]
    pub fn interpolate_at(&self, time: &JsTime) -> JsState {
        JsState(self.0.interpolate_at(time.inner()))
    }

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
#[wasm_bindgen(js_name = "Event")]
#[derive(Clone, Debug)]
pub struct JsEvent(Event<DynTimeScale>);

#[wasm_bindgen(js_class = "Event")]
impl JsEvent {
	pub fn time(&self) -> JsTime {
		JsTime::from_inner(self.0.time())
	}

	pub fn crossing(&self) -> String {
		self.0.crossing().to_string()
	}

	#[wasm_bindgen(js_name = "toString")]
	pub fn to_string(&self) -> String {
		format!("Event - {}crossing at {}", self.crossing(), self.time().to_string())
	}
}

/// Represents a time window (interval between two times).
#[wasm_bindgen(js_name = "Window")]
#[derive(Clone, Debug)]
pub struct JsWindow(Window<DynTimeScale>);

#[wasm_bindgen(js_class = "Window")]
impl JsWindow {
	pub fn start(&self) -> JsTime {
		JsTime::from_inner(self.0.start())
	}

	pub fn end(&self) -> JsTime {
		JsTime::from_inner(self.0.end())
	}

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

	#[wasm_bindgen(js_name = "propagateAt")]
	pub fn propagate_at(&self, time: JsTime) -> Result<JsState, JsValue> {
		let state = self.0.propagate(time.inner()).map_err(JsValladoError)?;
		Ok(JsState(state))
	}


    pub fn propagate(&self, steps: Vec<JsTime>) -> Result<JsTrajectory, JsValue> {
        let times: Vec<DynTime> = steps.into_iter().map(|t| t.inner()).collect();
        let traj = self
            .0
            .propagate_all(times.into_iter())
            .map_err(JsValladoError)?;
        Ok(JsTrajectory(traj))
    }
}

/// Represents a location on the surface of a celestial body.
#[wasm_bindgen(js_name = "GroundLocation")]
#[derive(Clone, Debug)]
pub struct JsGroundLocation(DynGroundLocation);

#[wasm_bindgen(js_class = "GroundLocation")]
impl JsGroundLocation {
	#[wasm_bindgen(constructor)]
	pub fn new(origin: JsOrigin, longitude: f64, latitude: f64, altitude: f64) -> Result<Self, JsValue> {
		Ok(JsGroundLocation(
			DynGroundLocation::with_dynamic(longitude, latitude, altitude, origin.inner())
				.map_err(|e| js_error_with_name(e, "ValueError"))?,
		))
	}

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

	#[wasm_bindgen(js_name = "rotationToTopocentric")]
	pub fn rotation_to_topocentric(&self) -> Result<Array, JsValue> {
		let rot = self.0.rotation_to_topocentric();
		let rot: Vec<Vec<f64>> = rot.to_cols_array_2d().iter().map(|v| v.to_vec()).collect();
		Ok(to_js_2d(&rot))
	}

	#[wasm_bindgen(getter)]
	pub fn longitude(&self) -> f64 {
		self.0.longitude()
	}

	#[wasm_bindgen(getter)]
	pub fn latitude(&self) -> f64 {
		self.0.latitude()
	}

	#[wasm_bindgen(getter)]
	pub fn altitude(&self) -> f64 {
		self.0.altitude()
	}
}

/// Propagator for ground station positions.
#[wasm_bindgen(js_name = "GroundPropagator")]
pub struct JsGroundPropagator(DynGroundPropagator);

#[wasm_bindgen(js_class = "GroundPropagator")]
impl JsGroundPropagator {
	#[wasm_bindgen(constructor)]
	pub fn new(location: JsGroundLocation) -> Self {
		JsGroundPropagator(DynGroundPropagator::with_dynamic(location.0))
	}

	#[wasm_bindgen(js_name = "propagateAt")]
	pub fn propagate_at(&self, time: JsTime) -> Result<JsState, JsValue> {
		let state = self
				.0
				.propagate_dyn(time.inner())
				.map_err(JsGroundPropagatorError)?;
		Ok(JsState(state).into())
	}


    pub fn propagate(&self, times: Vec<JsTime>) -> Result<JsTrajectory, JsValue> {
        let mut states: Vec<DynState> = Vec::with_capacity(times.len());
        for t in times {
            let state = self
                .0
                .propagate_dyn(t.inner())
                .map_err(JsGroundPropagatorError)?;
            states.push(state);
        }
        let traj = Trajectory::new(&states).map_err(JsTrajectoryError)?;
        Ok(JsTrajectory(traj))
	}
}

/// SGP4 orbit propagator.
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

	#[wasm_bindgen(getter)]
	pub fn time(&self) -> JsTime {
		JsTime::from_inner(
			self.0
				.time()
				.try_to_scale(DynTimeScale::Tai, &DefaultOffsetProvider)
				.unwrap(),
		)
	}

	pub fn propagate_at(&self, jstime: JsTime, provider: Option<JsEopProvider>) -> Result<JsState, JsValue> {
		let provider = provider.map(|p| p.inner());
		let (time, dyntime) = match provider.as_ref() {
			Some(provider) => (
				jstime
					.inner()
					.try_to_scale(Tai, provider)
					.map_err(JsEopProviderError)?,
				jstime
					.inner()
					.try_to_scale(DynTimeScale::Tai, provider)
					.map_err(JsEopProviderError)?,
			),
			None => (jstime.inner().to_scale(Tai), jstime.inner().to_scale(DynTimeScale::Tai)),
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

	pub fn propagate(&self, times: Vec<JsTime>, provider: Option<JsEopProvider>) -> Result<JsTrajectory, JsValue> {
		let provider = provider.map(|p| p.inner());
		let mut states: Vec<DynState> = Vec::with_capacity(times.len() as usize);
		for step in times.iter() {
			let (time, dyntime) = match provider.as_ref() {
				Some(provider) => (
					step.inner().try_to_scale(Tai, provider).map_err(JsEopProviderError)?,
					step
						.inner()
						.try_to_scale(DynTimeScale::Tai, provider)
						.map_err(JsEopProviderError)?,
				),
				None => (step.inner().to_scale(Tai), step.inner().to_scale(DynTimeScale::Tai)),
			};
			let s = self.0.propagate(time).map_err(JsSgp4Error)?;
			let s = State::new(
				dyntime,
				s.position(),
				s.velocity(),
				DynOrigin::default(),
				DynFrame::default(),
			);
			states.push(s);
		}
		Ok(JsTrajectory(Trajectory::new(&states).map_err(JsTrajectoryError)?).into())
	}
}

/// Defines elevation constraints for visibility analysis.
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

	pub fn fixed(min_elevation: f64) -> Self {
		JsElevationMask(ElevationMask::with_fixed_elevation(min_elevation))
	}

	pub fn variable(azimuth: Vec<f64>, elevation: Vec<f64>) -> Result<Self, JsValue> {
		Ok(JsElevationMask(
			ElevationMask::new(azimuth, elevation).map_err(JsElevationMaskError)?,
		))
	}

	pub fn azimuth(&self) -> Option<Vec<f64>> {
		match &self.0 {
			ElevationMask::Fixed(_) => None,
			ElevationMask::Variable(series) => Some(series.x().to_vec()),
		}
	}

	pub fn elevation(&self) -> Option<Vec<f64>> {
		match &self.0 {
			ElevationMask::Fixed(_) => None,
			ElevationMask::Variable(series) => Some(series.y().to_vec()),
		}
	}

	#[wasm_bindgen(js_name = "fixedElevation")]
	pub fn fixed_elevation(&self) -> Option<f64> {
		match &self.0 {
			ElevationMask::Fixed(min_elevation) => Some(*min_elevation),
			ElevationMask::Variable(_) => None,
		}
	}

	#[wasm_bindgen(js_name = "minElevation")]
	pub fn min_elevation(&self, azimuth: f64) -> f64 {
		self.0.min_elevation(azimuth)
	}
}

/// Observation data from a ground station to a target.
#[wasm_bindgen(js_name = "Observables")]
#[derive(Clone, Debug)]
pub struct JsObservables(Observables);

#[wasm_bindgen(js_class = "Observables")]
impl JsObservables {
	#[wasm_bindgen(constructor)]
	pub fn new(azimuth: f64, elevation: f64, range: f64, range_rate: f64) -> Self {
		JsObservables(Observables::new(azimuth, elevation, range, range_rate))
	}

	#[wasm_bindgen(getter)]
	pub fn azimuth(&self) -> f64 {
		self.0.azimuth()
	}

	#[wasm_bindgen(getter)]
	pub fn elevation(&self) -> f64 {
		self.0.elevation()
	}

	#[wasm_bindgen(getter)]
	pub fn range(&self) -> f64 {
		self.0.range()
	}

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
#[wasm_bindgen(js_name = "Pass")]
#[derive(Debug, Clone)]
pub struct JsPass(DynPass);

#[wasm_bindgen(js_class = "Pass")]
impl JsPass {
	#[wasm_bindgen(constructor)]
	pub fn new(window: JsWindow, times: Vec<JsTime>, observables: Vec<JsObservables>) -> Result<Self, JsValue> {
		let times: Vec<DynTime> = times.into_iter().map(|t| t.inner()).collect();
		let observables: Vec<Observables> = observables.into_iter().map(|o| o.0).collect();

		let pass = Pass::new(window.0, times, observables)
			.map_err(|e| js_error_with_name(e, "ValueError"))?;

		Ok(JsPass(pass))
	}

	#[wasm_bindgen(getter)]
	pub fn window(&self) -> JsWindow {
		JsWindow::from_inner(*self.0.window())
	}

	#[wasm_bindgen(getter)]
	pub fn times(&self) -> Vec<JsTime> {
		self.0.times().iter().map(|&t| JsTime::from_inner(t)).collect()
	}

	#[wasm_bindgen(getter)]
	pub fn observables(&self) -> Vec<JsObservables> {
		self.0
			.observables()
			.iter()
			.map(|o| JsObservables::from_inner(o.clone()))
			.collect()
	}

	pub fn interpolate(&self, time: JsTime) -> Option<JsObservables> {
		self.0.interpolate(time.inner()).map(JsObservables)
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

/// Collection of named trajectories for batch visibility analysis.
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

/// Compute visibility passes between a ground station and spacecraft.
#[wasm_bindgen]
pub fn visibility(
	times: Vec<JsTime>,
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
	let times: Vec<DynTime> = times.into_iter().map(|s| s.inner()).collect();
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

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn location(&self) -> JsGroundLocation {
        self.location.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn mask(&self) -> JsElevationMask {
        self.mask.clone()
    }
}

/// Compute visibility for multiple spacecraft and ground stations.
#[wasm_bindgen(js_name = "visibilityAll")]
pub fn visibility_all(
    times: Vec<JsTime>,
    ground_stations: Vec<JsGroundStation>,
    spacecraft: &JsEnsemble,
    ephemeris: &JsSpk,
    bodies: Option<Vec<JsOrigin>>,
) -> Result<Object, JsValue> {
    let times: Vec<DynTime> = times.into_iter().map(|s| s.inner()).collect();
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
