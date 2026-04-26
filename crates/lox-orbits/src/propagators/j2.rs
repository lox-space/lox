// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Analytical J2 orbit propagator using Kozai secular perturbation theory.
//!
//! Propagates mean Keplerian elements with first-order J2 secular rates for
//! RAAN, argument of periapsis, and mean anomaly. Semi-major axis,
//! eccentricity, and inclination are held constant.
//!
//! Optionally applies Kwok short-period corrections (Vallado pp. 708–710)
//! to produce osculating elements. Enable with
//! [`with_osculating`](J2Propagator::with_osculating).
//!
//! Non-singular for circular (e = 0) and equatorial (i = 0) orbits.
//!
//! # References
//!
//! - Kozai, Y. (1959). *The Astronomical Journal*, 64, 367.
//! - Vallado, D. A. (2013). *Fundamentals of Astrodynamics and
//!   Applications*, 4th ed. pp. 372, 708–710.

use lox_bodies::{
    DynOrigin, Origin, TryJ2, TryPointMass, TrySpheroid, UndefinedOriginPropertyError,
};
use lox_core::anomalies::AnomalyError;
use lox_core::elements::{Keplerian, OrbitType};
use lox_frames::{DynFrame, ReferenceFrame};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::{DynTimeScale, TimeScale};
use thiserror::Error;

use crate::orbits::{CartesianOrbit, KeplerianOrbit, TrajectorError, Trajectory};
use crate::propagators::Propagator;

use super::kozai::{self, BodyConstants, SecularRates};

/// Default time step for interval propagation (60 seconds).
const DEFAULT_STEP_SECONDS: f64 = 60.0;

/// Errors from the Kozai J2 propagator.
#[derive(Debug, Error)]
pub enum J2Error {
    /// The orbit is not elliptic.
    #[error("J2 propagation requires an elliptic orbit, got {0}")]
    NonElliptic(OrbitType),
    /// The origin body lacks a required physical property.
    #[error("undefined origin property: {0}")]
    UndefinedOriginProperty(#[from] UndefinedOriginPropertyError),
    /// Anomaly conversion failed.
    #[error("anomaly conversion failed: {0}")]
    Anomaly(#[from] AnomalyError),
    /// Error constructing the output trajectory.
    #[error(transparent)]
    Trajectory(#[from] TrajectorError),
}

impl From<std::convert::Infallible> for J2Error {
    fn from(x: std::convert::Infallible) -> Self {
        unreachable!("{}", x)
    }
}

/// Analytical J2 orbit propagator (Kozai secular, optionally osculating).
///
/// Propagates mean Keplerian elements with first-order J2 secular rates.
/// When [`osculating`](Self::with_osculating) is enabled, Kwok short-period
/// corrections are applied to the output state.
///
/// The input orbit is treated as **mean** elements. No osculating-to-mean
/// conversion is performed.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct J2Propagator<T: TimeScale, O: TryJ2 + TryPointMass + TrySpheroid, R: ReferenceFrame> {
    initial_orbit: KeplerianOrbit<T, O, R>,
    kep: Keplerian,
    m0: f64,
    rates: SecularRates,
    body: BodyConstants,
    osculating: bool,
    step: TimeDelta,
}

/// Type alias for a [`J2Propagator`] using dynamic time scale, origin, and frame.
pub type DynJ2Propagator = J2Propagator<DynTimeScale, DynOrigin, DynFrame>;

impl<T, O, R> J2Propagator<T, O, R>
where
    T: TimeScale + Copy,
    O: TryJ2 + TryPointMass + TrySpheroid + Origin + Copy,
    R: ReferenceFrame + Copy,
{
    /// Create a new J2 propagator from mean Keplerian elements.
    ///
    /// By default, produces secular-only output (no short-period corrections).
    /// Call [`with_osculating`](Self::with_osculating) to enable Kwok
    /// short-period corrections.
    ///
    /// Accepts any orbit type convertible to [`KeplerianOrbit`], including
    /// [`CartesianOrbit`].
    pub fn try_new(
        orbit: impl TryInto<KeplerianOrbit<T, O, R>, Error: Into<J2Error>>,
    ) -> Result<Self, J2Error> {
        let orbit = orbit.try_into().map_err(Into::into)?;
        let body = BodyConstants {
            mu: orbit.origin().try_gravitational_parameter()?.as_f64(),
            j2: orbit.origin().try_j2()?,
            r_eq: orbit.origin().try_equatorial_radius()?.as_f64(),
        };

        let kep = orbit.state();
        let ecc = kep.eccentricity();
        if !ecc.is_circular_or_elliptic() {
            return Err(J2Error::NonElliptic(ecc.orbit_type()));
        }

        let a = kep.semi_major_axis().as_f64();
        let e = ecc.as_f64();
        let i = kep.inclination().as_f64();
        let m0 = kep.true_anomaly().to_mean(ecc)?.as_f64();
        let rates = kozai::j2_secular_rates(a, e, i, body.mu, body.j2, body.r_eq);

        Ok(Self {
            initial_orbit: orbit,
            kep,
            m0,
            rates,
            body,
            osculating: false,
            step: TimeDelta::from_seconds_f64(DEFAULT_STEP_SECONDS),
        })
    }

    /// Enable or disable Kwok short-period corrections (Vallado pp. 708–710).
    ///
    /// When enabled, the propagator produces osculating elements; when
    /// disabled (default), it produces mean (secular-only) elements.
    pub fn with_osculating(mut self, osculating: bool) -> Self {
        self.osculating = osculating;
        self
    }

    /// Set the fixed time step for interval propagation.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Return the initial mean Keplerian orbit.
    pub fn initial_orbit(&self) -> &KeplerianOrbit<T, O, R> {
        &self.initial_orbit
    }

    /// Return the epoch.
    pub fn epoch(&self) -> Time<T> {
        self.initial_orbit.time()
    }

    /// Return whether Kwok short-period corrections are enabled.
    pub fn is_osculating(&self) -> bool {
        self.osculating
    }
}

impl<T, O, R> Propagator<T, O> for J2Propagator<T, O, R>
where
    T: TimeScale + Copy + PartialOrd,
    O: TryJ2 + TryPointMass + TrySpheroid + Origin + Copy,
    R: ReferenceFrame + Copy,
{
    type Frame = R;
    type Error = J2Error;

    fn state_at(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, R>, J2Error> {
        let dt = (time - self.initial_orbit.time()).to_seconds().to_f64();
        let el = kozai::propagate_mean(&self.kep, self.m0, &self.rates, dt);
        let cartesian = if self.osculating {
            kozai::kwok_osculating_cartesian(&el, &self.body)?
        } else {
            kozai::mean_to_cartesian(&el, self.body.mu)?
        };
        Ok(CartesianOrbit::new(
            cartesian,
            time,
            self.initial_orbit.origin(),
            self.initial_orbit.reference_frame(),
        ))
    }

    fn propagate(&self, interval: TimeInterval<T>) -> Result<Trajectory<T, O, R>, J2Error> {
        let states: Result<Vec<_>, _> = interval
            .step_by(self.step)
            .map(|t| self.state_at(t))
            .collect();
        Ok(Trajectory::try_new(states?)?)
    }
}

#[cfg(test)]
mod tests {
    use lox_bodies::{Earth, PointMass};
    use lox_frames::Icrf;
    use lox_test_utils::assert_approx_eq;
    use lox_time::time;
    use lox_time::time_scales::Tdb;
    use lox_units::AngleUnits;

    use lox_core::elements::Keplerian;
    use lox_core::units::Distance;

    use super::*;
    use crate::orbits::KeplerianOrbit;

    fn sso_orbit() -> KeplerianOrbit<Tdb, Earth, Icrf> {
        let time = time!(Tdb, 2025, 6, 1).unwrap();
        let kep = Keplerian::builder()
            .with_semi_major_axis(Distance::new(6_878_137.0), 0.001)
            .with_inclination(97.42_f64.to_radians().rad())
            .with_longitude_of_ascending_node(69.3_f64.to_radians().rad())
            .with_argument_of_periapsis(0.0.rad())
            .with_true_anomaly(0.0.rad())
            .build()
            .unwrap();
        KeplerianOrbit::new(kep, time, Earth, Icrf)
    }

    #[test]
    fn test_j2_construction() {
        assert!(J2Propagator::try_new(sso_orbit()).is_ok());
    }

    #[test]
    fn test_j2_circular_orbit_all_true_anomalies() {
        let time = time!(Tdb, 2025, 6, 1).unwrap();

        // Must work for ALL true anomalies, including the range that
        // makes Brouwer-Lyddane diverge (~70°–110°, ~250°–290°).
        for ta_deg in (0..360).step_by(5) {
            let kep = Keplerian::builder()
                .with_semi_major_axis(Distance::new(6_878_137.0), 0.0)
                .with_inclination(97.42_f64.to_radians().rad())
                .with_longitude_of_ascending_node(69.3_f64.to_radians().rad())
                .with_argument_of_periapsis(0.0.rad())
                .with_true_anomaly((ta_deg as f64).to_radians().rad())
                .build()
                .unwrap();
            let orbit = KeplerianOrbit::new(kep, time, Earth, Icrf);

            let sec = J2Propagator::try_new(orbit);
            assert!(
                sec.is_ok(),
                "J2 secular failed at TA={ta_deg}°: {}",
                sec.unwrap_err()
            );

            let osc = J2Propagator::try_new(orbit).map(|p| p.with_osculating(true));
            assert!(
                osc.is_ok(),
                "J2 osculating failed at TA={ta_deg}°: {}",
                osc.unwrap_err()
            );
        }
    }

    #[test]
    fn test_j2_raan_drift_sso() {
        let orbit = sso_orbit();
        let j2 = J2Propagator::try_new(orbit).unwrap();

        let mu = Earth.gravitational_parameter();
        let result = j2
            .state_at(j2.epoch() + TimeDelta::from_seconds_f64(86400.0))
            .unwrap();
        let kep_final = result.state().to_keplerian(mu);

        let draan = kep_final.longitude_of_ascending_node().as_f64()
            - orbit.state().longitude_of_ascending_node().as_f64();
        let draan_deg_per_day = draan.to_degrees();

        // SSO should drift ~+0.9856°/day
        assert_approx_eq!(draan_deg_per_day, 0.9856, atol <= 0.02);
    }

    #[test]
    fn test_j2_rejects_hyperbolic_orbit() {
        let time = time!(Tdb, 2025, 6, 1).unwrap();
        let kep = Keplerian::builder()
            .with_semi_major_axis(Distance::new(-7_000_000.0), 1.5)
            .with_inclination(0.5.rad())
            .with_longitude_of_ascending_node(0.0.rad())
            .with_argument_of_periapsis(0.0.rad())
            .with_true_anomaly(0.0.rad())
            .build()
            .unwrap();
        let orbit = KeplerianOrbit::new(kep, time, Earth, Icrf);
        let err = J2Propagator::try_new(orbit).unwrap_err();
        assert!(matches!(err, J2Error::NonElliptic(_)));
    }

    #[test]
    fn test_j2_propagate_interval() {
        let orbit = sso_orbit();
        let j2 = J2Propagator::try_new(orbit).unwrap();
        let dt = TimeDelta::from_minutes(90);
        let interval = lox_time::intervals::Interval::new(j2.epoch(), j2.epoch() + dt);
        let traj = j2.propagate(interval).unwrap();
        assert!(traj.states().len() > 1);
    }

    #[test]
    fn test_j2_accessors() {
        let orbit = sso_orbit();
        let j2 = J2Propagator::try_new(orbit).unwrap();
        assert!(!j2.is_osculating());
        assert_eq!(j2.epoch(), orbit.time());
        assert_eq!(
            j2.initial_orbit().state().semi_major_axis(),
            orbit.state().semi_major_axis()
        );

        let osc = j2.with_osculating(true);
        assert!(osc.is_osculating());
    }

    #[test]
    fn test_osculating_has_short_period_oscillations() {
        let orbit = sso_orbit();
        let sec = J2Propagator::try_new(orbit).unwrap();
        let osc = J2Propagator::try_new(orbit).unwrap().with_osculating(true);

        // Propagate half an orbit (~45 min for LEO)
        let t = sec.epoch() + TimeDelta::from_seconds_f64(2700.0);
        let pos_sec = sec.state_at(t).unwrap().position();
        let pos_osc = osc.state_at(t).unwrap().position();

        // The positions should differ by ~O(J2·a) ≈ several km
        let diff = (pos_osc - pos_sec).length();
        assert!(
            diff > 100.0 && diff < 50_000.0,
            "expected 100 m – 50 km short-period difference, got {diff:.0} m"
        );
    }
}
