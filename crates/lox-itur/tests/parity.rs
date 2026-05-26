// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0
//
// Parity tests: provider methods must return the same values as the
// legacy free fns. These tests are temporary and will be deleted in
// Phase D once the free fns are removed.

mod common;

use lox_core::units::Angle;
use lox_itur::p453;
use lox_itur::p836;
use lox_itur::p839;
use lox_itur::p1510;
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

#[test]
fn surface_mean_temperature_madrid() {
    let p = common::provider();
    let lat = Angle::degrees(40.4);
    let lon = Angle::degrees(-3.7);
    let a = p.surface_mean_temperature(lat, lon).unwrap().to_kelvin();
    let b = p1510::surface_mean_temperature(lat, lon).to_kelvin();
    assert!((a - b).abs() < 1e-9);
}

#[test]
fn surface_month_mean_temperature_madrid_july() {
    let p = common::provider();
    let lat = Angle::degrees(40.4);
    let lon = Angle::degrees(-3.7);
    let a = p
        .surface_month_mean_temperature(lat, lon, 7)
        .unwrap()
        .to_kelvin();
    let b = p1510::surface_month_mean_temperature(lat, lon, 7).to_kelvin();
    assert!((a - b).abs() < 1e-9);
}

#[test]
fn surface_water_vapour_density_madrid() {
    let p = common::provider();
    let lat = Angle::degrees(40.4);
    let lon = Angle::degrees(-3.7);
    let a = p.surface_water_vapour_density(lat, lon, 1.0).unwrap();
    let b = p836::surface_water_vapour_density(lat, lon, 1.0);
    assert!((a - b).abs() < 1e-9);
}

#[test]
fn total_water_vapour_content_madrid() {
    let p = common::provider();
    let lat = Angle::degrees(40.4);
    let lon = Angle::degrees(-3.7);
    let a = p.total_water_vapour_content(lat, lon, 50.0).unwrap();
    let b = p836::total_water_vapour_content(lat, lon, 50.0);
    assert!((a - b).abs() < 1e-9);
}

#[test]
fn map_wet_term_radio_refractivity_madrid() {
    let p = common::provider();
    let lat = Angle::degrees(40.4);
    let lon = Angle::degrees(-3.7);
    let a = p.map_wet_term_radio_refractivity(lat, lon, 50.0).unwrap();
    let b = p453::map_wet_term_radio_refractivity(lat, lon, 50.0);
    assert!((a - b).abs() < 1e-9);
}

#[test]
fn isotherm_0c_height_madrid() {
    let p = common::provider();
    let lat = Angle::degrees(40.4);
    let lon = Angle::degrees(-3.7);
    let a = p.isotherm_0c_height(lat, lon).unwrap().to_kilometers();
    let b = p839::isotherm_0c_height(lat, lon).to_kilometers();
    assert!((a - b).abs() < 1e-9);
}

#[test]
fn rain_height_madrid() {
    let p = common::provider();
    let lat = Angle::degrees(40.4);
    let lon = Angle::degrees(-3.7);
    let a = p.rain_height(lat, lon).unwrap().to_kilometers();
    let b = p839::rain_height(lat, lon).to_kilometers();
    assert!((a - b).abs() < 1e-9);
}
