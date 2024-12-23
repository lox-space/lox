/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::Earth;
use lox_orbits::analysis::elevation;
use lox_orbits::frames::{Icrf, NoOpFrameTransformationProvider, Topocentric};
use lox_orbits::ground::GroundLocation;
use lox_orbits::trajectories::Trajectory;
use lox_time::prelude::Tai;
use lox_time::{time, Time};

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn elevation_rotation(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            (
                time!(Tai, 2022, 2, 1).unwrap(),
                GroundLocation::new(-4f64.to_radians(), 41f64.to_radians(), 0.0, Earth),
                spacecraft_trajectory(),
            )
        })
        .bench_values(|(t, gs, sc)| elevation(t, &gs, &sc, &NoOpFrameTransformationProvider));
}

fn ground_station_trajectory() -> Trajectory<Time<Tai>, Earth, Icrf> {
    Trajectory::from_csv(
        include_str!("../../../data/trajectory_cebr.csv"),
        Earth,
        Icrf,
    )
    .unwrap()
}

fn spacecraft_trajectory() -> Trajectory<Time<Tai>, Earth, Icrf> {
    Trajectory::from_csv(
        include_str!("../../../data/trajectory_lunar.csv"),
        Earth,
        Icrf,
    )
    .unwrap()
}
