// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Coverage for the top-level format-agnostic `lox_odm::read_*` /
//! `write_*` / `read_*_file` / `write_*_file` functions in `lib.rs`,
//! plus `detect_format`, `Format`, and `OdmError`.

use lox_odm::types::ci::OdmCi;
use lox_odm::types::common::MessageKind;
use lox_odm::{
    Format, OdmError, detect_format, read_ci, read_ci_file, read_oem, read_oem_file, read_omm,
    read_omm_file, read_opm, read_opm_file, write_ci, write_ci_file, write_oem, write_oem_file,
    write_omm, write_omm_file, write_opm, write_opm_file,
};

const OPM_KVN: &str = "\
CCSDS_OPM_VERS = 3.0
CREATION_DATE = 2024-01-01T00:00:00
ORIGINATOR = TEST
OBJECT_NAME = SAT
OBJECT_ID = 2024-000A
CENTER_NAME = EARTH
REF_FRAME = ICRF
TIME_SYSTEM = TAI
EPOCH = 2024-01-01T00:00:00
X = 7000.0 [km]
Y = 0.0 [km]
Z = 0.0 [km]
X_DOT = 0.0 [km/s]
Y_DOT = 7.5 [km/s]
Z_DOT = 0.0 [km/s]
";

const OPM_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<opm id="CCSDS_OPM_VERS" version="3.0">
<header><CREATION_DATE>2024-01-01T00:00:00</CREATION_DATE><ORIGINATOR>TEST</ORIGINATOR></header>
<body><segment>
<metadata><OBJECT_NAME>SAT</OBJECT_NAME><OBJECT_ID>2024-000A</OBJECT_ID><CENTER_NAME>EARTH</CENTER_NAME><REF_FRAME>ICRF</REF_FRAME><TIME_SYSTEM>TAI</TIME_SYSTEM></metadata>
<data><stateVector><EPOCH>2024-01-01T00:00:00</EPOCH><X units="km">7000.0</X><Y units="km">0.0</Y><Z units="km">0.0</Z><X_DOT units="km/s">0.0</X_DOT><Y_DOT units="km/s">7.5</Y_DOT><Z_DOT units="km/s">0.0</Z_DOT></stateVector></data>
</segment></body>
</opm>"#;

const OEM_KVN: &str = "\
CCSDS_OEM_VERS = 3.0
CREATION_DATE = 2024-01-01T00:00:00
ORIGINATOR = TEST

META_START
OBJECT_NAME = SAT
OBJECT_ID = 2024-000A
CENTER_NAME = EARTH
REF_FRAME = ICRF
TIME_SYSTEM = TAI
START_TIME = 2024-01-01T00:00:00
STOP_TIME = 2024-01-01T00:01:00
META_STOP

2024-01-01T00:00:00 7000.0 0.0 0.0 0.0 7.5 0.0
2024-01-01T00:01:00 7001.0 0.0 0.0 0.0 7.5 0.0
";

const OMM_KVN: &str = "\
CCSDS_OMM_VERS = 3.0
CREATION_DATE = 2024-01-01T00:00:00
ORIGINATOR = TEST
OBJECT_NAME = SAT
OBJECT_ID = 2024-000A
CENTER_NAME = EARTH
REF_FRAME = TEME
TIME_SYSTEM = UTC
MEAN_ELEMENT_THEORY = SGP/SGP4
EPOCH = 2024-01-01T00:00:00
SEMI_MAJOR_AXIS = 7000.0 [km]
ECCENTRICITY = 0.001
INCLINATION = 51.6 [deg]
RA_OF_ASC_NODE = 247.5 [deg]
ARG_OF_PERICENTER = 130.5 [deg]
MEAN_ANOMALY = 325.0 [deg]
";

const OMM_JSON: &str = r#"{
"CCSDS_OMM_VERS": "2.0",
"CREATION_DATE": "2024-01-01T00:00:00",
"ORIGINATOR": "TEST",
"OBJECT_NAME": "SAT",
"OBJECT_ID": "2024-000A",
"CENTER_NAME": "EARTH",
"REF_FRAME": "TEME",
"TIME_SYSTEM": "UTC",
"MEAN_ELEMENT_THEORY": "SGP/SGP4",
"EPOCH": "2024-01-01T00:00:00",
"SEMI_MAJOR_AXIS": 7000.0,
"ECCENTRICITY": 0.001,
"INCLINATION": 51.6,
"RA_OF_ASC_NODE": 247.5,
"ARG_OF_PERICENTER": 130.5,
"MEAN_ANOMALY": 325.0
}"#;

// ---------------------------------------------------------------------------
// detect_format
// ---------------------------------------------------------------------------

#[test]
fn detect_format_kvn_xml_json() {
    assert_eq!(detect_format(OPM_KVN).unwrap(), Format::Kvn);
    assert_eq!(detect_format(OPM_XML).unwrap(), Format::Xml);
    assert_eq!(detect_format(OMM_JSON).unwrap(), Format::Json);
}

#[test]
fn detect_format_errors_on_empty() {
    assert!(matches!(
        detect_format("").unwrap_err(),
        OdmError::UndetectableFormat
    ));
}

// ---------------------------------------------------------------------------
// Top-level auto-detecting read_*
// ---------------------------------------------------------------------------

#[test]
fn read_opm_auto_detects_kvn() {
    let opm = read_opm(OPM_KVN).unwrap();
    assert_eq!(opm.metadata.object_name, "SAT");
}

#[test]
fn read_opm_auto_detects_xml() {
    let opm = read_opm(OPM_XML).unwrap();
    assert_eq!(opm.metadata.object_name, "SAT");
}

#[test]
fn read_opm_rejects_json() {
    let err = read_opm(OMM_JSON).unwrap_err();
    assert!(matches!(
        err,
        OdmError::UnsupportedFormat {
            kind: MessageKind::Opm,
            format: Format::Json
        }
    ));
}

#[test]
fn read_oem_auto_detects_kvn() {
    let oem = read_oem(OEM_KVN).unwrap();
    assert_eq!(oem.segments.len(), 1);
}

#[test]
fn read_oem_rejects_json() {
    let err = read_oem(OMM_JSON).unwrap_err();
    assert!(matches!(
        err,
        OdmError::UnsupportedFormat {
            kind: MessageKind::Oem,
            format: Format::Json
        }
    ));
}

#[test]
fn read_omm_auto_detects_kvn() {
    let omm = read_omm(OMM_KVN).unwrap();
    assert_eq!(omm.metadata.object_name, "SAT");
}

#[test]
fn read_omm_auto_detects_json() {
    let omm = read_omm(OMM_JSON).unwrap();
    assert_eq!(omm.metadata.object_name, "SAT");
}

#[test]
fn read_ci_kvn_dispatches_to_opm() {
    let ci = read_ci(OPM_KVN).unwrap();
    assert_eq!(ci.kind(), MessageKind::Opm);
}

#[test]
fn read_ci_xml_dispatches_to_opm() {
    let ci = read_ci(OPM_XML).unwrap();
    assert_eq!(ci.kind(), MessageKind::Opm);
}

#[test]
fn read_ci_json_dispatches_to_omm() {
    let ci = read_ci(OMM_JSON).unwrap();
    assert_eq!(ci.kind(), MessageKind::Omm);
}

// ---------------------------------------------------------------------------
// Top-level write_* with Format selection
// ---------------------------------------------------------------------------

#[test]
fn write_opm_each_supported_format() {
    let opm = read_opm(OPM_KVN).unwrap();
    let kvn = write_opm(&opm, Format::Kvn).unwrap();
    assert!(kvn.starts_with("CCSDS_OPM_VERS"));
    let xml = write_opm(&opm, Format::Xml).unwrap();
    assert!(xml.contains("<opm"));
    let err = write_opm(&opm, Format::Json).unwrap_err();
    assert!(matches!(err, OdmError::UnsupportedFormat { .. }));
}

#[test]
fn write_oem_each_supported_format() {
    let oem = read_oem(OEM_KVN).unwrap();
    assert!(
        write_oem(&oem, Format::Kvn)
            .unwrap()
            .starts_with("CCSDS_OEM_VERS")
    );
    assert!(write_oem(&oem, Format::Xml).unwrap().contains("<oem"));
    let err = write_oem(&oem, Format::Json).unwrap_err();
    assert!(matches!(err, OdmError::UnsupportedFormat { .. }));
}

#[test]
fn write_omm_each_supported_format() {
    let omm = read_omm(OMM_KVN).unwrap();
    assert!(
        write_omm(&omm, Format::Kvn)
            .unwrap()
            .starts_with("CCSDS_OMM_VERS")
    );
    assert!(write_omm(&omm, Format::Xml).unwrap().contains("<omm"));
    let json = write_omm(&omm, Format::Json).unwrap();
    assert!(json.contains("OBJECT_NAME"));
}

#[test]
fn write_ci_delegates_per_variant() {
    let opm_ci = read_ci(OPM_KVN).unwrap();
    assert!(
        write_ci(&opm_ci, Format::Kvn)
            .unwrap()
            .starts_with("CCSDS_OPM_VERS")
    );

    let omm_ci = read_ci(OMM_JSON).unwrap();
    let json = write_ci(&omm_ci, Format::Json).unwrap();
    assert!(json.contains("OBJECT_NAME"));

    // OPM ↛ JSON
    let err = write_ci(&opm_ci, Format::Json).unwrap_err();
    assert!(matches!(err, OdmError::UnsupportedFormat { .. }));
}

// ---------------------------------------------------------------------------
// File I/O helpers
// ---------------------------------------------------------------------------

fn tmpdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("lox-odm-test-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[test]
fn opm_file_round_trip() {
    let opm = read_opm(OPM_KVN).unwrap();
    let path = tmpdir().join("opm.kvn");
    write_opm_file(&opm, &path, Format::Kvn).unwrap();
    let opm2 = read_opm_file(&path).unwrap();
    assert_eq!(opm.metadata.object_name, opm2.metadata.object_name);

    let xml_path = tmpdir().join("opm.xml");
    write_opm_file(&opm, &xml_path, Format::Xml).unwrap();
    let opm3 = read_opm_file(&xml_path).unwrap();
    assert_eq!(opm.metadata.object_id, opm3.metadata.object_id);
}

#[test]
fn oem_file_round_trip() {
    let oem = read_oem(OEM_KVN).unwrap();
    let path = tmpdir().join("oem.kvn");
    write_oem_file(&oem, &path, Format::Kvn).unwrap();
    let oem2 = read_oem_file(&path).unwrap();
    assert_eq!(oem.segments.len(), oem2.segments.len());
}

#[test]
fn omm_file_round_trip_all_three_formats() {
    let omm = read_omm(OMM_KVN).unwrap();
    for (fmt, ext) in [
        (Format::Kvn, "kvn"),
        (Format::Xml, "xml"),
        (Format::Json, "json"),
    ] {
        let path = tmpdir().join(format!("omm.{ext}"));
        write_omm_file(&omm, &path, fmt).unwrap();
        let omm2 = read_omm_file(&path).unwrap();
        assert_eq!(omm.metadata.object_name, omm2.metadata.object_name);
    }
}

#[test]
fn ci_file_round_trip() {
    let ci = read_ci(OPM_KVN).unwrap();
    let path = tmpdir().join("ci.kvn");
    write_ci_file(&ci, &path, Format::Kvn).unwrap();
    let ci2 = read_ci_file(&path).unwrap();
    assert_eq!(ci.kind(), ci2.kind());
    assert!(matches!(ci2, OdmCi::Opm(_)));
}

#[test]
fn read_opm_file_errors_on_missing_path() {
    let err = read_opm_file("/nonexistent/path.kvn").unwrap_err();
    assert!(matches!(err, OdmError::Io(_)));
}

#[test]
fn read_oem_file_errors_on_missing_path() {
    let err = read_oem_file("/nonexistent/path.kvn").unwrap_err();
    assert!(matches!(err, OdmError::Io(_)));
}

#[test]
fn read_omm_file_errors_on_missing_path() {
    let err = read_omm_file("/nonexistent/path.kvn").unwrap_err();
    assert!(matches!(err, OdmError::Io(_)));
}

#[test]
fn read_ci_file_errors_on_missing_path() {
    let err = read_ci_file("/nonexistent/path.kvn").unwrap_err();
    assert!(matches!(err, OdmError::Io(_)));
}
