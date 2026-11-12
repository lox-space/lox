// SPDX-FileCopyrightText: 2026 Torkil Rein Gustavsen <torkilrg@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

//! Link-budget benchmarks: refining the visibility windows of a single
//! ground-space pair into the intervals in which a lumped X-band downlink
//! closes, with the fixed losses of the carrier plan and with an
//! elevation-dependent site profile.
//!
//! The profiled variant resolves the elevation at every sample and
//! interpolates the profile there, so the pair of benchmarks measures what
//! opting into site propagation costs.
//!
//! Run with `cargo bench -p lox-space --bench link_budget`.

use divan::Bencher;
use lox_space::analysis::assets::Scenario;
use lox_space::analysis::link_budget::{CarrierPlan, LinkBudgetAnalysis};
use lox_space::analysis::visibility::{VisibilityAnalysis, VisibilityResults};
use lox_space::bodies::Origin;
use lox_space::comms::channel::Channel;
use lox_space::comms::modcod::dvb_s2;
use lox_space::comms::terminal::{EirpModel, GtModel};
use lox_space::comms::{ElevationProfile, FrequencyBand, PropagationLosses, SitePropagation};
use lox_space::core::units::{Angle, Decibel, Frequency};
use lox_space::frames::Frame;

#[path = "common/mod.rs"]
mod common;

const CARRIER: Frequency = Frequency::gigahertz(8.2);

fn main() {
    divan::main();
}

/// The lunar ground-space pair, fitted with a lumped X-band downlink that
/// closes over part of each visibility window.
fn setup() -> (Scenario, common::DynamicEnsemble, VisibilityResults) {
    let (bare, ensemble) = common::setup_dynamic();
    let band = FrequencyBand::X;
    let tx = EirpModel::new(band, Decibel::decibels(20.0)).unwrap();
    let rx = GtModel::new(band, Decibel::decibels(20.0)).unwrap();
    let spacecraft = bare.spacecraft()[0]
        .clone()
        .with_tx_terminal("downlink", tx);
    let station = bare.ground_stations()[0]
        .clone()
        .with_rx_terminal("main", rx);
    let interval = bare.interval();
    let scenario = Scenario::new(interval.start(), interval.end(), Origin::Earth, Frame::Icrf)
        .with_spacecraft(&[spacecraft])
        .with_ground_stations(&[station]);
    let visibility = VisibilityAnalysis::new(&scenario, &ensemble)
        .compute()
        .unwrap();
    (scenario, ensemble, visibility)
}

fn plan() -> CarrierPlan {
    let channel = Channel::builder(Frequency::megahertz(1.0)).build().unwrap();
    CarrierPlan::new(CARRIER, channel, dvb_s2().to_vec()).unwrap()
}

/// Four atmospheric lines following the cosecant law, on a 14-point grid like
/// the one the ITU-R provider produces.
fn site() -> SitePropagation {
    let samples = [
        5.0_f64, 6.0, 7.5, 10.0, 12.5, 15.0, 20.0, 25.0, 30.0, 40.0, 50.0, 65.0, 80.0, 90.0,
    ]
    .into_iter()
    .map(|el_deg| {
        let csc = 1.0 / el_deg.to_radians().sin();
        let losses = PropagationLosses::builder()
            .rain(Decibel::decibels(0.4 * csc))
            .gaseous(Decibel::decibels(0.1 * csc))
            .cloud(Decibel::decibels(0.1 * csc))
            .scintillation(Decibel::decibels(0.05 * csc))
            .build()
            .unwrap();
        (Angle::degrees(el_deg), losses)
    })
    .collect();
    SitePropagation::new(vec![ElevationProfile::new(CARRIER, samples).unwrap()]).unwrap()
}

#[divan::bench]
fn link_budget_single_pair_fixed_losses(bencher: Bencher) {
    let (scenario, ensemble, visibility) = setup();
    bencher.bench(|| {
        LinkBudgetAnalysis::new(&scenario, &ensemble, &visibility, plan())
            .compute()
            .unwrap()
    });
}

#[divan::bench]
fn link_budget_single_pair_site_profile(bencher: Bencher) {
    let (scenario, ensemble, visibility) = setup();
    let site = site();
    bencher.bench(|| {
        LinkBudgetAnalysis::new(&scenario, &ensemble, &visibility, plan())
            .with_site_propagation("cebreros", site.clone())
            .compute()
            .unwrap()
    });
}
