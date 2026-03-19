// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Power budget analysis: sun beta angle, eclipse detection, solar flux.
//!
//! The shadow model uses cylindrical (umbra-only) geometry via the existing
//! [`line_of_sight`](crate::visibility::line_of_sight) function.  Penumbra is
//! **not** modelled.

use std::collections::HashMap;

use lox_bodies::{DynOrigin, Origin, Sun, TryMeanRadius, TrySpheroid};
use lox_core::glam::DVec3;
use lox_core::math::series::InterpolationType;
use lox_core::units::ASTRONOMICAL_UNIT;
use lox_ephem::Ephemeris;
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::{self, TimeInterval};
use lox_time::series::TimeSeries;
use lox_time::time_scales::{Tai, Tdb};
use rayon::prelude::*;
use thiserror::Error;

use crate::assets::{AssetId, ConstellationId, Scenario, Spacecraft};
use crate::visibility::{EvalError, LineOfSight};
use lox_frames::ReferenceFrame;
use lox_orbits::events::{DetectFn, EventsToIntervals, IntervalDetector, RootFindingDetector};
use lox_orbits::orbits::{Ensemble, Trajectory};

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

/// Per-spacecraft power-budget output tuple.
type SpacecraftPowerData = (
    AssetId,
    Vec<TimeInterval<Tai>>,
    TimeSeries<Tai>,
    TimeSeries<Tai>,
);

/// Errors from power-budget analysis.
#[derive(Debug, Error)]
pub enum PowerError {
    /// Event detection failed.
    #[error(transparent)]
    Detect(#[from] lox_orbits::events::DetectError),
    /// Evaluation error (frame rotation, ephemeris, …).
    #[error(transparent)]
    Eval(#[from] EvalError),
}

/// Eclipse detect function: positive when the spacecraft is sunlit, negative
/// when it is in eclipse (cylindrical shadow model, umbra only).
struct EclipseDetectFn<'a, O: Origin, R: ReferenceFrame, E> {
    sc: &'a Trajectory<Tai, O, R>,
    ephemeris: &'a E,
}

impl<O, R, E: Ephemeris> DetectFn<Tai> for EclipseDetectFn<'_, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy,
    R: ReferenceFrame + Copy,
    E::Error: 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time<Tai>) -> Result<f64, Self::Error> {
        let tdb = time.to_scale(Tdb);
        let r_sun = self
            .ephemeris
            .position(tdb, self.sc.origin(), Sun)
            .map_err(|e| EvalError::Ephemeris(Box::new(e)))?;
        let r_sc = self.sc.interpolate_at(time).position();
        // line_of_sight returns positive when the two vectors have mutual LOS
        // (spacecraft is sunlit) and negative when occluded (eclipse).
        Ok(self.sc.origin().line_of_sight(r_sc, r_sun)?)
    }
}

// ---------------------------------------------------------------------------
// PowerBudgetResults
// ---------------------------------------------------------------------------

/// Results of a power-budget analysis.
///
/// Contains eclipse intervals, beta-angle time series, and solar-flux time
/// series for each spacecraft.
pub struct PowerBudgetResults {
    eclipse_intervals: HashMap<AssetId, Vec<TimeInterval<Tai>>>,
    beta_angles: HashMap<AssetId, TimeSeries<Tai>>,
    solar_fluxes: HashMap<AssetId, TimeSeries<Tai>>,
    scenario_duration: f64,
}

impl PowerBudgetResults {
    /// Eclipse intervals for a given spacecraft.
    pub fn eclipse_intervals_for(&self, id: &AssetId) -> Option<&[TimeInterval<Tai>]> {
        self.eclipse_intervals.get(id).map(|v| v.as_slice())
    }

    /// All eclipse intervals keyed by spacecraft id.
    pub fn all_eclipse_intervals(&self) -> &HashMap<AssetId, Vec<TimeInterval<Tai>>> {
        &self.eclipse_intervals
    }

    /// Eclipse fraction for a given spacecraft (ratio of total eclipse time to
    /// scenario duration, in \[0, 1\]).
    pub fn eclipse_fraction(&self, id: &AssetId) -> Option<f64> {
        let intervals = self.eclipse_intervals.get(id)?;
        let total_eclipse: f64 = intervals
            .iter()
            .map(|i| (i.end() - i.start()).to_seconds().to_f64())
            .sum();
        Some(total_eclipse / self.scenario_duration)
    }

    /// Sunlit fraction for a given spacecraft (`1 − eclipse_fraction`).
    pub fn sunlit_fraction(&self, id: &AssetId) -> Option<f64> {
        self.eclipse_fraction(id).map(|f| 1.0 - f)
    }

    /// Beta-angle time series for a given spacecraft (radians).
    pub fn beta_angles_for(&self, id: &AssetId) -> Option<&TimeSeries<Tai>> {
        self.beta_angles.get(id)
    }

    /// Solar-flux time series for a given spacecraft (W/m²).
    pub fn solar_flux_for(&self, id: &AssetId) -> Option<&TimeSeries<Tai>> {
        self.solar_fluxes.get(id)
    }
}

// ---------------------------------------------------------------------------
// PowerBudgetAnalysis
// ---------------------------------------------------------------------------

/// Filter for restricting which spacecraft are analysed.
#[derive(Clone)]
pub enum SpacecraftFilter {
    /// Analyse only spacecraft whose id is in the given list.
    Ids(Vec<AssetId>),
    /// Analyse only spacecraft belonging to the given constellation.
    Constellation(ConstellationId),
}

/// Computes eclipse intervals, beta angles, and solar flux for spacecraft
/// in a scenario.
///
/// Generic over origin `O`, reference frame `R`, and ephemeris `E`.
/// The shadow model is cylindrical (umbra only) — penumbra is not modelled.
pub struct PowerBudgetAnalysis<'a, O: Origin, R: ReferenceFrame, E> {
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, Tai, O, R>,
    ephemeris: &'a E,
    step: TimeDelta,
    filter: Option<SpacecraftFilter>,
}

impl<'a, O, R, E> PowerBudgetAnalysis<'a, O, R, E>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Send + Sync,
    E: Ephemeris + Send + Sync,
    E::Error: 'static,
{
    /// Creates a new power-budget analysis.
    pub fn new(
        scenario: &'a Scenario<O, R>,
        ensemble: &'a Ensemble<AssetId, Tai, O, R>,
        ephemeris: &'a E,
    ) -> Self {
        Self {
            scenario,
            ensemble,
            ephemeris,
            step: TimeDelta::from_seconds(60),
            filter: None,
        }
    }

    /// Sets the time step for sampling and event detection.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Restricts the analysis to a subset of spacecraft.
    ///
    /// See [`SpacecraftFilter`] for the available filter modes.
    pub fn with_filter(mut self, filter: SpacecraftFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Compute the power-budget analysis for all (or filtered) spacecraft in
    /// the scenario.
    pub fn compute(&self) -> Result<PowerBudgetResults, PowerError> {
        let interval = *self.scenario.interval();
        let all_spacecraft = self.scenario.spacecraft();
        let spacecraft: Vec<&Spacecraft> = match &self.filter {
            Some(SpacecraftFilter::Ids(ids)) => all_spacecraft
                .iter()
                .filter(|sc| ids.contains(sc.id()))
                .collect(),
            Some(SpacecraftFilter::Constellation(cid)) => all_spacecraft
                .iter()
                .filter(|sc| sc.constellation_id() == Some(cid))
                .collect(),
            None => all_spacecraft.iter().collect(),
        };
        let duration_s = (interval.end() - interval.start()).to_seconds().to_f64();

        let results: Result<Vec<_>, PowerError> = spacecraft
            .par_iter()
            .map(|sc| self.compute_spacecraft(sc, interval))
            .collect();

        let mut eclipse_intervals = HashMap::new();
        let mut beta_angles = HashMap::new();
        let mut solar_fluxes = HashMap::new();

        for (id, eclipses, betas, fluxes) in results? {
            eclipse_intervals.insert(id.clone(), eclipses);
            beta_angles.insert(id.clone(), betas);
            solar_fluxes.insert(id, fluxes);
        }

        Ok(PowerBudgetResults {
            eclipse_intervals,
            beta_angles,
            solar_fluxes,
            scenario_duration: duration_s,
        })
    }

    /// Compute all quantities for a single spacecraft.
    fn compute_spacecraft(
        &self,
        sc: &Spacecraft,
        interval: TimeInterval<Tai>,
    ) -> Result<SpacecraftPowerData, PowerError> {
        let sc_traj = self.ensemble.get(sc.id()).expect(
            "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
        );

        // 1. Eclipse intervals via root-finding
        let eclipse_fn = EclipseDetectFn {
            sc: sc_traj,
            ephemeris: self.ephemeris,
        };
        let detector = RootFindingDetector::new(eclipse_fn, self.step);
        // EventsToIntervals gives intervals where the function is positive
        // (sunlit). We need the complement → eclipse intervals.
        let sunlit_intervals = EventsToIntervals::new(detector).detect(interval)?;

        // Complement sunlit intervals to get eclipse intervals
        let eclipse_intervals = intervals::complement_intervals(&sunlit_intervals, interval);

        // 2. Beta angle + solar flux sampled at `step`
        let epoch = interval.start();
        let mut offsets = Vec::new();
        let mut beta_values = Vec::new();
        let mut flux_values = Vec::new();

        for time in interval.step_by(self.step) {
            let tdb = time.to_scale(Tdb);
            let state = sc_traj.interpolate_at(time);
            let r = state.position();
            let v = state.velocity();
            let h = r.cross(v);
            let h_hat = h.normalize();

            let r_sun = self
                .ephemeris
                .position(tdb, sc_traj.origin(), Sun)
                .map_err(|e| PowerError::Eval(EvalError::Ephemeris(Box::new(e))))?;
            let sun_hat = r_sun.normalize();

            offsets.push((time - epoch).to_seconds().to_f64());
            beta_values.push(beta_angle(h_hat, sun_hat));
            flux_values.push(solar_flux(r_sun.length()));
        }

        let beta_series = TimeSeries::try_new(
            epoch,
            offsets.clone(),
            beta_values,
            InterpolationType::Linear,
        )
        .expect("sampled series should have valid dimensions");
        let flux_series =
            TimeSeries::try_new(epoch, offsets, flux_values, InterpolationType::Linear)
                .expect("sampled series should have valid dimensions");

        Ok((sc.id().clone(), eclipse_intervals, beta_series, flux_series))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};
    use std::sync::OnceLock;

    use lox_bodies::DynOrigin;
    use lox_core::glam::DVec3;
    use lox_ephem::spk::parser::Spk;
    use lox_frames::DynFrame;
    use lox_orbits::propagators::sgp4::{Elements, Sgp4};
    use lox_orbits::propagators::{OrbitSource, Propagator};
    use lox_test_utils::{assert_approx_eq, data_file};
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
            .propagate(Interval::new(t0, t1))
            .unwrap()
            .into_dyn();

        let tai_interval = TimeInterval::new(
            sc_traj.start_time().to_scale(Tai),
            sc_traj.end_time().to_scale(Tai),
        );

        let sc = Spacecraft::new("ISS", OrbitSource::Trajectory(sc_traj.clone()));
        let scenario = Scenario::with_interval(tai_interval, DynOrigin::Earth, DynFrame::Icrf)
            .with_spacecraft(std::slice::from_ref(&sc));

        // Build ensemble
        let (epoch, origin, frame, data) = sc_traj.into_parts();
        let typed = Trajectory::from_parts(epoch.with_scale(Tai), origin, frame, data);
        let mut map = HashMap::new();
        map.insert(sc.id().clone(), typed);
        let ensemble = Ensemble::new(map);

        let spk = ephemeris();
        let analysis = PowerBudgetAnalysis::new(&scenario, &ensemble, spk);
        let results = analysis.compute().expect("power budget analysis");

        // ISS completes ~15.5 orbits/day → expect roughly that many eclipses.
        let eclipses = results
            .eclipse_intervals_for(sc.id())
            .expect("eclipse intervals");
        assert!(
            eclipses.len() >= 10,
            "expected ≥10 eclipse intervals for ISS over 24h, got {}",
            eclipses.len()
        );

        // Eclipse fraction for ISS is typically ~35%.
        let eclipse_frac = results.eclipse_fraction(sc.id()).unwrap();
        assert!(
            (0.2..0.5).contains(&eclipse_frac),
            "unexpected eclipse fraction: {eclipse_frac}"
        );

        let sunlit_frac = results.sunlit_fraction(sc.id()).unwrap();
        assert_approx_eq!(eclipse_frac + sunlit_frac, 1.0, atol <= 1e-15);

        let betas = results.beta_angles_for(sc.id()).expect("beta angles");
        assert!(!betas.values().is_empty());
        for &b in betas.values() {
            assert!((-FRAC_PI_2..=FRAC_PI_2).contains(&b));
        }

        let fluxes = results.solar_flux_for(sc.id()).expect("solar flux");
        assert!(!fluxes.values().is_empty());
        for &f in fluxes.values() {
            // Solar flux near Earth should be ~1361 W/m².
            assert!(f > 1300.0 && f < 1420.0, "unexpected flux: {f}");
        }
    }
}
