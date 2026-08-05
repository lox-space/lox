// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Streams real ground-station passes for a multi-station, multi-spacecraft
//! scenario — the toy `stream_toy` example against actual orbits.
//!
//! ```text
//! cargo run -p lox-analysis --features async --example stream_passes [hours]
//! ```
//!
//! `hours` (default 24) sets the scenario length. A short window finishes in
//! well under a second, which is too fast to interrupt — pass something long if
//! you want to watch cancellation work.
//!
//! What to look for:
//!
//! - **Passes arrive per item and interleaved.** A pair emits each pass as its
//!   scan finds it, and pairs are not processed one after another, so the station
//!   column jumps around. Nothing about arrival order identifies a pass — the key
//!   does.
//! - **Every pair ends with one `<completed>`**, including pairs a spacecraft
//!   never rises above, which report zero passes rather than going missing.
//! - **Ctrl-C stops the scans**, not just the delivery: pairs still in flight
//!   report no `<completed>` and the process exits promptly.

use std::collections::BTreeMap;
use std::sync::Arc;

use futures_util::StreamExt;
use lox_analysis::assets::{AssetId, GroundStation, Scenario, Spacecraft};
use lox_analysis::pipeline::analyses::VisibilityStreamAnalysis;
use lox_analysis::stream::StreamEvent;
use lox_analysis::visibility::ElevationMask;
use lox_bodies::Origin;
use lox_core::coords::LonLatAlt;
use lox_core::sync::CancellationToken;
use lox_core::units::Angle;
use lox_ephem::spk::parser::Spk;
use lox_frames::Frame;
use lox_orbits::ground::GroundLocation;
use lox_orbits::orbits::{Ensemble, Trajectory};
use lox_orbits::propagators::sgp4::{Elements, Sgp4};
use lox_orbits::propagators::{OrbitSource, Propagator};
use lox_time::deltas::TimeDelta;
use lox_time::intervals::{Interval, TimeInterval};
use lox_time::time_scales::TimeScale;

/// ESTRACK-like stations, spread widely enough that a LEO satellite is visible
/// from a different one every few minutes.
const STATIONS: [(&str, f64, f64); 4] = [
    ("cebreros", -4.3676, 40.4527),
    ("kiruna", 20.9647, 67.8574),
    ("santiago", -70.6667, -33.1500),
    ("dongara", 115.3489, -29.0455),
];

const TLES: [(&str, &[u8], &[u8]); 3] = [
    (
        "ISS",
        b"1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996",
        b"2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731",
    ),
    (
        "ONEWEB-0012",
        b"1 44057U 19010A   24322.58825131  .00000088  00000+0  19693-3 0  9993",
        b"2 44057  87.9092 343.6767 0002420  76.7970 283.3431 13.16592150275693",
    ),
    (
        "ONEWEB-0017",
        b"1 45132U 20008B   24322.88240834 -.00000016  00000+0 -81930-4 0  9998",
        b"2 45132  87.8896 151.0343 0001369  78.1189 282.0092 13.10376984232476",
    ),
];

fn stations() -> Vec<GroundStation> {
    STATIONS
        .iter()
        .map(|&(name, lon, lat)| {
            let coords = LonLatAlt::from_degrees(lon, lat, 0.0).unwrap();
            let location = GroundLocation::try_new(coords, Origin::Earth).unwrap();
            GroundStation::new(
                name,
                location,
                // 5 degrees, a realistic operational horizon.
                ElevationMask::with_fixed_elevation(Angle::degrees(5.0).to_radians()),
            )
        })
        .collect()
}

fn main() {
    // The TLE epochs differ by months, so each spacecraft is propagated from its
    // own epoch and the scenario spans the window they overlap in — here just the
    // last one's, since a common window would be empty.
    let sgp4: Vec<(&str, Sgp4)> = TLES
        .iter()
        .map(|&(name, l1, l2)| {
            let tle = Elements::from_tle(Some(name.to_string()), l1, l2).unwrap();
            (name, Sgp4::new(tle).unwrap())
        })
        .collect();

    let hours: i64 = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(24);

    let start = sgp4
        .iter()
        .map(|(_, s)| s.time())
        .max()
        .expect("no spacecraft");
    let interval = Interval::new(start, start + TimeDelta::from_hours(hours));

    let trajectories: Vec<(&str, Trajectory)> = sgp4
        .into_iter()
        .map(|(name, s)| {
            let traj = s
                .with_step(TimeDelta::from_seconds(30))
                .propagate(interval.into_dynamic())
                .expect("propagation failed")
                .into_dynamic();
            (name, traj)
        })
        .collect();

    let spacecraft: Vec<Spacecraft> = trajectories
        .iter()
        .map(|(name, traj)| Spacecraft::new(*name, OrbitSource::Trajectory(traj.clone())))
        .collect();
    let stations = stations();

    let scenario_interval = interval.into_dynamic();
    let scenario = Scenario::with_interval(scenario_interval, Origin::Earth, Frame::Icrf)
        .with_ground_stations(&stations)
        .with_spacecraft(&spacecraft);

    let ensemble = Ensemble::new(
        trajectories
            .iter()
            .map(|(name, traj)| {
                let (epoch, origin, frame, data) = traj.clone().into_parts();
                (
                    AssetId::new(*name),
                    Trajectory::from_parts(epoch.with_scale(TimeScale::Tai), origin, frame, data),
                )
            })
            .collect(),
    );

    let ephemeris =
        Arc::new(Spk::from_file(lox_test_utils::data_file("spice/de440s.bsp")).expect("SPK"));

    run(scenario, ensemble, ephemeris, scenario_interval);
}

#[tokio::main]
async fn run(
    scenario: Scenario<Origin, Frame>,
    ensemble: Ensemble<AssetId, Origin, Frame>,
    ephemeris: Arc<Spk>,
    interval: TimeInterval,
) {
    let cancel = CancellationToken::new();
    let on_ctrl_c = cancel.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            eprintln!("\n-- Ctrl-C: cancelling --");
            on_ctrl_c.cancel();
        }
    });

    let analysis = VisibilityStreamAnalysis::new(&scenario, &ensemble, ephemeris)
        .with_step(TimeDelta::from_seconds(30))
        .with_min_pass_duration(TimeDelta::from_seconds(120));

    let pairs = scenario.ground_stations().len() * scenario.spacecraft().len();
    println!(
        "streaming {pairs} pairs over {:.1} h (Ctrl-C to cancel)\n",
        interval.duration().to_seconds().to_f64() / 3600.0
    );
    println!(
        "{:>10}  {:>12}  {:>26}  {:>8}  {:>7}",
        "station", "spacecraft", "pass start (TAI)", "dur [s]", "max el"
    );

    let mut events = std::pin::pin!(analysis.stream(interval, Some(cancel)));
    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut completed: BTreeMap<(String, String), usize> = BTreeMap::new();

    while let Some(((gs, sc), event)) = events.next().await {
        let key = (gs.as_str().to_string(), sc.as_str().to_string());
        match event {
            StreamEvent::Item(Ok(pass)) => {
                *counts.entry(key.clone()).or_default() += 1;
                let max_el = pass
                    .observables()
                    .iter()
                    .map(|o| o.elevation())
                    .fold(f64::NEG_INFINITY, f64::max);
                println!(
                    "{:>10}  {:>12}  {:>26}  {:>8.0}  {:>6.1}°",
                    key.0,
                    key.1,
                    pass.interval().start().to_string(),
                    pass.interval().duration().to_seconds().to_f64(),
                    max_el.to_degrees(),
                );
            }
            StreamEvent::Item(Err(e)) => println!("{:>10}  {:>12}  ERROR: {e}", key.0, key.1),
            StreamEvent::Completed => {
                *completed.entry(key.clone()).or_default() += 1;
                println!("{:>10}  {:>12}  <completed>", key.0, key.1);
            }
        }
    }

    println!("\n{:->72}", "");
    let mut multiples = vec![];
    for ((gs, sc), c) in &completed {
        if *c != 1 {
            multiples.push((gs.clone(), sc.clone(), *c));
        }
    }
    let total: usize = counts.values().sum();
    println!(
        "{total} passes across {} pairs; {} of {pairs} pairs completed",
        counts.len(),
        completed.len()
    );
    for (gs, sc) in completed.keys() {
        let n = counts.get(&(gs.clone(), sc.clone())).copied().unwrap_or(0);
        if n == 0 {
            println!("  {gs} <-> {sc}: no passes (completed, not missing)");
        }
    }
    assert!(
        multiples.is_empty(),
        "the one-Completed-per-target invariant broke for {multiples:?}"
    );
}
