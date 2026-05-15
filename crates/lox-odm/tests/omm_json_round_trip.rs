// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! End-to-end round-trip tests for the JSON OMM reader/writer against
//! real Space-Track OMM fixtures (NUSAT-8 / MARIE), including
//! non-CCSDS Space-Track extras (TLE_LINE0/1/2, OBJECT_TYPE, RCS_SIZE,
//! COUNTRY_CODE, LAUNCH_DATE, etc.) that the reader silently absorbs.
//!
//! Exercises the public API: [`lox_odm::json::read_omm`],
//! [`lox_odm::json::read_omm_list`], [`lox_odm::json::write_omm`],
//! [`lox_odm::json::write_omm_list`].

use lox_odm::json::{read_omm, read_omm_list, write_omm, write_omm_list};

const NUSAT_8: &str = r#"{
    "CCSDS_OMM_VERS": "2.0",
    "COMMENT": "GENERATED VIA SPACE-TRACK.ORG API",
    "CREATION_DATE": "2020-12-29T06:26:10",
    "ORIGINATOR": "18 SPCS",
    "OBJECT_NAME": "NUSAT-8 (MARIE)",
    "OBJECT_ID": "2020-003C",
    "CENTER_NAME": "EARTH",
    "REF_FRAME": "TEME",
    "TIME_SYSTEM": "UTC",
    "MEAN_ELEMENT_THEORY": "SGP4",
    "EPOCH": "2020-12-29T03:57:59.406624",
    "MEAN_MOTION": "15.27989249",
    "ECCENTRICITY": "0.00133560",
    "INCLINATION": "97.2970",
    "RA_OF_ASC_NODE": "66.4161",
    "ARG_OF_PERICENTER": "110.6345",
    "MEAN_ANOMALY": "334.7107",
    "EPHEMERIS_TYPE": "0",
    "CLASSIFICATION_TYPE": "U",
    "NORAD_CAT_ID": "45018",
    "ELEMENT_SET_NO": "999",
    "REV_AT_EPOCH": "5327",
    "BSTAR": "0.00008455300000",
    "MEAN_MOTION_DOT": "0.00002241",
    "MEAN_MOTION_DDOT": "0.0000000000000",
    "SEMIMAJOR_AXIS": "6859.961",
    "PERIOD": "94.242",
    "APOAPSIS": "490.988",
    "PERIAPSIS": "472.664",
    "OBJECT_TYPE": "PAYLOAD",
    "RCS_SIZE": "MEDIUM",
    "COUNTRY_CODE": "ARGN",
    "LAUNCH_DATE": "2020-01-15",
    "SITE": "TSC",
    "DECAY_DATE": null,
    "FILE": "2911831",
    "GP_ID": "168552672",
    "TLE_LINE0": "0 NUSAT-8 (MARIE)",
    "TLE_LINE1": "1 45018U 20003C   20364.16527091  .00002241  00000-0  84553-4 0  9997",
    "TLE_LINE2": "2 45018  97.2970  66.4161 0013356 110.6345 334.7107 15.27989249 53274"
}"#;

#[test]
fn read_nusat_8_succeeds_with_space_track_extras() {
    let omm = read_omm(NUSAT_8).expect("read");
    assert_eq!(omm.metadata.object_name, "NUSAT-8 (MARIE)");
    assert_eq!(omm.metadata.object_id, "2020-003C");
    assert_eq!(omm.metadata.mean_element_theory, "SGP4");
}

#[test]
fn nusat_8_tle_parameters_populated() {
    let omm = read_omm(NUSAT_8).expect("read");
    let tle = omm.tle_parameters.as_ref().expect("TLE params present");
    assert_eq!(tle.norad_cat_id, Some(45018));
    assert_eq!(tle.classification_type.as_deref(), Some("U"));
    assert_eq!(tle.element_set_no, Some(999));
    assert_eq!(tle.rev_at_epoch, Some(5327));
    assert!((tle.bstar.unwrap() - 8.4553e-5).abs() < 1e-12);
}

#[test]
fn nusat_8_mean_motion_resolves_via_canonical_earth_gm() {
    let omm = read_omm(NUSAT_8).expect("read");
    // 15.28 rev/day at Earth → ~482 km altitude → ~6860 km SMA.
    let a_km = omm.mean_elements.elements.a / 1000.0;
    assert!(
        (a_km - 6860.0).abs() < 50.0,
        "computed SMA = {a_km} km (expected ~6860)"
    );
}

#[test]
fn nusat_8_header_comment_preserved() {
    let omm = read_omm(NUSAT_8).expect("read");
    assert_eq!(omm.header.comments.len(), 1);
    assert_eq!(omm.header.comments[0], "GENERATED VIA SPACE-TRACK.ORG API");
}

#[test]
fn round_trip_nusat_8_preserves_structure() {
    let omm = read_omm(NUSAT_8).expect("read");
    let written = write_omm(&omm).unwrap();
    let omm2 = read_omm(&written).expect("re-read");
    assert_eq!(omm.metadata.object_name, omm2.metadata.object_name);
    assert_eq!(omm.metadata.object_id, omm2.metadata.object_id);
    assert_eq!(omm.metadata.center, omm2.metadata.center);
    assert_eq!(omm.metadata.frame, omm2.metadata.frame);
    assert_eq!(
        omm.metadata.mean_element_theory,
        omm2.metadata.mean_element_theory
    );
    assert_eq!(omm.header.comments, omm2.header.comments);

    let t1 = omm.tle_parameters.as_ref().unwrap();
    let t2 = omm2.tle_parameters.as_ref().unwrap();
    assert_eq!(t1.norad_cat_id, t2.norad_cat_id);
    assert_eq!(t1.classification_type, t2.classification_type);
    assert_eq!(t1.ephemeris_type, t2.ephemeris_type);
    assert_eq!(t1.element_set_no, t2.element_set_no);
    assert_eq!(t1.rev_at_epoch, t2.rev_at_epoch);
    assert_eq!(t1.bstar, t2.bstar);
}

#[test]
fn write_emits_native_numbers_not_strings() {
    let omm = read_omm(NUSAT_8).expect("read");
    let written = write_omm(&omm).unwrap();
    // The wire sends numbers as strings (`"MEAN_MOTION": "15.27989249"`);
    // we emit them as native numbers (no quotes).
    assert!(
        written.contains("\"NORAD_CAT_ID\": 45018"),
        "expected native integer; output excerpt: {}",
        &written[..200.min(written.len())]
    );
}

#[test]
fn read_omm_list_handles_array() {
    let list_json = format!("[{NUSAT_8}, {NUSAT_8}]");
    let omms = read_omm_list(&list_json).expect("read list");
    assert_eq!(omms.len(), 2);
    assert_eq!(omms[0].metadata.object_name, "NUSAT-8 (MARIE)");
    assert_eq!(omms[1].metadata.object_name, "NUSAT-8 (MARIE)");
}

#[test]
fn write_omm_list_emits_json_array() {
    let omm = read_omm(NUSAT_8).expect("read");
    let written = write_omm_list(&[omm.clone(), omm]).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&written).expect("valid JSON");
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 2);
}

#[test]
fn space_track_extras_round_trip_on_write() {
    // Operator-supplied non-CCSDS fields (TLE_LINE0/1/2, OBJECT_TYPE, …)
    // must round-trip through `provider_extras` so downstream callers
    // don't lose them on a read→write cycle.
    let omm = read_omm(NUSAT_8).expect("read");

    // The typed model exposes the extras.
    assert!(omm.provider_extras.contains_key("TLE_LINE0"));
    assert!(omm.provider_extras.contains_key("OBJECT_TYPE"));

    let written = write_omm(&omm).unwrap();
    assert!(
        written.contains("TLE_LINE0"),
        "TLE_LINE0 must survive write"
    );
    assert!(
        written.contains("OBJECT_TYPE"),
        "OBJECT_TYPE must survive write"
    );
    assert!(written.contains("RCS_SIZE"), "RCS_SIZE must survive write");
    assert!(
        written.contains("COUNTRY_CODE"),
        "COUNTRY_CODE must survive write"
    );

    // Reading the written JSON should reproduce the same extras.
    let reparsed = read_omm(&written).expect("re-parse");
    assert_eq!(omm.provider_extras, reparsed.provider_extras);
}
