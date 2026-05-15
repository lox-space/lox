// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Integration tests: XML OPM round-trip on the legacy fixture and synthetic
//! inputs, verifying that parse → write → re-parse produces structurally
//! identical values.

use lox_odm::xml::opm::{read_opm, write_opm};

const LEGACY_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<opm  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://sanaregistry.org/r/ndmxml/ndmxml-1.0-master.xsd"
        id="CCSDS_OPM_VERS" version="3.0">

    <header>
    <COMMENT>THIS IS AN XML VERSION OF THE OPM</COMMENT>
    <CREATION_DATE>2001-11-06T09:23:57</CREATION_DATE>
    <ORIGINATOR>JAXA</ORIGINATOR>
    <MESSAGE_ID>OPM 201113719185</MESSAGE_ID>
    </header>
    <body>
    <segment>
        <metadata>
            <COMMENT>GEOCENTRIC, CARTESIAN, EARTH FIXED</COMMENT>
            <OBJECT_NAME>OSPREY 5</OBJECT_NAME>
            <OBJECT_ID>1998-999A</OBJECT_ID>
            <CENTER_NAME>EARTH</CENTER_NAME>
            <REF_FRAME>TOD</REF_FRAME>
            <REF_FRAME_EPOCH>1998-12-18T14:28:15.1172</REF_FRAME_EPOCH>
            <TIME_SYSTEM>UTC</TIME_SYSTEM>
        </metadata>
        <data>
            <stateVector>
                <EPOCH>2008-09-20T12:25:40.104192</EPOCH>
                <X units="km">4086.147180</X>
                <Y units="km">-994.936814</Y>
                <Z units="km">5250.678791</Z>
                <X_DOT units="km/s">2.511071</X_DOT>
                <Y_DOT units="km/s">7.255240</Y_DOT>
                <Z_DOT units="km/s">-0.583165</Z_DOT>
            </stateVector>
            <keplerianElements>
                <SEMI_MAJOR_AXIS units="km">6730.96</SEMI_MAJOR_AXIS>
                <ECCENTRICITY>0.0006703</ECCENTRICITY>
                <INCLINATION units="deg">51.6416</INCLINATION>
                <RA_OF_ASC_NODE units="deg">247.463</RA_OF_ASC_NODE>
                <ARG_OF_PERICENTER units="deg">130.536</ARG_OF_PERICENTER>
                <TRUE_ANOMALY units="deg">324.985</TRUE_ANOMALY>
                <GM units="km**3/s**2">398600.9368</GM>
            </keplerianElements>
            <spacecraftParameters>
                <MASS>3000.000000</MASS>
                <SOLAR_RAD_AREA>18.770000</SOLAR_RAD_AREA>
                <SOLAR_RAD_COEFF>1.000000</SOLAR_RAD_COEFF>
                <DRAG_AREA>18.770000</DRAG_AREA>
                <DRAG_COEFF>2.500000</DRAG_COEFF>
            </spacecraftParameters>
            <covarianceMatrix>
                <COV_REF_FRAME>ITRF1997</COV_REF_FRAME>
                <CX_X>0.316</CX_X>
                <CY_X>0.722</CY_X>
                <CY_Y>0.518</CY_Y>
                <CZ_X>0.202</CZ_X>
                <CZ_Y>0.715</CZ_Y>
                <CZ_Z>0.002</CZ_Z>
                <CX_DOT_X>0.912</CX_DOT_X>
                <CX_DOT_Y>0.306</CX_DOT_Y>
                <CX_DOT_Z>0.276</CX_DOT_Z>
                <CX_DOT_X_DOT>0.797</CX_DOT_X_DOT>
                <CY_DOT_X>0.562</CY_DOT_X>
                <CY_DOT_Y>0.899</CY_DOT_Y>
                <CY_DOT_Z>0.022</CY_DOT_Z>
                <CY_DOT_X_DOT>0.079</CY_DOT_X_DOT>
                <CY_DOT_Y_DOT>0.415</CY_DOT_Y_DOT>
                <CZ_DOT_X>0.245</CZ_DOT_X>
                <CZ_DOT_Y>0.965</CZ_DOT_Y>
                <CZ_DOT_Z>0.950</CZ_DOT_Z>
                <CZ_DOT_X_DOT>0.435</CZ_DOT_X_DOT>
                <CZ_DOT_Y_DOT>0.621</CZ_DOT_Y_DOT>
                <CZ_DOT_Z_DOT>0.991</CZ_DOT_Z_DOT>
            </covarianceMatrix>
            <maneuverParameters>
                <COMMENT>Maneuver 1</COMMENT>
                <MAN_EPOCH_IGNITION>2008-09-20T12:41:09.984493</MAN_EPOCH_IGNITION>
                <MAN_DURATION units="s">180.000</MAN_DURATION>
                <MAN_DELTA_MASS units="kg">-0.001</MAN_DELTA_MASS>
                <MAN_REF_FRAME>RSW</MAN_REF_FRAME>
                <MAN_DV_1 units="km/s">0.000000</MAN_DV_1>
                <MAN_DV_2 units="km/s">0.280000</MAN_DV_2>
                <MAN_DV_3 units="km/s">0.000000</MAN_DV_3>
            </maneuverParameters>
            <maneuverParameters>
                <MAN_EPOCH_IGNITION>2008-09-20T13:33:11.374985</MAN_EPOCH_IGNITION>
                <MAN_DURATION units="s">180.000</MAN_DURATION>
                <MAN_DELTA_MASS units="kg">-0.001</MAN_DELTA_MASS>
                <MAN_REF_FRAME>RSW</MAN_REF_FRAME>
                <MAN_DV_1 units="km/s">0.000000</MAN_DV_1>
                <MAN_DV_2 units="km/s">0.270000</MAN_DV_2>
                <MAN_DV_3 units="km/s">0.000000</MAN_DV_3>
            </maneuverParameters>
        </data>
    </segment>
    </body>
</opm>"#;

// -----------------------------------------------------------------------
// 1. AST-ish round-trip on the legacy XML fixture
//
// We do a double round-trip: parse → write → re-parse → write → re-parse.
// The first write normalises the epoch string precision (lox-time UTC Display
// emits 3 decimal places by default); after that normalisation the two
// parses must be structurally identical.
// -----------------------------------------------------------------------

#[test]
fn legacy_fixture_ast_round_trip() {
    // First pass: parse the fixture and normalise
    let first = read_opm(LEGACY_FIXTURE).expect("first parse failed");
    let serialised = write_opm(&first).unwrap();
    // Second pass: parse the normalised XML — this and a third pass should agree
    let second = read_opm(&serialised).expect("second parse failed");
    let serialised2 = write_opm(&second).unwrap();
    let third = read_opm(&serialised2).expect("third parse failed");
    assert_eq!(second, third, "double round-trip mismatch");
}

// -----------------------------------------------------------------------
// 2. Multiple maneuvers preserved (count + per-maneuver fields)
// -----------------------------------------------------------------------

#[test]
fn legacy_fixture_multiple_maneuvers_preserved() {
    let opm = read_opm(LEGACY_FIXTURE).expect("parse failed");
    assert_eq!(opm.maneuvers.len(), 2, "expected 2 maneuvers");

    // First maneuver
    assert_eq!(opm.maneuvers[0].comments, vec!["Maneuver 1".to_string()]);
    let dv2_man1 = opm.maneuvers[0].delta_v[1].to_kilometers_per_second();
    let diff1 = (dv2_man1 - 0.28).abs();
    assert!(diff1 < 1e-9, "MAN_DV_2 man1 unexpected: {dv2_man1}");

    // Second maneuver (no comment)
    assert!(opm.maneuvers[1].comments.is_empty());
    let dv2_man2 = opm.maneuvers[1].delta_v[1].to_kilometers_per_second();
    let diff2 = (dv2_man2 - 0.27).abs();
    assert!(diff2 < 1e-9, "MAN_DV_2 man2 unexpected: {dv2_man2}");
}

// -----------------------------------------------------------------------
// 3. Keplerian elements preserved with wire GM
// -----------------------------------------------------------------------

#[test]
fn legacy_fixture_keplerian_with_wire_gm() {
    let opm = read_opm(LEGACY_FIXTURE).expect("parse failed");
    let kep = opm.keplerian.expect("Keplerian block missing");

    // Semi-major axis round-trip (km)
    let sma_km = kep.elements.semi_major_axis().to_kilometers();
    let diff_sma = (sma_km - 6730.96).abs();
    assert!(diff_sma < 1e-6, "SMA unexpected: {sma_km}");

    // Wire GM preserved: 398600.9368 km³/s² = 3.986009368e14 m³/s²
    let gm_m3s2 = kep.gm.expect("GM missing").as_f64();
    let expected = 398600.9368e9;
    let diff_gm = (gm_m3s2 - expected).abs();
    assert!(diff_gm < 1.0, "GM unexpected: {gm_m3s2}");
}

// -----------------------------------------------------------------------
// 4. Covariance round-trip (specific matrix values)
// -----------------------------------------------------------------------

#[test]
fn legacy_fixture_covariance_round_trip() {
    // Normalise epoch precision first
    let first = read_opm(LEGACY_FIXTURE).expect("first parse failed");
    let serialised = write_opm(&first).unwrap();
    let second = read_opm(&serialised).expect("second parse failed");
    let serialised2 = write_opm(&second).unwrap();
    let third = read_opm(&serialised2).expect("third parse failed");

    let cov1 = second
        .covariance
        .as_ref()
        .expect("covariance missing on second parse");
    let cov2 = third
        .covariance
        .as_ref()
        .expect("covariance missing on third parse");

    // CX_X = 0.316
    let diff_cx_x = (cov1.matrix[(0, 0)] - 0.316).abs();
    assert!(
        diff_cx_x < 1e-12,
        "CX_X unexpected: {}",
        cov1.matrix[(0, 0)]
    );

    // CZ_DOT_Z_DOT = 0.991
    let diff_last = (cov1.matrix[(5, 5)] - 0.991).abs();
    assert!(
        diff_last < 1e-12,
        "CZ_DOT_Z_DOT unexpected: {}",
        cov1.matrix[(5, 5)]
    );

    // Symmetry preserved
    assert_eq!(cov1.matrix[(1, 0)], cov1.matrix[(0, 1)]);
    assert_eq!(cov1.matrix[(3, 0)], cov1.matrix[(0, 3)]);

    // Structural equality between first and second parse
    assert_eq!(
        cov1.matrix, cov2.matrix,
        "covariance matrix changed across round-trip"
    );
}
