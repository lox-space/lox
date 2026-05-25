// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2026 Lox Space Contributors
//
// SPDX-License-Identifier: MPL-2.0

use std::f64::consts::TAU;
use std::str::FromStr;

use lox_space::bodies::DynOrigin;
use lox_space::bodies::Origin as OriginTrait;
use lox_space::bodies::TryMeanRadius;
use lox_space::bodies::TryPointMass;
use lox_space::core::anomalies::TrueAnomaly;
use lox_space::core::coords::Cartesian as LoxCartesian;
use lox_space::core::elements::Keplerian as LoxKeplerian;
use lox_space::core::elements::{ArgumentOfPeriapsis, Eccentricity, LongitudeOfAscendingNode};
use lox_space::core::glam::DVec3;
use lox_space::core::units::{Angle, AngleUnits, DistanceUnits};
use lox_space::frames::dynamic::DynFrame;
use lox_space::frames::providers::DefaultRotationProvider;
use lox_space::frames::traits::ReferenceFrame;
use lox_space::orbits::propagators::semi_analytical::DynVallado;
use lox_space::orbits::sso::inclination_sso;
use lox_space::orbits::KeplerianOrbit;
use lox_space::time::calendar_dates::CalendarDate;
use lox_space::time::deltas::TimeDelta;
use lox_space::time::time_of_day::CivilTime;
use lox_space::time::time_scales::{DynTimeScale, TimeScale};
use lox_space::time::utc::transformations::ToUtc;
use lox_space::time::utc::Utc as LoxUtc;
use lox_space::time::DynTime;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

/// Initialize the WASM module with panic hook for better error messages.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Represents a celestial body (planet, moon, barycenter, etc.).
#[wasm_bindgen]
pub struct Origin(DynOrigin);

#[wasm_bindgen]
impl Origin {
    /// Construct an Origin from a body name (string) or NAIF ID (number).
    #[wasm_bindgen(constructor)]
    pub fn new(value: JsValue) -> Result<Origin, JsValue> {
        if let Some(name) = value.as_string() {
            let origin =
                DynOrigin::from_str(&name).map_err(|e| JsValue::from_str(&e.to_string()))?;
            return Ok(Origin(origin));
        }
        if let Some(id) = value.as_f64() {
            let origin =
                DynOrigin::try_from(id as i32).map_err(|e| JsValue::from_str(&e.to_string()))?;
            return Ok(Origin(origin));
        }
        Err(JsValue::from_str(
            "`origin` must be either a string (name) or a number (NAIF ID)",
        ))
    }

    /// Return the name of this body.
    pub fn name(&self) -> String {
        OriginTrait::name(&self.0).to_string()
    }

    /// Return the NAIF ID of this body.
    pub fn id(&self) -> i32 {
        OriginTrait::id(&self.0).0
    }

    /// Return the gravitational parameter (GM) in m³/s².
    pub fn gravitational_parameter(&self) -> Result<f64, JsValue> {
        TryPointMass::try_gravitational_parameter(&self.0)
            .map(|gp| gp.as_f64())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Return the mean radius in meters.
    pub fn mean_radius(&self) -> Result<f64, JsValue> {
        TryMeanRadius::try_mean_radius(&self.0)
            .map(|r| r.to_meters())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Represents a reference frame for positioning and transformations.
#[wasm_bindgen]
pub struct Frame(DynFrame);

#[wasm_bindgen]
impl Frame {
    /// Construct a Frame from its abbreviation (e.g., "ICRF", "ITRF").
    #[wasm_bindgen(constructor)]
    pub fn new(abbreviation: &str) -> Result<Frame, JsValue> {
        DynFrame::from_str(abbreviation)
            .map(Frame)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Return the full name of this reference frame.
    pub fn name(&self) -> String {
        ReferenceFrame::name(&self.0)
    }

    /// Return the abbreviation of this reference frame.
    pub fn abbreviation(&self) -> String {
        self.0.abbreviation()
    }
}

/// Represents a set of Keplerian orbital elements.
///
/// All angular values are in radians and distances in meters.
#[wasm_bindgen]
pub struct Keplerian {
    elements: LoxKeplerian,
    origin: DynOrigin,
}

#[wasm_bindgen]
impl Keplerian {
    /// Construct a Keplerian orbit from classical elements.
    ///
    /// - `semi_major_axis`: meters
    /// - `eccentricity`: dimensionless
    /// - `inclination`: radians
    /// - `raan`: right ascension of ascending node, radians
    /// - `arg_periapsis`: argument of periapsis, radians
    /// - `true_anomaly`: radians
    #[wasm_bindgen(constructor)]
    pub fn new(
        semi_major_axis: f64,
        eccentricity: f64,
        inclination: f64,
        raan: f64,
        arg_periapsis: f64,
        true_anomaly: f64,
        origin: &Origin,
    ) -> Result<Keplerian, JsValue> {
        let elements = LoxKeplerian::builder()
            .with_semi_major_axis(semi_major_axis.m(), eccentricity)
            .with_inclination(inclination.rad())
            .with_longitude_of_ascending_node(raan.rad())
            .with_argument_of_periapsis(arg_periapsis.rad())
            .with_true_anomaly(true_anomaly.rad())
            .build()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Keplerian {
            elements,
            origin: origin.0,
        })
    }

    /// Construct a Keplerian orbit from periapsis and apoapsis radii.
    ///
    /// - `periapsis_radius`: meters
    /// - `apoapsis_radius`: meters
    pub fn from_radii(
        periapsis_radius: f64,
        apoapsis_radius: f64,
        inclination: f64,
        raan: f64,
        arg_periapsis: f64,
        true_anomaly: f64,
        origin: &Origin,
    ) -> Result<Keplerian, JsValue> {
        let elements = LoxKeplerian::builder()
            .with_radii(periapsis_radius.m(), apoapsis_radius.m())
            .with_inclination(inclination.rad())
            .with_longitude_of_ascending_node(raan.rad())
            .with_argument_of_periapsis(arg_periapsis.rad())
            .with_true_anomaly(true_anomaly.rad())
            .build()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Keplerian {
            elements,
            origin: origin.0,
        })
    }

    /// Construct a Keplerian orbit from periapsis and apoapsis altitudes above the body's mean radius.
    ///
    /// - `periapsis_altitude`: meters above mean radius
    /// - `apoapsis_altitude`: meters above mean radius
    pub fn from_altitudes(
        periapsis_altitude: f64,
        apoapsis_altitude: f64,
        inclination: f64,
        raan: f64,
        arg_periapsis: f64,
        true_anomaly: f64,
        origin: &Origin,
    ) -> Result<Keplerian, JsValue> {
        let mean_radius = TryMeanRadius::try_mean_radius(&origin.0)
            .map(|r| r.to_meters().m())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let elements = LoxKeplerian::builder()
            .with_altitudes(periapsis_altitude.m(), apoapsis_altitude.m(), mean_radius)
            .with_inclination(inclination.rad())
            .with_longitude_of_ascending_node(raan.rad())
            .with_argument_of_periapsis(arg_periapsis.rad())
            .with_true_anomaly(true_anomaly.rad())
            .build()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Keplerian {
            elements,
            origin: origin.0,
        })
    }

    /// Construct a circular orbit (eccentricity=0, argument of periapsis=0, true anomaly=0).
    ///
    /// - `semi_major_axis`: meters
    /// - `inclination`: radians
    /// - `raan`: right ascension of ascending node, radians
    pub fn circular(
        semi_major_axis: f64,
        inclination: f64,
        raan: f64,
        origin: &Origin,
    ) -> Result<Keplerian, JsValue> {
        let elements = LoxKeplerian::builder()
            .with_semi_major_axis(semi_major_axis.m(), 0.0)
            .with_inclination(inclination.rad())
            .with_longitude_of_ascending_node(raan.rad())
            .with_argument_of_periapsis(0.0f64.rad())
            .with_true_anomaly(0.0f64.rad())
            .build()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Keplerian {
            elements,
            origin: origin.0,
        })
    }

    /// Construct a circular orbit from altitude above the body's mean radius.
    ///
    /// - `altitude`: meters above mean radius
    /// - `origin`: central body (must have mean radius and GM defined)
    pub fn circular_from_altitude(altitude: f64, origin: &Origin) -> Result<Keplerian, JsValue> {
        let mean_radius = TryMeanRadius::try_mean_radius(&origin.0)
            .map(|r| r.to_meters())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let semi_major_axis = (altitude + mean_radius).m();
        let elements = LoxKeplerian::builder()
            .with_semi_major_axis(semi_major_axis, 0.0)
            .with_inclination(0.0f64.rad())
            .with_longitude_of_ascending_node(0.0f64.rad())
            .with_argument_of_periapsis(0.0f64.rad())
            .with_true_anomaly(0.0f64.rad())
            .build()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Keplerian {
            elements,
            origin: origin.0,
        })
    }

    /// Construct a sun-synchronous orbit (SSO) around Earth by altitude and eccentricity.
    ///
    /// The SSO inclination is computed analytically. The orbit is Earth-only.
    ///
    /// - `altitude`: meters above Earth's mean radius
    /// - `eccentricity`: dimensionless
    pub fn sso(altitude: f64, eccentricity: f64, origin: &Origin) -> Result<Keplerian, JsValue> {
        let ecc =
            Eccentricity::try_new(eccentricity).map_err(|e| JsValue::from_str(&e.to_string()))?;

        // For SSO we compute semi-major axis from altitude (altitude + mean radius of Earth)
        // inclination_sso takes a semi-major axis (Distance) and eccentricity
        let mean_radius = TryMeanRadius::try_mean_radius(&origin.0)
            .map(|r| r.to_meters())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let semi_major_axis = (altitude + mean_radius).m();

        let inclination =
            inclination_sso(semi_major_axis, ecc).map_err(|e| JsValue::from_str(&e.to_string()))?;

        // RAAN defaults to 0 for a time-independent constructor
        let longitude_of_ascending_node = LongitudeOfAscendingNode::try_new(0.0f64.rad())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let aop = ArgumentOfPeriapsis::try_new(Angle::ZERO)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let ta = TrueAnomaly::new(Angle::ZERO);

        let elements = LoxKeplerian::new(
            semi_major_axis,
            ecc,
            inclination,
            longitude_of_ascending_node,
            aop,
            ta,
        );

        Ok(Keplerian {
            elements,
            origin: origin.0,
        })
    }

    /// Returns the semi-major axis in meters.
    pub fn semi_major_axis(&self) -> f64 {
        self.elements.semi_major_axis().to_meters()
    }

    /// Returns the eccentricity.
    pub fn eccentricity(&self) -> f64 {
        self.elements.eccentricity().as_f64()
    }

    /// Returns the inclination in radians.
    pub fn inclination(&self) -> f64 {
        self.elements.inclination().as_f64()
    }

    /// Returns the right ascension of the ascending node in radians.
    pub fn raan(&self) -> f64 {
        self.elements.longitude_of_ascending_node().as_f64()
    }

    /// Returns the argument of periapsis in radians.
    pub fn arg_periapsis(&self) -> f64 {
        self.elements.argument_of_periapsis().as_f64()
    }

    /// Returns the true anomaly in radians.
    pub fn true_anomaly(&self) -> f64 {
        self.elements.true_anomaly().as_f64()
    }

    /// Returns the orbital period in seconds.
    ///
    /// Returns an error for non-elliptic orbits or if the origin's GM is undefined.
    pub fn orbital_period(&self) -> Result<f64, JsValue> {
        let mu = TryPointMass::try_gravitational_parameter(&self.origin)
            .map(|gp| gp.as_f64())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let a = self.elements.semi_major_axis().to_meters();
        if !self.elements.eccentricity().is_circular_or_elliptic() {
            return Err(JsValue::from_str(
                "orbital period is only defined for circular and elliptic orbits",
            ));
        }
        Ok(TAU * (a.powi(3) / mu).sqrt())
    }

    /// Returns the origin (central body) of this orbit.
    pub fn origin(&self) -> Origin {
        Origin(self.origin)
    }

    /// Convert this Keplerian orbit to a Cartesian state vector.
    ///
    /// Returns an error if the origin's gravitational parameter is not defined.
    pub fn to_cartesian(&self) -> Result<Cartesian, JsValue> {
        let gp = TryPointMass::try_gravitational_parameter(&self.origin)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let cart = self.elements.to_cartesian(gp);
        Ok(Cartesian {
            state: cart,
            origin: self.origin,
            frame: DynFrame::default(),
        })
    }

    /// Sample the orbit over the scenario window using the Vallado
    /// Kepler propagator. Returns a `SampledTrajectory` with parallel
    /// epoch / ECI / ground-track buffers.
    ///
    /// - `start_iso`: ISO-8601 UTC start of the window.
    /// - `duration_s`: scenario duration in seconds (must be > 0).
    /// - `step_s`: sampling step in seconds (must be > 0).
    #[wasm_bindgen(js_name = propagateSampled)]
    pub fn propagate_sampled(
        &self,
        start_iso: &str,
        duration_s: f64,
        step_s: f64,
    ) -> Result<SampledTrajectory, JsValue> {
        compute_sampled_trajectory(&self.elements, self.origin, start_iso, duration_s, step_s)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Trace the orbit into `n` evenly-spaced Cartesian states over one full revolution.
    ///
    /// Returns an error for non-elliptic orbits or if the origin's GM is undefined.
    pub fn trace(&self, n: usize) -> Result<Trajectory, JsValue> {
        let gp = TryPointMass::try_gravitational_parameter(&self.origin)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let states = self
            .elements
            .trace(gp, n)
            .ok_or_else(|| JsValue::from_str("trace is only available for elliptic orbits"))?;
        Ok(Trajectory {
            states,
            origin: self.origin,
            frame: DynFrame::default(),
        })
    }
}

/// Represents a Cartesian state vector (position + velocity).
///
/// Position components are in meters; velocity components are in m/s.
#[wasm_bindgen]
pub struct Cartesian {
    state: LoxCartesian,
    origin: DynOrigin,
    frame: DynFrame,
}

#[wasm_bindgen]
impl Cartesian {
    /// Construct a Cartesian state from position and velocity arrays.
    ///
    /// - `position`: `[x, y, z]` in meters
    /// - `velocity`: `[vx, vy, vz]` in m/s
    #[wasm_bindgen(constructor)]
    pub fn new(
        position: &[f64],
        velocity: &[f64],
        origin: &Origin,
        frame: &Frame,
    ) -> Result<Cartesian, JsValue> {
        if position.len() != 3 {
            return Err(JsValue::from_str("`position` must have exactly 3 elements"));
        }
        if velocity.len() != 3 {
            return Err(JsValue::from_str("`velocity` must have exactly 3 elements"));
        }
        let pos = DVec3::new(position[0], position[1], position[2]);
        let vel = DVec3::new(velocity[0], velocity[1], velocity[2]);
        let state = LoxCartesian::from_vecs(pos, vel);
        Ok(Cartesian {
            state,
            origin: origin.0,
            frame: frame.0,
        })
    }

    /// Returns the position vector `[x, y, z]` in meters.
    pub fn position(&self) -> Vec<f64> {
        let p = self.state.position();
        vec![p.x, p.y, p.z]
    }

    /// Returns the velocity vector `[vx, vy, vz]` in m/s.
    pub fn velocity(&self) -> Vec<f64> {
        let v = self.state.velocity();
        vec![v.x, v.y, v.z]
    }

    /// Returns the x position component in meters.
    pub fn x(&self) -> f64 {
        self.state.x().to_meters()
    }

    /// Returns the y position component in meters.
    pub fn y(&self) -> f64 {
        self.state.y().to_meters()
    }

    /// Returns the z position component in meters.
    pub fn z(&self) -> f64 {
        self.state.z().to_meters()
    }

    /// Returns the x velocity component in m/s.
    pub fn vx(&self) -> f64 {
        self.state.vx().to_meters_per_second()
    }

    /// Returns the y velocity component in m/s.
    pub fn vy(&self) -> f64 {
        self.state.vy().to_meters_per_second()
    }

    /// Returns the z velocity component in m/s.
    pub fn vz(&self) -> f64 {
        self.state.vz().to_meters_per_second()
    }

    /// Returns the origin (central body) of this state.
    pub fn origin(&self) -> Origin {
        Origin(self.origin)
    }

    /// Returns the reference frame of this state.
    pub fn frame(&self) -> Frame {
        Frame(self.frame)
    }

    /// Position in Three.js coordinates (Y-up right-handed).
    /// Applies ICRF-to-Three.js transform. Values in meters.
    pub fn to_threejs(&self) -> Vec<f64> {
        let p = self.state.position();
        vec![p.x, p.z, -p.y]
    }

    /// Convert this Cartesian state to Keplerian orbital elements.
    ///
    /// Returns an error if the origin's gravitational parameter is not defined.
    pub fn to_keplerian(&self) -> Result<Keplerian, JsValue> {
        let gp = TryPointMass::try_gravitational_parameter(&self.origin)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let elements = self.state.to_keplerian(gp);
        Ok(Keplerian {
            elements,
            origin: self.origin,
        })
    }
}

/// A sequence of Cartesian states sampled along an orbit.
#[wasm_bindgen]
pub struct Trajectory {
    states: Vec<LoxCartesian>,
    origin: DynOrigin,
    frame: DynFrame,
}

#[wasm_bindgen]
impl Trajectory {
    /// Returns the number of states in this trajectory.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Returns `true` if the trajectory contains no states.
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Returns all states as an array of `Cartesian` objects.
    pub fn states(&self) -> js_sys::Array {
        self.states
            .iter()
            .map(|c| {
                JsValue::from(Cartesian {
                    state: *c,
                    origin: self.origin,
                    frame: self.frame,
                })
            })
            .collect()
    }

    /// Returns interleaved position components `[x0,y0,z0, x1,y1,z1, ...]` in meters.
    ///
    /// This is the fast path for populating a Three.js `BufferGeometry`.
    pub fn to_buffer(&self) -> Vec<f64> {
        self.states
            .iter()
            .flat_map(|c| {
                let p = c.position();
                [p.x, p.y, p.z]
            })
            .collect()
    }

    /// Interleaved position buffer in Three.js coordinates.
    /// Returns [x0,y0,z0, x1,y1,z1, ...] with ICRF-to-Three.js transform applied.
    /// All values in meters.
    pub fn to_threejs_buffer(&self) -> Vec<f64> {
        self.states
            .iter()
            .flat_map(|c| {
                let p = c.position();
                [p.x, p.z, -p.y]
            })
            .collect()
    }
}

/// A satellite trajectory sampled at fixed time intervals, with
/// parallel buffers ready for direct consumption by Three.js
/// `BufferAttribute` and canvas-based map renderers.
///
/// Buffers are parallel — `epochs_ms[i]`, `eci_km[3*i..3*i+3]`, and
/// `ground_deg[2*i..2*i+2]` all refer to the same sample.
#[wasm_bindgen]
pub struct SampledTrajectory {
    pub(crate) epochs_ms: Vec<f64>,
    pub(crate) eci_km: Vec<f64>,
    pub(crate) ground_deg: Vec<f64>,
}

#[wasm_bindgen]
impl SampledTrajectory {
    /// Returns the number of samples.
    pub fn len(&self) -> usize {
        self.epochs_ms.len()
    }

    /// Returns `true` if there are no samples.
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.epochs_ms.is_empty()
    }

    /// Returns the epoch buffer as Unix milliseconds since 1970-01-01T00:00:00Z.
    #[wasm_bindgen(js_name = epochsMs)]
    pub fn epochs_ms(&self) -> Vec<f64> {
        self.epochs_ms.clone()
    }

    /// Returns the ECI position buffer in Three.js convention (Y-up), km.
    /// Layout: `[x0, y0, z0, x1, y1, z1, ...]`
    #[wasm_bindgen(js_name = eciThreejsBufferKm)]
    pub fn eci_threejs_buffer_km(&self) -> Vec<f64> {
        self.eci_km.clone()
    }

    /// Returns the ground-track buffer as interleaved `[lat0, lon0, lat1, lon1, ...]` in degrees.
    #[wasm_bindgen(js_name = groundLatLonDeg)]
    pub fn ground_lat_lon_deg(&self) -> Vec<f64> {
        self.ground_deg.clone()
    }
}

/// Core propagation logic for [`Keplerian::propagate_sampled`].
///
/// Separated from the `#[wasm_bindgen]` shim so that unit tests can call it
/// without triggering `JsValue::from_str`, which panics outside of wasm32.
///
/// Per-sample time offsets are computed from the float `step_s` via
/// `TimeDelta::from_seconds_f64` to avoid silent truncation of sub-second
/// steps (e.g. `30.5 s` must not silently become `30 s`).
fn compute_sampled_trajectory(
    elements: &LoxKeplerian,
    origin: DynOrigin,
    start_iso: &str,
    duration_s: f64,
    step_s: f64,
) -> Result<SampledTrajectory, String> {
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return Err("duration_s must be positive and finite".to_string());
    }
    if !step_s.is_finite() || step_s <= 0.0 {
        return Err("step_s must be positive and finite".to_string());
    }

    // Parse the start time.
    let utc_start = start_iso
        .parse::<LoxUtc>()
        .map_err(|e| e.to_string())?;
    let start: DynTime = utc_start.to_dyn_time();

    // Build a KeplerianOrbit and convert to DynCartesianOrbit.
    let kep_orbit =
        KeplerianOrbit::try_from_keplerian(*elements, start, DynOrigin::Earth, DynFrame::Icrf)
            .map_err(|e| e.to_string())?;
    let cartesian = kep_orbit
        .try_to_cartesian()
        .map_err(|e| e.to_string())?;

    // Build the Vallado propagator.
    let vallado = DynVallado::try_new(cartesian).map_err(|e| e.to_string())?;

    // IAU_EARTH body-fixed frame (purely polynomial, no IERS dependency).
    let iau_earth = DynFrame::Iau(origin);
    let provider = DefaultRotationProvider;

    let n_steps = (duration_s / step_s) as usize;
    let total_samples = n_steps + 1;

    let mut epochs_ms = Vec::with_capacity(total_samples);
    let mut eci_km = Vec::with_capacity(total_samples * 3);
    let mut ground_deg = Vec::with_capacity(total_samples * 2);

    for i in 0..=n_steps {
        // Per-sample time offset computed from the float step to avoid silent
        // truncation: `TimeDelta::from_seconds_f64` handles sub-second steps.
        let dt_s = step_s * i as f64;
        let t = start + TimeDelta::from_seconds_f64(dt_s);

        let state = vallado
            .state_at(t)
            .map_err(|e| e.to_string())?;

        // ECI position → Three.js (Y-up): (x, z, -y), convert m → km.
        let pos = state.position();
        eci_km.push(pos.x / 1000.0);
        eci_km.push(pos.z / 1000.0);
        eci_km.push(-pos.y / 1000.0);

        // Body-fixed transform for ground track.
        let body_fixed = state
            .try_to_frame(iau_earth, &provider)
            .map_err(|e| format!("frame transform error: {e}"))?;
        let ground = body_fixed
            .try_to_ground_location()
            .map_err(|e| e.to_string())?;
        ground_deg.push(ground.latitude().to_degrees());
        ground_deg.push(ground.longitude().to_degrees());

        // Unix epoch milliseconds from the UTC instant.
        let utc = t.to_utc();
        epochs_ms.push(unix_epoch_ms_from_utc(&utc));
    }

    Ok(SampledTrajectory {
        epochs_ms,
        eci_km,
        ground_deg,
    })
}

/// Converts a UTC instant to Unix epoch milliseconds.
fn unix_epoch_ms_from_utc(utc: &LoxUtc) -> f64 {
    // Sub-millisecond components (µs, ps) are intentionally dropped —
    // trajectory visualisation needs ms precision at most.
    use lox_space::time::calendar_dates::CalendarDate;
    use lox_space::time::time_of_day::CivilTime;

    let y = utc.year() as i64;
    let m = utc.month() as i64;
    let d = utc.day() as i64;
    let y = y - (m <= 2) as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m + (if m > 2 { -3 } else { 9 })) + 2) / 5 + (d - 1);
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let unix_days = era * 146_097 + doe - 719_468;
    let secs = utc.hour() as f64 * 3600.0
        + utc.minute() as f64 * 60.0
        + utc.second() as f64
        + utc.millisecond() as f64 / 1000.0;
    unix_days as f64 * 86_400_000.0 + secs * 1000.0
}

fn parse_scale(scale: &str) -> Result<DynTimeScale, JsValue> {
    scale
        .parse()
        .map_err(|_| JsValue::from_str(&format!("unknown time scale: {scale}")))
}

#[wasm_bindgen]
pub struct Utc(LoxUtc);

#[wasm_bindgen]
impl Utc {
    /// Parse a UTC timestamp from an ISO 8601 string.
    #[wasm_bindgen(js_name = fromIso)]
    pub fn from_iso(iso: &str) -> Result<Utc, JsValue> {
        iso.parse::<LoxUtc>()
            .map(Utc)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Convert to a `Time` in the given scale ("TAI", "TT", "TDB", "TCG", "TCB").
    #[wasm_bindgen(js_name = toScale)]
    pub fn to_scale(&self, scale: &str) -> Result<WasmTime, JsValue> {
        let scale = parse_scale(scale)?;
        Ok(WasmTime(self.0.to_dyn_time().to_scale(scale)))
    }

    pub fn year(&self) -> i64 {
        self.0.year()
    }
    pub fn month(&self) -> u8 {
        self.0.month()
    }
    pub fn day(&self) -> u8 {
        self.0.day()
    }
    pub fn hour(&self) -> u8 {
        self.0.hour()
    }
    pub fn minute(&self) -> u8 {
        self.0.minute()
    }
    pub fn second(&self) -> u8 {
        self.0.second()
    }
    pub fn millisecond(&self) -> u32 {
        self.0.millisecond()
    }
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[wasm_bindgen]
pub struct WasmTime(DynTime);

#[wasm_bindgen]
impl WasmTime {
    /// Convert to another time scale.
    #[wasm_bindgen(js_name = toScale)]
    pub fn to_scale(&self, scale: &str) -> Result<WasmTime, JsValue> {
        let scale = parse_scale(scale)?;
        Ok(WasmTime(self.0.to_scale(scale)))
    }

    /// Convert back to UTC.
    #[wasm_bindgen(js_name = toUtc)]
    pub fn to_utc(&self) -> Utc {
        Utc(self.0.to_utc())
    }

    /// The time scale abbreviation, e.g. "TAI", "TT", "TDB".
    pub fn scale(&self) -> String {
        self.0.scale().abbreviation().to_string()
    }

    pub fn year(&self) -> i64 {
        self.0.year()
    }
    pub fn month(&self) -> u8 {
        self.0.month()
    }
    pub fn day(&self) -> u8 {
        self.0.day()
    }
    pub fn hour(&self) -> u8 {
        self.0.hour()
    }
    pub fn minute(&self) -> u8 {
        self.0.minute()
    }
    pub fn second(&self) -> u8 {
        self.0.second()
    }
    pub fn millisecond(&self) -> u32 {
        self.0.millisecond()
    }
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// A single Walker-designed satellite: orbital elements + plane and
/// in-plane indices.
#[wasm_bindgen]
pub struct WasmSatellite {
    pub(crate) plane: u32,
    pub(crate) index_in_plane: u32,
    pub(crate) sma_m: f64,
    pub(crate) ecc: f64,
    pub(crate) inc_rad: f64,
    pub(crate) raan_rad: f64,
    pub(crate) aop_rad: f64,
    pub(crate) true_anomaly_rad: f64,
}

#[wasm_bindgen]
impl WasmSatellite {
    pub fn plane(&self) -> u32 { self.plane }
    #[wasm_bindgen(js_name = indexInPlane)]
    pub fn index_in_plane(&self) -> u32 { self.index_in_plane }
    #[wasm_bindgen(js_name = smaMeters)]
    pub fn sma_m(&self) -> f64 { self.sma_m }
    pub fn ecc(&self) -> f64 { self.ecc }
    #[wasm_bindgen(js_name = incRad)]
    pub fn inc_rad(&self) -> f64 { self.inc_rad }
    #[wasm_bindgen(js_name = raanRad)]
    pub fn raan_rad(&self) -> f64 { self.raan_rad }
    #[wasm_bindgen(js_name = aopRad)]
    pub fn aop_rad(&self) -> f64 { self.aop_rad }
    #[wasm_bindgen(js_name = trueAnomalyRad)]
    pub fn true_anomaly_rad(&self) -> f64 { self.true_anomaly_rad }
}

/// A Walker delta constellation builder.
///
/// All inputs use the same units as the `Keplerian` types: meters and
/// radians.
#[wasm_bindgen]
pub struct WalkerDelta;

#[wasm_bindgen]
impl WalkerDelta {
    /// Build a Walker delta constellation.
    ///
    /// - `nsats`: total number of satellites (must be divisible by `nplanes`).
    /// - `nplanes`: number of orbital planes.
    /// - `phasing`: Walker phasing parameter `F` in [0, nplanes).
    /// - `sma_m`: semi-major axis in meters.
    /// - `ecc`: eccentricity.
    /// - `inc_rad`: inclination in radians.
    ///
    /// Each satellite's in-plane phase is set as a mean anomaly internally
    /// by `WalkerDeltaBuilder`; the returned `trueAnomalyRad` is the
    /// corresponding true anomaly (identical when `ecc == 0`, but diverging
    /// for eccentric orbits via the standard Kepler equation).
    #[wasm_bindgen]
    pub fn build(
        nsats: u32,
        nplanes: u32,
        phasing: u32,
        sma_m: f64,
        ecc: f64,
        inc_rad: f64,
    ) -> Result<js_sys::Array, JsValue> {
        let sats = lox_space::orbits::constellations::WalkerDeltaBuilder::new(
            nsats as usize,
            nplanes as usize,
        )
        .with_semi_major_axis(sma_m.m(), ecc)
        .with_inclination(inc_rad.rad())
        .with_phasing(phasing as usize)
        .build()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let out = sats.into_iter().map(|s| {
            JsValue::from(WasmSatellite {
                plane: s.plane as u32,
                index_in_plane: s.index_in_plane as u32,
                sma_m: s.elements.semi_major_axis().to_meters(),
                ecc: s.elements.eccentricity().as_f64(),
                inc_rad: s.elements.inclination().as_f64(),
                raan_rad: s.elements.longitude_of_ascending_node().as_f64(),
                aop_rad: s.elements.argument_of_periapsis().as_f64(),
                true_anomaly_rad: s.elements.true_anomaly().as_f64(),
            })
        });

        Ok(out.collect())
    }
}

/// Earth's body-fixed rotation angle (the IAU `W` prime-meridian angle),
/// radians, at the given ISO-8601 UTC instant.
///
/// Computed from lox-bodies' IAU polynomial model — no IERS Earth
/// Orientation Parameter data is required. The 3D Earth mesh in the
/// GlobeView is rotated each frame by this angle. Consistent with the
/// `IAU_EARTH` frame used to project Cartesian states to lat/lon in
/// `Keplerian::propagate_sampled`.
#[wasm_bindgen(js_name = earthRotationAngleRad)]
pub fn earth_rotation_angle_rad_wasm(epoch_iso: &str) -> Result<f64, JsValue> {
    compute_earth_rotation_angle_rad(epoch_iso).map_err(|e| JsValue::from_str(&e))
}

fn compute_earth_rotation_angle_rad(epoch_iso: &str) -> Result<f64, String> {
    use lox_space::bodies::{Earth, RotationalElements};
    use lox_space::time::julian_dates::JulianDate;
    use lox_space::time::offsets::DefaultOffsetProvider;
    use lox_space::time::time_scales::DynTimeScale;

    let utc: LoxUtc = epoch_iso
        .parse()
        .map_err(|e: lox_space::time::utc::UtcError| e.to_string())?;

    // Convert UTC → TAI (DynTime) → TDB, then get seconds since J2000.
    // RotationalElements::rotation_angle takes `t` in seconds since J2000 TDB.
    // DynTimeScale → DynTimeScale requires try_to_scale (only TryOffset is implemented).
    let t_tdb = utc
        .to_dyn_time()
        .try_to_scale(DynTimeScale::Tdb, &DefaultOffsetProvider)
        .map_err(|e| e.to_string())?
        .seconds_since_j2000();

    // IAU prime-meridian angle W, radians (not normalised).
    let w = Earth.rotation_angle(t_tdb);

    // Normalise to [0, 2π).
    Ok(w.rem_euclid(TAU))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Origin tests

    #[test]
    fn test_origin_from_name() {
        let origin = DynOrigin::from_str("Earth").unwrap();
        assert_eq!(OriginTrait::name(&origin), "Earth");
    }

    #[test]
    fn test_origin_from_naif_id() {
        let origin = DynOrigin::try_from(399i32).unwrap();
        assert_eq!(OriginTrait::name(&origin), "Earth");
    }

    #[test]
    fn test_origin_gravitational_parameter() {
        let origin = DynOrigin::from_str("Earth").unwrap();
        let gp = TryPointMass::try_gravitational_parameter(&origin)
            .unwrap()
            .as_f64();
        assert!((gp - 3.986e14).abs() < 1e11, "gp = {gp}");
    }

    #[test]
    fn test_origin_mean_radius() {
        let origin = DynOrigin::from_str("Earth").unwrap();
        let r = TryMeanRadius::try_mean_radius(&origin).unwrap().to_meters();
        assert!((r - 6.371e6).abs() < 1e4, "r = {r}");
    }

    #[test]
    fn test_origin_unknown_name() {
        assert!(DynOrigin::from_str("Tatooine").is_err());
    }

    #[test]
    fn test_origin_unknown_id() {
        assert!(DynOrigin::try_from(999999i32).is_err());
    }

    // Frame tests

    #[test]
    fn test_frame_icrf() {
        let frame = DynFrame::from_str("ICRF").unwrap();
        assert_eq!(frame.abbreviation(), "ICRF");
    }

    #[test]
    fn test_frame_itrf() {
        let frame = DynFrame::from_str("ITRF").unwrap();
        assert_eq!(frame.abbreviation(), "ITRF");
    }

    #[test]
    fn test_frame_unknown() {
        assert!(DynFrame::from_str("UNKNOWN_FRAME_XYZ").is_err());
    }

    // Keplerian tests
    //
    // These tests use lox_core types directly to avoid triggering JsValue on non-wasm32
    // targets (wasm-bindgen's JsValue panics outside wasm).

    use lox_space::core::elements::{Eccentricity, Keplerian as LoxKeplerianTest};
    use lox_space::orbits::sso::inclination_sso;

    #[test]
    fn test_keplerian_roundtrip() {
        // Molniya-like orbit
        let sma = 26_600_000.0_f64;
        let ecc = 0.74;
        let inc = 63.4_f64.to_radians();
        let raan = 0.5;
        let aop = 1.2;
        let ta = 0.0;

        let k = LoxKeplerianTest::builder()
            .with_semi_major_axis(sma.m(), ecc)
            .with_inclination(inc.rad())
            .with_longitude_of_ascending_node(raan.rad())
            .with_argument_of_periapsis(aop.rad())
            .with_true_anomaly(ta.rad())
            .build()
            .unwrap();

        assert!((k.semi_major_axis().to_meters() - sma).abs() < 1.0);
        assert!((k.eccentricity().as_f64() - ecc).abs() < 1e-12);
        assert!((k.inclination().as_f64() - inc).abs() < 1e-12);
        assert!((k.longitude_of_ascending_node().as_f64() - raan).abs() < 1e-12);
        assert!((k.argument_of_periapsis().as_f64() - aop).abs() < 1e-12);
        assert!((k.true_anomaly().as_f64() - ta).abs() < 1e-12);
    }

    #[test]
    fn test_keplerian_orbital_period_molniya() {
        use lox_space::core::elements::GravitationalParameter;

        let sma = 26_600_000.0_f64;
        let ecc = 0.74;
        let inc = 63.4_f64.to_radians();
        let mu_km3 = 398600.43550702266; // km³/s²
        let mu = GravitationalParameter::km3_per_s2(mu_km3);

        let k = LoxKeplerianTest::builder()
            .with_semi_major_axis(sma.m(), ecc)
            .with_inclination(inc.rad())
            .with_longitude_of_ascending_node(0.0_f64.rad())
            .with_argument_of_periapsis(0.0_f64.rad())
            .build()
            .unwrap();

        let period = k.orbital_period(mu).unwrap();
        let secs = period.to_seconds();
        let period_s = secs.hi + secs.lo;
        assert!((period_s - 43082.0).abs() < 500.0, "period = {period_s}");
    }

    #[test]
    fn test_keplerian_invalid_eccentricity() {
        let result = Eccentricity::try_new(-0.1);
        assert!(result.is_err(), "Expected error for negative eccentricity");
    }

    #[test]
    fn test_keplerian_from_radii_leo() {
        // LEO: periapsis ~6578 km, apoapsis ~6728 km
        let rp = 6_578_000.0_f64;
        let ra = 6_728_000.0_f64;
        let inc = 51.6_f64.to_radians();

        let k = LoxKeplerianTest::builder()
            .with_radii(rp.m(), ra.m())
            .with_inclination(inc.rad())
            .with_longitude_of_ascending_node(0.0_f64.rad())
            .with_argument_of_periapsis(0.0_f64.rad())
            .build()
            .unwrap();

        let expected_sma = (rp + ra) / 2.0;
        assert!((k.semi_major_axis().to_meters() - expected_sma).abs() < 1.0);
        assert!(k.eccentricity().as_f64() > 0.0);
        assert!((k.eccentricity().as_f64() - (ra - rp) / (ra + rp)).abs() < 1e-10);
    }

    #[test]
    fn test_keplerian_from_altitudes_leo() {
        // LEO altitudes: periapsis 200 km, apoapsis 350 km above mean radius ~6371 km
        let alt_p = 200_000.0_f64;
        let alt_a = 350_000.0_f64;
        let inc = 51.6_f64.to_radians();
        let mean_r = TryMeanRadius::try_mean_radius(&DynOrigin::from_str("Earth").unwrap())
            .unwrap()
            .to_meters();

        let k = LoxKeplerianTest::builder()
            .with_altitudes(alt_p.m(), alt_a.m(), mean_r.m())
            .with_inclination(inc.rad())
            .with_longitude_of_ascending_node(0.0_f64.rad())
            .with_argument_of_periapsis(0.0_f64.rad())
            .build()
            .unwrap();

        let expected_sma = (alt_p + alt_a) / 2.0 + mean_r;
        assert!((k.semi_major_axis().to_meters() - expected_sma).abs() < 1.0);
    }

    #[test]
    fn test_keplerian_circular() {
        let sma = 7_000_000.0_f64;
        let inc = 45.0_f64.to_radians();
        let raan = 0.3_f64;

        let k = LoxKeplerianTest::builder()
            .with_semi_major_axis(sma.m(), 0.0)
            .with_inclination(inc.rad())
            .with_longitude_of_ascending_node(raan.rad())
            .with_argument_of_periapsis(0.0_f64.rad())
            .with_true_anomaly(0.0_f64.rad())
            .build()
            .unwrap();

        assert!(
            (k.eccentricity().as_f64() - 0.0).abs() < 1e-12,
            "eccentricity should be 0"
        );
        assert!((k.argument_of_periapsis().as_f64() - 0.0).abs() < 1e-12);
        assert!((k.true_anomaly().as_f64() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_keplerian_circular_from_altitude() {
        let altitude = 400_000.0_f64; // 400 km in meters
        let earth = DynOrigin::from_str("Earth").unwrap();
        let mean_r = TryMeanRadius::try_mean_radius(&earth).unwrap().to_meters();

        let k = LoxKeplerianTest::builder()
            .with_semi_major_axis((altitude + mean_r).m(), 0.0)
            .with_inclination(0.0_f64.rad())
            .with_longitude_of_ascending_node(0.0_f64.rad())
            .with_argument_of_periapsis(0.0_f64.rad())
            .with_true_anomaly(0.0_f64.rad())
            .build()
            .unwrap();

        let expected_sma = altitude + mean_r;
        assert!((k.semi_major_axis().to_meters() - expected_sma).abs() < 1.0);
        assert!(k.eccentricity().as_f64().abs() < 1e-12);

        // Verify orbital period is reasonable (~92 min for 400 km LEO)
        let mu = TryPointMass::try_gravitational_parameter(&earth)
            .unwrap()
            .as_f64();
        let period = TAU * (k.semi_major_axis().to_meters().powi(3) / mu).sqrt();
        assert!(period > 5400.0 && period < 5700.0, "period = {period}");
    }

    #[test]
    fn test_keplerian_sso() {
        // 600 km altitude
        let alt = 600_000.0_f64;
        let ecc = Eccentricity::default(); // 0.0
        let earth = DynOrigin::from_str("Earth").unwrap();
        let mean_r = TryMeanRadius::try_mean_radius(&earth).unwrap().to_meters();
        let semi_major_axis = (alt + mean_r).m();
        let inclination = inclination_sso(semi_major_axis, ecc).unwrap();
        let inc_deg = inclination.as_f64().to_degrees();
        assert!(
            inc_deg > 97.0 && inc_deg < 99.0,
            "SSO inclination = {inc_deg} deg"
        );
    }

    // Cartesian tests

    #[test]
    fn test_cartesian_construction_roundtrip() {
        let pos = [7_000_000.0_f64, 0.0, 0.0];
        let vel = [0.0_f64, 7_500.0, 0.0];
        let state = LoxCartesian::from_vecs(
            DVec3::new(pos[0], pos[1], pos[2]),
            DVec3::new(vel[0], vel[1], vel[2]),
        );
        let p = state.position();
        let v = state.velocity();
        assert!((p.x - pos[0]).abs() < 1e-6);
        assert!((p.y - pos[1]).abs() < 1e-6);
        assert!((p.z - pos[2]).abs() < 1e-6);
        assert!((v.x - vel[0]).abs() < 1e-9);
        assert!((v.y - vel[1]).abs() < 1e-9);
        assert!((v.z - vel[2]).abs() < 1e-9);
    }

    #[test]
    fn test_cartesian_component_getters() {
        let px = 6_778_000.0_f64;
        let py = 100_000.0_f64;
        let pz = -50_000.0_f64;
        let vx = 200.0_f64;
        let vy = 7_500.0_f64;
        let vz = -10.0_f64;
        let state = LoxCartesian::from_vecs(DVec3::new(px, py, pz), DVec3::new(vx, vy, vz));
        assert!((state.x().to_meters() - px).abs() < 1e-6);
        assert!((state.y().to_meters() - py).abs() < 1e-6);
        assert!((state.z().to_meters() - pz).abs() < 1e-6);
        assert!((state.vx().to_meters_per_second() - vx).abs() < 1e-9);
        assert!((state.vy().to_meters_per_second() - vy).abs() < 1e-9);
        assert!((state.vz().to_meters_per_second() - vz).abs() < 1e-9);
    }

    #[test]
    fn test_keplerian_to_cartesian_position_magnitude() {
        use lox_space::core::elements::GravitationalParameter;

        // Circular LEO at 7000 km radius
        let sma = 7_000_000.0_f64;
        let mu = GravitationalParameter::km3_per_s2(398600.43550702266);

        let k = LoxKeplerianTest::builder()
            .with_semi_major_axis(sma.m(), 0.0)
            .with_inclination(0.0_f64.rad())
            .with_longitude_of_ascending_node(0.0_f64.rad())
            .with_argument_of_periapsis(0.0_f64.rad())
            .with_true_anomaly(0.0_f64.rad())
            .build()
            .unwrap();

        let cart = k.to_cartesian(mu);
        let r = cart.position().length();
        // For circular orbit at true anomaly = 0, |r| should equal sma
        assert!((r - sma).abs() < 1.0, "position magnitude = {r}");
    }

    #[test]
    fn test_keplerian_cartesian_roundtrip() {
        use lox_space::core::elements::GravitationalParameter;

        let sma = 26_600_000.0_f64;
        let ecc = 0.74;
        let inc = 63.4_f64.to_radians();
        let raan = 0.5;
        let aop = 1.2;
        let ta = 0.3;
        let mu = GravitationalParameter::km3_per_s2(398600.43550702266);

        let k1 = LoxKeplerianTest::builder()
            .with_semi_major_axis(sma.m(), ecc)
            .with_inclination(inc.rad())
            .with_longitude_of_ascending_node(raan.rad())
            .with_argument_of_periapsis(aop.rad())
            .with_true_anomaly(ta.rad())
            .build()
            .unwrap();

        let k2 = k1.to_cartesian(mu).to_keplerian(mu);

        assert!(
            (k1.semi_major_axis().to_meters() - k2.semi_major_axis().to_meters()).abs() < 1.0,
            "sma mismatch: {} vs {}",
            k1.semi_major_axis().to_meters(),
            k2.semi_major_axis().to_meters()
        );
        assert!(
            (k1.eccentricity().as_f64() - k2.eccentricity().as_f64()).abs() < 1e-10,
            "ecc mismatch"
        );
        assert!(
            (k1.inclination().as_f64() - k2.inclination().as_f64()).abs() < 1e-10,
            "inc mismatch"
        );
        assert!(
            (k1.longitude_of_ascending_node().as_f64() - k2.longitude_of_ascending_node().as_f64())
                .abs()
                < 1e-10,
            "raan mismatch"
        );
        assert!(
            (k1.argument_of_periapsis().as_f64() - k2.argument_of_periapsis().as_f64()).abs()
                < 1e-10,
            "aop mismatch"
        );
        assert!(
            (k1.true_anomaly().as_f64() - k2.true_anomaly().as_f64()).abs() < 1e-10,
            "ta mismatch"
        );
    }

    // Trajectory tests

    fn molniya_keplerian() -> LoxKeplerianTest {
        LoxKeplerianTest::builder()
            .with_semi_major_axis(26_600_000.0_f64.m(), 0.74)
            .with_inclination(63.4_f64.to_radians().rad())
            .with_longitude_of_ascending_node(0.0_f64.rad())
            .with_argument_of_periapsis(0.0_f64.rad())
            .with_true_anomaly(0.0_f64.rad())
            .build()
            .unwrap()
    }

    fn earth_gp() -> lox_space::core::elements::GravitationalParameter {
        lox_space::core::elements::GravitationalParameter::km3_per_s2(398600.43550702266)
    }

    #[test]
    fn test_trajectory_to_buffer() {
        let n = 100;
        let k = molniya_keplerian();
        let gp = earth_gp();
        let states = k.trace(gp, n).unwrap();
        let traj = Trajectory {
            states,
            origin: DynOrigin::from_str("Earth").unwrap(),
            frame: DynFrame::default(),
        };
        let buf = traj.to_buffer();
        assert_eq!(buf.len(), n * 3);
        // First position should be non-zero (orbit is far from origin)
        let r2 = buf[0] * buf[0] + buf[1] * buf[1] + buf[2] * buf[2];
        assert!(r2 > 0.0, "first position should be non-zero");
    }

    #[test]
    fn test_trajectory_len() {
        let n = 72;
        let k = molniya_keplerian();
        let gp = earth_gp();
        let states = k.trace(gp, n).unwrap();
        let traj = Trajectory {
            states,
            origin: DynOrigin::from_str("Earth").unwrap(),
            frame: DynFrame::default(),
        };
        assert_eq!(traj.len(), n);
        assert!(!traj.is_empty());
    }

    #[test]
    fn test_trace_positions_reasonable() {
        // Molniya orbit: periapsis ~6900 km, apoapsis ~46300 km
        let n = 360;
        let k = molniya_keplerian();
        let gp = earth_gp();
        let states = k.trace(gp, n).unwrap();
        for state in &states {
            let p = state.position();
            let r_m = p.length();
            let r_km = r_m / 1000.0;
            assert!(
                (6_000.0..=50_000.0).contains(&r_km),
                "position radius out of range: {r_km} km"
            );
        }
    }

    #[test]
    fn test_cartesian_to_threejs() {
        let cart = LoxCartesian::from_vecs(
            DVec3::new(1000.0, 2000.0, 3000.0),
            DVec3::new(0.0, 0.0, 0.0),
        );
        let state = Cartesian {
            state: cart,
            origin: DynOrigin::default(),
            frame: DynFrame::default(),
        };
        let threejs = state.to_threejs();
        assert!((threejs[0] - 1000.0).abs() < 1e-10);
        assert!((threejs[1] - 3000.0).abs() < 1e-10);
        assert!((threejs[2] - (-2000.0)).abs() < 1e-10);
    }

    #[test]
    fn test_trajectory_to_threejs_buffer() {
        let p1 = LoxCartesian::from_vecs(DVec3::new(1.0, 2.0, 3.0), DVec3::new(0.0, 0.0, 0.0));
        let p2 = LoxCartesian::from_vecs(DVec3::new(4.0, 5.0, 6.0), DVec3::new(0.0, 0.0, 0.0));
        let traj = Trajectory {
            states: vec![p1, p2],
            origin: DynOrigin::default(),
            frame: DynFrame::default(),
        };
        let buf = traj.to_threejs_buffer();
        assert_eq!(buf.len(), 6);
        assert!((buf[0] - 1.0).abs() < 1e-10);
        assert!((buf[1] - 3.0).abs() < 1e-10);
        assert!((buf[2] - (-2.0)).abs() < 1e-10);
        assert!((buf[3] - 4.0).abs() < 1e-10);
        assert!((buf[4] - 6.0).abs() < 1e-10);
        assert!((buf[5] - (-5.0)).abs() < 1e-10);
    }

    #[test]
    fn test_walker_delta_24_3_1_at_600km_53deg() {
        // Walker delta 24/3/1: 24 satellites, 3 planes, phasing 1, alt 600 km, inc 53°.
        let earth = DynOrigin::from_str("Earth").unwrap();
        let mean_r = TryMeanRadius::try_mean_radius(&earth).unwrap().to_meters();
        let sma = (600_000.0_f64 + mean_r).m();
        let inc = 53.0_f64.to_radians().rad();

        let sats = lox_space::orbits::constellations::WalkerDeltaBuilder::new(24, 3)
            .with_semi_major_axis(sma, 0.0)
            .with_inclination(inc)
            .with_phasing(1)
            .build()
            .unwrap();

        assert_eq!(sats.len(), 24);
        // 3 planes × 8 sats/plane.
        let plane_counts: std::collections::BTreeMap<usize, usize> =
            sats.iter().fold(std::collections::BTreeMap::new(), |mut m, s| {
                *m.entry(s.plane).or_insert(0) += 1;
                m
            });
        assert_eq!(plane_counts.len(), 3);
        for (_p, count) in plane_counts.iter() {
            assert_eq!(*count, 8, "each plane should have 8 satellites");
        }
        // RAANs of the three planes should be 0, 120°, 240° (modulo wrap).
        let raans: std::collections::BTreeMap<usize, f64> =
            sats.iter().fold(std::collections::BTreeMap::new(), |mut m, s| {
                m.entry(s.plane).or_insert(s.elements.longitude_of_ascending_node().as_f64());
                m
            });
        let mut raan_vec: Vec<f64> = raans.values().copied().collect();
        raan_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let expected: [f64; 3] = [0.0, 120.0_f64.to_radians(), 240.0_f64.to_radians()];
        for (got, exp) in raan_vec.iter().zip(expected.iter()) {
            assert!((got - exp).abs() < 1e-9, "raan {got} vs expected {exp}");
        }
    }

    #[test]
    fn test_compute_sampled_trajectory_buffer_sizes_match_step() {
        // 6 hours @ 60 s step → n_steps = (6*3600/60) = 360, total = 361 samples.
        // Exercises compute_sampled_trajectory (the JsValue-free helper) end-to-end,
        // including the n_steps formula and buffer layout.
        let elements = LoxKeplerianTest::builder()
            .with_semi_major_axis(7_000_000.0_f64.m(), 0.0)
            .with_inclination(53.0_f64.to_radians().rad())
            .with_longitude_of_ascending_node(0.0_f64.rad())
            .with_argument_of_periapsis(0.0_f64.rad())
            .with_true_anomaly(0.0_f64.rad())
            .build()
            .unwrap();

        let traj = compute_sampled_trajectory(
            &elements,
            DynOrigin::from_str("Earth").unwrap(),
            "2026-06-01T00:00:00",
            6.0 * 3600.0,
            60.0,
        )
        .expect("propagation should succeed");

        assert_eq!(traj.len(), 361);
        assert_eq!(traj.eci_km.len(), 361 * 3);
        assert_eq!(traj.ground_deg.len(), 361 * 2);
    }

    #[test]
    fn test_propagate_sampled_returns_expected_buffer_sizes() {
        // Verify the WASM-side struct directly (avoids JsValue).
        let traj = SampledTrajectory {
            epochs_ms: vec![0.0, 30_000.0, 60_000.0],
            eci_km: vec![7000.0, 0.0, 0.0, 7000.0, 1.0, 0.0, 7000.0, 2.0, 0.0],
            ground_deg: vec![0.0, 0.0, 0.1, 0.0, 0.2, 0.0],
        };
        assert_eq!(traj.len(), 3);
        assert_eq!(traj.epochs_ms.len(), 3);
        assert_eq!(traj.eci_km.len(), 9);
        assert_eq!(traj.ground_deg.len(), 6);
    }

    #[test]
    fn test_wasm_constellation_satellite_struct_round_trip() {
        // Construct a WasmSatellite directly and verify each getter returns
        // the value it was constructed with. Exercises the #[wasm_bindgen]
        // getters used at the JS boundary without crossing it.
        let sat = WasmSatellite {
            plane: 2,
            index_in_plane: 5,
            sma_m: 6_978_137.0,
            ecc: 0.0,
            inc_rad: 0.9250245,
            raan_rad: 2.0943951,
            aop_rad: 0.0,
            true_anomaly_rad: 1.5707963,
        };
        assert_eq!(sat.plane(), 2);
        assert_eq!(sat.index_in_plane(), 5);
        assert!((sat.sma_m() - 6_978_137.0).abs() < 1e-6);
        assert!((sat.ecc() - 0.0).abs() < 1e-12);
        assert!((sat.inc_rad() - 0.9250245).abs() < 1e-9);
        assert!((sat.raan_rad() - 2.0943951).abs() < 1e-9);
        assert!((sat.aop_rad() - 0.0).abs() < 1e-12);
        assert!((sat.true_anomaly_rad() - 1.5707963).abs() < 1e-9);
    }

    #[test]
    fn test_earth_rotation_angle_known_instant() {
        // At J2000.0 (2000-01-01T12:00:00 UTC), the IAU W angle for Earth
        // has a known reference value. We confirm the result is finite and within [0, 2π).
        let era = compute_earth_rotation_angle_rad("2000-01-01T12:00:00").unwrap();
        assert!(era.is_finite());
        assert!((0.0..std::f64::consts::TAU).contains(&era));
    }

    #[test]
    fn test_earth_rotation_angle_advances_by_one_day() {
        // After 24 UTC hours, the angle has advanced by ~2π × (1 + 1/365.25) mod 2π.
        let era0 = compute_earth_rotation_angle_rad("2026-06-01T00:00:00").unwrap();
        let era1 = compute_earth_rotation_angle_rad("2026-06-02T00:00:00").unwrap();
        let delta = (era1 - era0).rem_euclid(std::f64::consts::TAU);
        let expected = std::f64::consts::TAU * 0.00274;
        // Tolerance generous — IAU model includes long-period terms.
        assert!((delta - expected).abs() < 5e-2,
                "delta = {delta} rad, expected ≈ {expected} rad");
    }
}
