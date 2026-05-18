// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2013-2021 NumFOCUS Foundation
//
// SPDX-License-Identifier: MPL-2.0 AND LicenseRef-ERFA

//! Coordinate types for representing positions, velocities, and trajectories.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::f64::consts::{FRAC_PI_2, TAU};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use glam::{DMat3, DVec3};
use lox_test_utils::ApproxEq;

use crate::math::float::{abs, atan2, powi, sin_cos, sqrt};
use thiserror::Error;

use crate::{
    math::series::{InterpolationType, Series},
    time::deltas::TimeDelta,
    units::{Angle, AngularRate, Distance, Velocity},
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

/// Reference ellipsoid (rotational, oblate) defined by its equatorial
/// radius and flattening. Construction validates that `equatorial_radius
/// > 0` and `flattening ∈ [0, 1)`.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ellipsoid {
    equatorial_radius: Distance,
    flattening: f64,
}

impl Ellipsoid {
    /// Creates an `Ellipsoid` from equatorial radius and flattening.
    ///
    /// `const fn` for compile-time use (e.g., associated constants).
    /// **Panics** if `equatorial_radius <= 0` or `flattening` is not in
    /// `[0, 1)`. Use [`Ellipsoid::try_new`] for runtime values that may
    /// not satisfy the invariants.
    pub const fn new(equatorial_radius: Distance, flattening: f64) -> Self {
        assert!(
            equatorial_radius.to_meters() > 0.0,
            "equatorial radius must be > 0"
        );
        assert!(
            flattening >= 0.0 && flattening < 1.0,
            "flattening must be in [0, 1)"
        );
        Self {
            equatorial_radius,
            flattening,
        }
    }

    /// Fallible constructor for runtime values.
    pub fn try_new(equatorial_radius: Distance, flattening: f64) -> Result<Self, EllipsoidError> {
        if equatorial_radius.to_meters() <= 0.0 {
            return Err(EllipsoidError::InvalidEquatorialRadius(equatorial_radius));
        }
        if !(0.0..1.0).contains(&flattening) {
            return Err(EllipsoidError::InvalidFlattening(flattening));
        }
        Ok(Self {
            equatorial_radius,
            flattening,
        })
    }

    /// Returns the equatorial radius.
    pub const fn equatorial_radius(&self) -> Distance {
        self.equatorial_radius
    }

    /// Returns the flattening factor.
    pub const fn flattening(&self) -> f64 {
        self.flattening
    }

    /// WGS84 reference ellipsoid (`a = 6378137.0 m`, `f ≈ 1/298.257223563`).
    pub const WGS84: Self = Self::new(Distance::meters(6378137.0), 1.0 / 298.257223563);
    /// GRS80 reference ellipsoid (`a = 6378137.0 m`, `f ≈ 1/298.257222101`).
    pub const GRS80: Self = Self::new(Distance::meters(6378137.0), 1.0 / 298.257222101);
    /// WGS72 reference ellipsoid (`a = 6378135.0 m`, `f ≈ 1/298.26`).
    pub const WGS72: Self = Self::new(Distance::meters(6378135.0), 1.0 / 298.26);
}

/// Error returned when constructing an [`Ellipsoid`] with invalid parameters.
#[derive(Copy, Clone, Debug, Error, PartialEq)]
pub enum EllipsoidError {
    /// Equatorial radius must be strictly positive.
    #[error("equatorial radius must be > 0, got {0}")]
    InvalidEquatorialRadius(Distance),
    /// Flattening must be in `[0, 1)`.
    #[error("flattening must be in [0, 1), got {0}")]
    InvalidFlattening(f64),
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

    /// Converts geodetic coordinates (LLA) to body-fixed Cartesian position
    /// in meters.
    ///
    /// # References
    ///
    /// - ERFA [`gd2gce`](https://github.com/liberfa/erfa/blob/master/src/gd2gce.c)
    pub fn to_body_fixed(&self, ellipsoid: &Ellipsoid) -> DVec3 {
        let alt = self.alt().to_meters();
        let (lon_sin, lon_cos) = self.lon().sin_cos();
        let (lat_sin, lat_cos) = self.lat().sin_cos();
        let r_eq = ellipsoid.equatorial_radius().to_meters();
        let f = ellipsoid.flattening();
        let e = sqrt(2.0 * f - powi(f, 2));
        let c = r_eq / sqrt(1.0 - powi(e, 2) * powi(lat_sin, 2));
        let s = c * (1.0 - powi(e, 2));
        let r_delta = (c + alt) * lat_cos;
        let r_kappa = (s + alt) * lat_sin;
        DVec3::new(r_delta * lon_cos, r_delta * lon_sin, r_kappa)
    }

    /// Converts a body-fixed Cartesian position (meters) to geodetic coordinates (LLA).
    ///
    /// # Errors
    ///
    /// - [`FromBodyFixedError::ZeroPosition`] if `pos` is the zero vector.
    ///
    /// # References
    ///
    /// - ERFA [`gc2gde`](https://github.com/liberfa/erfa/blob/master/src/gc2gde.c)
    pub fn from_body_fixed(pos: DVec3, ellipsoid: &Ellipsoid) -> Result<Self, FromBodyFixedError> {
        let f = ellipsoid.flattening();
        let a = ellipsoid.equatorial_radius().to_meters();

        // Only exact zero is rejected. Small but non-zero positions are
        // routed to the polar case by ERFA `gc2gde`'s `p² > aeps2` test
        // below; the Halley iteration is numerically stable there. This
        // matches ERFA; the previous secant-based implementation used a
        // 1e-10 m tolerance, which is intentionally relaxed.
        if pos.length_squared() == 0.0 {
            return Err(FromBodyFixedError::ZeroPosition);
        }

        // Functions of ellipsoid parameters.
        let aeps2 = a * a * 1e-32;
        let e2 = (2.0 - f) * f;
        let e4t = e2 * e2 * 1.5;
        // f ∈ [0, 1) implies e2 < 1 implies ec2 > 0; no guard needed.
        let ec2 = 1.0 - e2;
        let ec = sqrt(ec2);
        let b = a * ec;

        let x = pos.x;
        let y = pos.y;
        let z = pos.z;
        let p2 = x * x + y * y;

        // Longitude.
        let lon = if p2 > 0.0 { atan2(y, x) } else { 0.0 };

        // Unsigned z.
        let absz = abs(z);

        // Latitude and height.
        let (mut phi, height) = if p2 > aeps2 {
            // General case.
            let p = sqrt(p2);
            let s0 = absz / a;
            let pn = p / a;
            let zc = ec * s0;

            // Newton correction factors.
            let c0 = ec * pn;
            let c02 = c0 * c0;
            let c03 = c02 * c0;
            let s02 = s0 * s0;
            let s03 = s02 * s0;
            let a02 = c02 + s02;
            let a0 = sqrt(a02);
            let a03 = a02 * a0;
            let d0 = zc * a03 + e2 * s03;
            let f0 = pn * a03 - e2 * c03;

            // Halley correction factor.
            let b0 = e4t * s02 * c02 * pn * (a0 - ec);
            let s1 = d0 * f0 - b0 * s0;
            let cc = ec * (f0 * f0 - b0 * c0);

            let phi = atan2(s1, cc);
            let s12 = s1 * s1;
            let cc2 = cc * cc;
            let height = (p * cc + absz * s1 - a * sqrt(ec2 * s12 + cc2)) / sqrt(s12 + cc2);
            (phi, height)
        } else {
            // Polar case.
            (FRAC_PI_2, absz - b)
        };

        // Restore sign of latitude.
        if z < 0.0 {
            phi = -phi;
        }

        Ok(LonLatAlt(
            Angle::radians(lon),
            Angle::radians(phi),
            Distance::meters(height),
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

/// Error returned by [`LonLatAlt::from_body_fixed`].
#[derive(Copy, Clone, Debug, Error, PartialEq)]
pub enum FromBodyFixedError {
    /// The position vector has zero length.
    #[error("position vector has zero length")]
    ZeroPosition,
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

/// Spherical coordinates: longitude (θ), latitude (φ), and radius.
///
/// `lon` is not auto-normalized (callers choose whether to work in
/// `[-π, π]` or `[0, 2π]`). `lat` is expected in `[-π/2, π/2]` but is not
/// enforced.
#[derive(Copy, Clone, Debug, Default, PartialEq, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Spherical {
    lon: Angle,
    lat: Angle,
    r: Distance,
}

impl Spherical {
    /// Creates new spherical coordinates.
    pub const fn new(lon: Angle, lat: Angle, r: Distance) -> Self {
        Self { lon, lat, r }
    }

    /// Returns the longitude (θ).
    pub fn lon(&self) -> Angle {
        self.lon
    }

    /// Returns the latitude (φ).
    pub fn lat(&self) -> Angle {
        self.lat
    }

    /// Returns the radius.
    pub fn r(&self) -> Distance {
        self.r
    }

    /// Converts spherical direction (θ, φ) to a unit Cartesian vector.
    /// `r` is ignored.
    ///
    /// # References
    ///
    /// - ERFA [`s2c`](https://github.com/liberfa/erfa/blob/master/src/s2c.c)
    pub fn to_unit_vector(&self) -> DVec3 {
        let (sin_lat, cos_lat) = self.lat.sin_cos();
        let (sin_lon, cos_lon) = self.lon.sin_cos();
        DVec3::new(cos_lon * cos_lat, sin_lon * cos_lat, sin_lat)
    }

    /// Converts spherical (θ, φ, r) to a Cartesian position vector in
    /// meters (`r * to_unit_vector()`).
    ///
    /// # References
    ///
    /// - ERFA [`s2p`](https://github.com/liberfa/erfa/blob/master/src/s2p.c)
    pub fn to_cartesian(&self) -> DVec3 {
        self.to_unit_vector() * self.r.to_meters()
    }

    /// Converts a Cartesian position vector to spherical (θ, φ, r).
    ///
    /// At the origin returns `(0, 0, 0)`. On the polar axis (`x=y=0`)
    /// returns `lon = 0` and `lat = ±π/2`.
    ///
    /// # References
    ///
    /// - ERFA [`p2s`](https://github.com/liberfa/erfa/blob/master/src/p2s.c)
    pub fn from_cartesian(pos: DVec3) -> Self {
        let xy2 = pos.x * pos.x + pos.y * pos.y;
        let lon = if xy2 == 0.0 { 0.0 } else { atan2(pos.y, pos.x) };
        let lat = if xy2 == 0.0 && pos.z == 0.0 {
            0.0
        } else {
            atan2(pos.z, sqrt(xy2))
        };
        let r = sqrt(xy2 + pos.z * pos.z);
        Self {
            lon: Angle::radians(lon),
            lat: Angle::radians(lat),
            r: Distance::meters(r),
        }
    }
}

/// Spherical state: position in spherical coordinates and its first
/// time derivative.
#[derive(Copy, Clone, Debug, Default, PartialEq, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SphericalState {
    pos: Spherical,
    lon_dot: AngularRate,
    lat_dot: AngularRate,
    r_dot: Velocity,
}

impl SphericalState {
    /// Creates a new spherical state.
    pub const fn new(
        pos: Spherical,
        lon_dot: AngularRate,
        lat_dot: AngularRate,
        r_dot: Velocity,
    ) -> Self {
        Self {
            pos,
            lon_dot,
            lat_dot,
            r_dot,
        }
    }

    /// Returns the spherical position component.
    pub fn position(&self) -> Spherical {
        self.pos
    }

    /// Returns the rate of change of longitude.
    pub fn lon_dot(&self) -> AngularRate {
        self.lon_dot
    }

    /// Returns the rate of change of latitude.
    pub fn lat_dot(&self) -> AngularRate {
        self.lat_dot
    }

    /// Returns the radial velocity (rate of change of radius).
    pub fn r_dot(&self) -> Velocity {
        self.r_dot
    }

    /// Converts spherical state to a Cartesian state (position + velocity)
    /// in meters and m/s.
    ///
    /// # References
    ///
    /// - ERFA [`s2pv`](https://github.com/liberfa/erfa/blob/master/src/s2pv.c)
    pub fn to_cartesian(&self) -> Cartesian {
        let theta = self.pos.lon.to_radians();
        let phi = self.pos.lat.to_radians();
        let r = self.pos.r.to_meters();
        let td = self.lon_dot.to_radians_per_second();
        let pd = self.lat_dot.to_radians_per_second();
        let rd = self.r_dot.to_meters_per_second();

        let (st, ct) = sin_cos(theta);
        let (sp, cp) = sin_cos(phi);
        let rcp = r * cp;
        let x = rcp * ct;
        let y = rcp * st;
        let rpd = r * pd;
        let w = rpd * sp - cp * rd;
        let vx = -y * td - w * ct;
        let vy = x * td - w * st;
        let vz = rpd * cp + sp * rd;
        let z = r * sp;

        Cartesian::from_vecs(DVec3::new(x, y, z), DVec3::new(vx, vy, vz))
    }

    /// Converts a Cartesian state to a spherical state.
    ///
    /// Degenerate cases:
    ///
    /// - At the origin (`position == 0`), spatial direction is taken from
    ///   the velocity vector; the returned radius is 0.
    /// - On the polar axis (`x == y == 0`), the returned longitude is 0
    ///   and `lon_dot` is 0; `lat_dot` is derived from the velocity.
    ///
    /// # References
    ///
    /// - ERFA [`pv2s`](https://github.com/liberfa/erfa/blob/master/src/pv2s.c)
    pub fn from_cartesian(state: Cartesian) -> Self {
        let p = state.position();
        let v = state.velocity();
        let x = p.x;
        let y = p.y;
        let z = p.z;
        let xd = v.x;
        let yd = v.y;
        let zd = v.z;

        let rxy2 = x * x + y * y;
        let r2 = rxy2 + z * z;
        let rtrue = sqrt(r2);

        // Degenerate case: at the origin, take direction from the velocity.
        let (rw, p_x, p_y, p_z) = if rtrue == 0.0 {
            let rw = sqrt(xd * xd + yd * yd + zd * zd);
            (if rw == 0.0 { 1.0 } else { rw }, xd, yd, zd)
        } else {
            (rtrue, x, y, z)
        };

        let rxy2 = p_x * p_x + p_y * p_y;
        let rxy = sqrt(rxy2);
        let xyp = p_x * xd + p_y * yd;
        let (lon_dot, lat_dot) = if rxy2 != 0.0 {
            let lon_dot = (p_x * yd - p_y * xd) / rxy2;
            let lat_dot = (zd * rxy2 - p_z * xyp) / (rxy * rw * rw);
            (lon_dot, lat_dot)
        } else {
            // On the polar axis.
            let lat_dot = if p_z != 0.0 { -xyp / (p_z * p_z) } else { 0.0 };
            (0.0, lat_dot)
        };
        let r_dot = (xyp + p_z * zd) / rw;

        let lon = if rxy2 == 0.0 { 0.0 } else { atan2(p_y, p_x) };
        let lat = if rxy == 0.0 && p_z == 0.0 {
            0.0
        } else {
            atan2(p_z, rxy)
        };

        Self {
            pos: Spherical {
                lon: Angle::radians(lon),
                lat: Angle::radians(lat),
                r: Distance::meters(rtrue),
            },
            lon_dot: AngularRate::radians_per_second(lon_dot),
            lat_dot: AngularRate::radians_per_second(lat_dot),
            r_dot: Velocity::meters_per_second(r_dot),
        }
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

/// Serde helper for `[T; N]` with const-generic `N`.
///
/// The derive macro cannot handle `[T; N]` when `N` is a const generic parameter,
/// so we serialize/deserialize as a `Vec<T>` on the wire.
#[cfg(feature = "serde")]
mod const_array_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S, T, const N: usize>(arr: &[T; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        arr.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let vec = Vec::<T>::deserialize(deserializer)?;
        vec.try_into().map_err(|v: Vec<T>| {
            serde::de::Error::custom(format!("expected array of length {N}, got {}", v.len()))
        })
    }
}

/// Generic interpolated time series data with `N` components.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrajectoryData<const N: usize> {
    epoch: TimeDelta,
    time_steps: Arc<[f64]>,
    #[cfg_attr(feature = "serde", serde(with = "const_array_serde"))]
    data: [Arc<[f64]>; N],
    #[cfg_attr(feature = "serde", serde(with = "const_array_serde"))]
    series: [Series; N],
}

impl<const N: usize> TrajectoryData<N> {
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

/// A 6-component trajectory (x, y, z, vx, vy, vz) with Hermite cubic interpolation.
///
/// Position components use Hermite cubic splines constructed from both position
/// values and velocity derivatives at each knot point. Velocity is obtained as the
/// analytical derivative of the position spline, guaranteeing physical consistency
/// between interpolated position and velocity.
pub type CartesianTrajectory = TrajectoryData<6>;

impl CartesianTrajectory {
    /// Creates a trajectory from an iterator of timestamped Cartesian states.
    ///
    /// Position components (x, y, z) are interpolated with Hermite cubic splines
    /// using velocity (vx, vy, vz) as the known derivatives. Velocity components
    /// are stored as cubic splines for raw data access but the [`velocity`] and
    /// [`at`] methods derive velocity from the position spline derivative.
    ///
    /// # Panics
    ///
    /// Panics if the iterator yields fewer than 2 states. Hermite cubic
    /// interpolation requires at least two knots; callers must enforce this
    /// invariant.
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

        // Position series use Hermite cubic interpolation with velocity as derivatives.
        let sx = Series::hermite_cubic(time_steps.clone(), x, vx.clone());
        let sy = Series::hermite_cubic(time_steps.clone(), y, vy.clone());
        let sz = Series::hermite_cubic(time_steps.clone(), z, vz.clone());

        // Velocity series use standard cubic splines (used only for raw data access).
        let svx = Series::new(time_steps.clone(), vx, InterpolationType::CubicSpline);
        let svy = Series::new(time_steps.clone(), vy, InterpolationType::CubicSpline);
        let svz = Series::new(time_steps.clone(), vz, InterpolationType::CubicSpline);

        let series = [sx, sy, sz, svx, svy, svz];

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
    ///
    /// Derived as the analytical derivative of the Hermite position spline.
    #[inline]
    pub fn interpolate_vx(&self, t: f64) -> f64 {
        let idx = self.series[0].find_index(t);
        self.series[0].derivative_at_index(t, idx)
    }

    /// Interpolates the y velocity at time `t` (seconds since epoch).
    ///
    /// Derived as the analytical derivative of the Hermite position spline.
    #[inline]
    pub fn interpolate_vy(&self, t: f64) -> f64 {
        let idx = self.series[1].find_index(t);
        self.series[1].derivative_at_index(t, idx)
    }

    /// Interpolates the z velocity at time `t` (seconds since epoch).
    ///
    /// Derived as the analytical derivative of the Hermite position spline.
    #[inline]
    pub fn interpolate_vz(&self, t: f64) -> f64 {
        let idx = self.series[2].find_index(t);
        self.series[2].derivative_at_index(t, idx)
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
    ///
    /// Velocity is derived as the analytical derivative of the Hermite position
    /// spline, ensuring physical consistency with the interpolated position.
    #[inline]
    pub fn velocity(&self, t: f64) -> DVec3 {
        let idx = self.series[0].find_index(t);
        DVec3::new(
            self.series[0].derivative_at_index(t, idx),
            self.series[1].derivative_at_index(t, idx),
            self.series[2].derivative_at_index(t, idx),
        )
    }

    /// Interpolates the full Cartesian state at time `t` (seconds since epoch).
    ///
    /// Position is interpolated from the Hermite spline; velocity is derived as
    /// the analytical derivative of the same spline.
    #[inline]
    pub fn at(&self, t: f64) -> Cartesian {
        let idx = self.series[0].find_index(t);
        let pos = DVec3::new(
            self.series[0].interpolate_at_index(t, idx),
            self.series[1].interpolate_at_index(t, idx),
            self.series[2].interpolate_at_index(t, idx),
        );
        let vel = DVec3::new(
            self.series[0].derivative_at_index(t, idx),
            self.series[1].derivative_at_index(t, idx),
            self.series[2].derivative_at_index(t, idx),
        );
        Cartesian::from_vecs(pos, vel)
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
    use lox_test_utils::assert_approx_eq;
    use rstest::rstest;

    use crate::units::{AngleUnits, AngularRate, DistanceUnits, VelocityUnits};

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
        let ellipsoid = Ellipsoid::new(Distance::meters(EARTH_R_EQ), EARTH_F);
        let result = coords.to_body_fixed(&ellipsoid);
        let expected = DVec3::new(4846130.017870638, -370132.8551351891, 4116364.272747229);
        assert!((result - expected).length() < 1e-3);
    }

    #[test]
    fn test_lla_from_body_fixed_roundtrip() {
        let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 100.0).unwrap();
        let ellipsoid = Ellipsoid::new(Distance::meters(EARTH_R_EQ), EARTH_F);
        let body_fixed = coords.to_body_fixed(&ellipsoid);
        let roundtrip = LonLatAlt::from_body_fixed(body_fixed, &ellipsoid).unwrap();
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
        let ellipsoid = Ellipsoid::new(Distance::meters(EARTH_R_EQ), EARTH_F);
        let e = (2.0 * EARTH_F - EARTH_F.powi(2)).sqrt();
        let r_polar = EARTH_R_EQ * (1.0 - e.powi(2)).sqrt();
        let pos = DVec3::new(0.0, 0.0, r_polar);
        let result = LonLatAlt::from_body_fixed(pos, &ellipsoid).unwrap();
        assert!((result.lat().to_degrees() - 90.0).abs() < 1e-10);
        assert!(result.alt().to_meters().abs() < 1e-3);
    }

    #[test]
    fn test_from_body_fixed_south_pole() {
        let ellipsoid = Ellipsoid::new(Distance::meters(EARTH_R_EQ), EARTH_F);
        let e = (2.0 * EARTH_F - EARTH_F.powi(2)).sqrt();
        let r_polar = EARTH_R_EQ * (1.0 - e.powi(2)).sqrt();
        let pos = DVec3::new(0.0, 0.0, -r_polar - 1000.0);
        let result = LonLatAlt::from_body_fixed(pos, &ellipsoid).unwrap();
        assert!((result.lat().to_degrees() + 90.0).abs() < 1e-10);
        assert!((result.alt().to_meters() - 1000.0).abs() < 1e-3);
    }

    #[test]
    fn test_from_body_fixed_zero_position() {
        let ellipsoid = Ellipsoid::new(Distance::meters(EARTH_R_EQ), EARTH_F);
        let pos = DVec3::ZERO;
        let result = LonLatAlt::from_body_fixed(pos, &ellipsoid);
        assert!(matches!(result, Err(FromBodyFixedError::ZeroPosition)));
    }

    #[test]
    fn test_from_body_fixed_equator() {
        // A point on the equator at WGS84-ish radius: z = 0, p² >> aeps2.
        // Expected: lat = 0, alt = a + h, lon = atan2(y, x).
        let ellipsoid = Ellipsoid::new(Distance::meters(EARTH_R_EQ), EARTH_F);
        let h = 100.0;
        let surface_radius = EARTH_R_EQ + h;
        let pos = DVec3::new(surface_radius, 0.0, 0.0);
        let lla = LonLatAlt::from_body_fixed(pos, &ellipsoid).unwrap();
        assert_approx_eq!(lla.lon().to_radians(), 0.0, atol <= 1e-12);
        assert_approx_eq!(lla.lat().to_radians(), 0.0, atol <= 1e-12);
        assert_approx_eq!(lla.alt().to_meters(), h, atol <= 1e-6);
    }

    #[test]
    fn test_spherical_api() {
        let s = Spherical::new(1.0.rad(), 0.5.rad(), 100.0.m());
        assert_eq!(s.lon(), 1.0.rad());
        assert_eq!(s.lat(), 0.5.rad());
        assert_eq!(s.r(), 100.0.m());
    }

    #[test]
    fn test_spherical_default_is_zero() {
        let s = Spherical::default();
        assert_eq!(s.lon(), Angle::ZERO);
        assert_eq!(s.lat(), Angle::ZERO);
        assert_eq!(s.r(), Distance::default());
    }

    #[test]
    fn test_spherical_to_unit_vector_erfa_s2c() {
        let s = Spherical::new(3.0123.rad(), (-0.999).rad(), 1.0.m());
        let c = s.to_unit_vector();
        assert_approx_eq!(c.x, -0.536_626_766_726_052_4, atol <= 1e-12);
        assert_approx_eq!(c.y, 0.069_771_110_976_514_53, atol <= 1e-12);
        assert_approx_eq!(c.z, -0.840_930_261_856_621_5, atol <= 1e-12);
    }

    #[test]
    fn test_spherical_to_cartesian_erfa_s2p() {
        let s = Spherical::new((-3.21).rad(), 0.123.rad(), 0.456.m());
        let p = s.to_cartesian();
        assert_approx_eq!(p.x, -0.451_496_467_388_016_5, atol <= 1e-12);
        assert_approx_eq!(p.y, 0.030_933_942_773_425_867, atol <= 1e-12);
        assert_approx_eq!(p.z, 0.055_946_681_051_087_79, atol <= 1e-12);
    }

    #[test]
    fn test_spherical_from_cartesian_erfa_p2s() {
        let p = DVec3::new(100.0, -50.0, 25.0);
        let s = Spherical::from_cartesian(p);
        assert_approx_eq!(
            s.lon().to_radians(),
            -0.463_647_609_000_806_1,
            atol <= 1e-12
        );
        assert_approx_eq!(
            s.lat().to_radians(),
            0.219_987_977_395_459_44,
            atol <= 1e-12
        );
        assert_approx_eq!(s.r().to_meters(), 114.564_392_373_896, atol <= 1e-9);
    }

    #[test]
    fn test_spherical_from_cartesian_origin() {
        let s = Spherical::from_cartesian(DVec3::ZERO);
        assert_eq!(s.lon(), Angle::ZERO);
        assert_eq!(s.lat(), Angle::ZERO);
        assert_eq!(s.r(), Distance::default());
    }

    #[test]
    fn test_spherical_from_cartesian_north_pole() {
        let s = Spherical::from_cartesian(DVec3::new(0.0, 0.0, 10.0));
        assert_eq!(s.lon(), Angle::ZERO);
        assert_approx_eq!(
            s.lat().to_radians(),
            core::f64::consts::FRAC_PI_2,
            atol <= 1e-12
        );
        assert_approx_eq!(s.r().to_meters(), 10.0, atol <= 1e-12);
    }

    #[test]
    fn test_spherical_from_cartesian_south_pole() {
        // On the -z axis: lat = -π/2, lon = 0.
        let s = Spherical::from_cartesian(DVec3::new(0.0, 0.0, -10.0));
        assert_eq!(s.lon(), Angle::ZERO);
        assert_approx_eq!(
            s.lat().to_radians(),
            -core::f64::consts::FRAC_PI_2,
            atol <= 1e-12
        );
        assert_approx_eq!(s.r().to_meters(), 10.0, atol <= 1e-12);
    }

    #[test]
    fn test_spherical_state_from_cartesian_erfa_pv2s() {
        let cart = Cartesian::from_array([
            -0.4514964673880165,
            0.03093394277342585,
            0.05594668105108779,
            1.292_270_850_663_26e-5,
            2.652814182060692e-6,
            2.568431853930293e-6,
        ]);
        let s = SphericalState::from_cartesian(cart);
        assert_approx_eq!(
            s.position().lon().to_radians(),
            3.073_185_307_179_586_7,
            atol <= 1e-12
        );
        assert_approx_eq!(s.position().lat().to_radians(), 0.123, atol <= 1e-12);
        assert_approx_eq!(
            s.position().r().to_meters(),
            0.455_999_999_999_999_96,
            atol <= 1e-12
        );
        assert_approx_eq!(s.lon_dot().to_radians_per_second(), -7.8e-6, atol <= 1e-16);
        assert_approx_eq!(
            s.lat_dot().to_radians_per_second(),
            9.010_000_000_000_002e-6,
            atol <= 1e-16
        );
        assert_approx_eq!(
            s.r_dot().to_meters_per_second(),
            -1.229_999_999_999_999_9e-5,
            atol <= 1e-16
        );
    }

    #[test]
    fn test_spherical_state_from_cartesian_polar_axis() {
        // On the +z axis with non-zero z. Exercises the rxy² == 0 branch:
        // lon = 0, lon_dot = 0, lat_dot = -xyp / z² where xyp = x*xd + y*yd = 0 here,
        // so all derivatives in xy contribute zero to lat_dot.
        let cart = Cartesian::from_array([0.0, 0.0, 10.0, 1.0, 2.0, 0.5]);
        let s = SphericalState::from_cartesian(cart);
        assert_eq!(s.position().lon(), Angle::ZERO);
        assert_approx_eq!(
            s.position().lat().to_radians(),
            core::f64::consts::FRAC_PI_2,
            atol <= 1e-12
        );
        assert_approx_eq!(s.position().r().to_meters(), 10.0, atol <= 1e-12);
        assert_eq!(s.lon_dot(), AngularRate::radians_per_second(0.0));
        assert_approx_eq!(s.lat_dot().to_radians_per_second(), 0.0, atol <= 1e-12);
        assert_approx_eq!(s.r_dot().to_meters_per_second(), 0.5, atol <= 1e-12);
    }

    #[test]
    fn test_spherical_state_to_cartesian_erfa_s2pv() {
        let pos = Spherical::new((-3.21).rad(), 0.123.rad(), 0.456.m());
        let state = SphericalState::new(
            pos,
            AngularRate::radians_per_second(-7.8e-6),
            AngularRate::radians_per_second(9.01e-6),
            Velocity::meters_per_second(-1.23e-5),
        );
        let c = state.to_cartesian();
        assert_approx_eq!(c.position().x, -0.451_496_467_388_016_5, atol <= 1e-12);
        assert_approx_eq!(c.position().y, 0.030_933_942_773_425_867, atol <= 1e-12);
        assert_approx_eq!(c.position().z, 0.055_946_681_051_087_79, atol <= 1e-12);
        assert_approx_eq!(c.velocity().x, 1.292_270_850_663_260_2e-5, atol <= 1e-16);
        assert_approx_eq!(c.velocity().y, 2.652_814_182_060_691_4e-6, atol <= 1e-16);
        assert_approx_eq!(c.velocity().z, 2.568_431_853_930_292e-6, atol <= 1e-16);
    }

    #[test]
    fn test_spherical_state_api() {
        let pos = Spherical::new(1.0.rad(), 0.5.rad(), 100.0.m());
        let s = SphericalState::new(
            pos,
            AngularRate::radians_per_second(1e-3),
            AngularRate::radians_per_second(2e-3),
            Velocity::meters_per_second(0.5),
        );
        assert_eq!(s.position(), pos);
        assert_eq!(s.lon_dot(), AngularRate::radians_per_second(1e-3));
        assert_eq!(s.lat_dot(), AngularRate::radians_per_second(2e-3));
        assert_eq!(s.r_dot(), Velocity::meters_per_second(0.5));
    }

    #[test]
    fn test_lla_to_body_fixed_erfa_gd2gce() {
        // ERFA t_erfa_c.c::t_gd2gce
        let lla = LonLatAlt::builder()
            .longitude(Angle::radians(3.1))
            .latitude(Angle::radians(-0.5))
            .altitude(Distance::meters(2500.0))
            .build()
            .unwrap();
        let ellipsoid = Ellipsoid::new(Distance::meters(6378136.0), 0.0033528);
        let xyz = lla.to_body_fixed(&ellipsoid);
        assert_approx_eq!(xyz.x, -5598999.6665116328, atol <= 1e-7);
        assert_approx_eq!(xyz.y, 233_011.635_146_305_72, atol <= 1e-7);
        assert_approx_eq!(xyz.z, -3_040_909.051_731_413, atol <= 1e-7);
    }

    #[test]
    fn test_lla_from_body_fixed_erfa_gc2gde() {
        // ERFA t_erfa_c.c::t_gc2gde
        let xyz = DVec3::new(2e6, 3e6, 5.244e6);
        let ellipsoid = Ellipsoid::new(Distance::meters(6378136.0), 0.0033528);
        let lla = LonLatAlt::from_body_fixed(xyz, &ellipsoid).unwrap();
        assert_approx_eq!(lla.lon().to_radians(), 0.982_793_723_247_329, atol <= 1e-14);
        assert_approx_eq!(
            lla.lat().to_radians(),
            0.971_601_837_757_041_1,
            atol <= 1e-14
        );
        assert_approx_eq!(lla.alt().to_meters(), 332.368_624_957_644, atol <= 1e-8);
    }

    #[test]
    fn test_ellipsoid_try_new_valid() {
        let e = Ellipsoid::try_new(Distance::meters(6378137.0), 1.0 / 298.257223563).unwrap();
        assert_eq!(e.equatorial_radius(), Distance::meters(6378137.0));
        assert_approx_eq!(e.flattening(), 1.0 / 298.257223563, atol <= 1e-15);
    }

    #[test]
    fn test_ellipsoid_try_new_invalid_radius() {
        assert!(matches!(
            Ellipsoid::try_new(Distance::meters(0.0), 0.003),
            Err(EllipsoidError::InvalidEquatorialRadius(_))
        ));
        assert!(matches!(
            Ellipsoid::try_new(Distance::meters(-1.0), 0.003),
            Err(EllipsoidError::InvalidEquatorialRadius(_))
        ));
    }

    #[test]
    fn test_ellipsoid_try_new_invalid_flattening() {
        assert!(matches!(
            Ellipsoid::try_new(Distance::meters(6378137.0), -0.1),
            Err(EllipsoidError::InvalidFlattening(_))
        ));
        assert!(matches!(
            Ellipsoid::try_new(Distance::meters(6378137.0), 1.0),
            Err(EllipsoidError::InvalidFlattening(_))
        ));
    }

    #[test]
    fn test_ellipsoid_constants_are_valid() {
        // const fn `new` would have panicked at compile time if invalid.
        // Just verify the values look right.
        assert_eq!(
            Ellipsoid::WGS84.equatorial_radius(),
            Distance::meters(6378137.0)
        );
        assert_approx_eq!(
            Ellipsoid::WGS84.flattening(),
            1.0 / 298.257223563,
            atol <= 1e-15
        );
        assert_approx_eq!(
            Ellipsoid::GRS80.flattening(),
            1.0 / 298.257222101,
            atol <= 1e-15
        );
        assert_approx_eq!(Ellipsoid::WGS72.flattening(), 1.0 / 298.26, atol <= 1e-15);
    }

    // Round-trip tolerance: combined atol+rtol so that both small (near-zero) components
    // and large-magnitude vectors pass. atol=1e-14 absorbs cos(π/2) style FP noise;
    // rtol=1e-12 keeps the relative error bound tight for km-scale coordinates.
    #[rstest]
    #[case(DVec3::new(1.0, 0.0, 0.0))]
    #[case(DVec3::new(0.0, 1.0, 0.0))]
    #[case(DVec3::new(1.0, 2.0, 3.0))]
    #[case(DVec3::new(-100.0, 50.0, -25.0))]
    #[case(DVec3::new(1e7, 1e7, 1e7))]
    #[case(DVec3::new(0.0, 0.0, 1.0))]
    fn test_spherical_roundtrip(#[case] pos: DVec3) {
        let s = Spherical::from_cartesian(pos);
        let pos_back = s.to_cartesian();
        assert_approx_eq!(pos_back.x, pos.x, atol <= 1e-14, rtol <= 1e-12);
        assert_approx_eq!(pos_back.y, pos.y, atol <= 1e-14, rtol <= 1e-12);
        assert_approx_eq!(pos_back.z, pos.z, atol <= 1e-14, rtol <= 1e-12);
    }

    #[rstest]
    #[case(Cartesian::from_array([1.0, 2.0, 3.0, 0.1, 0.2, 0.3]))]
    #[case(Cartesian::from_array([-100.0, 50.0, -25.0, -0.5, 0.5, 1.5]))]
    #[case(Cartesian::from_array([1e6, 2e6, 3e6, 10.0, 20.0, 30.0]))]
    fn test_spherical_state_roundtrip(#[case] cart: Cartesian) {
        let s = SphericalState::from_cartesian(cart);
        let back = s.to_cartesian();
        assert_approx_eq!(
            back.position().x,
            cart.position().x,
            atol <= 1e-14,
            rtol <= 1e-12
        );
        assert_approx_eq!(
            back.position().y,
            cart.position().y,
            atol <= 1e-14,
            rtol <= 1e-12
        );
        assert_approx_eq!(
            back.position().z,
            cart.position().z,
            atol <= 1e-14,
            rtol <= 1e-12
        );
        assert_approx_eq!(
            back.velocity().x,
            cart.velocity().x,
            atol <= 1e-14,
            rtol <= 1e-12
        );
        assert_approx_eq!(
            back.velocity().y,
            cart.velocity().y,
            atol <= 1e-14,
            rtol <= 1e-12
        );
        assert_approx_eq!(
            back.velocity().z,
            cart.velocity().z,
            atol <= 1e-14,
            rtol <= 1e-12
        );
    }
}
