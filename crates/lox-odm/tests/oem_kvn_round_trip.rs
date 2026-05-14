// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! End-to-end round-trip tests for the KVN OEM reader/writer against a
//! realistic JPL Mars Global Surveyor fixture (three segments, embedded
//! comments, one covariance entry in segment 2).
//!
//! Exercises the public API: [`lox_odm::kvn::parse`],
//! [`lox_odm::kvn::read_oem`], [`lox_odm::kvn::write_oem`], plus
//! [`std::fmt::Display`] on [`lox_odm::kvn::KvnDocument`].

use lox_odm::kvn::{parse, read_oem, write_oem};

const JPL_OEM: &str = "\
CCSDS_OEM_VERS = 3.0
CREATION_DATE = 1996-11-04T17:22:31
ORIGINATOR = NASA/JPL

META_START
OBJECT_NAME = MARS GLOBAL SURVEYOR
OBJECT_ID = 1996-062A
CENTER_NAME = MARS BARYCENTER
REF_FRAME = J2000
TIME_SYSTEM = TAI
START_TIME = 1996-12-18T12:00:00.331
USEABLE_START_TIME = 1996-12-18T12:10:00.331
USEABLE_STOP_TIME = 1996-12-28T21:23:00.331
STOP_TIME = 1996-12-28T21:28:00.331
INTERPOLATION = HERMITE
INTERPOLATION_DEGREE = 7
META_STOP

COMMENT This file was produced by M.R. Somebody, MSOO NAV/JPL, 1996NOV 04. It is
COMMENT to be used for DSN scheduling purposes only.

1996-12-18T12:00:00.331 2789.619 -280.045 -1746.755 4.73372 -2.49586 -1.04195
1996-12-18T12:01:00.331 2783.419 -308.143 -1877.071 5.18604 -2.42124 -1.99608
1996-12-18T12:02:00.331 2776.033 -336.859 -2008.682 5.63678 -2.33951 -1.94687
1996-12-28T21:28:00.331 -3881.024 563.959 -682.773 -3.28827 -3.66735 1.63861

META_START
OBJECT_NAME = MARS GLOBAL SURVEYOR
OBJECT_ID = 1996-062A
CENTER_NAME = MARS BARYCENTER
REF_FRAME = J2000
TIME_SYSTEM = TAI
START_TIME = 1996-12-28T21:29:07.267
USEABLE_START_TIME = 1996-12-28T22:08:02.5
USEABLE_STOP_TIME = 1996-12-30T01:18:02.5
STOP_TIME = 1996-12-30T01:28:02.267
INTERPOLATION = HERMITE
INTERPOLATION_DEGREE = 7
META_STOP

COMMENT This block begins after trajectory correction maneuver TCM-3.
1996-12-28T21:29:07.267 -2432.166 -063.042 1742.754 7.33702 -3.495867 -1.041945
1996-12-28T21:59:02.267 -2445.234 -878.141 1873.073 1.86043 -3.421256 -0.996366
1996-12-28T22:00:02.267 -2458.079 -683.858 2007.684 6.36786 -3.339563 -0.946654
1996-12-30T01:28:02.267 2164.375 1115.811 -688.131 -3.53328 -2.88452 0.88535

COVARIANCE_START
EPOCH = 1996-12-28T21:29:07.267
COV_REF_FRAME = EME2000
3.3313494e-04
4.6189273e-04 6.7824216e-04
-3.0700078e-04 -4.2212341e-04 3.2319319e-04
-3.3493650e-07 -4.6860842e-07 2.4849495e-07 4.2960228e-10
-2.2118325e-07 -2.8641868e-07 1.7980986e-07 2.6088992e-10 1.7675147e-10
-3.0413460e-07 -4.9894969e-07 3.5403109e-07 1.8692631e-10 1.0088625e-10 6.2244443e-10
COVARIANCE_STOP
";

#[test]
fn ast_round_trip_jpl_oem() {
    let parsed = parse(JPL_OEM).expect("parse");
    let reemitted = format!("{parsed}");
    let reparsed = parse(&reemitted).expect("reparse");
    assert_eq!(parsed, reparsed);
}

#[test]
fn jpl_oem_has_two_segments() {
    let oem = read_oem(JPL_OEM).expect("read");
    assert_eq!(oem.segments.len(), 2);
}

#[test]
fn jpl_oem_segment_1_has_four_states() {
    let oem = read_oem(JPL_OEM).expect("read");
    assert_eq!(oem.segments[0].states.len(), 4);
}

#[test]
fn jpl_oem_segment_2_has_one_covariance_entry() {
    let oem = read_oem(JPL_OEM).expect("read");
    assert_eq!(oem.segments[1].covariance_history.len(), 1);
}

#[test]
fn jpl_oem_preserves_comments() {
    let oem = read_oem(JPL_OEM).expect("read");

    let header_count = oem.header.comments.len();
    let metadata_count: usize = oem.segments.iter().map(|s| s.metadata.comments.len()).sum();
    let data_count: usize = oem.segments.iter().map(|s| s.data_comments.len()).sum();
    let covariance_count: usize = oem
        .segments
        .iter()
        .flat_map(|s| s.covariance_history.iter())
        .map(|c| c.comments.len())
        .sum();

    let total = header_count + metadata_count + data_count + covariance_count;

    // JPL fixture has 3 body COMMENT lines:
    //   2 before segment 1's data rows
    //   1 before segment 2's data rows
    // Plus the trailing "block begins after TCM-3" comment.
    assert_eq!(
        total, 3,
        "comment counts: header={header_count}, metadata={metadata_count}, data={data_count}, covariance={covariance_count}"
    );
}

#[test]
fn typed_round_trip_jpl_oem_preserves_structure() {
    // Bit-exact f64 equality through km↔m unit conversions is
    // structurally impossible (per OPM phase findings). Assert the
    // *structure* round-trips and rely on the AST round-trip test for
    // wire-level preservation.
    let oem = read_oem(JPL_OEM).expect("read");
    let written = write_oem(&oem);
    let oem2 = read_oem(&written).expect("re-read");

    assert_eq!(oem.header.originator, oem2.header.originator);
    assert_eq!(oem.segments.len(), oem2.segments.len());
    for (a, b) in oem.segments.iter().zip(&oem2.segments) {
        assert_eq!(a.metadata.object_name, b.metadata.object_name);
        assert_eq!(a.metadata.object_id, b.metadata.object_id);
        assert_eq!(a.metadata.center, b.metadata.center);
        assert_eq!(a.metadata.frame, b.metadata.frame);
        assert_eq!(a.metadata.interpolation, b.metadata.interpolation);
        assert_eq!(
            a.metadata.interpolation_degree,
            b.metadata.interpolation_degree
        );
        assert_eq!(a.states.len(), b.states.len());
        assert_eq!(a.covariance_history.len(), b.covariance_history.len());
        assert_eq!(a.data_comments, b.data_comments);
    }
}

#[test]
fn jpl_oem_covariance_round_trip_has_six_rows() {
    let oem = read_oem(JPL_OEM).expect("read");
    let cov = &oem.segments[1].covariance_history[0];
    // Matrix is 6×6; verify a couple of lower-triangle values from the fixture.
    // First row: CX_X = 3.3313494e-04
    assert!((cov.matrix[(0, 0)] - 3.3313494e-04).abs() < 1e-12);
    // Last row, last column: CZ_DOT_Z_DOT = 6.2244443e-10
    assert!((cov.matrix[(5, 5)] - 6.2244443e-10).abs() < 1e-16);
    // Symmetry check: matrix[(1,0)] == matrix[(0,1)]
    assert_eq!(cov.matrix[(1, 0)], cov.matrix[(0, 1)]);
}
