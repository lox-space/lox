// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Data types for representing orbital elements.

use alloc::vec::Vec;
use core::f64::consts::PI;
use core::f64::consts::TAU;
use core::fmt::Display;

use glam::DVec3;
use lox_approx::ApproxEq;
use lox_approx::approx_eq;
use thiserror::Error;

use crate::math::float::{abs, powf, powi, signum, sqrt};

use crate::anomalies::MeanAnomaly;
use crate::anomalies::{AnomalyError, hyperbolic_asymptote_angle};
use crate::anomalies::{EccentricAnomaly, TrueAnomaly};
use crate::time::deltas::TimeDelta;
use crate::utils::Linspace;
use crate::{
    coords::Cartesian,
    glam::Azimuth,
    units::{Angle, AngleUnits, AngularRate, Distance, DistanceUnits},
};

/// The standard gravitational parameter of a celestial body µ = GM.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct GravitationalParameter(f64);

impl GravitationalParameter {
    /// Creates a new gravitational parameter from an `f64` value in m³/s².
    pub const fn m3_per_s2(mu: f64) -> Self {
        Self(mu)
    }

    /// Creates a new gravitational parameter from an `f64` value in km³/s².
    pub const fn km3_per_s2(mu: f64) -> Self {
        Self(1e9 * mu)
    }

    /// Returns the value of the gravitational parameters as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }
}

impl Display for GravitationalParameter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        (self.0 * 1e-9).fmt(f)?;
        write!(f, " km³/s²")
    }
}

/// The Keplerian orbit types or conic sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OrbitType {
    /// Circular orbit (e ≈ 0).
    Circular,
    /// Elliptic orbit (0 < e < 1).
    Elliptic,
    /// Parabolic orbit (e ≈ 1).
    Parabolic,
    /// Hyperbolic orbit (e > 1).
    Hyperbolic,
}

impl Display for OrbitType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            OrbitType::Circular => "circular".fmt(f),
            OrbitType::Elliptic => "elliptic".fmt(f),
            OrbitType::Parabolic => "parabolic".fmt(f),
            OrbitType::Hyperbolic => "hyperbolic".fmt(f),
        }
    }
}

/// Error returned when attempting to create an [`Eccentricity`] from a negative or non-finite value.
#[derive(Debug, Clone, Error)]
#[error("eccentricity must be finite and non-negative but was {0}")]
pub struct EccentricityError(f64);

/// Orbital eccentricity.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Eccentricity(f64);

impl Eccentricity {
    /// Tries to create a new [`Eccentricity`] instance from an `f64` value.
    ///
    /// # Errors
    ///
    /// Returns an [`EccentricityError`] if the value is negative or not finite.
    pub const fn try_new(ecc: f64) -> Result<Eccentricity, EccentricityError> {
        if !ecc.is_finite() || ecc < 0.0 {
            return Err(EccentricityError(ecc));
        }
        Ok(Eccentricity(ecc))
    }

    /// Returns the value of the eccentricity as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the [`OrbitType`] based on the eccentricity.
    pub fn orbit_type(&self) -> OrbitType {
        match self.0 {
            ecc if approx_eq!(ecc, 0.0, atol <= 1e-8) => OrbitType::Circular,
            ecc if approx_eq!(ecc, 1.0, rtol <= 1e-8) => OrbitType::Parabolic,
            ecc if ecc > 0.0 && ecc < 1.0 => OrbitType::Elliptic,
            _ => OrbitType::Hyperbolic,
        }
    }

    /// Checks if the orbit is circular.
    pub fn is_circular(&self) -> bool {
        matches!(self.orbit_type(), OrbitType::Circular)
    }

    /// Checks if the orbit is elliptic.
    pub fn is_elliptic(&self) -> bool {
        matches!(self.orbit_type(), OrbitType::Elliptic)
    }

    /// Checks if the orbit is parabolic.
    pub fn is_parabolic(&self) -> bool {
        matches!(self.orbit_type(), OrbitType::Parabolic)
    }

    /// Checks if the orbit is hyperbolic.
    pub fn is_hyperbolic(&self) -> bool {
        matches!(self.orbit_type(), OrbitType::Hyperbolic)
    }

    /// Checks if the orbit is circular or elliptic.
    pub fn is_circular_or_elliptic(&self) -> bool {
        matches!(self.orbit_type(), OrbitType::Circular | OrbitType::Elliptic)
    }
}

impl Display for Eccentricity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

/// Error returned when an [`Inclination`] is outside the valid range [0°, 180°].
#[derive(Debug, Clone, Error)]
#[error("inclination must be between 0 and 180 deg but was {0}")]
pub struct InclinationError(Angle);

/// Orbital inclination.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Inclination(Angle);

impl Inclination {
    /// Tries to create a new [`Inclination`] from an angle. Must be between 0 and π.
    pub const fn try_new(inclination: Angle) -> Result<Inclination, InclinationError> {
        let inc = inclination.as_f64();
        if !(inc >= 0.0 && inc <= PI) {
            return Err(InclinationError(inclination));
        }
        Ok(Inclination(inclination))
    }

    /// Returns the inclination in radians as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0.as_f64()
    }

    /// Returns the inclination as an [`Angle`].
    pub const fn as_angle(&self) -> Angle {
        self.0
    }
}

impl Display for Inclination {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

/// Error returned when a [`LongitudeOfAscendingNode`] is outside the valid range [0°, 360°].
#[derive(Debug, Clone, Error)]
#[error("longitude of ascending node must be between 0 and 360 deg but was {0}")]
pub struct LongitudeOfAscendingNodeError(Angle);

/// Longitude of ascending node.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct LongitudeOfAscendingNode(#[approx(periodic = TAU)] Angle);

impl LongitudeOfAscendingNode {
    /// Tries to create a new [`LongitudeOfAscendingNode`]. Must be between 0 and 2π.
    pub const fn try_new(
        longitude_of_ascending_node: Angle,
    ) -> Result<LongitudeOfAscendingNode, LongitudeOfAscendingNodeError> {
        let node = longitude_of_ascending_node.as_f64();
        if !(node >= 0.0 && node <= TAU) {
            return Err(LongitudeOfAscendingNodeError(longitude_of_ascending_node));
        }
        Ok(LongitudeOfAscendingNode(longitude_of_ascending_node))
    }

    /// Returns the longitude of ascending node in radians as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0.as_f64()
    }

    /// Returns the longitude of ascending node as an [`Angle`].
    pub const fn as_angle(&self) -> Angle {
        self.0
    }
}

impl Display for LongitudeOfAscendingNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

/// Error returned when an [`ArgumentOfPeriapsis`] is outside the valid range [0°, 360°].
#[derive(Debug, Clone, Error)]
#[error("argument of periapsis must be between 0 and 360 deg but was {0}")]
pub struct ArgumentOfPeriapsisError(Angle);

/// Argument of periapsis.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct ArgumentOfPeriapsis(#[approx(periodic = TAU)] Angle);

impl ArgumentOfPeriapsis {
    /// Tries to create a new [`ArgumentOfPeriapsis`]. Must be between 0 and 2π.
    pub const fn try_new(
        argument_of_periapsis: Angle,
    ) -> Result<ArgumentOfPeriapsis, ArgumentOfPeriapsisError> {
        let arg = argument_of_periapsis.as_f64();
        if !(arg >= 0.0 && arg <= TAU) {
            return Err(ArgumentOfPeriapsisError(argument_of_periapsis));
        }
        Ok(ArgumentOfPeriapsis(argument_of_periapsis))
    }

    /// Returns the argument of periapsis in radians as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0.as_f64()
    }

    /// Returns the argument of periapsis as an [`Angle`].
    pub const fn as_angle(&self) -> Angle {
        self.0
    }
}

impl Display for ArgumentOfPeriapsis {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

/// A set of Keplerian orbital elements.
///
/// The orbit's shape is stored as the semi-parameter (semi-latus rectum) `p` and the eccentricity,
/// which are finite for every conic section, including parabolic orbits.
///
/// Approximate comparisons treat the longitude of the ascending node, the argument of periapsis,
/// and the true anomaly as angles modulo 2π. For these elements, `rtol` is relative to π rather
/// than to their magnitude, see
/// [`ApproxEqResult::periodic`](lox_approx::ApproxEqResult::periodic).
#[derive(Debug, Clone, Copy, PartialEq, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Keplerian {
    semi_parameter: Distance,
    eccentricity: Eccentricity,
    inclination: Inclination,
    longitude_of_ascending_node: LongitudeOfAscendingNode,
    argument_of_periapsis: ArgumentOfPeriapsis,
    true_anomaly: TrueAnomaly,
}

impl Keplerian {
    /// Validates the components and normalises the angles. Every [`Keplerian`] is constructed
    /// through this function, so the elements always describe a valid conic section.
    fn try_new(
        semi_parameter: Distance,
        eccentricity: Eccentricity,
        inclination: Angle,
        longitude_of_ascending_node: Angle,
        argument_of_periapsis: Angle,
        true_anomaly: TrueAnomaly,
    ) -> Result<Self, KeplerianError> {
        let p = semi_parameter.as_f64();
        if !p.is_finite() || p <= 0.0 {
            return Err(KeplerianError::InvalidSemiParameter(semi_parameter));
        }
        let inclination = Inclination::try_new(inclination)?;
        let longitude_of_ascending_node =
            LongitudeOfAscendingNode::try_new(longitude_of_ascending_node.mod_two_pi())?;
        let argument_of_periapsis =
            ArgumentOfPeriapsis::try_new(argument_of_periapsis.mod_two_pi())?;
        if !true_anomaly.as_f64().is_finite() {
            return Err(KeplerianError::NonFiniteAnomaly(true_anomaly.as_angle()));
        }
        let true_anomaly = check_true_anomaly(true_anomaly, eccentricity)?;
        Ok(Self {
            semi_parameter,
            eccentricity,
            inclination,
            longitude_of_ascending_node,
            argument_of_periapsis,
            true_anomaly: normalize_true_anomaly(true_anomaly, eccentricity),
        })
    }

    /// Returns a new [`KeplerianBuilder`].
    pub fn builder() -> KeplerianBuilder {
        KeplerianBuilder::default()
    }

    /// Returns the semi-major axis, or `None` for parabolic orbits.
    ///
    /// The semi-major axis is negative for hyperbolic orbits.
    pub fn semi_major_axis(&self) -> Option<Distance> {
        if self.eccentricity.is_parabolic() {
            return None;
        }
        if self.eccentricity.is_circular() {
            return Some(self.semi_parameter);
        }
        Some(Distance::new(
            self.semi_parameter.as_f64() / (1.0 - powi(self.eccentricity.0, 2)),
        ))
    }

    /// Returns the eccentricity.
    pub fn eccentricity(&self) -> Eccentricity {
        self.eccentricity
    }

    /// Returns the inclination.
    pub fn inclination(&self) -> Inclination {
        self.inclination
    }

    /// Returns the longitude of ascending node.
    pub fn longitude_of_ascending_node(&self) -> LongitudeOfAscendingNode {
        self.longitude_of_ascending_node
    }

    /// Returns the argument of periapsis.
    pub fn argument_of_periapsis(&self) -> ArgumentOfPeriapsis {
        self.argument_of_periapsis
    }

    /// Returns the true anomaly.
    pub fn true_anomaly(&self) -> TrueAnomaly {
        self.true_anomaly
    }

    /// Returns the semi-parameter (semi-latus rectum) of the orbit.
    pub fn semi_parameter(&self) -> Distance {
        self.semi_parameter
    }

    /// Returns the periapsis radius.
    pub fn periapsis_radius(&self) -> Distance {
        Distance::new(self.semi_parameter.as_f64() / (1.0 + self.eccentricity.0))
    }

    /// Returns the apoapsis radius, or `None` for open orbits.
    pub fn apoapsis_radius(&self) -> Option<Distance> {
        if !self.eccentricity.is_circular_or_elliptic() {
            return None;
        }
        Some(Distance::new(
            self.semi_parameter.as_f64() / (1.0 - self.eccentricity.0),
        ))
    }

    /// Returns the periapsis altitude above a body with the given mean radius.
    pub fn periapsis_altitude(&self, mean_radius: Distance) -> Distance {
        self.periapsis_radius() - mean_radius
    }

    /// Returns the apoapsis altitude above a body with the given mean radius, or `None` for
    /// open orbits.
    pub fn apoapsis_altitude(&self, mean_radius: Distance) -> Option<Distance> {
        Some(self.apoapsis_radius()? - mean_radius)
    }

    /// Returns the argument of latitude, i.e. the angle from the ascending node to the position,
    /// in the interval [0, 2π).
    pub fn argument_of_latitude(&self) -> Angle {
        (self.argument_of_periapsis.0 + self.true_anomaly.as_angle()).mod_two_pi()
    }

    /// Returns the true longitude in the interval [0, 2π).
    pub fn true_longitude(&self) -> Angle {
        (self.longitude_of_ascending_node.0
            + self.argument_of_periapsis.0
            + self.true_anomaly.as_angle())
        .mod_two_pi()
    }

    /// Returns the mean longitude.
    ///
    /// For closed orbits the result lies in the interval [0, 2π). The mean anomaly it is based on
    /// depends on the [`OrbitType`], see [`Keplerian::mean_motion`].
    ///
    /// # Errors
    ///
    /// Returns an [`AnomalyError`] if the true anomaly cannot be converted to a mean anomaly.
    pub fn mean_longitude(&self) -> Result<Angle, AnomalyError> {
        let mean_anomaly = self.true_anomaly.to_mean(self.eccentricity)?;
        let mean_longitude = self.longitude_of_ascending_node.0
            + self.argument_of_periapsis.0
            + mean_anomaly.as_angle();
        if self.eccentricity.is_circular_or_elliptic() {
            Ok(mean_longitude.mod_two_pi())
        } else {
            Ok(mean_longitude)
        }
    }

    /// Returns the flight-path angle, i.e. the angle between the velocity vector and the local
    /// horizontal. It is positive while the orbit is moving away from periapsis.
    pub fn flight_path_angle(&self) -> Angle {
        let ecc = self.eccentricity.as_f64();
        let (sin_nu, cos_nu) = self.true_anomaly.as_angle().sin_cos();
        Angle::from_atan2(ecc * sin_nu, 1.0 + ecc * cos_nu)
    }

    /// Returns the mean motion, i.e. the rate of change of the [`MeanAnomaly`].
    ///
    /// The mean anomaly follows the conventions of the [`anomalies`](crate::anomalies) module,
    /// so the mean motion is defined for every orbit type. These conventions normalise the mean
    /// anomaly differently for each [`OrbitType`], however, so the mean motion is discontinuous
    /// at the parabolic threshold of [`Eccentricity::orbit_type`]: for a fixed semi-parameter
    /// `p`, it tends to zero as the eccentricity approaches one from either side but is
    /// `2·√(µ/p³)` for parabolic orbits. Mean anomalies and mean motions are therefore only
    /// comparable between orbits of the same [`OrbitType`].
    pub fn mean_motion(&self, grav_param: GravitationalParameter) -> AngularRate {
        let mu = grav_param.as_f64();
        match self.semi_major_axis() {
            Some(a) => AngularRate::new(sqrt(mu / powi(abs(a.as_f64()), 3))),
            None => {
                let p = self.semi_parameter.as_f64();
                AngularRate::new(2.0 * sqrt(mu / powi(p, 3)))
            }
        }
    }

    // TODO: Return unitful types from `specific_orbital_energy` and `specific_angular_momentum`
    // once `lox_core::units` has types for specific energy and specific angular momentum.

    /// Returns the specific orbital energy in J/kg (m²/s²).
    ///
    /// The energy is negative for closed orbits, zero for parabolic orbits, and positive for
    /// hyperbolic orbits.
    pub fn specific_orbital_energy(&self, grav_param: GravitationalParameter) -> f64 {
        let mu = grav_param.as_f64();
        let p = self.semi_parameter.as_f64();
        -mu * (1.0 - powi(self.eccentricity.0, 2)) / (2.0 * p)
    }

    /// Returns the magnitude of the specific angular momentum in m²/s.
    pub fn specific_angular_momentum(&self, grav_param: GravitationalParameter) -> f64 {
        sqrt(grav_param.as_f64() * self.semi_parameter.as_f64())
    }

    /// Converts the orbital elements to position and velocity in the perifocal frame.
    pub fn to_perifocal(&self, grav_param: GravitationalParameter) -> (DVec3, DVec3) {
        let ecc = self.eccentricity.as_f64();
        let mu = grav_param.as_f64();
        let semiparameter = self.semi_parameter.as_f64();
        let (sin_nu, cos_nu) = self.true_anomaly.as_angle().sin_cos();
        let sqrt_mu_p = sqrt(mu / semiparameter);

        let pos = DVec3::new(cos_nu, sin_nu, 0.0) * (semiparameter / (1.0 + ecc * cos_nu));
        let vel = DVec3::new(-sin_nu, ecc + cos_nu, 0.0) * sqrt_mu_p;

        (pos, vel)
    }

    /// Converts the orbital elements to a Cartesian state vector.
    pub fn to_cartesian(&self, grav_param: GravitationalParameter) -> Cartesian {
        let (pos, vel) = self.to_perifocal(grav_param);
        let rot = self.longitude_of_ascending_node.0.rotation_z().transpose()
            * self.inclination.0.rotation_x().transpose()
            * self.argument_of_periapsis.0.rotation_z().transpose();
        Cartesian::from_vecs(rot * pos, rot * vel)
    }

    /// Returns the orbital period, or `None` for non-elliptic orbits.
    pub fn orbital_period(&self, grav_param: GravitationalParameter) -> Option<TimeDelta> {
        if !self.eccentricity().is_circular_or_elliptic() {
            return None;
        }
        let mu = grav_param.as_f64();
        let a = self.semi_major_axis()?.as_f64();
        Some(TimeDelta::from_seconds_f64(TAU * sqrt(powf(a, 3.0) / mu)))
    }

    /// Returns an iterator over `n` Cartesian positions equally spaced in eccentric anomaly around
    /// the orbit, or `None` for non-elliptic orbits.
    pub fn iter_trace(
        &self,
        grav_param: GravitationalParameter,
        n: usize,
    ) -> Option<impl Iterator<Item = Cartesian>> {
        if !self.eccentricity().is_circular_or_elliptic() {
            return None;
        }
        let elements = *self;
        Some(Linspace::new(0.0, TAU, n).map(move |ecc| {
            let true_anomaly = EccentricAnomaly::new(ecc.rad()).to_true(elements.eccentricity);
            Keplerian {
                true_anomaly,
                ..elements
            }
            .to_cartesian(grav_param)
        }))
    }

    /// Returns `n` Cartesian positions equally spaced in eccentric anomaly around the orbit, or
    /// `None` for non-elliptic orbits.
    pub fn trace(&self, grav_param: GravitationalParameter, n: usize) -> Option<Vec<Cartesian>> {
        Some(self.iter_trace(grav_param, n)?.collect())
    }
}

/// Checks that the true anomaly of an open orbit lies strictly between the asymptotes.
fn check_true_anomaly(
    true_anomaly: TrueAnomaly,
    eccentricity: Eccentricity,
) -> Result<TrueAnomaly, AnomalyError> {
    let max_nu = match eccentricity.orbit_type() {
        OrbitType::Circular | OrbitType::Elliptic => return Ok(true_anomaly),
        OrbitType::Parabolic => Angle::new(PI),
        OrbitType::Hyperbolic => hyperbolic_asymptote_angle(eccentricity),
    };
    let nu = true_anomaly.as_angle().normalize_two_pi(Angle::ZERO);
    // Close to the asymptotes, 1 + e·cos(ν) can round to zero even though ν lies inside them,
    // which would make the radius infinite.
    let (_, cos_nu) = nu.sin_cos();
    if abs(nu.as_f64()) >= max_nu.as_f64() || 1.0 + eccentricity.as_f64() * cos_nu <= 0.0 {
        return Err(AnomalyError::InvalidTrueAnomaly {
            nu: true_anomaly.as_angle(),
            max_nu,
        });
    }
    Ok(TrueAnomaly::new(nu))
}

/// Normalises a true anomaly to [0, 2π) for closed orbits and to [-π, π) for open orbits.
fn normalize_true_anomaly(true_anomaly: TrueAnomaly, eccentricity: Eccentricity) -> TrueAnomaly {
    let nu = true_anomaly.as_angle();
    if eccentricity.is_circular_or_elliptic() {
        TrueAnomaly::new(nu.mod_two_pi())
    } else {
        TrueAnomaly::new(nu.normalize_two_pi(Angle::ZERO))
    }
}

impl Cartesian {
    /// Computes the eccentricity vector from the Cartesian state.
    pub fn eccentricity_vector(&self, grav_param: GravitationalParameter) -> DVec3 {
        let mu = grav_param.as_f64();
        let r = self.position();
        let v = self.velocity();

        let rm = r.length();
        let v2 = v.dot(v);
        let rv = r.dot(v);

        ((v2 - mu / rm) * r - rv * v) / mu
    }

    /// Converts the Cartesian state to Keplerian orbital elements.
    ///
    /// # Errors
    ///
    /// Returns a [`KeplerianError`] if the state is not finite, if its angular momentum is zero,
    /// i.e. the orbit is rectilinear, or if the resulting elements are invalid. The latter can
    /// happen for nearly rectilinear orbits.
    pub fn to_keplerian(
        &self,
        grav_param: GravitationalParameter,
    ) -> Result<Keplerian, KeplerianError> {
        let r = self.position();
        let v = self.velocity();
        let mu = grav_param.as_f64();

        if !r.is_finite() || !v.is_finite() {
            return Err(KeplerianError::NonFiniteState);
        }

        let rm = r.length();
        let h = r.cross(v);
        let hm = h.length();
        if hm == 0.0 {
            return Err(KeplerianError::ZeroAngularMomentum);
        }
        let node = DVec3::Z.cross(h);
        let e = self.eccentricity_vector(grav_param);
        let eccentricity = Eccentricity::try_new(e.length())?;
        let inclination = h.angle_between(DVec3::Z).rad();

        let equatorial = approx_eq!(inclination, Angle::ZERO, atol <= 1e-8)
            || approx_eq!(inclination, Angle::new(PI), atol <= 1e-8);
        let circular = eccentricity.is_circular();
        // In-plane angles of retrograde equatorial orbits are measured clockwise from the x-axis.
        let direction = signum(h.z);

        let semi_parameter = powi(hm, 2) / mu;

        let (longitude_of_ascending_node, argument_of_periapsis, true_anomaly) =
            if equatorial && !circular {
                (
                    Angle::ZERO,
                    direction * e.azimuth(),
                    TrueAnomaly::new(Angle::from_atan2(h.dot(e.cross(r)) / hm, r.dot(e))),
                )
            } else if !equatorial && circular {
                (
                    node.azimuth(),
                    Angle::ZERO,
                    TrueAnomaly::new(Angle::from_atan2(r.dot(h.cross(node)) / hm, r.dot(node))),
                )
            } else if equatorial && circular {
                (
                    Angle::ZERO,
                    Angle::ZERO,
                    TrueAnomaly::new(direction * r.azimuth()),
                )
            } else {
                // e·sin(ν) = h·(r·v) / (µ·r) and e·cos(ν) = p/r - 1 hold for every conic section.
                let true_anomaly = TrueAnomaly::new(Angle::from_atan2(
                    hm * r.dot(v) / (mu * rm),
                    semi_parameter / rm - 1.0,
                ));
                let px = r.dot(node);
                let py = r.dot(h.cross(node)) / hm;
                (
                    node.azimuth(),
                    Angle::from_atan2(py, px) - true_anomaly.as_angle(),
                    true_anomaly,
                )
            };

        Keplerian::try_new(
            semi_parameter.m(),
            eccentricity,
            inclination,
            longitude_of_ascending_node,
            argument_of_periapsis,
            true_anomaly,
        )
    }
}

/// Error returned when constructing [`Keplerian`] elements.
#[derive(Debug, Clone, Error)]
pub enum KeplerianError {
    /// The eccentricity is invalid.
    #[error(transparent)]
    InvalidEccentricity(#[from] EccentricityError),
    /// The semi-major axis is inconsistent with the eccentricity.
    #[error(
        "semi-major axis ({semi_major_axis}) {} for {} eccentricity ({eccentricity})",
        semi_major_axis_requirement(.eccentricity),
        .eccentricity.orbit_type()
    )]
    InvalidShape {
        /// The invalid semi-major axis.
        semi_major_axis: Distance,
        /// The eccentricity that conflicts with the semi-major axis.
        eccentricity: Eccentricity,
    },
    /// The semi-parameter is not finite and positive.
    #[error("semi-parameter must be finite and positive but was {0}")]
    InvalidSemiParameter(Distance),
    /// No shape parameters were provided.
    #[error(
        "no orbital shape parameters (semi-major axis or semi-parameter and eccentricity, radii, or altitudes) were provided"
    )]
    MissingShape,
    /// The inclination is invalid.
    #[error(transparent)]
    InvalidInclination(#[from] InclinationError),
    /// The longitude of ascending node is invalid.
    #[error(transparent)]
    InvalidLongitudeOfAscendingNode(#[from] LongitudeOfAscendingNodeError),
    /// The argument of periapsis is invalid.
    #[error(transparent)]
    InvalidArgumentOfPeriapsis(#[from] ArgumentOfPeriapsisError),
    /// More than one anomaly was provided.
    #[error("only one of true, eccentric, or mean anomaly may be specified")]
    AmbiguousAnomaly,
    /// The anomaly is not finite.
    #[error("anomaly must be finite but was {0}")]
    NonFiniteAnomaly(Angle),
    /// The anomaly conversion failed or the true anomaly is outside the valid range.
    #[error(transparent)]
    Anomaly(#[from] AnomalyError),
    /// The Cartesian state is not finite.
    #[error("Cartesian state must be finite")]
    NonFiniteState,
    /// The Cartesian state has zero angular momentum, e.g. a rectilinear orbit.
    #[error("Keplerian elements are undefined for states with zero angular momentum")]
    ZeroAngularMomentum,
}

fn semi_major_axis_requirement(eccentricity: &Eccentricity) -> &'static str {
    match eccentricity.orbit_type() {
        OrbitType::Circular | OrbitType::Elliptic => "must be finite and positive",
        OrbitType::Parabolic => "is undefined",
        OrbitType::Hyperbolic => "must be finite and negative",
    }
}

#[derive(Debug, Clone, Copy)]
enum Shape {
    SemiMajorAxis(Distance),
    SemiParameter(Distance),
}

#[derive(Debug, Clone, Copy)]
enum Anomaly {
    True(Angle),
    Eccentric(Angle),
    Mean(Angle),
}

/// Builder for constructing validated [`Keplerian`] elements.
#[derive(Debug, Default, Clone)]
pub struct KeplerianBuilder {
    shape: Option<(Shape, Result<Eccentricity, EccentricityError>)>,
    inclination: Angle,
    longitude_of_ascending_node: Angle,
    argument_of_periapsis: Angle,
    anomaly: Option<Anomaly>,
    ambiguous_anomaly: bool,
}

impl KeplerianBuilder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the semi-major axis and eccentricity.
    ///
    /// The semi-major axis must be positive for closed orbits and negative for hyperbolic orbits.
    /// It is undefined for parabolic orbits; use
    /// [`with_semi_parameter`](Self::with_semi_parameter) for those.
    pub fn with_semi_major_axis(mut self, semi_major_axis: Distance, eccentricity: f64) -> Self {
        self.shape = Some((
            Shape::SemiMajorAxis(semi_major_axis),
            Eccentricity::try_new(eccentricity),
        ));
        self
    }

    /// Sets the semi-parameter (semi-latus rectum) and eccentricity.
    pub fn with_semi_parameter(mut self, semi_parameter: Distance, eccentricity: f64) -> Self {
        self.shape = Some((
            Shape::SemiParameter(semi_parameter),
            Eccentricity::try_new(eccentricity),
        ));
        self
    }

    /// Sets the orbit shape from periapsis and apoapsis radii.
    pub fn with_radii(self, periapsis_radius: Distance, apoapsis_radius: Distance) -> Self {
        let rp = periapsis_radius.as_f64();
        let ra = apoapsis_radius.as_f64();
        self.with_semi_major_axis(Distance::new((rp + ra) / 2.0), (ra - rp) / (ra + rp))
    }

    /// Sets the orbit shape from periapsis and apoapsis altitudes above a mean radius.
    pub fn with_altitudes(
        self,
        periapsis_altitude: Distance,
        apoapsis_altitude: Distance,
        mean_radius: Distance,
    ) -> Self {
        let rp = periapsis_altitude + mean_radius;
        let ra = apoapsis_altitude + mean_radius;
        self.with_radii(rp, ra)
    }

    /// Sets the inclination.
    pub fn with_inclination(mut self, inclination: Angle) -> Self {
        self.inclination = inclination;
        self
    }

    /// Sets the longitude of ascending node. Any finite angle is accepted.
    pub fn with_longitude_of_ascending_node(mut self, longitude_of_ascending_node: Angle) -> Self {
        self.longitude_of_ascending_node = longitude_of_ascending_node;
        self
    }

    /// Sets the argument of periapsis. Any finite angle is accepted.
    pub fn with_argument_of_periapsis(mut self, argument_of_periapsis: Angle) -> Self {
        self.argument_of_periapsis = argument_of_periapsis;
        self
    }

    /// Sets the true anomaly.
    ///
    /// Only one of the true, eccentric, or mean anomaly may be set.
    pub fn with_true_anomaly(self, true_anomaly: Angle) -> Self {
        self.with_anomaly(Anomaly::True(true_anomaly))
    }

    /// Sets the eccentric anomaly (converted to true anomaly during build).
    ///
    /// For hyperbolic and parabolic orbits, this is the hyperbolic and parabolic anomaly
    /// respectively, see [`EccentricAnomaly`]. Only one of the true, eccentric, or mean anomaly
    /// may be set.
    pub fn with_eccentric_anomaly(self, eccentric_anomaly: Angle) -> Self {
        self.with_anomaly(Anomaly::Eccentric(eccentric_anomaly))
    }

    /// Sets the mean anomaly (converted to true anomaly during build).
    ///
    /// The mean anomaly is interpreted according to the [`OrbitType`] of the eccentricity, so the
    /// same value describes a different position on either side of the parabolic threshold, see
    /// [`Keplerian::mean_motion`]. Only one of the true, eccentric, or mean anomaly may be set.
    pub fn with_mean_anomaly(self, mean_anomaly: Angle) -> Self {
        self.with_anomaly(Anomaly::Mean(mean_anomaly))
    }

    fn with_anomaly(mut self, anomaly: Anomaly) -> Self {
        self.ambiguous_anomaly |= self.anomaly.is_some();
        self.anomaly = Some(anomaly);
        self
    }

    /// Validates all parameters and builds the [`Keplerian`] elements.
    ///
    /// The longitude of ascending node and the argument of periapsis are normalised to [0, 2π).
    /// The true anomaly is normalised to [0, 2π) for closed orbits and to [-π, π) for open orbits.
    ///
    /// # Errors
    ///
    /// Returns a [`KeplerianError`] if any parameter is invalid or not finite, if more than one
    /// anomaly was set, or if the true anomaly lies outside the asymptotes of a hyperbolic orbit.
    pub fn build(self) -> Result<Keplerian, KeplerianError> {
        let (shape, eccentricity) = self.shape.ok_or(KeplerianError::MissingShape)?;

        let eccentricity = eccentricity?;

        let semi_parameter = Self::semi_parameter(shape, eccentricity)?;

        if self.ambiguous_anomaly {
            return Err(KeplerianError::AmbiguousAnomaly);
        }

        if let Some(Anomaly::True(anomaly) | Anomaly::Eccentric(anomaly) | Anomaly::Mean(anomaly)) =
            self.anomaly
            && !anomaly.as_f64().is_finite()
        {
            return Err(KeplerianError::NonFiniteAnomaly(anomaly));
        }

        let true_anomaly = match self.anomaly {
            None => TrueAnomaly::new(Angle::ZERO),
            Some(Anomaly::True(nu)) => TrueAnomaly::new(nu),
            Some(Anomaly::Eccentric(ecc)) => EccentricAnomaly::new(ecc).to_true(eccentricity),
            Some(Anomaly::Mean(mean)) => MeanAnomaly::new(mean).to_true(eccentricity)?,
        };

        Keplerian::try_new(
            semi_parameter,
            eccentricity,
            self.inclination,
            self.longitude_of_ascending_node,
            self.argument_of_periapsis,
            true_anomaly,
        )
    }

    fn semi_parameter(
        shape: Shape,
        eccentricity: Eccentricity,
    ) -> Result<Distance, KeplerianError> {
        let semi_parameter = match shape {
            Shape::SemiMajorAxis(semi_major_axis) => {
                let ecc = eccentricity.as_f64();
                let sma = semi_major_axis.as_f64();
                let valid = sma.is_finite()
                    && match eccentricity.orbit_type() {
                        OrbitType::Circular | OrbitType::Elliptic => sma > 0.0,
                        OrbitType::Parabolic => false,
                        OrbitType::Hyperbolic => sma < 0.0,
                    };
                if !valid {
                    return Err(KeplerianError::InvalidShape {
                        semi_major_axis,
                        eccentricity,
                    });
                }
                if eccentricity.is_circular() {
                    semi_major_axis
                } else {
                    Distance::new(sma * (1.0 - powi(ecc, 2)))
                }
            }
            Shape::SemiParameter(semi_parameter) => semi_parameter,
        };
        Ok(semi_parameter)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use core::f64::consts::FRAC_PI_2;

    use lox_approx::{ApproxEqResult, assert_approx_eq, assert_approx_ne};
    use proptest::prelude::*;

    use crate::math::float::{asin, sin_cos};
    use crate::units::VelocityUnits;

    use super::*;

    #[test]
    fn test_eccentricity_rejects_infinity() {
        assert!(Eccentricity::try_new(f64::INFINITY).is_err());
    }

    #[test]
    fn test_element_validators_reject_nan() {
        assert!(Eccentricity::try_new(f64::NAN).is_err());
        assert!(Inclination::try_new(Angle::radians(f64::NAN)).is_err());
        assert!(LongitudeOfAscendingNode::try_new(Angle::radians(f64::NAN)).is_err());
        assert!(ArgumentOfPeriapsis::try_new(Angle::radians(f64::NAN)).is_err());
    }

    #[test]
    fn test_element_angles_as_angle() {
        assert_eq!(
            Inclination::try_new(0.5.rad()).unwrap().as_angle(),
            0.5.rad()
        );
        assert_eq!(
            LongitudeOfAscendingNode::try_new(1.5.rad())
                .unwrap()
                .as_angle(),
            1.5.rad()
        );
        assert_eq!(
            ArgumentOfPeriapsis::try_new(2.5.rad()).unwrap().as_angle(),
            2.5.rad()
        );
    }

    #[test]
    fn test_element_angles_approx_eq_wraps_around() {
        let near_zero = 1e-12.rad();
        let near_two_pi = (TAU - 1e-12).rad();
        assert_approx_eq!(
            LongitudeOfAscendingNode(near_two_pi),
            LongitudeOfAscendingNode(near_zero),
            atol <= 1e-11
        );
        assert_approx_eq!(
            ArgumentOfPeriapsis(near_zero),
            ArgumentOfPeriapsis(near_two_pi),
            atol <= 1e-11
        );
        assert_approx_ne!(
            ArgumentOfPeriapsis(near_zero),
            ArgumentOfPeriapsis(PI.rad()),
            atol <= 1e-11
        );
        // Inclination is bounded to [0, π] and does not wrap.
        assert_approx_ne!(Inclination(near_zero), Inclination(PI.rad()), atol <= 1e-11);
    }

    #[test]
    fn test_keplerian_approx_eq_wraps_around() {
        let k = |node: f64, argp: f64, nu: f64| {
            Keplerian::builder()
                .with_semi_major_axis(7000.0.km(), 0.1)
                .with_inclination(45.0.deg())
                .with_longitude_of_ascending_node(node.rad())
                .with_argument_of_periapsis(argp.rad())
                .with_true_anomaly(nu.rad())
                .build()
                .unwrap()
        };
        assert_approx_eq!(k(TAU - 1e-12, TAU - 1e-12, TAU - 1e-12), k(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_cartesian_to_keplerian_roundtrip() {
        let mu = GravitationalParameter::km3_per_s2(398600.43550702266f64);

        let cartesian = Cartesian::builder()
            .position(
                -0.107622532467967e7.m(),
                -0.676589636432773e7.m(),
                -0.332308783350379e6.m(),
            )
            .velocity(
                0.935685775154103e4.mps(),
                -0.331234775037644e4.mps(),
                -0.118801577532701e4.mps(),
            )
            .build();

        let cartesian1 = cartesian.to_keplerian(mu).unwrap().to_cartesian(mu);

        assert_approx_eq!(cartesian.position(), cartesian1.position(), rtol <= 1e-8);
        assert_approx_eq!(cartesian.velocity(), cartesian1.velocity(), rtol <= 1e-6);
    }

    #[test]
    fn test_retrograde_equatorial_round_trip() {
        // Exactly in the equatorial plane, so the node vector is exactly zero.
        let states = [
            // Elliptic
            Cartesian::from_vecs(
                DVec3::new(-4838.214e3, -4489.062e3, 0.0),
                DVec3::new(-5.0e3, 6.0e3, 0.0),
            ),
            // Circular
            Cartesian::from_vecs(
                DVec3::new(7000e3, 0.0, 0.0),
                DVec3::new(0.0, -sqrt(MU_EARTH.as_f64() / 7000e3), 0.0),
            ),
        ];
        for state in states {
            let k = state.to_keplerian(MU_EARTH).unwrap();
            assert_approx_eq!(k.inclination().as_f64(), PI, rtol <= 1e-15);
            let state1 = k.to_cartesian(MU_EARTH);
            assert_approx_eq!(state.position(), state1.position(), atol <= 1e-6);
            assert_approx_eq!(state.velocity(), state1.velocity(), atol <= 1e-9);
        }
    }

    #[test]
    fn test_to_keplerian_rejects_degenerate_states() {
        let r = DVec3::new(7000e3, 0.0, 0.0);
        let v = DVec3::new(0.0, 7.5e3, 0.0);
        for (name, state) in [
            ("radial", Cartesian::from_vecs(r, DVec3::new(1e3, 0.0, 0.0))),
            ("zero velocity", Cartesian::from_vecs(r, DVec3::ZERO)),
            ("zero position", Cartesian::from_vecs(DVec3::ZERO, v)),
        ] {
            let result = state.to_keplerian(MU_EARTH);
            assert!(
                matches!(result, Err(KeplerianError::ZeroAngularMomentum)),
                "{name}: {result:?}"
            );
        }

        for state in [
            Cartesian::from_vecs(DVec3::new(f64::NAN, 0.0, 0.0), v),
            Cartesian::from_vecs(r, DVec3::new(0.0, f64::INFINITY, 0.0)),
        ] {
            let result = state.to_keplerian(MU_EARTH);
            assert!(
                matches!(result, Err(KeplerianError::NonFiniteState)),
                "{result:?}"
            );
        }

        // Nearly radial: every element is finite, but the true anomaly lies so close to the
        // asymptote that the radius 1 + e·cos(ν) rounds to zero.
        let nearly_radial = Cartesian::from_vecs(r, DVec3::new(1e3, 1e-9, 0.0));
        let result = nearly_radial.to_keplerian(MU_EARTH);
        assert!(
            matches!(
                result,
                Err(KeplerianError::Anomaly(
                    AnomalyError::InvalidTrueAnomaly { .. }
                ))
            ),
            "{result:?}"
        );
    }

    #[test]
    fn test_keplerian_builder() {
        let mu = GravitationalParameter::km3_per_s2(398600.43550702266f64);

        let semi_major_axis = 24464.560.km();
        let eccentricity = 0.7311;
        let inclination = 0.122138.rad();
        let ascending_node = 1.00681.rad();
        let argument_of_periapsis = 3.10686.rad();
        let true_anomaly = 0.44369564302687126.rad();

        let k = Keplerian::builder()
            .with_semi_major_axis(semi_major_axis, eccentricity)
            .with_inclination(inclination)
            .with_longitude_of_ascending_node(ascending_node)
            .with_argument_of_periapsis(argument_of_periapsis)
            .with_true_anomaly(true_anomaly)
            .build()
            .unwrap();
        let k1 = k.to_cartesian(mu).to_keplerian(mu).unwrap();
        assert_approx_eq!(k, k1);
    }

    const MU_EARTH: GravitationalParameter = GravitationalParameter::km3_per_s2(398600.43550702266);

    fn elliptic() -> Keplerian {
        Keplerian::builder()
            .with_semi_major_axis(24464.560.km(), 0.7311)
            .with_inclination(0.122138.rad())
            .with_longitude_of_ascending_node(1.00681.rad())
            .with_argument_of_periapsis(3.10686.rad())
            .with_true_anomaly(0.44369564302687126.rad())
            .build()
            .unwrap()
    }

    #[test]
    fn test_parabolic_round_trip() {
        let k = Keplerian::builder()
            .with_semi_parameter(10000.0.km(), 1.0)
            .with_inclination(0.5.rad())
            .with_longitude_of_ascending_node(1.0.rad())
            .with_argument_of_periapsis(2.0.rad())
            .with_true_anomaly(1.0.rad())
            .build()
            .unwrap();
        assert!(k.eccentricity().is_parabolic());
        assert!(k.semi_major_axis().is_none());
        assert_eq!(k.periapsis_radius(), 5000.0.km());
        assert!(k.apoapsis_radius().is_none());
        assert_approx_eq!(k.specific_orbital_energy(MU_EARTH), 0.0, atol <= 1e-12);

        let k1 = k.to_cartesian(MU_EARTH).to_keplerian(MU_EARTH).unwrap();
        assert_approx_eq!(k, k1, rtol <= 1e-10);
    }

    #[test]
    fn test_hyperbolic_round_trip() {
        let k = Keplerian::builder()
            .with_semi_major_axis(-20000.0.km(), 1.5)
            .with_inclination(0.5.rad())
            .with_longitude_of_ascending_node(1.0.rad())
            .with_argument_of_periapsis(2.0.rad())
            .with_true_anomaly(-1.0.rad())
            .build()
            .unwrap();
        assert_approx_eq!(k.semi_major_axis().unwrap(), -20000.0.km(), rtol <= 1e-14);
        assert!(k.specific_orbital_energy(MU_EARTH) > 0.0);

        let k1 = k.to_cartesian(MU_EARTH).to_keplerian(MU_EARTH).unwrap();
        assert_approx_eq!(k, k1, rtol <= 1e-10);
    }

    #[test]
    fn test_semi_major_axis_rejected_for_parabolic() {
        let result = Keplerian::builder()
            .with_semi_major_axis(10000.0.km(), 1.0)
            .build();
        assert!(matches!(result, Err(KeplerianError::InvalidShape { .. })));
    }

    #[test]
    fn test_semi_major_axis_rejected_for_wrong_sign_or_zero() {
        for (sma, ecc, requirement) in [
            (0.0.km(), 0.5, "must be finite and positive"),
            (
                Distance::new(f64::INFINITY),
                0.0,
                "must be finite and positive",
            ),
            (-7000.0.km(), 0.1, "must be finite and positive"),
            (7000.0.km(), 1.5, "must be finite and negative"),
            (7000.0.km(), 1.0, "is undefined"),
        ] {
            let err = Keplerian::builder()
                .with_semi_major_axis(sma, ecc)
                .build()
                .unwrap_err();
            assert!(matches!(err, KeplerianError::InvalidShape { .. }));
            assert!(err.to_string().contains(requirement), "{err}");
        }
    }

    #[test]
    fn test_invalid_semi_parameter() {
        let result = Keplerian::builder()
            .with_semi_parameter(-1.0.km(), 0.5)
            .build();
        assert!(matches!(
            result,
            Err(KeplerianError::InvalidSemiParameter(_))
        ));
    }

    #[test]
    fn test_builder_ambiguous_anomaly() {
        let result = Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.1)
            .with_true_anomaly(0.1.rad())
            .with_mean_anomaly(0.1.rad())
            .build();
        assert!(matches!(result, Err(KeplerianError::AmbiguousAnomaly)));
    }

    #[test]
    fn test_builder_eccentric_anomaly() {
        let ecc = Eccentricity::try_new(0.3).unwrap();
        let k = Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.3)
            .with_eccentric_anomaly(1.0.rad())
            .build()
            .unwrap();
        let expected = EccentricAnomaly::new(1.0.rad()).to_true(ecc);
        assert_approx_eq!(k.true_anomaly(), expected, rtol <= 1e-14);
    }

    #[test]
    fn test_builder_rejects_true_anomaly_beyond_asymptote() {
        // The asymptote angle for e = 2 is 120°.
        let result = Keplerian::builder()
            .with_semi_major_axis(-20000.0.km(), 2.0)
            .with_true_anomaly(130.0.deg())
            .build();
        assert!(matches!(
            result,
            Err(KeplerianError::Anomaly(
                AnomalyError::InvalidTrueAnomaly { .. }
            ))
        ));
    }

    #[test]
    fn test_builder_normalizes_true_anomaly() {
        let k = Keplerian::builder()
            .with_semi_major_axis(-20000.0.km(), 2.0)
            .with_true_anomaly(350.0.deg())
            .build()
            .unwrap();
        assert_approx_eq!(k.true_anomaly().as_angle(), -10.0.deg(), rtol <= 1e-12);

        let k = Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.1)
            .with_true_anomaly(-10.0.deg())
            .build()
            .unwrap();
        assert_approx_eq!(k.true_anomaly().as_angle(), 350.0.deg(), rtol <= 1e-12);
    }

    #[test]
    fn test_builder_normalizes_node_and_argument_of_periapsis() {
        let k = Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.1)
            .with_inclination(45.0.deg())
            .with_longitude_of_ascending_node(-30.0.deg())
            .with_argument_of_periapsis(400.0.deg())
            .build()
            .unwrap();
        assert_approx_eq!(
            k.longitude_of_ascending_node().as_f64(),
            330.0_f64.to_radians(),
            rtol <= 1e-12
        );
        assert_approx_eq!(
            k.argument_of_periapsis().as_f64(),
            40.0_f64.to_radians(),
            rtol <= 1e-12
        );
    }

    #[test]
    fn test_builder_rejects_non_finite_angles() {
        let result = Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.1)
            .with_longitude_of_ascending_node(Angle::new(f64::INFINITY))
            .build();
        assert!(matches!(
            result,
            Err(KeplerianError::InvalidLongitudeOfAscendingNode(_))
        ));
        let result = Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.1)
            .with_argument_of_periapsis(Angle::new(f64::NAN))
            .build();
        assert!(matches!(
            result,
            Err(KeplerianError::InvalidArgumentOfPeriapsis(_))
        ));
        for builder in [
            Keplerian::builder().with_true_anomaly(Angle::new(f64::NAN)),
            Keplerian::builder().with_eccentric_anomaly(Angle::new(f64::INFINITY)),
            Keplerian::builder().with_mean_anomaly(Angle::new(f64::NEG_INFINITY)),
        ] {
            let result = builder.with_semi_major_axis(7000.0.km(), 0.1).build();
            assert!(matches!(result, Err(KeplerianError::NonFiniteAnomaly(_))));
        }
    }

    #[test]
    fn test_derived_quantities() {
        let k = elliptic();
        let a = 24464.560e3;
        let e = 0.7311;
        let mu = MU_EARTH.as_f64();
        let state = k.to_cartesian(MU_EARTH);
        let r = state.position();
        let v = state.velocity();

        assert_approx_eq!(k.semi_major_axis().unwrap().as_f64(), a, rtol <= 1e-14);
        assert_approx_eq!(k.periapsis_radius().as_f64(), a * (1.0 - e), rtol <= 1e-14);
        assert_approx_eq!(
            k.apoapsis_radius().unwrap().as_f64(),
            a * (1.0 + e),
            rtol <= 1e-14
        );
        assert_approx_eq!(
            k.periapsis_altitude(6371.0.km()).as_f64(),
            a * (1.0 - e) - 6371e3,
            rtol <= 1e-14
        );
        assert_approx_eq!(
            k.mean_motion(MU_EARTH).as_f64(),
            sqrt(mu / powi(a, 3)),
            rtol <= 1e-14
        );
        assert_approx_eq!(
            k.specific_orbital_energy(MU_EARTH),
            v.length_squared() / 2.0 - mu / r.length(),
            rtol <= 1e-10
        );
        assert_approx_eq!(
            k.specific_angular_momentum(MU_EARTH),
            r.cross(v).length(),
            rtol <= 1e-10
        );
        assert_approx_eq!(
            k.flight_path_angle().as_f64(),
            asin(r.dot(v) / (r.length() * v.length())),
            rtol <= 1e-10
        );
        assert_approx_eq!(
            k.argument_of_latitude().as_f64(),
            (3.10686 + 0.44369564302687126) % TAU,
            rtol <= 1e-14
        );
        assert_approx_eq!(
            k.true_longitude().as_f64(),
            (1.00681 + 3.10686 + 0.44369564302687126) % TAU,
            rtol <= 1e-14
        );
        let mean_anomaly = k.true_anomaly().to_mean(k.eccentricity()).unwrap();
        assert_approx_eq!(
            k.mean_longitude().unwrap().as_f64(),
            (1.00681 + 3.10686 + mean_anomaly.as_f64()) % TAU,
            rtol <= 1e-14
        );
    }

    #[test]
    fn test_iter_trace_open_orbit() {
        let k = Keplerian::builder()
            .with_semi_major_axis(-20000.0.km(), 1.5)
            .build()
            .unwrap();
        assert!(k.iter_trace(MU_EARTH, 10).is_none());
        assert!(k.trace(MU_EARTH, 10).is_none());
        assert_eq!(elliptic().trace(MU_EARTH, 10).unwrap().len(), 10);
    }

    /// Eccentricities covering every orbit type, including the exact edge values and both
    /// sides of the circular and parabolic classification thresholds.
    fn any_eccentricity() -> impl Strategy<Value = f64> {
        prop_oneof![
            Just(0.0),
            0.0..2e-8,
            2e-8..0.95,
            1.0 - 2e-8..1.0 + 2e-8,
            Just(1.0),
            1.05..5.0,
        ]
    }

    fn any_inclination() -> impl Strategy<Value = f64> {
        prop_oneof![Just(0.0), Just(PI), 0.0..=PI]
    }

    /// Builds elements from a true anomaly given as a fraction of the valid range, i.e.
    /// [-π, π) for closed orbits and the open interval between the asymptotes otherwise.
    fn elements(p: f64, ecc: f64, inc: f64, node: f64, argp: f64, nu_fraction: f64) -> Keplerian {
        let eccentricity = Eccentricity::try_new(ecc).unwrap();
        let max_nu = match eccentricity.orbit_type() {
            OrbitType::Circular | OrbitType::Elliptic => PI,
            OrbitType::Parabolic => 0.95 * PI,
            OrbitType::Hyperbolic => 0.95 * hyperbolic_asymptote_angle(eccentricity).as_f64(),
        };
        Keplerian::builder()
            .with_semi_parameter(p.m(), ecc)
            .with_inclination(inc.rad())
            .with_longitude_of_ascending_node(node.rad())
            .with_argument_of_periapsis(argp.rad())
            .with_true_anomaly((nu_fraction * max_nu).rad())
            .build()
            .unwrap()
    }

    fn assert_states_close(expected: Cartesian, actual: Cartesian, rtol: f64) {
        let r = expected.position();
        let v = expected.velocity();
        let dr = (actual.position() - r).length();
        let dv = (actual.velocity() - v).length();
        assert!(
            dr <= rtol * r.length(),
            "position: {r} vs {}",
            actual.position()
        );
        assert!(
            dv <= rtol * v.length(),
            "velocity: {v} vs {}",
            actual.velocity()
        );
    }

    proptest! {
        #[test]
        fn prop_cartesian_round_trip_from_elements(
            p in 6.6e6..1e8f64,
            ecc in any_eccentricity(),
            inc in any_inclination(),
            node in 0.0..TAU,
            argp in 0.0..TAU,
            nu_fraction in -1.0..1.0f64,
        ) {
            let k = elements(p, ecc, inc, node, argp, nu_fraction);
            let state = k.to_cartesian(MU_EARTH);
            let state1 = state.to_keplerian(MU_EARTH).unwrap().to_cartesian(MU_EARTH);
            assert_states_close(state, state1, 1e-7);
        }

        #[test]
        fn prop_cartesian_round_trip_from_states(
            r in 6.6e6..1e8f64,
            lon in 0.0..TAU,
            lat in -FRAC_PI_2..FRAC_PI_2,
            // Classical elements are singular for rectilinear orbits (h → 0), where the
            // reconstruction loses accuracy as eps / (1 - e), so radial velocities are excluded.
            flight_path_angle in -1.5..1.5f64,
            heading in 0.0..TAU,
            // Up to 1.5 times the escape velocity
            v_scale in 0.05..1.5f64,
            equatorial in prop_oneof![Just(None), Just(Some(1.0)), Just(Some(-1.0))],
        ) {
            let (sin_lon, cos_lon) = sin_cos(lon);
            let east = DVec3::new(-sin_lon, cos_lon, 0.0);
            let (r_dir, horizontal) = match equatorial {
                // Exactly in the equatorial plane, prograde or retrograde
                Some(direction) => (DVec3::new(cos_lon, sin_lon, 0.0), direction * east),
                None => {
                    let (sin_lat, cos_lat) = sin_cos(lat);
                    let r_dir = DVec3::new(cos_lat * cos_lon, cos_lat * sin_lon, sin_lat);
                    let north = DVec3::new(-sin_lat * cos_lon, -sin_lat * sin_lon, cos_lat);
                    let (sin_h, cos_h) = sin_cos(heading);
                    (r_dir, cos_h * east + sin_h * north)
                }
            };
            let (sin_fpa, cos_fpa) = sin_cos(flight_path_angle);
            let v = v_scale * sqrt(2.0 * MU_EARTH.as_f64() / r);
            let state = Cartesian::from_vecs(r * r_dir, v * (sin_fpa * r_dir + cos_fpa * horizontal));
            let state1 = state.to_keplerian(MU_EARTH).unwrap().to_cartesian(MU_EARTH);
            assert_states_close(state, state1, 1e-8);
        }

        #[test]
        fn prop_keplerian_round_trip(
            p in 6.6e6..1e8f64,
            ecc in prop_oneof![1e-3..0.95, 1.05..5.0],
            inc in 1e-3..PI - 1e-3,
            node in 0.0..TAU,
            argp in 0.0..TAU,
            nu_fraction in -1.0..1.0f64,
        ) {
            let k = elements(p, ecc, inc, node, argp, nu_fraction);
            let k1 = k.to_cartesian(MU_EARTH).to_keplerian(MU_EARTH).unwrap();
            assert_approx_eq!(k.semi_parameter(), k1.semi_parameter(), rtol <= 1e-10);
            assert_approx_eq!(k.eccentricity(), k1.eccentricity(), rtol <= 1e-8);
            assert_approx_eq!(k.inclination(), k1.inclination(), atol <= 1e-10);
            assert_approx_eq!(
                k.longitude_of_ascending_node(),
                k1.longitude_of_ascending_node(),
                atol <= 1e-7
            );
            assert_approx_eq!(k.argument_of_periapsis(), k1.argument_of_periapsis(), atol <= 1e-7);
            assert_approx_eq!(k.true_anomaly(), k1.true_anomaly(), atol <= 1e-7);
            let nu = k1.true_anomaly().as_f64();
            if k1.eccentricity().is_circular_or_elliptic() {
                prop_assert!((0.0..TAU).contains(&nu));
            } else {
                prop_assert!((-PI..PI).contains(&nu));
            }
        }

        /// Ω and ω are undefined for circular and equatorial orbits, so only the angles that
        /// remain defined are compared.
        #[test]
        fn prop_keplerian_round_trip_degenerate(
            p in 6.6e6..1e8f64,
            ecc in prop_oneof![Just(0.0), 1e-3..0.95, 1.05..5.0],
            inc in prop_oneof![Just(0.0), Just(PI), 1e-3..PI - 1e-3],
            node in 0.0..TAU,
            argp in 0.0..TAU,
            nu_fraction in -1.0..1.0f64,
        ) {
            let k = elements(p, ecc, inc, node, argp, nu_fraction);
            let k1 = k.to_cartesian(MU_EARTH).to_keplerian(MU_EARTH).unwrap();
            let (expected, actual) = if ecc != 0.0 && inc != 0.0 && inc != PI {
                return Ok(());
            } else if inc == 0.0 || inc == PI {
                // The azimuth of the position in the equatorial plane
                let azimuth = |k: &Keplerian| k.to_cartesian(MU_EARTH).position().azimuth();
                (azimuth(&k), azimuth(&k1))
            } else {
                (k.argument_of_latitude(), k1.argument_of_latitude())
            };
            prop_assert!(
                ApproxEqResult::periodic(expected.into(), actual.into(), TAU, 1e-7, 0.0).is_pass(),
                "{expected} vs {actual}"
            );
            for angle in [
                k1.longitude_of_ascending_node().as_f64(),
                k1.argument_of_periapsis().as_f64(),
                k1.true_anomaly().as_f64(),
            ] {
                prop_assert!(angle.is_finite());
            }
        }
    }
}
