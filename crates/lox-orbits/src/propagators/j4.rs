// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Analytical J4 orbit propagator using Kozai secular perturbation theory.
//!
//! Extends [`super::j2::J2Propagator`] with J2², and J4 zonal harmonic terms
//! in the secular rates. Optionally applies Kwok J2 short-period corrections.
//!
//! Non-singular for circular (e = 0) and equatorial (i = 0) orbits.
//!
//! # References
//!
//! - Kozai, Y. (1959). *The Astronomical Journal*, 64, 367.
//! - Vallado, D. A. (2013). *Fundamentals of Astrodynamics and
//!   Applications*, 4th ed. pp. 647–654, 708–710.
//! - Hoots, F. R. & Roehrich, R. L. (1980). Spacetrack Report No. 3.

use lox_bodies::{
    DynOrigin, Origin, TryJ2, TryJ4, TryPointMass, TrySpheroid, UndefinedOriginPropertyError,
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

const DEFAULT_STEP_SECONDS: f64 = 60.0;

/// Errors from the Kozai J4 propagator.
#[derive(Debug, Error)]
pub enum J4Error {
    /// The orbit is not elliptic.
    #[error("J4 propagation requires an elliptic orbit, got {0}")]
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

impl From<std::convert::Infallible> for J4Error {
    fn from(x: std::convert::Infallible) -> Self {
        unreachable!("{}", x)
    }
}

/// Analytical J4 orbit propagator (Kozai secular with J2+J2²+J4,
/// optionally osculating).
///
/// Same interface as [`super::j2::J2Propagator`] but uses higher-order
/// secular rates. Short-period corrections (when enabled) are J2-only
/// per the standard reference.
#[derive(Debug, Clone, Copy)]
pub struct J4Propagator<
    T: TimeScale,
    O: TryJ2 + TryJ4 + TryPointMass + TrySpheroid,
    R: ReferenceFrame,
> {
    initial_orbit: KeplerianOrbit<T, O, R>,
    kep: Keplerian,
    m0: f64,
    rates: SecularRates,
    body: BodyConstants,
    osculating: bool,
    step: TimeDelta,
}

/// Type alias for a [`J4Propagator`] using dynamic time scale, origin, and frame.
pub type DynJ4Propagator = J4Propagator<DynTimeScale, DynOrigin, DynFrame>;

impl<T, O, R> J4Propagator<T, O, R>
where
    T: TimeScale + Copy,
    O: TryJ2 + TryJ4 + TryPointMass + TrySpheroid + Origin + Copy,
    R: ReferenceFrame + Copy,
{
    /// Create a new J4 propagator from mean Keplerian elements.
    ///
    /// By default, produces secular-only output. Call
    /// [`with_osculating`](Self::with_osculating) to enable Kwok
    /// short-period corrections.
    pub fn try_new(
        orbit: impl TryInto<KeplerianOrbit<T, O, R>, Error: Into<J4Error>>,
    ) -> Result<Self, J4Error> {
        let orbit = orbit.try_into().map_err(Into::into)?;
        let body = BodyConstants {
            mu: orbit.origin().try_gravitational_parameter()?.as_f64(),
            j2: orbit.origin().try_j2()?,
            r_eq: orbit.origin().try_equatorial_radius()?.as_f64(),
        };
        let j4 = orbit.origin().try_j4()?;

        let kep = orbit.state();
        let ecc = kep.eccentricity();
        if !ecc.is_circular_or_elliptic() {
            return Err(J4Error::NonElliptic(ecc.orbit_type()));
        }

        let a = kep.semi_major_axis().as_f64();
        let e = ecc.as_f64();
        let i = kep.inclination().as_f64();
        let m0 = kep.true_anomaly().to_mean(ecc)?.as_f64();
        let rates = kozai::j4_secular_rates(a, e, i, body.mu, body.j2, j4, body.r_eq);

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

    /// Enable or disable Kwok short-period corrections.
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

impl<T, O, R> Propagator<T, O> for J4Propagator<T, O, R>
where
    T: TimeScale + Copy + PartialOrd,
    O: TryJ2 + TryJ4 + TryPointMass + TrySpheroid + Origin + Copy,
    R: ReferenceFrame + Copy,
{
    type Frame = R;
    type Error = J4Error;

    fn state_at(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, R>, J4Error> {
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

    fn propagate(&self, interval: TimeInterval<T>) -> Result<Trajectory<T, O, R>, J4Error> {
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
    use lox_time::time;
    use lox_time::time_scales::Tdb;
    use lox_units::AngleUnits;

    use lox_core::elements::Keplerian;
    use lox_core::units::Distance;

    use super::*;
    use crate::orbits::KeplerianOrbit;
    use crate::propagators::j2::J2Propagator;

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
    fn test_j4_construction() {
        assert!(J4Propagator::try_new(sso_orbit()).is_ok());
    }

    #[test]
    fn test_j4_circular_orbit_all_true_anomalies() {
        let time = time!(Tdb, 2025, 6, 1).unwrap();
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

            let sec = J4Propagator::try_new(orbit);
            assert!(
                sec.is_ok(),
                "J4 secular failed at TA={ta_deg}°: {}",
                sec.unwrap_err()
            );

            let osc = J4Propagator::try_new(orbit).map(|p| p.with_osculating(true));
            assert!(
                osc.is_ok(),
                "J4 osculating failed at TA={ta_deg}°: {}",
                osc.unwrap_err()
            );
        }
    }

    #[test]
    fn test_j4_rejects_hyperbolic_orbit() {
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
        let err = J4Propagator::try_new(orbit).unwrap_err();
        assert!(matches!(err, J4Error::NonElliptic(_)));
    }

    #[test]
    fn test_j4_propagate_interval() {
        let orbit = sso_orbit();
        let j4 = J4Propagator::try_new(orbit).unwrap();
        let dt = TimeDelta::from_minutes(90);
        let interval = lox_time::intervals::Interval::new(j4.epoch(), j4.epoch() + dt);
        let traj = j4.propagate(interval).unwrap();
        assert!(traj.states().len() > 1);
    }

    #[test]
    fn test_j4_accessors() {
        let orbit = sso_orbit();
        let j4 = J4Propagator::try_new(orbit).unwrap();
        assert!(!j4.is_osculating());
        assert_eq!(j4.epoch(), orbit.time());

        let osc = j4.with_osculating(true);
        assert!(osc.is_osculating());
    }

    #[test]
    fn test_j4_secular_rates_differ_from_j2() {
        let orbit = sso_orbit();
        let j2 = J2Propagator::try_new(orbit).unwrap();
        let j4 = J4Propagator::try_new(orbit).unwrap();

        let t = j2.epoch() + TimeDelta::from_seconds_f64(86400.0);
        let mu = Earth.gravitational_parameter();

        let kep_j2 = j2.state_at(t).unwrap().state().to_keplerian(mu);
        let kep_j4 = j4.state_at(t).unwrap().state().to_keplerian(mu);

        // RAAN should differ by a small amount after 1 day.
        let draan = (kep_j4.longitude_of_ascending_node().as_f64()
            - kep_j2.longitude_of_ascending_node().as_f64())
        .abs();
        assert!(
            draan > 1e-8 && draan < 1e-3,
            "unexpected RAAN diff: {draan}"
        );
    }

    #[test]
    fn test_j4_osculating_has_short_period_oscillations() {
        let orbit = sso_orbit();
        let sec = J4Propagator::try_new(orbit).unwrap();
        let osc = J4Propagator::try_new(orbit).unwrap().with_osculating(true);

        let t = sec.epoch() + TimeDelta::from_seconds_f64(2700.0);
        let pos_sec = sec.state_at(t).unwrap().position();
        let pos_osc = osc.state_at(t).unwrap().position();

        let diff = (pos_osc - pos_sec).length();
        assert!(
            diff > 100.0 && diff < 50_000.0,
            "expected 100 m – 50 km short-period difference, got {diff:.0} m"
        );
    }
}
