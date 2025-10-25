// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Data types for representing orbital elements.

use glam::DVec3;
use lox_test_utils::approx_eq;
use lox_test_utils::{
    ApproxEq,
    approx_eq::{ApproxEq, ApproxEqResult, ApproxEqResults},
};
use thiserror::Error;

use crate::{Angle, AngleUnits, Distance, DistanceUnits, coords::Cartesian, glam::Azimuth};

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

#[derive(Debug, Error)]
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
    pub fn try_new(ecc: f64) -> Result<Eccentricity, NegativeEccentricityError> {
        if ecc < 0.0 {
            return Err(NegativeEccentricityError(ecc));
        }
        Ok(Eccentricity(ecc))
    }

    /// Returns the value of the eccentricity as an `f64`.
    pub fn as_f64(&self) -> f64 {
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
}

fn hyperbolic_to_true(hyperbolic_anomaly: f64, eccentricity: f64) -> f64 {
    2.0 * (((1.0 + eccentricity) / (eccentricity - 1.0)).sqrt() * (hyperbolic_anomaly / 2.0).tanh())
        .atan()
}

fn eccentric_to_true(eccentric_anomaly: f64, eccentricity: f64) -> f64 {
    2.0 * (((1.0 + eccentricity) / (1.0 - eccentricity)).sqrt() * (eccentric_anomaly / 2.0).tan())
        .atan()
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Anomaly {
    True(Angle),
    Eccentric(Angle),
    Mean(Angle),
}

impl Anomaly {
    pub fn new(anomaly: Angle) -> Self {
        Self::True(anomaly.normalize_two_pi(Angle::ZERO))
    }

    pub fn eccentric(anomaly: Angle) -> Self {
        Self::Eccentric(anomaly.normalize_two_pi(Angle::ZERO))
    }

    pub fn mean(anomaly: Angle) -> Self {
        Self::Mean(anomaly.normalize_two_pi(Angle::ZERO))
    }

    pub fn to_true(&self, ecc: Eccentricity) -> Self {
        let orbit = ecc.orbit_type();
        let ecc = ecc.as_f64();
        match (self, orbit) {
            (Anomaly::Eccentric(anomaly), OrbitType::Circular | OrbitType::Elliptic) => {
                Self::new(eccentric_to_true(anomaly.as_f64(), ecc).rad())
            }
            (Anomaly::Eccentric(anomaly), OrbitType::Hyperbolic) => {
                Self::new(hyperbolic_to_true(anomaly.as_f64(), ecc).rad())
            }
            (Anomaly::Eccentric(_), OrbitType::Parabolic) => todo!(),
            (Anomaly::Mean(_), OrbitType::Circular) => todo!(),
            (Anomaly::Mean(_), OrbitType::Elliptic) => todo!(),
            (Anomaly::Mean(_), OrbitType::Parabolic) => todo!(),
            (Anomaly::Mean(_), OrbitType::Hyperbolic) => todo!(),
            (Anomaly::True(_), _) => *self,
        }
    }

    pub fn normalize(&self) -> Self {
        match self {
            Anomaly::True(angle) => Anomaly::True(angle.normalize_two_pi(Angle::ZERO)),
            Anomaly::Eccentric(angle) => Anomaly::Eccentric(angle.normalize_two_pi(Angle::ZERO)),
            Anomaly::Mean(angle) => Anomaly::Mean(angle.normalize_two_pi(Angle::ZERO)),
        }
    }

    pub fn as_angle(&self) -> Angle {
        match self {
            Anomaly::True(angle) | Anomaly::Eccentric(angle) | Anomaly::Mean(angle) => *angle,
        }
    }

    pub fn as_f64(&self) -> f64 {
        self.as_angle().as_f64()
    }
}

impl ApproxEq for Anomaly {
    fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> ApproxEqResults {
        match (self, rhs) {
            (Anomaly::True(lhs), Anomaly::True(rhs))
            | (Anomaly::Eccentric(lhs), Anomaly::Eccentric(rhs))
            | (Anomaly::Mean(lhs), Anomaly::Mean(rhs)) => lhs.approx_eq(rhs, atol, rtol),
            (_, _) => ApproxEqResults::single(ApproxEqResult::fail(self.as_f64(), rhs.as_f64())),
        }
    }
}

/// The standard gravitational parameter of a celestial body µ = GM.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, ApproxEq)]
#[repr(transparent)]
pub struct GravitationalParameter(f64);

impl GravitationalParameter {
    /// Creates a new gravitational parameter from an `f64` value in m³/s².
    pub fn m3_per_s2(mu: f64) -> Self {
        Self(mu)
    }

    /// Creates a new gravitational parameter from an `f64` value in km³/s².
    pub fn km3_per_s2(mu: f64) -> Self {
        Self(1e9 * mu)
    }

    /// Returns the value of the gravitational parameters as an `f64`.
    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, ApproxEq)]
pub struct Keplerian {
    pub semi_major_axis: Distance,
    pub eccentricity: Eccentricity,
    pub inclination: Angle,
    pub longitude_of_ascending_node: Angle,
    pub argument_of_periapsis: Angle,
    pub anomaly: Anomaly,
}

impl Keplerian {
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
        let (sin_nu, cos_nu) = self.anomaly.to_true(self.eccentricity).as_angle().sin_cos();
        let sqrt_mu_p = (mu / semiparameter).sqrt();

        let pos = DVec3::new(cos_nu, sin_nu, 0.0) * (semiparameter / (1.0 + ecc * cos_nu));
        let vel = DVec3::new(-sin_nu, ecc + cos_nu, 0.0) * sqrt_mu_p;

        (pos, vel)
    }

    pub fn to_cartesian(&self, grav_param: GravitationalParameter) -> Cartesian {
        let (pos, vel) = self.to_perifocal(grav_param);
        let rot = self.longitude_of_ascending_node.rotation_z().transpose()
            * self.inclination.rotation_x().transpose()
            * self.argument_of_periapsis.rotation_z().transpose();
        Cartesian::from_vecs(rot * pos, rot * vel)
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
                    Anomaly::True(Angle::from_atan2(h.dot(e.cross(r)) / hm, r.dot(e))),
                )
            } else if !equatorial && circular {
                (
                    node.azimuth(),
                    Angle::ZERO,
                    Anomaly::True(Angle::from_atan2(r.dot(h.cross(node)) / hm, r.dot(node))),
                )
            } else if equatorial && circular {
                (Angle::ZERO, Angle::ZERO, Anomaly::new(r.azimuth()))
            } else {
                let true_anomaly = if semi_major_axis > 0.0 {
                    let e_se = r.dot(v) / (mu * semi_major_axis).sqrt();
                    let e_ce = (rm * vm.powi(2)) / mu - 1.0;
                    Anomaly::Eccentric(Angle::from_atan2(e_se, e_ce)).to_true(eccentricity)
                } else {
                    let e_sh = r.dot(v) / (-mu * semi_major_axis).sqrt();
                    let e_ch = (rm * vm.powi(2)) / mu - 1.0;
                    Anomaly::eccentric((((e_ch + e_sh) / (e_ch - e_sh)).ln() / 2.0).rad())
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
            inclination,
            longitude_of_ascending_node: longitude_of_ascending_node.mod_two_pi(),
            argument_of_periapsis: argument_of_periapsis.mod_two_pi(),
            anomaly: true_anomaly.normalize(),
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use crate::VelocityUnits;

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
}
