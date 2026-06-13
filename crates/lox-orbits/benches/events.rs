// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Baseline benchmarks for event detection.
//!
//! The scenario mimics ground-station visibility: an elevation-like signal
//! (orbital-period sinusoid above a mask) intersected with an
//! occultation-like condition, over one day at LEO rates. The interesting
//! quantity alongside wall time is the evaluation count, which the unit
//! tests track; these benches pin the end-to-end cost.

use divan::Bencher;
use lox_orbits::events::{EventsToIntervals, FnDetect, IntervalDetector, RootFindingDetector};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time;
use lox_time::time_scales::Tai;

fn main() {
    divan::main();
}

const DAY: f64 = 86_400.0;
/// LEO orbital period [s].
const PERIOD: f64 = 5_700.0;

fn horizon() -> TimeInterval<Tai> {
    let start = time!(Tai, 2026, 6, 1).unwrap();
    TimeInterval::new(start, start + TimeDelta::from_seconds_f64(DAY))
}

fn seconds_since(start: Time<Tai>, t: Time<Tai>) -> f64 {
    (t - start).to_seconds().to_f64()
}

/// Elevation-like condition: sinusoid at the orbital period above a mask,
/// positive for ~8% of each orbit (a realistic pass fraction).
fn elevation(start: Time<Tai>) -> impl Fn(Time<Tai>) -> f64 + Copy {
    move |t| {
        let s = seconds_since(start, t);
        (std::f64::consts::TAU * s / PERIOD).sin() - 0.97
    }
}

/// Occultation-like condition on an incommensurate period.
fn occultation(start: Time<Tai>) -> impl Fn(Time<Tai>) -> f64 + Copy {
    move |t| {
        let s = seconds_since(start, t);
        (std::f64::consts::TAU * s / (PERIOD * 1.37) + 0.7).sin() + 0.4
    }
}

#[divan::bench]
fn single_level_windows(bencher: Bencher) {
    let interval = horizon();
    let f = elevation(interval.start());
    bencher.bench(|| {
        let detector = RootFindingDetector::new(FnDetect(f), TimeDelta::from_seconds(10));
        EventsToIntervals::new(detector).detect(interval).unwrap()
    });
}

#[divan::bench]
fn two_level_windows(bencher: Bencher) {
    let interval = horizon();
    let f = elevation(interval.start());
    bencher.bench(|| {
        let detector = RootFindingDetector::new(FnDetect(f), TimeDelta::from_seconds(10))
            .with_coarse_step(TimeDelta::from_seconds(300));
        EventsToIntervals::new(detector).detect(interval).unwrap()
    });
}

#[divan::bench]
fn signal_windows(bencher: Bencher) {
    use lox_orbits::signals::{Detector, SignalFn, UniformSampler};

    let interval = horizon();
    let f = elevation(interval.start());
    bencher.bench(|| {
        Detector::new(SignalFn(f))
            .windows(interval, &UniformSampler::new(TimeDelta::from_seconds(10)))
            .unwrap()
    });
}

#[divan::bench]
fn signal_min_windows(bencher: Bencher) {
    use lox_orbits::signals::{Detector, SignalExt, SignalFn, UniformSampler};

    let interval = horizon();
    let f = elevation(interval.start());
    let g = occultation(interval.start());
    bencher.bench(|| {
        Detector::new(SignalFn(f).min(SignalFn(g)))
            .windows(interval, &UniformSampler::new(TimeDelta::from_seconds(10)))
            .unwrap()
    });
}
