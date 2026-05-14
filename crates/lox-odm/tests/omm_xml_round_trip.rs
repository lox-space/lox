// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Integration tests: XML OMM round-trip on a realistic SGP4-tuned fixture
//! (GOES-13) and synthetic inputs, verifying structural round-trip correctness.

use lox_odm::xml::omm::{read_omm, write_omm};

/// Realistic GOES-13 OMM in XML wire format.
const GOES_13_OMM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<omm xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
     id="CCSDS_OMM_VERS" version="3.0">
    <header>
        <COMMENT>GEOCENTRIC, CARTESIAN, EARTH FIXED</COMMENT>
        <CREATION_DATE>2007-03-06T16:00:00</CREATION_DATE>
        <ORIGINATOR>NOAA/USA</ORIGINATOR>
        <MESSAGE_ID>OMM-2007-03-06-001</MESSAGE_ID>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>GOES-13</OBJECT_NAME>
                <OBJECT_ID>2006-018A</OBJECT_ID>
                <CENTER_NAME>EARTH</CENTER_NAME>
                <REF_FRAME>TEME</REF_FRAME>
                <TIME_SYSTEM>UTC</TIME_SYSTEM>
                <MEAN_ELEMENT_THEORY>SGP/SGP4</MEAN_ELEMENT_THEORY>
            </metadata>
            <data>
                <meanElements>
                    <COMMENT>Mean elements</COMMENT>
                    <EPOCH>2007-03-05T10:34:41.4264</EPOCH>
                    <SEMI_MAJOR_AXIS units="km">6655.9942</SEMI_MAJOR_AXIS>
                    <ECCENTRICITY>0.0006703</ECCENTRICITY>
                    <INCLINATION units="deg">51.6416</INCLINATION>
                    <RA_OF_ASC_NODE units="deg">247.4627</RA_OF_ASC_NODE>
                    <ARG_OF_PERICENTER units="deg">130.5360</ARG_OF_PERICENTER>
                    <MEAN_ANOMALY units="deg">325.0288</MEAN_ANOMALY>
                    <GM units="km**3/s**2">398600.8</GM>
                </meanElements>
                <tleParameters>
                    <COMMENT>TLE-related parameters</COMMENT>
                    <EPHEMERIS_TYPE>0</EPHEMERIS_TYPE>
                    <CLASSIFICATION_TYPE>U</CLASSIFICATION_TYPE>
                    <NORAD_CAT_ID>25544</NORAD_CAT_ID>
                    <ELEMENT_SET_NO>999</ELEMENT_SET_NO>
                    <REV_AT_EPOCH>4453</REV_AT_EPOCH>
                    <BSTAR>0.00011328</BSTAR>
                    <MEAN_MOTION_DOT>0.00000264</MEAN_MOTION_DOT>
                    <MEAN_MOTION_DDOT>0</MEAN_MOTION_DDOT>
                </tleParameters>
            </data>
        </segment>
    </body>
</omm>"#;

// -----------------------------------------------------------------------
// 1. Realistic OMM XML fixture — parse + structural round-trip
// -----------------------------------------------------------------------

#[test]
fn goes_13_omm_parse_succeeds() {
    let omm = read_omm(GOES_13_OMM).expect("parse failed");

    assert_eq!(omm.metadata.object_name, "GOES-13");
    assert_eq!(omm.metadata.object_id, "2006-018A");
    assert_eq!(omm.metadata.mean_element_theory, "SGP/SGP4");
    assert_eq!(omm.header.originator, "NOAA/USA");
    assert_eq!(omm.header.message_id.as_deref(), Some("OMM-2007-03-06-001"));
    assert!(omm.tle_parameters.is_some(), "TLE parameters missing");

    // SMA round-trip: 6655.9942 km → m → km
    let sma_km = omm.mean_elements.elements.a / 1000.0;
    let diff_sma = (sma_km - 6655.9942).abs();
    assert!(diff_sma < 1e-6, "SMA unexpected: {sma_km}");

    // Wire GM: 398600.8 km³/s² = 3.988008e14 m³/s²
    let gm = omm.mean_elements.gm.expect("wire GM missing");
    let diff_gm = (gm.as_f64() - 398600.8e9).abs();
    assert!(diff_gm < 1e8, "GM unexpected: {}", gm.as_f64());
}

#[test]
fn goes_13_omm_structural_round_trip() {
    // First pass normalises epoch-string precision; subsequent passes must agree.
    let first = read_omm(GOES_13_OMM).expect("first parse failed");
    let serialised = write_omm(&first);
    let second = read_omm(&serialised).expect("second parse failed");
    let serialised2 = write_omm(&second);
    let third = read_omm(&serialised2).expect("third parse failed");

    // Structural identity (field-by-field, not full PartialEq, to tolerate
    // tiny floating-point drift through deg↔rad and km↔m conversions).
    assert_eq!(second.header.originator, third.header.originator);
    assert_eq!(second.metadata.object_name, third.metadata.object_name);
    assert_eq!(second.metadata.object_id, third.metadata.object_id);
    assert_eq!(second.metadata.center, third.metadata.center);
    assert_eq!(second.metadata.frame, third.metadata.frame);
    assert_eq!(
        second.metadata.mean_element_theory,
        third.metadata.mean_element_theory
    );
    assert_eq!(second.mean_elements.gm, third.mean_elements.gm);

    let diff_sma = (second.mean_elements.elements.a - third.mean_elements.elements.a).abs();
    assert!(
        diff_sma < 1e-6,
        "SMA drifted across round-trips: {diff_sma}"
    );

    let diff_ecc = (second.mean_elements.elements.e - third.mean_elements.elements.e).abs();
    assert!(diff_ecc < 1e-12, "eccentricity drifted: {diff_ecc}");

    // TLE preservation
    assert_eq!(
        second.tle_parameters.is_some(),
        third.tle_parameters.is_some()
    );
    let tle2 = second.tle_parameters.as_ref().unwrap();
    let tle3 = third.tle_parameters.as_ref().unwrap();
    assert_eq!(tle2.norad_cat_id, tle3.norad_cat_id);
    assert_eq!(tle2.classification_type, tle3.classification_type);
    assert_eq!(tle2.ephemeris_type, tle3.ephemeris_type);
    assert_eq!(tle2.element_set_no, tle3.element_set_no);
    assert_eq!(tle2.rev_at_epoch, tle3.rev_at_epoch);
    assert_eq!(tle2.bstar, tle3.bstar);

    assert!(
        second.tle_parameters.is_some(),
        "TLE parameters lost after round-trip"
    );
    assert!(
        (serialised2.len() as isize - serialised.len() as isize).abs() < 20,
        "serialised length changed unexpectedly across round-trips"
    );
}

// -----------------------------------------------------------------------
// 2. TLE parameters round-trip (NORAD_CAT_ID, BSTAR, BTERM, AGOM, etc.)
// -----------------------------------------------------------------------

#[test]
fn tle_parameters_round_trip_full_fields() {
    let omm = read_omm(GOES_13_OMM).expect("parse failed");
    let tle = omm.tle_parameters.as_ref().expect("TLE parameters missing");

    assert_eq!(tle.ephemeris_type, Some(0));
    assert_eq!(tle.classification_type.as_deref(), Some("U"));
    assert_eq!(tle.norad_cat_id, Some(25544));
    assert_eq!(tle.element_set_no, Some(999));
    assert_eq!(tle.rev_at_epoch, Some(4453));
    assert!(
        (tle.bstar.unwrap() - 0.00011328).abs() < 1e-12,
        "BSTAR mismatch: {:?}",
        tle.bstar
    );
    assert!(
        (tle.mean_motion_dot.unwrap() - 0.00000264).abs() < 1e-12,
        "MEAN_MOTION_DOT mismatch: {:?}",
        tle.mean_motion_dot
    );
    assert_eq!(tle.mean_motion_ddot, Some(0.0));

    // Write and re-read to confirm TLE fields survive serialisation.
    let written = write_omm(&omm);
    let reparsed = read_omm(&written).expect("re-parse after write failed");
    let tle2 = reparsed
        .tle_parameters
        .as_ref()
        .expect("TLE missing after re-parse");

    assert_eq!(tle.norad_cat_id, tle2.norad_cat_id);
    assert_eq!(tle.classification_type, tle2.classification_type);
    assert_eq!(tle.bstar, tle2.bstar);
    assert_eq!(tle.mean_motion_dot, tle2.mean_motion_dot);
    assert_eq!(tle.mean_motion_ddot, tle2.mean_motion_ddot);
}

// -----------------------------------------------------------------------
// 3. Header comments preserved
// -----------------------------------------------------------------------

#[test]
fn header_comments_preserved() {
    let omm = read_omm(GOES_13_OMM).expect("parse failed");

    // Fixture has one header comment: "GEOCENTRIC, CARTESIAN, EARTH FIXED"
    assert_eq!(
        omm.header.comments.len(),
        1,
        "expected exactly 1 header comment"
    );
    assert_eq!(
        omm.header.comments[0].trim(),
        "GEOCENTRIC, CARTESIAN, EARTH FIXED"
    );

    // The meanElements block also has a comment: "Mean elements"
    assert!(
        !omm.mean_elements.comments.is_empty(),
        "mean elements comments lost"
    );

    // The tleParameters block also has a comment: "TLE-related parameters"
    let tle_comments = &omm.tle_parameters.as_ref().unwrap().comments;
    assert!(!tle_comments.is_empty(), "TLE comments lost");

    // After write + re-read, comments are preserved.
    let written = write_omm(&omm);
    let reparsed = read_omm(&written).expect("re-parse failed");
    assert_eq!(omm.header.comments, reparsed.header.comments);
    assert_eq!(omm.mean_elements.comments, reparsed.mean_elements.comments);
}
