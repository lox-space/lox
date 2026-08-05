// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Power budget analysis: sun beta angle, eclipse detection, solar flux.
//!
//! The shadow model uses cylindrical (umbra-only) geometry via the existing
//! [`line_of_sight`](crate::visibility::line_of_sight) function.  Penumbra is
//! **not** modelled.

use lox_bodies::{CoordinateOrigin, Sun, TryMeanRadius, TrySpheroid};
use lox_core::glam::DVec3;
use lox_core::units::ASTRONOMICAL_UNIT;
use lox_ephem::Ephemeris;
use lox_time::Time;
use lox_time::time_scales::Tdb;

use crate::events::DetectFn;
use crate::visibility::{EvalError, LineOfSight};
use lox_frames::ReferenceFrame;
use lox_orbits::orbits::Trajectory;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Solar constant at 1 AU in W/m².
pub const SOLAR_CONSTANT: f64 = 1361.0;

// ---------------------------------------------------------------------------
// Pure geometry functions
// ---------------------------------------------------------------------------

/// Computes the Sun beta angle — the angle between the orbit plane and the
/// Sun direction.
///
/// Returns radians in \[-π/2, π/2\].
///
/// # Arguments
/// * `orbit_normal` — unit normal of the orbital plane (`(r × v).normalize()`)
/// * `sun_direction` — unit vector towards the Sun
pub fn beta_angle(orbit_normal: DVec3, sun_direction: DVec3) -> f64 {
    orbit_normal.dot(sun_direction).clamp(-1.0, 1.0).asin()
}

/// Computes the solar flux at the given distance from the Sun.
///
/// Returns W/m² using the inverse-square law relative to
/// [`SOLAR_CONSTANT`] at 1 AU.
///
/// # Arguments
/// * `distance_m` — distance from the Sun in **meters**
pub fn solar_flux(distance_m: f64) -> f64 {
    let ratio = ASTRONOMICAL_UNIT / distance_m;
    SOLAR_CONSTANT * ratio * ratio
}

// ---------------------------------------------------------------------------
// Eclipse DetectFn
// ---------------------------------------------------------------------------

/// Eclipse detect function: positive when the spacecraft is sunlit, negative
/// when it is in eclipse (cylindrical shadow model, umbra only).
pub(crate) struct EclipseDetectFn<'a, O: CoordinateOrigin, R: ReferenceFrame, E> {
    pub(crate) sc: &'a Trajectory<O, R>,
    pub(crate) ephemeris: &'a E,
}

impl<O, R, E: Ephemeris> DetectFn for EclipseDetectFn<'_, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy,
    R: ReferenceFrame + Copy,
    E::Error: 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        let tdb = time.to_scale(Tdb);
        let r_sun = self
            .ephemeris
            .position(tdb, self.sc.origin(), Sun)
            .map_err(|e| EvalError::Ephemeris(Box::new(e)))?;
        let r_sc = self.sc.at(time).position();
        // line_of_sight returns positive when the two vectors have mutual LOS
        // (spacecraft is sunlit) and negative when occluded (eclipse).
        Ok(self.sc.origin().line_of_sight(r_sc, r_sun)?)
    }
}

// The eager `PowerBudgetAnalysis` / `PowerBudgetResults` implementation moved to
// `crate::legacy` for one commit while the Python bindings are ported. Power is
// the one analysis the pipeline does not cover uniformly: only eclipses are
// event-shaped, so the replacement exposes those as items and the continuous
// beta-angle and solar-flux channels as sampled series.
pub use crate::pipeline::analyses::{PowerBudgetAnalysis, SpacecraftPower};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    // See the note in `visibility`'s test module.
    use crate::assets::{Scenario, Spacecraft};
    use crate::pipeline::Parallelism;
    use lox_orbits::orbits::Ensemble;
    use lox_time::deltas::TimeDelta;
    use lox_time::intervals::TimeInterval;
    use std::collections::HashMap;

    use lox_time::time_scales::TimeScale;
    use std::f64::consts::{FRAC_PI_2, PI};
    use std::sync::OnceLock;

    use lox_approx::assert_approx_eq;
    use lox_bodies::Origin;
    use lox_core::glam::DVec3;
    use lox_ephem::spk::parser::Spk;
    use lox_frames::Frame;
    use lox_orbits::propagators::sgp4::{Elements, Sgp4};
    use lox_orbits::propagators::{OrbitSource, Propagator};
    use lox_test_utils::data_file;
    use lox_time::intervals::Interval;

    use super::*;

    #[test]
    fn test_beta_angle_sun_in_orbit_plane() {
        // Sun in the orbit plane → beta = 0
        let h = DVec3::Z;
        let sun = DVec3::X;
        assert_approx_eq!(beta_angle(h, sun), 0.0, atol <= 1e-15);
    }

    #[test]
    fn test_beta_angle_sun_perpendicular() {
        // Sun along orbit normal → beta = π/2
        let h = DVec3::Z;
        let sun = DVec3::Z;
        assert_approx_eq!(beta_angle(h, sun), FRAC_PI_2, atol <= 1e-15);
    }

    #[test]
    fn test_beta_angle_sun_opposite() {
        // Sun opposite to orbit normal → beta = -π/2
        let h = DVec3::Z;
        let sun = -DVec3::Z;
        assert_approx_eq!(beta_angle(h, sun), -FRAC_PI_2, atol <= 1e-15);
    }

    #[test]
    fn test_beta_angle_45_degrees() {
        let h = DVec3::Z;
        let sun = DVec3::new(1.0, 0.0, 1.0).normalize();
        assert_approx_eq!(beta_angle(h, sun), PI / 4.0, atol <= 1e-15);
    }

    #[test]
    fn test_solar_flux_at_1au() {
        assert_approx_eq!(solar_flux(ASTRONOMICAL_UNIT), SOLAR_CONSTANT, rtol <= 1e-10);
    }

    #[test]
    fn test_solar_flux_inverse_square() {
        let d = 2.0 * ASTRONOMICAL_UNIT;
        assert_approx_eq!(solar_flux(d), SOLAR_CONSTANT / 4.0, rtol <= 1e-10);
    }

    #[test]
    fn test_power_budget_integration() {
        fn ephemeris() -> &'static Spk {
            static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
            EPHEMERIS.get_or_init(|| Spk::from_file(data_file("spice/de440s.bsp")).unwrap())
        }

        // ISS in LEO — guaranteed multiple eclipses per day.
        let iss = Elements::from_tle(
            Some("ISS (ZARYA)".to_string()),
            b"1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996",
            b"2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731",
        )
        .unwrap();
        let sgp4 = Sgp4::new(iss).unwrap();
        let t0 = sgp4.time();
        let t1 = t0 + TimeDelta::from_hours(24);
        let sc_traj = sgp4
            .with_step(TimeDelta::from_seconds(30))
            .propagate(Interval::new(t0, t1).into_dynamic())
            .unwrap()
            .into_dynamic();

        let scenario_interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());

        let sc = Spacecraft::new("ISS", OrbitSource::Trajectory(sc_traj.clone()));
        let scenario = Scenario::with_interval(scenario_interval, Origin::Earth, Frame::Icrf)
            .with_spacecraft(std::slice::from_ref(&sc));

        // Build ensemble
        let (epoch, origin, frame, data) = sc_traj.into_parts();
        let typed = Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data);
        let mut map = HashMap::new();
        map.insert(sc.id().clone(), typed);
        let ensemble = Ensemble::new(map);

        let spk = ephemeris();
        let analysis = PowerBudgetAnalysis::new(&scenario, &ensemble, spk);
        let eclipses = analysis
            .eclipses(&sc, scenario_interval)
            .expect("power budget analysis");
        assert!(
            eclipses.len() >= 10,
            "expected ≥10 eclipse intervals for ISS over 24h, got {}",
            eclipses.len()
        );

        // Eclipse fraction for ISS is typically ~35%. Computed here rather than
        // read off a results object: the aggregate is the caller's to derive.
        let eclipsed: f64 = eclipses
            .iter()
            .map(|e| e.0.duration().to_seconds().to_f64())
            .sum();
        let total = scenario_interval.duration().to_seconds().to_f64();
        let eclipse_frac = eclipsed / total;
        assert!(
            (0.2..0.5).contains(&eclipse_frac),
            "unexpected eclipse fraction: {eclipse_frac}"
        );

        let betas = analysis
            .beta_angle(&sc, scenario_interval)
            .expect("beta angles");
        assert!(!betas.values().is_empty());
        for &b in betas.values() {
            assert!((-FRAC_PI_2..=FRAC_PI_2).contains(&b));
        }

        let fluxes = analysis
            .solar_flux(&sc, scenario_interval)
            .expect("solar flux");
        assert!(!fluxes.values().is_empty());
        for &f in fluxes.values() {
            // Solar flux near Earth should be ~1361 W/m².
            assert!(f > 1300.0 && f < 1420.0, "unexpected flux: {f}");
        }

        // And the batch path agrees with the per-channel calls.
        let run = analysis.run(scenario_interval, Parallelism::Sequential);
        assert_eq!(run.len(), 1);
        let power = run[0].1.as_ref().expect("run failed");
        assert_eq!(power.eclipses.len(), eclipses.len());
        assert_approx_eq!(
            power.eclipse_fraction_over(scenario_interval),
            eclipse_frac,
            rtol <= 1e-12
        );
    }
}
