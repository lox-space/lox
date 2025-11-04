// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Data types for representing orbital elements.

use std::f64::consts::PI;
use std::f64::consts::TAU;
use std::fmt::Display;

use glam::DVec3;
use lox_test_utils::ApproxEq;
use lox_test_utils::approx_eq;
use thiserror::Error;

use crate::anomalies::AnomalyError;
use crate::anomalies::MeanAnomaly;
use crate::anomalies::{EccentricAnomaly, TrueAnomaly};
use crate::time::deltas::TimeDelta;
use crate::utils::Linspace;
use crate::{
    coords::Cartesian,
    glam::Azimuth,
    units::{Angle, AngleUnits, Distance, DistanceUnits},
};

/// The standard gravitational parameter of a celestial body µ = GM.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0 * 1e-9).fmt(f)?;
        write!(f, " km³/s²")
    }
}

pub type SemiMajorAxis = Distance;

/// The Keplerian orbit types or conic sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrbitType::Circular => "circular".fmt(f),
            OrbitType::Elliptic => "elliptic".fmt(f),
            OrbitType::Parabolic => "parabolic".fmt(f),
            OrbitType::Hyperbolic => "hyperbolic".fmt(f),
        }
    }
}

#[derive(Debug, Clone, Error)]
#[error("eccentricity cannot be negative but was {0}")]
pub struct NegativeEccentricityError(f64);

/// Orbital eccentricity.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
#[repr(transparent)]
pub struct Eccentricity(f64);

impl Eccentricity {
    /// Tries to create a new [`Eccentricity`] instance from an `f64` value.
    ///
    /// # Errors
    ///
    /// Returns a [`NegativeEccentricityError`] if the value is smaller than zero.
    pub const fn try_new(ecc: f64) -> Result<Eccentricity, NegativeEccentricityError> {
        if ecc < 0.0 {
            return Err(NegativeEccentricityError(ecc));
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Error)]
#[error("inclination must be between 0 and 180 deg but was {0}")]
pub struct InclinationError(Angle);

/// Orbital inclination.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
#[repr(transparent)]
pub struct Inclination(Angle);

impl Inclination {
    pub const fn try_new(inclination: Angle) -> Result<Inclination, InclinationError> {
        let inc = inclination.as_f64();
        if inc < 0.0 || inc > PI {
            return Err(InclinationError(inclination));
        }
        Ok(Inclination(inclination))
    }

    pub const fn as_f64(&self) -> f64 {
        self.0.as_f64()
    }
}

impl Display for Inclination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Error)]
#[error("longitude of ascending node must be between 0 and 360 deg but was {0}")]
pub struct LongitudeOfAscendingNodeError(Angle);

/// Longitude of ascending node.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
#[repr(transparent)]
pub struct LongitudeOfAscendingNode(Angle);

impl LongitudeOfAscendingNode {
    pub const fn try_new(
        longitude_of_ascending_node: Angle,
    ) -> Result<LongitudeOfAscendingNode, LongitudeOfAscendingNodeError> {
        let node = longitude_of_ascending_node.as_f64();
        if node < 0.0 || node > TAU {
            return Err(LongitudeOfAscendingNodeError(longitude_of_ascending_node));
        }
        Ok(LongitudeOfAscendingNode(longitude_of_ascending_node))
    }

    pub const fn as_f64(&self) -> f64 {
        self.0.as_f64()
    }
}

impl Display for LongitudeOfAscendingNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Error)]
#[error("argument of periapsis must be between 0 and 360 deg but was {0}")]
pub struct ArgumentOfPeriapsisError(Angle);

/// Argument of periapsis.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
#[repr(transparent)]
pub struct ArgumentOfPeriapsis(Angle);

impl ArgumentOfPeriapsis {
    pub const fn try_new(
        argument_of_periapsis: Angle,
    ) -> Result<ArgumentOfPeriapsis, ArgumentOfPeriapsisError> {
        let arg = argument_of_periapsis.as_f64();
        if arg < 0.0 || arg > TAU {
            return Err(ArgumentOfPeriapsisError(argument_of_periapsis));
        }
        Ok(ArgumentOfPeriapsis(argument_of_periapsis))
    }

    pub const fn as_f64(&self) -> f64 {
        self.0.as_f64()
    }
}

impl Display for ArgumentOfPeriapsis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, ApproxEq)]
pub struct Keplerian {
    semi_major_axis: SemiMajorAxis,
    eccentricity: Eccentricity,
    inclination: Inclination,
    longitude_of_ascending_node: LongitudeOfAscendingNode,
    argument_of_periapsis: ArgumentOfPeriapsis,
    true_anomaly: TrueAnomaly,
}

impl Keplerian {
    pub fn new(
        semi_major_axis: SemiMajorAxis,
        eccentricity: Eccentricity,
        inclination: Inclination,
        longitude_of_ascending_node: LongitudeOfAscendingNode,
        argument_of_periapsis: ArgumentOfPeriapsis,
        true_anomaly: TrueAnomaly,
    ) -> Self {
        Self {
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_of_ascending_node,
            argument_of_periapsis,
            true_anomaly,
        }
    }

    pub fn builder() -> KeplerianBuilder {
        KeplerianBuilder::default()
    }

    pub fn semi_major_axis(&self) -> SemiMajorAxis {
        self.semi_major_axis
    }

    pub fn eccentricity(&self) -> Eccentricity {
        self.eccentricity
    }

    pub fn inclination(&self) -> Inclination {
        self.inclination
    }

    pub fn longitude_of_ascending_node(&self) -> LongitudeOfAscendingNode {
        self.longitude_of_ascending_node
    }

    pub fn argument_of_periapsis(&self) -> ArgumentOfPeriapsis {
        self.argument_of_periapsis
    }

    pub fn true_anomaly(&self) -> TrueAnomaly {
        self.true_anomaly
    }

    pub fn semi_parameter(&self) -> Distance {
        if self.eccentricity.is_circular() {
            self.semi_major_axis
        } else {
            Distance::new(self.semi_major_axis.as_f64() * (1.0 - self.eccentricity.0.powi(2)))
        }
    }

    pub fn to_perifocal(&self, grav_param: GravitationalParameter) -> (DVec3, DVec3) {
        let ecc = self.eccentricity.as_f64();
        let mu = grav_param.as_f64();
        let semiparameter = self.semi_parameter().as_f64();
        let (sin_nu, cos_nu) = self.true_anomaly.as_angle().sin_cos();
        let sqrt_mu_p = (mu / semiparameter).sqrt();

        let pos = DVec3::new(cos_nu, sin_nu, 0.0) * (semiparameter / (1.0 + ecc * cos_nu));
        let vel = DVec3::new(-sin_nu, ecc + cos_nu, 0.0) * sqrt_mu_p;

        (pos, vel)
    }

    pub fn to_cartesian(&self, grav_param: GravitationalParameter) -> Cartesian {
        let (pos, vel) = self.to_perifocal(grav_param);
        let rot = self.longitude_of_ascending_node.0.rotation_z().transpose()
            * self.inclination.0.rotation_x().transpose()
            * self.argument_of_periapsis.0.rotation_z().transpose();
        Cartesian::from_vecs(rot * pos, rot * vel)
    }

    pub fn orbital_period(&self, grav_param: GravitationalParameter) -> Option<TimeDelta> {
        if !self.eccentricity().is_circular_or_elliptic() {
            return None;
        }
        let mu = grav_param.as_f64();
        let a = self.semi_major_axis.as_f64();
        Some(TimeDelta::from_seconds_f64(TAU * (a.powf(3.0) / mu).sqrt()))
    }

    pub fn iter_trace(
        &self,
        grav_param: GravitationalParameter,
        n: usize,
    ) -> impl Iterator<Item = Cartesian> {
        assert!(self.eccentricity().is_circular_or_elliptic());
        Linspace::new(-PI, PI, n).map(move |ecc| {
            let true_anomaly = EccentricAnomaly::new(ecc.rad()).to_true(self.eccentricity);
            Keplerian {
                true_anomaly,
                ..*self
            }
            .to_cartesian(grav_param)
        })
    }

    pub fn trace(&self, grav_param: GravitationalParameter, n: usize) -> Option<Vec<Cartesian>> {
        if !self.eccentricity().is_circular_or_elliptic() {
            return None;
        }
        Some(self.iter_trace(grav_param, n).collect())
    }
}

impl Cartesian {
    pub fn eccentricity_vector(&self, grav_param: GravitationalParameter) -> DVec3 {
        let mu = grav_param.as_f64();
        let r = self.position();
        let v = self.velocity();

        let rm = r.length();
        let v2 = v.dot(v);
        let rv = r.dot(v);

        ((v2 - mu / rm) * r - rv * v) / mu
    }

    pub fn to_keplerian(&self, grav_param: GravitationalParameter) -> Keplerian {
        let r = self.position();
        let v = self.velocity();
        let mu = grav_param.as_f64();

        let rm = r.length();
        let vm = v.length();
        let h = r.cross(v);
        let hm = h.length();
        let node = DVec3::Z.cross(h);
        let e = self.eccentricity_vector(grav_param);
        let eccentricity = Eccentricity(e.length());
        let inclination = h.angle_between(DVec3::Z).rad();

        let equatorial = approx_eq!(inclination, Angle::ZERO, atol <= 1e-8);
        let circular = eccentricity.is_circular();

        let semi_major_axis = if circular {
            hm.powi(2) / mu
        } else {
            -mu / (2.0 * (vm.powi(2) / 2.0 - mu / rm))
        };

        let (longitude_of_ascending_node, argument_of_periapsis, true_anomaly) =
            if equatorial && !circular {
                (
                    Angle::ZERO,
                    e.azimuth(),
                    TrueAnomaly::new(Angle::from_atan2(h.dot(e.cross(r)) / hm, r.dot(e))),
                )
            } else if !equatorial && circular {
                (
                    node.azimuth(),
                    Angle::ZERO,
                    TrueAnomaly::new(Angle::from_atan2(r.dot(h.cross(node)) / hm, r.dot(node))),
                )
            } else if equatorial && circular {
                (Angle::ZERO, Angle::ZERO, TrueAnomaly::new(r.azimuth()))
            } else {
                let true_anomaly = if semi_major_axis > 0.0 {
                    let e_se = r.dot(v) / (mu * semi_major_axis).sqrt();
                    let e_ce = (rm * vm.powi(2)) / mu - 1.0;
                    EccentricAnomaly::new(Angle::from_atan2(e_se, e_ce)).to_true(eccentricity)
                } else {
                    let e_sh = r.dot(v) / (-mu * semi_major_axis).sqrt();
                    let e_ch = (rm * vm.powi(2)) / mu - 1.0;
                    EccentricAnomaly::new((((e_ch + e_sh) / (e_ch - e_sh)).ln() / 2.0).rad())
                        .to_true(eccentricity)
                };
                let px = r.dot(node);
                let py = r.dot(h.cross(node)) / hm;
                (
                    node.azimuth(),
                    Angle::from_atan2(py, px) - true_anomaly.as_angle(),
                    true_anomaly,
                )
            };

        Keplerian {
            semi_major_axis: semi_major_axis.m(),
            eccentricity,
            inclination: Inclination(inclination),
            longitude_of_ascending_node: LongitudeOfAscendingNode(
                longitude_of_ascending_node.mod_two_pi(),
            ),
            argument_of_periapsis: ArgumentOfPeriapsis(argument_of_periapsis.mod_two_pi()),
            true_anomaly,
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum KeplerianError {
    #[error(transparent)]
    NegativeEccentricity(#[from] NegativeEccentricityError),
    #[error(
        "{} semi-major axis ({semi_major_axis}) for {} eccentricity ({eccentricity})",
        if .semi_major_axis.as_f64().signum() == -1.0 {"negative"} else {"positive"},
        .eccentricity.orbit_type()
    )]
    InvalidShape {
        semi_major_axis: SemiMajorAxis,
        eccentricity: Eccentricity,
    },
    #[error(
        "no orbital shape parameters (semi-major axis and eccentricity, radii, or altitudes) were provided"
    )]
    MissingShape,
    #[error(transparent)]
    InvalidInclination(#[from] InclinationError),
    #[error(transparent)]
    InvalidLongitudeOfAscendingNode(#[from] LongitudeOfAscendingNodeError),
    #[error(transparent)]
    InvalidArgumentOfPeriapsis(#[from] ArgumentOfPeriapsisError),
    #[error(transparent)]
    Anomaly(#[from] AnomalyError),
}

#[derive(Debug, Default, Clone)]
pub struct KeplerianBuilder {
    shape: Option<(
        SemiMajorAxis,
        Result<Eccentricity, NegativeEccentricityError>,
    )>,
    inclination: Angle,
    longitude_of_ascending_node: Angle,
    argument_of_periapsis: Angle,
    true_anomaly: Option<Angle>,
    mean_anomaly: Option<Angle>,
}

impl KeplerianBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_semi_major_axis(
        mut self,
        semi_major_axis: SemiMajorAxis,
        eccentricity: f64,
    ) -> Self {
        self.shape = Some((semi_major_axis, Eccentricity::try_new(eccentricity)));
        self
    }

    pub fn with_radii(mut self, periapsis_radius: Distance, apoapsis_radius: Distance) -> Self {
        let rp = periapsis_radius.as_f64();
        let ra = apoapsis_radius.as_f64();
        let semi_major_axis = SemiMajorAxis::new((rp + ra) / 2.0);

        let eccentricity = Eccentricity::try_new((ra - rp) / (ra + rp));

        self.shape = Some((semi_major_axis, eccentricity));

        self
    }

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

    pub fn with_inclination(mut self, inclination: Angle) -> Self {
        self.inclination = inclination;
        self
    }

    pub fn with_longitude_of_ascending_node(mut self, longitude_of_ascending_node: Angle) -> Self {
        self.longitude_of_ascending_node = longitude_of_ascending_node;
        self
    }

    pub fn with_argument_of_periapsis(mut self, argument_of_periapsis: Angle) -> Self {
        self.argument_of_periapsis = argument_of_periapsis;
        self
    }

    pub fn with_true_anomaly(mut self, true_anomaly: Angle) -> Self {
        self.true_anomaly = Some(true_anomaly);
        self
    }

    pub fn with_mean_anomaly(mut self, mean_anomaly: Angle) -> Self {
        self.mean_anomaly = Some(mean_anomaly);
        self
    }

    pub fn build(self) -> Result<Keplerian, KeplerianError> {
        let (semi_major_axis, eccentricity) = self.shape.ok_or(KeplerianError::MissingShape)?;

        let eccentricity = eccentricity?;

        Self::check_shape(semi_major_axis, eccentricity)?;

        let inclination = Inclination::try_new(self.inclination)?;
        let longitude_of_ascending_node =
            LongitudeOfAscendingNode::try_new(self.longitude_of_ascending_node)?;
        let argument_of_periapsis = ArgumentOfPeriapsis::try_new(self.argument_of_periapsis)?;

        let true_anomaly = match self.true_anomaly {
            Some(true_anomaly) => TrueAnomaly::new(true_anomaly),
            None => match self.mean_anomaly {
                Some(mean_anomaly) => MeanAnomaly::new(mean_anomaly).to_true(eccentricity)?,
                None => TrueAnomaly::new(Angle::ZERO),
            },
        };

        Ok(Keplerian {
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_of_ascending_node,
            argument_of_periapsis,
            true_anomaly,
        })
    }

    fn check_shape(
        semi_major_axis: SemiMajorAxis,
        eccentricity: Eccentricity,
    ) -> Result<(), KeplerianError> {
        let ecc = eccentricity.as_f64();
        let sma = semi_major_axis.as_f64();
        if (ecc > 1.0 && sma > 0.0) || (ecc < 1.0 && sma < 0.0) {
            return Err(KeplerianError::InvalidShape {
                semi_major_axis,
                eccentricity,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use crate::units::VelocityUnits;

    use super::*;

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

        let cartesian1 = cartesian.to_keplerian(mu).to_cartesian(mu);

        assert_approx_eq!(cartesian.position(), cartesian1.position(), rtol <= 1e-8);
        assert_approx_eq!(cartesian.velocity(), cartesian1.velocity(), rtol <= 1e-6);
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
        let k1 = k.to_cartesian(mu).to_keplerian(mu);
        assert_approx_eq!(k, k1);
    }
}
