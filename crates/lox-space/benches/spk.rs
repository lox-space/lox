// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::OnceLock;

use divan::Bencher;
use lox_space::bodies::{Earth, MercuryBarycenter, Moon, SolarSystemBarycenter};
use lox_space::ephem::Ephemeris;
use lox_space::ephem::spk::parser::Spk;
use lox_space::time::{Time, deltas::TimeDelta, time_scales::Tdb};

fn main() {
    divan::main();
}

fn ephemeris() -> &'static Spk {
    static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
    EPHEMERIS.get_or_init(|| Spk::from_file(lox_test_utils::data_file("spice/de440s.bsp")).unwrap())
}

fn test_epoch() -> Time<Tdb> {
    Time::j2000(Tdb) + TimeDelta::from_seconds_f64(-14200747200.0)
}

#[divan::bench]
fn state_ssb_mercury(bencher: Bencher) {
    let spk = ephemeris();
    let epoch = test_epoch();
    bencher.bench(|| spk.state(epoch, SolarSystemBarycenter, MercuryBarycenter));
}

#[divan::bench]
fn position_ssb_mercury(bencher: Bencher) {
    let spk = ephemeris();
    let epoch = test_epoch();
    bencher.bench(|| spk.position(epoch, SolarSystemBarycenter, MercuryBarycenter));
}

#[divan::bench]
fn state_earth_moon(bencher: Bencher) {
    let spk = ephemeris();
    let epoch = test_epoch();
    bencher.bench(|| spk.state(epoch, Earth, Moon));
}
