// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Shared fixtures for the `lox-space` analysis benchmarks.
//!
//! This module is `#[path]`-included into each bench file (divan bench files
//! are separate crate roots and cannot `use` one another), so any individual
//! benchmark only exercises a subset of these helpers — hence the blanket
//! `dead_code` allow.
//!
//! All expensive inputs (the SPK kernel, the OneWeb TLE catalogue) are loaded
//! and parsed exactly once via `OnceLock` so that benchmark closures never pay
//! for fixture I/O. The only per-call cost is the unavoidable SGP4 propagation
//! of `n` spacecraft, which callers run *outside* the timed region.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::OnceLock;

use lox_space::analysis::assets::{AssetId, GroundStation, Scenario, Spacecraft};
use lox_space::analysis::visibility::ElevationMask;
use lox_space::bodies::{Earth, Origin};
use lox_space::core::coords::LonLatAlt;
use lox_space::core::units::AngularRate;
use lox_space::ephem::spk::parser::Spk;
use lox_space::frames::providers::DefaultRotationProvider;
use lox_space::frames::{Frame, Icrf};
use lox_space::orbits::ground::GroundLocation;
use lox_space::orbits::propagators::sgp4::{Elements, Sgp4};
use lox_space::orbits::propagators::{OrbitSource, Propagator};
use lox_space::orbits::{Ensemble, Trajectory};
use lox_space::time::deltas::TimeDelta;
use lox_space::time::intervals::{Interval, TimeInterval};
use lox_space::time::time_scales::TimeScale;

pub type DynamicEnsemble = Ensemble<AssetId, Origin, Frame>;
pub type MonoEnsemble = Ensemble<AssetId, Earth, Icrf>;
/// A parsed TLE: `(name, line1, line2)`.
pub type TleEntry = (String, Vec<u8>, Vec<u8>);

/// Solar-system ephemeris, loaded once.
pub fn ephemeris() -> &'static Spk {
    static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
    EPHEMERIS.get_or_init(|| Spk::from_file(lox_test_utils::data_file("spice/de440s.bsp")).unwrap())
}

// ---------------------------------------------------------------------------
// Single ground-space pair (lunar trajectory + Cebreros) — dynamic and mono
// ---------------------------------------------------------------------------

fn spacecraft_trajectory_dynamic() -> Trajectory {
    Trajectory::from_csv_dynamic(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        Origin::Earth,
        Frame::Icrf,
    )
    .unwrap()
}

fn ground_location_dynamic() -> GroundLocation<Origin> {
    let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
    GroundLocation::try_new(coords, Origin::Earth).unwrap()
}

/// Ground-space scenario over the lunar trajectory using dynamic dispatch.
pub fn setup_dynamic() -> (Scenario, DynamicEnsemble) {
    let sc_traj = spacecraft_trajectory_dynamic();
    let gs_loc = ground_location_dynamic();
    let mask = ElevationMask::with_fixed_elevation(0.0);
    let gs = GroundStation::new("cebreros", gs_loc, mask);
    let interval = TimeInterval::new(sc_traj.start_time(), sc_traj.end_time());
    let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(sc_traj));
    let scenario = Scenario::new(interval.start(), interval.end(), Origin::Earth, Frame::Icrf)
        .with_spacecraft(&[sc])
        .with_ground_stations(&[gs]);
    let ensemble = scenario.propagate(&DefaultRotationProvider).unwrap();
    (scenario, ensemble)
}

fn spacecraft_trajectory_mono() -> Trajectory<Earth, Icrf> {
    Trajectory::from_csv(
        &lox_test_utils::read_data_file("trajectory_lunar.csv"),
        Earth,
        Icrf,
    )
    .unwrap()
}

fn ground_location_mono() -> GroundLocation<Earth> {
    let coords = LonLatAlt::from_degrees(-4.3676, 40.4527, 0.0).unwrap();
    GroundLocation::try_new(coords, Earth).unwrap()
}

/// Ground-space scenario over the lunar trajectory using concrete types
/// (no `Frame` dispatch).
pub fn setup_mono() -> (Scenario<Earth, Icrf>, MonoEnsemble) {
    let traj = spacecraft_trajectory_mono();
    let gs_loc = ground_location_mono();
    let mask = ElevationMask::with_fixed_elevation(0.0);
    let interval = TimeInterval::new(traj.start_time(), traj.end_time());
    let sc = Spacecraft::new("lunar", OrbitSource::Trajectory(traj.into_dynamic()));
    let gs = GroundStation::new("cebreros", gs_loc.into_dynamic(), mask);
    let scenario = Scenario::new(interval.start(), interval.end(), Earth, Icrf)
        .with_spacecraft(&[sc])
        .with_ground_stations(&[gs]);
    let ensemble = scenario.propagate(&DefaultRotationProvider).unwrap();
    (scenario, ensemble)
}

// ---------------------------------------------------------------------------
// OneWeb constellation fixtures (TLE → SGP4) for many-asset scaling
// ---------------------------------------------------------------------------

/// The full OneWeb TLE catalogue, parsed once into `(name, line1, line2)`.
pub fn oneweb_tles() -> &'static Vec<TleEntry> {
    static TLES: OnceLock<Vec<TleEntry>> = OnceLock::new();
    TLES.get_or_init(|| {
        let raw = lox_test_utils::read_data_file("oneweb_tle.txt");
        let lines: Vec<&str> = raw.lines().collect();
        lines
            .chunks(3)
            .filter(|c| c.len() == 3)
            .map(|c| {
                (
                    c[0].trim().to_string(),
                    c[1].as_bytes().to_vec(),
                    c[2].as_bytes().to_vec(),
                )
            })
            .collect()
    })
}

/// SGP4-propagate the first `n` OneWeb spacecraft over a `window_hours` window.
///
/// Returns the propagated `Trajectory` for each, sharing a common interval
/// (the latest TLE epoch as start, so every element is valid).
pub fn propagate_oneweb_trajectories(n: usize, window_hours: i64) -> Vec<(String, Trajectory)> {
    let tles = oneweb_tles();
    let n = n.min(tles.len());
    let propagators: Vec<(String, Sgp4)> = tles[..n]
        .iter()
        .map(|(name, l1, l2)| {
            let el = Elements::from_tle(Some(name.clone()), l1, l2).unwrap();
            (name.clone(), Sgp4::new(el).unwrap())
        })
        .collect();
    // Latest epoch keeps every element within its valid propagation window.
    let t0 = propagators
        .iter()
        .map(|(_, p)| p.time())
        .max()
        .expect("at least one TLE");
    let t1 = t0 + TimeDelta::from_hours(window_hours);
    let interval = Interval::new(t0, t1);
    propagators
        .into_iter()
        .map(|(name, p)| {
            let traj = p
                .with_step(TimeDelta::from_seconds(10))
                .propagate(interval.into_dynamic())
                .unwrap()
                .into_dynamic();
            (name, traj)
        })
        .collect()
}

/// Assemble a `Scenario` + ensemble from named spacecraft (each carrying a
/// pre-propagated trajectory). Ground stations, if any, are added verbatim.
pub fn assemble_scenario(
    spacecraft: Vec<Spacecraft>,
    trajectories: Vec<Trajectory>,
    ground_stations: &[GroundStation],
) -> (Scenario, DynamicEnsemble) {
    let interval = TimeInterval::new(trajectories[0].start_time(), trajectories[0].end_time());
    let tai_interval = interval;
    let scenario = Scenario::with_interval(tai_interval, Origin::Earth, Frame::Icrf)
        .with_spacecraft(&spacecraft)
        .with_ground_stations(ground_stations);
    let mut map = HashMap::new();
    for (sc, traj) in spacecraft.iter().zip(trajectories) {
        let (epoch, origin, frame, data) = traj.into_parts();
        let typed = Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data);
        map.insert(sc.id().clone(), typed);
    }
    (scenario, Ensemble::new(map))
}

/// `n`-spacecraft OneWeb constellation, no ground stations. Used for both
/// inter-satellite visibility scaling and power-budget scaling.
pub fn propagate_oneweb(n: usize, window_hours: i64) -> (Scenario, DynamicEnsemble) {
    let trajs = propagate_oneweb_trajectories(n, window_hours);
    let (names, trajectories): (Vec<String>, Vec<Trajectory>) = trajs.into_iter().unzip();
    let spacecraft: Vec<Spacecraft> = names
        .into_iter()
        .zip(trajectories.iter())
        .map(|(name, traj)| Spacecraft::new(name, OrbitSource::Trajectory(traj.clone())))
        .collect();
    assemble_scenario(spacecraft, trajectories, &[])
}

/// Alias for `propagate_oneweb` with a 24 h window — the natural span for a
/// power-budget (eclipse) analysis.
pub fn build_power_scenario(n: usize) -> (Scenario, DynamicEnsemble) {
    propagate_oneweb(n, 24)
}

/// A spread of ground stations for ground-space scaling benchmarks.
fn scaling_ground_stations() -> Vec<GroundStation> {
    const SITES: [(&str, f64, f64); 5] = [
        ("cebreros", -4.3676, 40.4527),
        ("goldstone", -116.8900, 35.4267),
        ("canberra", 148.9819, -35.4014),
        ("kourou", -52.8047, 5.2517),
        ("svalbard", 15.4072, 78.2298),
    ];
    SITES
        .iter()
        .map(|&(id, lon, lat)| {
            let coords = LonLatAlt::from_degrees(lon, lat, 0.0).unwrap();
            let loc = GroundLocation::try_new(coords, Origin::Earth).unwrap();
            GroundStation::new(id, loc, ElevationMask::with_fixed_elevation(0.0))
        })
        .collect()
}

/// `n`-spacecraft OneWeb constellation plus a 5-station ground network
/// (`5 * n` ground-space pairs) for ground-space scaling.
pub fn build_groundspace_scenario(n: usize) -> (Scenario, DynamicEnsemble) {
    let trajs = propagate_oneweb_trajectories(n, 2);
    let (names, trajectories): (Vec<String>, Vec<Trajectory>) = trajs.into_iter().unzip();
    let spacecraft: Vec<Spacecraft> = names
        .into_iter()
        .zip(trajectories.iter())
        .map(|(name, traj)| Spacecraft::new(name, OrbitSource::Trajectory(traj.clone())))
        .collect();
    assemble_scenario(spacecraft, trajectories, &scaling_ground_stations())
}

// ---------------------------------------------------------------------------
// Inter-satellite pair (two crossing-orbit OneWeb spacecraft)
// ---------------------------------------------------------------------------

fn oneweb_pair_trajectories() -> (Trajectory, Trajectory) {
    let tle1 = Elements::from_tle(
        Some("ONEWEB-0012".to_string()),
        b"1 44057U 19010A   24322.58825131  .00000088  00000+0  19693-3 0  9993",
        b"2 44057  87.9092 343.6767 0002420  76.7970 283.3431 13.16592150275693",
    )
    .unwrap();
    let tle2 = Elements::from_tle(
        Some("ONEWEB-0017".to_string()),
        b"1 45132U 20008B   24322.88240834 -.00000016  00000+0 -81930-4 0  9998",
        b"2 45132  87.8896 151.0343 0001369  78.1189 282.0092 13.10376984232476",
    )
    .unwrap();
    let sgp4_1 = Sgp4::new(tle1).unwrap();
    let sgp4_2 = Sgp4::new(tle2).unwrap();
    let t0 = sgp4_1.time().max(sgp4_2.time());
    let t1 = t0 + TimeDelta::from_hours(2);
    let interval = Interval::new(t0, t1);
    let traj1 = sgp4_1
        .with_step(TimeDelta::from_seconds(10))
        .propagate(interval.into_dynamic())
        .unwrap()
        .into_dynamic();
    let traj2 = sgp4_2
        .with_step(TimeDelta::from_seconds(10))
        .propagate(interval.into_dynamic())
        .unwrap()
        .into_dynamic();
    (traj1, traj2)
}

/// Two-spacecraft inter-satellite scenario. When `slew_rate` is `Some`, both
/// spacecraft carry that maximum slew-rate limit.
pub fn setup_intersat_pair(slew_rate: Option<AngularRate>) -> (Scenario, DynamicEnsemble) {
    let (traj1, traj2) = oneweb_pair_trajectories();
    let mut sc1 = Spacecraft::new("oneweb-0012", OrbitSource::Trajectory(traj1.clone()));
    let mut sc2 = Spacecraft::new("oneweb-0017", OrbitSource::Trajectory(traj2.clone()));
    if let Some(rate) = slew_rate {
        sc1 = sc1.with_max_slew_rate(rate);
        sc2 = sc2.with_max_slew_rate(rate);
    }
    assemble_scenario(vec![sc1, sc2], vec![traj1, traj2], &[])
}
