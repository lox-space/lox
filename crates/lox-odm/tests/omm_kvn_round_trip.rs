// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! End-to-end round-trip tests for the KVN OMM reader/writer against
//! a realistic SGP/SGP4-tuned OMM fixture (GOES-13).
//!
//! Exercises the public API: [`lox_odm::kvn::parse`],
//! [`lox_odm::kvn::read_omm`], [`lox_odm::kvn::write_omm`], plus
//! [`std::fmt::Display`] on [`lox_odm::kvn::KvnDocument`].

use lox_odm::kvn::{parse, read_omm, write_omm};

const GOES_13_OMM: &str = "\
CCSDS_OMM_VERS = 3.0
COMMENT GEOCENTRIC, CARTESIAN, EARTH FIXED
CREATION_DATE = 2007-03-06T16:00:00
ORIGINATOR = NOAA/USA
OBJECT_NAME = GOES-13
OBJECT_ID = 2006-018A
CENTER_NAME = EARTH
REF_FRAME = TEME
TIME_SYSTEM = UTC
MEAN_ELEMENT_THEORY = SGP/SGP4
COMMENT Mean elements
EPOCH = 2007-03-05T10:34:41.4264
SEMI_MAJOR_AXIS = 6655.9942 [km]
ECCENTRICITY = 0.0006703
INCLINATION = 51.6416 [deg]
RA_OF_ASC_NODE = 247.4627 [deg]
ARG_OF_PERICENTER = 130.5360 [deg]
MEAN_ANOMALY = 325.0288 [deg]
GM = 398600.8 [km**3/s**2]
COMMENT TLE-related parameters
EPHEMERIS_TYPE = 0
CLASSIFICATION_TYPE = U
NORAD_CAT_ID = 25544
ELEMENT_SET_NO = 999
REV_AT_EPOCH = 4453
BSTAR = 0.00011328
MEAN_MOTION_DOT = 0.00000264
MEAN_MOTION_DDOT = 0
";

/// A second fixture using `MEAN_MOTION` instead of `SEMI_MAJOR_AXIS`.
/// 15.5 rev/day under Earth's canonical GM ≈ 6798 km SMA — the exact
/// value doesn't matter as long as the read succeeds and SMA is
/// computed without error.
const MEAN_MOTION_OMM: &str = "\
CCSDS_OMM_VERS = 3.0
CREATION_DATE = 2024-01-01T00:00:00
ORIGINATOR = TEST
OBJECT_NAME = TEST-SAT
OBJECT_ID = 2024-001A
CENTER_NAME = EARTH
REF_FRAME = TEME
TIME_SYSTEM = UTC
MEAN_ELEMENT_THEORY = SGP/SGP4
EPOCH = 2024-01-01T00:00:00
MEAN_MOTION = 15.5
ECCENTRICITY = 0.001
INCLINATION = 51.6 [deg]
RA_OF_ASC_NODE = 247.5 [deg]
ARG_OF_PERICENTER = 130.5 [deg]
MEAN_ANOMALY = 325.0 [deg]
";

#[test]
fn ast_round_trip_goes_13_omm() {
    let parsed = parse(GOES_13_OMM).expect("parse");
    let reemitted = format!("{parsed}");
    let reparsed = parse(&reemitted).expect("reparse");
    assert_eq!(parsed, reparsed);
}

#[test]
fn typed_round_trip_goes_13_omm_preserves_structure() {
    // Bit-exact f64 equality through km↔m and deg↔rad conversions is
    // structurally impossible — assert structure only.
    let omm = read_omm(GOES_13_OMM).expect("read");
    let written = write_omm(&omm);
    let omm2 = read_omm(&written).expect("re-read");

    assert_eq!(omm.header.originator, omm2.header.originator);
    assert_eq!(omm.metadata.object_name, omm2.metadata.object_name);
    assert_eq!(omm.metadata.object_id, omm2.metadata.object_id);
    assert_eq!(omm.metadata.center, omm2.metadata.center);
    assert_eq!(omm.metadata.frame, omm2.metadata.frame);
    assert_eq!(
        omm.metadata.mean_element_theory,
        omm2.metadata.mean_element_theory
    );
    assert_eq!(omm.mean_elements.gm, omm2.mean_elements.gm);
    assert_eq!(omm.tle_parameters.is_some(), omm2.tle_parameters.is_some());

    let a = omm.tle_parameters.as_ref().unwrap();
    let b = omm2.tle_parameters.as_ref().unwrap();
    assert_eq!(a.norad_cat_id, b.norad_cat_id);
    assert_eq!(a.classification_type, b.classification_type);
    assert_eq!(a.ephemeris_type, b.ephemeris_type);
    assert_eq!(a.element_set_no, b.element_set_no);
    assert_eq!(a.rev_at_epoch, b.rev_at_epoch);
    assert_eq!(a.bstar, b.bstar);
}

#[test]
fn goes_13_omm_preserves_comments() {
    let omm = read_omm(GOES_13_OMM).expect("read");

    let header_count = omm.header.comments.len();
    let metadata_count = omm.metadata.comments.len();
    let mean_elements_count = omm.mean_elements.comments.len();
    let tle_count = omm.tle_parameters.as_ref().map_or(0, |t| t.comments.len());
    let spacecraft_count = omm.spacecraft.as_ref().map_or(0, |s| s.comments.len());
    let covariance_count = omm.covariance.as_ref().map_or(0, |c| c.comments.len());

    let total = header_count
        + metadata_count
        + mean_elements_count
        + tle_count
        + spacecraft_count
        + covariance_count;

    // Three COMMENT lines in the fixture:
    //   GEOCENTRIC, CARTESIAN, EARTH FIXED (header)
    //   Mean elements (mean_elements)
    //   TLE-related parameters (tle)
    assert_eq!(
        total, 3,
        "header={header_count}, metadata={metadata_count}, mean_elements={mean_elements_count}, tle={tle_count}, spacecraft={spacecraft_count}, covariance={covariance_count}"
    );
}

#[test]
fn goes_13_omm_wire_gm_preserved() {
    let omm = read_omm(GOES_13_OMM).expect("read");
    let gm = omm.mean_elements.gm.expect("wire GM");
    // Wire value was 398600.8 km^3/s^2 → 3.986008e14 m^3/s^2
    assert!((gm.as_f64() - 3.986008e14).abs() < 1e8);
}

#[test]
fn mean_motion_input_succeeds_with_canonical_gm() {
    // Custom center with no wire GM → must fail; Known center → succeeds
    // via canonical Earth GM. This fixture uses Known EARTH.
    let omm = read_omm(MEAN_MOTION_OMM).expect("read");

    // Sanity-check the computed SMA is in a reasonable orbital regime
    // (15.5 rev/day ≈ 5500 km altitude → ~ 11900 km SMA). The exact
    // value depends on the GM constant.
    let a_m = omm.mean_elements.elements.a;
    let a_km = a_m / 1000.0;
    assert!(a_km > 5000.0 && a_km < 15000.0, "computed SMA = {a_km} km");

    // gm() returns canonical body GM since wire didn't include GM.
    assert!(omm.gm().is_some());
}

#[test]
fn typed_round_trip_after_mean_motion_input_always_emits_sma() {
    let omm = read_omm(MEAN_MOTION_OMM).expect("read");
    let written = write_omm(&omm);
    // Typed write always emits SEMI_MAJOR_AXIS regardless of wire form.
    assert!(written.contains("SEMI_MAJOR_AXIS"));
    assert!(!written.contains("MEAN_MOTION ="));
    // Note: MEAN_MOTION_DOT etc may appear in TLE params but `MEAN_MOTION =`
    // (with the equals sign) is only the size element. None of the
    // MEAN_MOTION_DOT/DDOT TLE fields are set in our fixture.
}
