// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0
//
// Parity tests: provider methods must return the same values as the
// legacy free fns. These tests are temporary and will be deleted in
// Phase D once the free fns are removed.

mod common;

use lox_core::units::Angle;
use lox_itur::p1511;

#[test]
fn topographic_altitude_madrid() {
    let p = common::provider();
    let lat = Angle::degrees(40.4);
    let lon = Angle::degrees(-3.7);
    let from_provider = p.topographic_altitude(lat, lon).unwrap().to_meters();
    let from_free = p1511::topographic_altitude(lat, lon).to_meters();
    assert!(
        (from_provider - from_free).abs() < 1e-9,
        "provider={from_provider} free={from_free}"
    );
}
