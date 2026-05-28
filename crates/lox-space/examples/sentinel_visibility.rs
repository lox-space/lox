// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Compute and print visibility between two Sentinel satellites and two
//! ground stations using SGP4 propagation.
//!
//! Run from the workspace root:
//!
//! ```text
//! cargo run --release --example sentinel_visibility -p lox-space
//! ```

use std::error::Error;

use lox_space::analysis::assets::{GroundStation, Scenario, Spacecraft};
use lox_space::analysis::visibility::{ElevationMask, VisibilityAnalysis, VisibilityResults};
use lox_space::bodies::DynOrigin;
use lox_space::core::coords::LonLatAlt;
use lox_space::frames::DynFrame;
use lox_space::frames::providers::DefaultRotationProvider;
use lox_space::orbits::ground::GroundLocation;
use lox_space::orbits::propagators::OrbitSource;
use lox_space::orbits::propagators::sgp4::{Elements, Sgp4};
use lox_space::time::deltas::TimeDelta;
use lox_space::time::intervals::TimeInterval;
use lox_space::time::time_scales::Tai;

const SENTINEL_1A_NAME: &str = "SENTINEL-1A";
const SENTINEL_1A_LINE_1: &[u8] =
    b"1 39634U 14016A   26148.27607743  .00000381  00000+0  90449-4 0  9995";
const SENTINEL_1A_LINE_2: &[u8] =
    b"2 39634  98.1782 156.0996 0001411  84.8085 275.3276 14.59200187647116";

const SENTINEL_2A_NAME: &str = "SENTINEL-2A";
const SENTINEL_2A_LINE_1: &[u8] =
    b"1 40697U 15028A   26148.28462749 -.00000050  00000+0 -22796-5 0  9992";
const SENTINEL_2A_LINE_2: &[u8] =
    b"2 40697  98.5688 223.4180 0001080  87.9265 272.2042 14.30816700570885";

// Propagation window: 24 hours, starting at the later of the two TLE epochs.
const WINDOW_HOURS: i64 = 24;
const PROPAGATION_STEP_SECS: i64 = 10;

// Minimum elevation for a contact, in degrees.
const ELEVATION_MASK_DEG: f64 = 5.0;

fn tle_to_sgp4(name: &str, line_1: &[u8], line_2: &[u8]) -> Result<Sgp4, Box<dyn Error>> {
    let elements = Elements::from_tle(Some(name.to_string()), line_1, line_2)?;
    Ok(Sgp4::new(elements)?)
}

fn format_duration(d: TimeDelta) -> String {
    let total = d.to_seconds().to_f64() as i64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    format!("{hours}h {minutes:02}m {seconds:02}s")
}

fn print_summary(results: &VisibilityResults, interval: TimeInterval<Tai>) {
    let title = format!(
        "Sentinel visibility — {} → {} ({}h window)",
        interval.start(),
        interval.end(),
        WINDOW_HOURS,
    );
    println!("{title}");
    println!("{}", "=".repeat(title.len()));

    for ((gs_id, sc_id), intervals) in results.all_intervals() {
        let total: TimeDelta = intervals
            .iter()
            .fold(TimeDelta::default(), |acc, iv| acc + iv.duration());
        println!(
            "  {} ↔ {}:  {} passes,  {}",
            gs_id,
            sc_id,
            intervals.len(),
            format_duration(total),
        );
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Parse both TLEs. We need to know each TLE's epoch before we
    //    can pick a common propagation window, so the propagators are
    //    built first.
    let s1a_prop = tle_to_sgp4(SENTINEL_1A_NAME, SENTINEL_1A_LINE_1, SENTINEL_1A_LINE_2)?;
    let s2a_prop = tle_to_sgp4(SENTINEL_2A_NAME, SENTINEL_2A_LINE_1, SENTINEL_2A_LINE_2)?;

    // Window: start at the later of the two TLE epochs (so both TLEs
    // are valid throughout) and run for WINDOW_HOURS.
    let t0 = s1a_prop.time().max(s2a_prop.time());
    let t1 = t0 + TimeDelta::from_hours(WINDOW_HOURS);
    let interval = TimeInterval::new(t0, t1);

    // 2. Build ground stations.
    let mask = ElevationMask::with_fixed_elevation(ELEVATION_MASK_DEG.to_radians());
    let stations = vec![
        GroundStation::new(
            "svalbard",
            GroundLocation::try_new(
                LonLatAlt::from_degrees(15.4078, 78.2297, 450.0)?,
                DynOrigin::Earth,
            )?,
            mask.clone(),
        ),
        GroundStation::new(
            "maspalomas",
            GroundLocation::try_new(
                LonLatAlt::from_degrees(-15.6336, 27.7629, 205.0)?,
                DynOrigin::Earth,
            )?,
            mask,
        ),
    ];

    // 3. Build the spacecraft assets with SGP4 orbit sources.
    let step = TimeDelta::from_seconds(PROPAGATION_STEP_SECS);
    let spacecraft = vec![
        Spacecraft::new("s1a", OrbitSource::Sgp4(s1a_prop.with_step(step))),
        Spacecraft::new("s2a", OrbitSource::Sgp4(s2a_prop.with_step(step))),
    ];

    // 4. Assemble the scenario.
    let scenario = Scenario::with_interval(interval, DynOrigin::Earth, DynFrame::Icrf)
        .with_ground_stations(&stations)
        .with_spacecraft(&spacecraft);

    // 5. Propagate trajectories into ICRF using the default rotation provider.
    let ensemble = scenario.propagate(&DefaultRotationProvider)?;

    // 6. Run the analysis.
    let results = VisibilityAnalysis::new(&scenario, &ensemble).compute()?;

    print_summary(&results, interval);
    Ok(())
}
