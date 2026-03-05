// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Coordinate types for representing positions, velocities, and trajectories.

use core::f64::consts::{FRAC_PI_2, PI, TAU};
use std::{
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    sync::Arc,
};

use glam::{DMat3, DVec3};
use lox_test_utils::ApproxEq;
use thiserror::Error;

use crate::{
    math::{
        roots::{FindRoot, RootFinderError, Secant},
        series::{InterpolationType, Series},
    },
    time::deltas::TimeDelta,
    units::{Angle, Distance, Velocity},
};

/// Azimuth-elevation pair for representing direction in a topocentric frame.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AzEl(Angle, Angle);

impl AzEl {
    /// Returns a new [`AzElBuilder`].
    pub fn builder() -> AzElBuilder {
        AzElBuilder::default()
    }

    /// Returns the azimuth angle.
    pub fn azimuth(&self) -> Angle {
        self.0
    }

    /// Returns the elevation angle.
    pub fn elevation(&self) -> Angle {
        self.1
    }
}

/// Error returned when constructing an [`AzEl`] with invalid angles.
#[derive(Copy, Clone, Debug, Error, PartialEq)]
pub enum AzElError {
    /// The azimuth angle is outside the valid range [0°, 360°].
    #[error("azimuth must be between 0 deg and 360 deg but was {0}")]
    InvalidAzimuth(Angle),
    /// The elevation angle is outside the valid range [0°, 360°].
    #[error("elevation must be between 0 deg and 360 deg but was {0}")]
    InvalidElevation(Angle),
}

/// Builder for constructing validated [`AzEl`] instances.
#[derive(Copy, Clone, Debug)]
pub struct AzElBuilder {
    azimuth: Result<Angle, AzElError>,
    elevation: Result<Angle, AzElError>,
}

impl Default for AzElBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AzElBuilder {
    /// Creates a new builder with default (zero) angles.
    pub fn new() -> Self {
        Self {
            azimuth: Ok(Angle::default()),
            elevation: Ok(Angle::default()),
        }
    }

    /// Sets the azimuth angle. Must be between 0° and 360°.
    pub fn azimuth(&mut self, azimuth: Angle) -> &mut Self {
        self.azimuth = match azimuth.to_radians() {
            lon if lon < 0.0 => Err(AzElError::InvalidAzimuth(azimuth)),
            lon if lon > TAU => Err(AzElError::InvalidAzimuth(azimuth)),
            _ => Ok(azimuth),
        };
        self
    }

    /// Sets the elevation angle. Must be between 0° and 360°.
    pub fn elevation(&mut self, elevation: Angle) -> &mut Self {
        self.elevation = match elevation.to_radians() {
            lat if lat < 0.0 => Err(AzElError::InvalidElevation(elevation)),
            lat if lat > TAU => Err(AzElError::InvalidElevation(elevation)),
            _ => Ok(elevation),
        };
        self
    }

    /// Builds the [`AzEl`], returning an error if any angle is invalid.
    pub fn build(&self) -> Result<AzEl, AzElError> {
        Ok(AzEl(self.azimuth?, self.elevation?))
    }
}

/// Geodetic coordinates: longitude, latitude, and altitude.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LonLatAlt(Angle, Angle, Distance);

impl LonLatAlt {
    /// Creates a `LonLatAlt` from longitude and latitude in degrees and altitude in meters.
    pub fn from_degrees(lon_deg: f64, lat_deg: f64, alt_m: f64) -> Result<Self, LonLatAltError> {
        LonLatAltBuilder::new()
            .longitude(Angle::degrees(lon_deg))
            .latitude(Angle::degrees(lat_deg))
            .altitude(Distance::meters(alt_m))
            .build()
    }

    /// Returns a new [`LonLatAltBuilder`].
    pub fn builder() -> LonLatAltBuilder {
        LonLatAltBuilder::default()
    }

    /// Returns the longitude.
    pub fn lon(&self) -> Angle {
        self.0
    }

    /// Returns the latitude.
    pub fn lat(&self) -> Angle {
        self.1
    }

    /// Returns the altitude.
    pub fn alt(&self) -> Distance {
        self.2
    }

    /// Converts geodetic coordinates (LLA) to body-fixed Cartesian position (meters).
    pub fn to_body_fixed(&self, equatorial_radius: Distance, flattening: f64) -> DVec3 {
        let alt = self.alt().to_meters();
        let (lon_sin, lon_cos) = self.lon().sin_cos();
        let (lat_sin, lat_cos) = self.lat().sin_cos();
        let r_eq = equatorial_radius.to_meters();
        let e = (2.0 * flattening - flattening.powi(2)).sqrt();
        let c = r_eq / (1.0 - e.powi(2) * lat_sin.powi(2)).sqrt();
        let s = c * (1.0 - e.powi(2));
        let r_delta = (c + alt) * lat_cos;
        let r_kappa = (s + alt) * lat_sin;
        DVec3::new(r_delta * lon_cos, r_delta * lon_sin, r_kappa)
    }

    /// Converts a body-fixed Cartesian position (meters) to geodetic coordinates (LLA).
    ///
    /// Returns [`FromBodyFixedError::ZeroPosition`] if the position vector has
    /// zero length. Polar positions (where the equatorial projection is zero)
    /// are handled as a special case without root-finding.
    pub fn from_body_fixed(
        pos: DVec3,
        equatorial_radius: Distance,
        flattening: f64,
    ) -> Result<Self, FromBodyFixedError> {
        let r_eq = equatorial_radius.to_meters();
        let rm = pos.length();

        if rm < 1e-10 {
            return Err(FromBodyFixedError::ZeroPosition);
        }

        let r_delta = (pos.x.powi(2) + pos.y.powi(2)).sqrt();

        // Polar special case: r_delta ≈ 0 means we're on or near a pole.
        // The iterative solver divides by r_delta so we handle this directly.
        if r_delta < 1e-10 {
            let lat = if pos.z >= 0.0 { FRAC_PI_2 } else { -FRAC_PI_2 };
            let e = (2.0 * flattening - flattening.powi(2)).sqrt();
            let r_polar = r_eq * (1.0 - e.powi(2)).sqrt();
            let alt = pos.z.abs() - r_polar;
            return Ok(LonLatAlt(
                Angle::radians(0.0),
                Angle::radians(lat),
                Distance::meters(alt),
            ));
        }

        let mut lon = pos.y.atan2(pos.x);

        if lon.abs() >= PI {
            if lon < 0.0 {
                lon += TAU;
            } else {
                lon -= TAU;
            }
        }

        let delta = (pos.z / rm).asin();

        let root_finder = Secant::default();

        let f = flattening;
        let lat = root_finder.find(
            |lat: f64| {
                let e = (2.0 * f - f.powi(2)).sqrt();
                let c = r_eq / (1.0 - e.powi(2) * lat.sin().powi(2)).sqrt();
                Ok(pos.z + c * e.powi(2) * lat.sin()).map(|v| v / r_delta - lat.tan())
            },
            delta,
        )?;

        let e = (2.0 * f - f.powi(2)).sqrt();
        let c = r_eq / (1.0 - e.powi(2) * lat.sin().powi(2)).sqrt();
        let alt = r_delta / lat.cos() - c;

        Ok(LonLatAlt(
            Angle::radians(lon),
            Angle::radians(lat),
            Distance::meters(alt),
        ))
    }

    /// Returns the rotation matrix from body-fixed to topocentric (SEZ) frame.
    pub fn rotation_to_topocentric(&self) -> DMat3 {
        let rot1 = DMat3::from_rotation_z(self.lon().to_radians()).transpose();
        let rot2 = DMat3::from_rotation_y(FRAC_PI_2 - self.lat().to_radians()).transpose();
        rot2 * rot1
    }
}

/// Error returned when constructing a [`LonLatAlt`] with invalid values.
#[derive(Copy, Clone, Debug, Error, PartialEq)]
pub enum LonLatAltError {
    /// The longitude is outside the valid range [-180°, 180°].
    #[error("longitude must be between -180 deg and 180 deg but was {0}")]
    InvalidLongitude(Angle),
    /// The latitude is outside the valid range [-90°, 90°].
    #[error("latitude must between -90 deg and 90 deg but was {0}")]
    InvalidLatitude(Angle),
    /// The altitude is not a finite value.
    #[error("invalid altitude {0}")]
    InvalidAltitude(Distance),
}

/// Error returned when converting from body-fixed coordinates to geodetic.
#[derive(Debug, Error)]
pub enum FromBodyFixedError {
    /// The position vector has zero length.
    #[error("position vector has zero length")]
    ZeroPosition,
    /// The root finder failed to converge.
    #[error(transparent)]
    RootFinder(#[from] RootFinderError),
}

/// Builder for constructing validated [`LonLatAlt`] instances.
#[derive(Copy, Clone, Debug)]
pub struct LonLatAltBuilder {
    longitude: Result<Angle, LonLatAltError>,
    latitude: Result<Angle, LonLatAltError>,
    altitude: Result<Distance, LonLatAltError>,
}

impl Default for LonLatAltBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LonLatAltBuilder {
    /// Creates a new builder with default (zero) values.
    pub fn new() -> Self {
        Self {
            longitude: Ok(Angle::default()),
            latitude: Ok(Angle::default()),
            altitude: Ok(Distance::default()),
        }
    }

    /// Sets the longitude. Must be between -180° and 180°.
    pub fn longitude(&mut self, longitude: Angle) -> &mut Self {
        self.longitude = match longitude.to_degrees() {
            lon if lon < -180.0 => Err(LonLatAltError::InvalidLongitude(longitude)),
            lon if lon > 180.0 => Err(LonLatAltError::InvalidLongitude(longitude)),
            _ => Ok(longitude),
        };
        self
    }

    /// Sets the latitude. Must be between -90° and 90°.
    pub fn latitude(&mut self, latitude: Angle) -> &mut Self {
        self.latitude = match latitude.to_degrees() {
            lat if lat < -90.0 => Err(LonLatAltError::InvalidLatitude(latitude)),
            lat if lat > 90.0 => Err(LonLatAltError::InvalidLatitude(latitude)),
            _ => Ok(latitude),
        };
        self
    }

    /// Sets the altitude. Must be a finite value.
    pub fn altitude(&mut self, altitude: Distance) -> &mut Self {
        self.altitude = if !altitude.to_meters().is_finite() {
            Err(LonLatAltError::InvalidAltitude(altitude))
        } else {
            Ok(altitude)
        };
        self
    }

    /// Builds the [`LonLatAlt`], returning an error if any value is invalid.
    pub fn build(&self) -> Result<LonLatAlt, LonLatAltError> {
        Ok(LonLatAlt(self.longitude?, self.latitude?, self.altitude?))
    }
}

/// Cartesian state vector with position and velocity in meters and m/s.
#[derive(Copy, Clone, Debug, Default, PartialEq, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cartesian {
    pos: DVec3,
    vel: DVec3,
}

impl Cartesian {
    /// Creates a new Cartesian state from individual position and velocity components.
    pub const fn new(
        x: Distance,
        y: Distance,
        z: Distance,
        vx: Velocity,
        vy: Velocity,
        vz: Velocity,
    ) -> Self {
        Self {
            pos: DVec3::new(x.to_meters(), y.to_meters(), z.to_meters()),
            vel: DVec3::new(
                vx.to_meters_per_second(),
                vy.to_meters_per_second(),
                vz.to_meters_per_second(),
            ),
        }
    }

    /// Creates a new Cartesian state from position and velocity vectors in meters and m/s.
    #[inline]
    pub const fn from_vecs(pos: DVec3, vel: DVec3) -> Self {
        Self { pos, vel }
    }

    /// Creates a new Cartesian state from a `[x, y, z, vx, vy, vz]` array in meters and m/s.
    pub const fn from_array([x, y, z, vx, vy, vz]: [f64; 6]) -> Self {
        Self {
            pos: DVec3::new(x, y, z),
            vel: DVec3::new(vx, vy, vz),
        }
    }

    /// Returns a new [`CartesianBuilder`].
    pub const fn builder() -> CartesianBuilder {
        CartesianBuilder::new()
    }

    /// Returns the position vector in meters.
    #[inline]
    pub fn position(&self) -> DVec3 {
        self.pos
    }

    /// Sets the position vector in meters.
    pub fn set_position(&mut self, position: DVec3) {
        self.pos = position
    }

    /// Returns the velocity vector in m/s.
    #[inline]
    pub fn velocity(&self) -> DVec3 {
        self.vel
    }

    /// Sets the velocity vector in m/s.
    pub fn set_velocity(&mut self, velocity: DVec3) {
        self.vel = velocity
    }

    /// Returns the x position component.
    pub fn x(&self) -> Distance {
        Distance::meters(self.pos.x)
    }

    /// Sets the `N`-th component (0–5 for x, y, z, vx, vy, vz) to `value`.
    pub fn set<const N: usize>(&mut self, value: f64) {
        const { assert!(N < 6, "index out of bounds") }

        match N {
            0 => {
                self.pos.x = value;
            }
            1 => {
                self.pos.y = value;
            }
            2 => {
                self.pos.z = value;
            }
            3 => {
                self.vel.x = value;
            }
            4 => {
                self.vel.y = value;
            }
            5 => {
                self.vel.z = value;
            }
            _ => unreachable!(),
        }
    }

    /// Returns the y position component.
    pub fn y(&self) -> Distance {
        Distance::meters(self.pos.y)
    }

    /// Returns the z position component.
    pub fn z(&self) -> Distance {
        Distance::meters(self.pos.z)
    }

    /// Returns the x velocity component.
    pub fn vx(&self) -> Velocity {
        Velocity::meters_per_second(self.vel.x)
    }

    /// Returns the y velocity component.
    pub fn vy(&self) -> Velocity {
        Velocity::meters_per_second(self.vel.y)
    }

    /// Returns the z velocity component.
    pub fn vz(&self) -> Velocity {
        Velocity::meters_per_second(self.vel.z)
    }
}

/// Builder for constructing [`Cartesian`] states from unitful components.
#[derive(Debug, Default, Clone, Copy)]
pub struct CartesianBuilder {
    pos: Option<DVec3>,
    vel: Option<DVec3>,
}

impl CartesianBuilder {
    /// Creates a new builder with no position or velocity set.
    pub const fn new() -> Self {
        Self {
            pos: None,
            vel: None,
        }
    }

    /// Sets the position components.
    pub const fn position(&mut self, x: Distance, y: Distance, z: Distance) -> &mut Self {
        self.pos = Some(DVec3::new(x.to_meters(), y.to_meters(), z.to_meters()));
        self
    }

    /// Sets the velocity components.
    pub const fn velocity(&mut self, vx: Velocity, vy: Velocity, vz: Velocity) -> &mut Self {
        self.vel = Some(DVec3::new(
            vx.to_meters_per_second(),
            vy.to_meters_per_second(),
            vz.to_meters_per_second(),
        ));
        self
    }

    /// Builds the [`Cartesian`] state. Unset components default to zero.
    pub const fn build(&self) -> Cartesian {
        Cartesian {
            pos: match self.pos {
                Some(pos) => pos,
                None => DVec3::ZERO,
            },
            vel: match self.vel {
                Some(vel) => vel,
                None => DVec3::ZERO,
            },
        }
    }
}

impl Add for Cartesian {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_vecs(self.pos + rhs.pos, self.vel + rhs.vel)
    }
}

impl AddAssign for Cartesian {
    fn add_assign(&mut self, rhs: Self) {
        self.pos += rhs.pos;
        self.vel += rhs.vel;
    }
}

impl Sub for Cartesian {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_vecs(self.pos - rhs.pos, self.vel - rhs.vel)
    }
}

impl SubAssign for Cartesian {
    fn sub_assign(&mut self, rhs: Self) {
        self.pos -= rhs.pos;
        self.vel -= rhs.vel;
    }
}

impl Mul<f64> for Cartesian {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            pos: self.pos * rhs,
            vel: self.vel * rhs,
        }
    }
}

impl MulAssign<f64> for Cartesian {
    fn mul_assign(&mut self, rhs: f64) {
        self.pos *= rhs;
        self.vel *= rhs;
    }
}

impl Div<f64> for Cartesian {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            pos: self.pos / rhs,
            vel: self.vel / rhs,
        }
    }
}

impl DivAssign<f64> for Cartesian {
    fn div_assign(&mut self, rhs: f64) {
        self.pos /= rhs;
        self.vel /= rhs;
    }
}

impl Neg for Cartesian {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from_vecs(-self.pos, -self.vel)
    }
}

/// Generic interpolated time series data with `N` components.
#[derive(Debug, Clone)]
pub struct TrajectoryData<const N: usize> {
    epoch: TimeDelta,
    time_steps: Arc<[f64]>,
    data: [Arc<[f64]>; N],
    series: [Series; N],
}

impl<const N: usize> TrajectoryData<N> {
    /// Creates trajectory data from fixed-size arrays of time steps and component data.
    pub fn from_arrays<const M: usize>(
        epoch: TimeDelta,
        time_steps: [TimeDelta; M],
        data: &[[f64; M]; N],
    ) -> Self {
        let time_steps: Arc<[f64]> = Arc::from_iter(
            time_steps
                .into_iter()
                .map(|t| (t - epoch).to_seconds().to_f64()),
        );
        let data: [Arc<[f64]>; N] = data.map(Arc::from);
        let series = data.clone().map(|d| {
            Series::new(
                time_steps.clone(),
                d.clone(),
                InterpolationType::CubicSpline,
            )
        });
        Self {
            epoch,
            time_steps,
            data,
            series,
        }
    }

    /// Returns the time steps relative to the epoch in seconds.
    pub fn time_steps(&self) -> Arc<[f64]> {
        self.time_steps.clone()
    }

    /// Interpolates the `M`-th component at time `t` (seconds since epoch).
    #[inline]
    pub fn interpolate<const M: usize>(&self, t: f64) -> f64 {
        const { assert!(M < N, "index is out-of-bounds") }

        self.series[M].interpolate(t)
    }

    /// Interpolates all `N` components at time `t` (seconds since epoch).
    #[inline]
    pub fn interpolate_all(&self, t: f64) -> [f64; N] {
        let idx = self.series[0].find_index(t);
        self.series
            .each_ref()
            .map(|s| s.interpolate_at_index(t, idx))
    }
}

/// A Cartesian state paired with a timestamp.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeStampedCartesian {
    /// The timestamp of the state.
    pub time: TimeDelta,
    /// The Cartesian state vector.
    pub state: Cartesian,
}

/// A 6-component trajectory (x, y, z, vx, vy, vz) with cubic spline interpolation.
pub type CartesianTrajectory = TrajectoryData<6>;

impl CartesianTrajectory {
    /// Creates a trajectory from an iterator of timestamped Cartesian states.
    pub fn from_states(states: impl IntoIterator<Item = TimeStampedCartesian>) -> Self {
        let mut iter = states.into_iter().peekable();
        let epoch = iter.peek().expect("should have at least two states").time;
        let _ = iter.peek().expect("should have at least two states");
        let (n, _) = iter.size_hint();

        let mut time_steps: Vec<f64> = Vec::with_capacity(n);
        let mut x: Vec<f64> = Vec::with_capacity(n);
        let mut y: Vec<f64> = Vec::with_capacity(n);
        let mut z: Vec<f64> = Vec::with_capacity(n);
        let mut vx: Vec<f64> = Vec::with_capacity(n);
        let mut vy: Vec<f64> = Vec::with_capacity(n);
        let mut vz: Vec<f64> = Vec::with_capacity(n);

        iter.for_each(|TimeStampedCartesian { time, state }| {
            time_steps.push((time - epoch).to_seconds().to_f64());
            x.push(state.x().as_f64());
            y.push(state.y().as_f64());
            z.push(state.z().as_f64());
            vx.push(state.vx().as_f64());
            vy.push(state.vy().as_f64());
            vz.push(state.vz().as_f64());
        });

        let time_steps: Arc<[f64]> = Arc::from(time_steps);

        let x: Arc<[f64]> = Arc::from(x);
        let y: Arc<[f64]> = Arc::from(y);
        let z: Arc<[f64]> = Arc::from(z);
        let vx: Arc<[f64]> = Arc::from(vx);
        let vy: Arc<[f64]> = Arc::from(vy);
        let vz: Arc<[f64]> = Arc::from(vz);

        let data = [
            x.clone(),
            y.clone(),
            z.clone(),
            vx.clone(),
            vy.clone(),
            vz.clone(),
        ];

        let series = data.clone().map(|d| {
            Series::new(
                time_steps.clone(),
                d.clone(),
                InterpolationType::CubicSpline,
            )
        });

        Self {
            epoch,
            time_steps,
            data,
            series,
        }
    }

    /// Returns the x position data points.
    pub fn x(&self) -> Arc<[f64]> {
        self.data[0].clone()
    }

    /// Returns the y position data points.
    pub fn y(&self) -> Arc<[f64]> {
        self.data[1].clone()
    }

    /// Returns the z position data points.
    pub fn z(&self) -> Arc<[f64]> {
        self.data[2].clone()
    }

    /// Returns the x velocity data points.
    pub fn vx(&self) -> Arc<[f64]> {
        self.data[3].clone()
    }

    /// Returns the y velocity data points.
    pub fn vy(&self) -> Arc<[f64]> {
        self.data[4].clone()
    }

    /// Returns the z velocity data points.
    pub fn vz(&self) -> Arc<[f64]> {
        self.data[5].clone()
    }

    /// Interpolates the x position at time `t` (seconds since epoch).
    #[inline]
    pub fn interpolate_x(&self, t: f64) -> f64 {
        self.interpolate::<0>(t)
    }

    /// Interpolates the y position at time `t` (seconds since epoch).
    #[inline]
    pub fn interpolate_y(&self, t: f64) -> f64 {
        self.interpolate::<1>(t)
    }

    /// Interpolates the z position at time `t` (seconds since epoch).
    #[inline]
    pub fn interpolate_z(&self, t: f64) -> f64 {
        self.interpolate::<2>(t)
    }

    /// Interpolates the x velocity at time `t` (seconds since epoch).
    #[inline]
    pub fn interpolate_vx(&self, t: f64) -> f64 {
        self.interpolate::<3>(t)
    }

    /// Interpolates the y velocity at time `t` (seconds since epoch).
    #[inline]
    pub fn interpolate_vy(&self, t: f64) -> f64 {
        self.interpolate::<4>(t)
    }

    /// Interpolates the z velocity at time `t` (seconds since epoch).
    #[inline]
    pub fn interpolate_vz(&self, t: f64) -> f64 {
        self.interpolate::<5>(t)
    }

    /// Interpolates the position vector at time `t` (seconds since epoch).
    #[inline]
    pub fn position(&self, t: f64) -> DVec3 {
        let idx = self.series[0].find_index(t);
        DVec3::new(
            self.series[0].interpolate_at_index(t, idx),
            self.series[1].interpolate_at_index(t, idx),
            self.series[2].interpolate_at_index(t, idx),
        )
    }

    /// Interpolates the velocity vector at time `t` (seconds since epoch).
    #[inline]
    pub fn velocity(&self, t: f64) -> DVec3 {
        let idx = self.series[3].find_index(t);
        DVec3::new(
            self.series[3].interpolate_at_index(t, idx),
            self.series[4].interpolate_at_index(t, idx),
            self.series[5].interpolate_at_index(t, idx),
        )
    }

    /// Interpolates the full Cartesian state at time `t` (seconds since epoch).
    #[inline]
    pub fn at(&self, t: f64) -> Cartesian {
        let vals = self.interpolate_all(t);
        Cartesian::from_array(vals)
    }
}

/// Iterator over the discrete states in a [`CartesianTrajectory`].
pub struct CartesianTrajectoryIterator {
    data: CartesianTrajectory,
    curr: usize,
}

impl CartesianTrajectoryIterator {
    fn new(data: CartesianTrajectory) -> Self {
        Self { data, curr: 0 }
    }

    fn len(&self) -> usize {
        self.data.time_steps.len()
    }

    fn get_item(&self, idx: usize) -> Option<TimeStampedCartesian> {
        let n = self.len();
        if idx >= n {
            return None;
        }

        let time = self.data.time_steps[idx];
        let state = Cartesian::from_array(self.data.data.clone().map(|d| d[idx]));
        Some(TimeStampedCartesian {
            time: self.data.epoch + TimeDelta::from_seconds_f64(time),
            state,
        })
    }
}

impl Iterator for CartesianTrajectoryIterator {
    type Item = TimeStampedCartesian;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.get_item(self.curr);
        self.curr += 1;
        item
    }
}

impl IntoIterator for CartesianTrajectory {
    type Item = TimeStampedCartesian;

    type IntoIter = CartesianTrajectoryIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl FromIterator<TimeStampedCartesian> for CartesianTrajectory {
    fn from_iter<T: IntoIterator<Item = TimeStampedCartesian>>(iter: T) -> Self {
        TrajectoryData::from_states(iter)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::units::{AngleUnits, DistanceUnits, VelocityUnits};

    use super::*;

    #[test]
    fn test_azel_api() {
        let azel = AzEl::builder()
            .azimuth(45.0.deg())
            .elevation(45.0.deg())
            .build()
            .unwrap();
        assert_eq!(azel.azimuth(), 45.0.deg());
        assert_eq!(azel.elevation(), 45.0.deg());
    }

    #[rstest]
    #[case(0.0.deg(), 0.0.deg(), Ok(AzEl(0.0.deg(), 0.0.deg())))]
    #[case(-1.0.deg(), 0.0.deg(), Err(AzElError::InvalidAzimuth(-1.0.deg())))]
    #[case(361.0.deg(), 0.0.deg(), Err(AzElError::InvalidAzimuth(361.0.deg())))]
    #[case(0.0.deg(), -1.0.deg(), Err(AzElError::InvalidElevation(-1.0.deg())))]
    #[case(0.0.deg(), 361.0.deg(), Err(AzElError::InvalidElevation(361.0.deg())))]
    fn test_azel(#[case] az: Angle, #[case] el: Angle, #[case] exp: Result<AzEl, AzElError>) {
        let act = AzEl::builder().azimuth(az).elevation(el).build();
        assert_eq!(act, exp)
    }

    #[test]
    fn test_lla_api() {
        let lla = LonLatAlt::builder()
            .longitude(45.0.deg())
            .latitude(45.0.deg())
            .altitude(100.0.m())
            .build()
            .unwrap();
        assert_eq!(lla.lon(), 45.0.deg());
        assert_eq!(lla.lat(), 45.0.deg());
        assert_eq!(lla.alt(), 100.0.m());
    }

    #[rstest]
    #[case(0.0.deg(), 0.0.deg(), 0.0.m(), Ok(LonLatAlt(0.0.deg(), 0.0.deg(), 0.0.m())))]
    #[case(-181.0.deg(), 0.0.deg(), 0.0.m(), Err(LonLatAltError::InvalidLongitude(-181.0.deg())))]
    #[case(181.0.deg(), 0.0.deg(), 0.0.m(), Err(LonLatAltError::InvalidLongitude(181.0.deg())))]
    #[case(0.0.deg(), -91.0.deg(), 0.0.m(), Err(LonLatAltError::InvalidLatitude(-91.0.deg())))]
    #[case(0.0.deg(), 91.0.deg(), 0.0.m(), Err(LonLatAltError::InvalidLatitude(91.0.deg())))]
    #[case(0.0.deg(), 0.0.deg(), f64::INFINITY.m(), Err(LonLatAltError::InvalidAltitude(f64::INFINITY.m())))]
    fn test_lla(
        #[case] lon: Angle,
        #[case] lat: Angle,
        #[case] alt: Distance,
        #[case] exp: Result<LonLatAlt, LonLatAltError>,
    ) {
        let act = LonLatAlt::builder()
            .longitude(lon)
            .latitude(lat)
            .altitude(alt)
            .build();
        assert_eq!(act, exp)
    }

    #[test]
    fn test_cartesian() {
        let c = Cartesian::builder()
            .position(1000.0.km(), 1000.0.km(), 1000.0.km())
            .velocity(1.0.kps(), 1.0.kps(), 1.0.kps())
            .build();
        assert_eq!(c.position(), DVec3::new(1e6, 1e6, 1e6));
        assert_eq!(c.velocity(), DVec3::new(1e3, 1e3, 1e3));
        assert_eq!(c.x(), 1e6.m());
        assert_eq!(c.y(), 1e6.m());
        assert_eq!(c.z(), 1e6.m());
        assert_eq!(c.vx(), 1e3.mps());
        assert_eq!(c.vy(), 1e3.mps());
        assert_eq!(c.vz(), 1e3.mps());
    }

    // Earth constants matching lox-bodies generated values
    const EARTH_R_EQ: f64 = 6378136.6; // meters (6378.1366 km)
    const EARTH_F: f64 = (6378.1366 - 6356.7519) / 6378.1366;

    #[test]
    fn test_lla_to_body_fixed() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let r_eq = Distance::meters(EARTH_R_EQ);
        let result = coords.to_body_fixed(r_eq, EARTH_F);
        let expected = DVec3::new(4846130.017870638, -370132.8551351891, 4116364.272747229);
        assert!((result - expected).length() < 1e-3);
    }

    #[test]
    fn test_lla_from_body_fixed_roundtrip() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 100.0).unwrap();
        let r_eq = Distance::meters(EARTH_R_EQ);
        let body_fixed = coords.to_body_fixed(r_eq, EARTH_F);
        let roundtrip = LonLatAlt::from_body_fixed(body_fixed, r_eq, EARTH_F).unwrap();
        assert!((roundtrip.lon().to_degrees() - coords.lon().to_degrees()).abs() < 1e-6);
        assert!((roundtrip.lat().to_degrees() - coords.lat().to_degrees()).abs() < 1e-6);
        assert!((roundtrip.alt().to_meters() - coords.alt().to_meters()).abs() < 1e-3);
    }

    #[test]
    fn test_lla_rotation_to_topocentric() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
        let act = coords.rotation_to_topocentric();
        let exp = DMat3::from_cols(
            DVec3::new(0.6469358921661584, 0.07615519584215287, 0.7587320591443464),
            DVec3::new(
                -0.049411020334552434,
                0.9970959763965771,
                -0.05794967578213965,
            ),
            DVec3::new(-0.7609418522440956, 0.0, 0.6488200809957448),
        );
        // Check element-wise within tolerance
        for i in 0..3 {
            assert!((act.col(i) - exp.col(i)).length() < 1e-10);
        }
    }

    #[test]
    fn test_from_body_fixed_north_pole() {
        let r_eq = Distance::meters(EARTH_R_EQ);
        let e = (2.0 * EARTH_F - EARTH_F.powi(2)).sqrt();
        let r_polar = EARTH_R_EQ * (1.0 - e.powi(2)).sqrt();
        let pos = DVec3::new(0.0, 0.0, r_polar);
        let result = LonLatAlt::from_body_fixed(pos, r_eq, EARTH_F).unwrap();
        assert!((result.lat().to_degrees() - 90.0).abs() < 1e-10);
        assert!(result.alt().to_meters().abs() < 1e-3);
    }

    #[test]
    fn test_from_body_fixed_south_pole() {
        let r_eq = Distance::meters(EARTH_R_EQ);
        let e = (2.0 * EARTH_F - EARTH_F.powi(2)).sqrt();
        let r_polar = EARTH_R_EQ * (1.0 - e.powi(2)).sqrt();
        let pos = DVec3::new(0.0, 0.0, -r_polar - 1000.0);
        let result = LonLatAlt::from_body_fixed(pos, r_eq, EARTH_F).unwrap();
        assert!((result.lat().to_degrees() + 90.0).abs() < 1e-10);
        assert!((result.alt().to_meters() - 1000.0).abs() < 1e-3);
    }

    #[test]
    fn test_from_body_fixed_zero_position() {
        let r_eq = Distance::meters(EARTH_R_EQ);
        let pos = DVec3::ZERO;
        let result = LonLatAlt::from_body_fixed(pos, r_eq, EARTH_F);
        assert!(matches!(result, Err(FromBodyFixedError::ZeroPosition)));
    }
}
