// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! XML ↔ typed [`OdmCi`] dispatch.
//!
//! Sniffs the root element name (`<opm>`, `<oem>`, `<omm>`) and
//! delegates to the per-kind reader. `<ndm>` (the multi-message
//! NDM-CI wrapper) is not currently supported and returns
//! [`XmlError::UnsupportedCiRoot`].

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::types::ci::OdmCi;
use crate::xml::error::XmlError;
use crate::xml::{oem, omm, opm};

/// Inspect the first non-prolog element to identify the message kind.
fn sniff_root_element(input: &str) -> Result<String, XmlError> {
    let mut reader = Reader::from_str(input);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) => {
                let name = e.name();
                let name_str = std::str::from_utf8(name.as_ref())
                    .map_err(|e| XmlError::InvalidValue {
                        keyword: "root_element".to_string(),
                        reason: e.to_string(),
                    })?
                    .to_string();
                return Ok(name_str);
            }
            Ok(Event::Eof) => {
                return Err(XmlError::MissingRequiredField("root element".to_string()));
            }
            Ok(_) => continue,
            Err(e) => {
                return Err(XmlError::InvalidValue {
                    keyword: "root_element".to_string(),
                    reason: e.to_string(),
                });
            }
        }
    }
}

/// Parse an XML ODM message of any kind, dispatching on the root
/// element name.
pub fn read_ci(input: &str) -> Result<OdmCi, XmlError> {
    let root = sniff_root_element(input)?;
    match root.as_str() {
        "opm" => opm::read_opm(input).map(OdmCi::Opm),
        "oem" => oem::read_oem(input).map(OdmCi::Oem),
        "omm" => omm::read_omm(input).map(OdmCi::Omm),
        other => Err(XmlError::UnsupportedCiRoot(other.to_string())),
    }
}

/// Serialise an [`OdmCi`] to XML, delegating to the per-kind writer.
pub fn write_ci(ci: &OdmCi) -> String {
    match ci {
        OdmCi::Opm(o) => opm::write_opm(o),
        OdmCi::Oem(o) => oem::write_oem(o),
        OdmCi::Omm(o) => omm::write_omm(o),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::MessageKind;

    const MINIMAL_OPM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<opm id="CCSDS_OPM_VERS" version="3.0">
    <header>
        <CREATION_DATE>2024-01-01T00:00:00</CREATION_DATE>
        <ORIGINATOR>TEST</ORIGINATOR>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>SAT</OBJECT_NAME>
                <OBJECT_ID>2024-000A</OBJECT_ID>
                <CENTER_NAME>EARTH</CENTER_NAME>
                <REF_FRAME>ICRF</REF_FRAME>
                <TIME_SYSTEM>TAI</TIME_SYSTEM>
            </metadata>
            <data>
                <stateVector>
                    <EPOCH>2024-01-01T00:00:00</EPOCH>
                    <X units="km">7000.0</X>
                    <Y units="km">0.0</Y>
                    <Z units="km">0.0</Z>
                    <X_DOT units="km/s">0.0</X_DOT>
                    <Y_DOT units="km/s">7.5</Y_DOT>
                    <Z_DOT units="km/s">0.0</Z_DOT>
                </stateVector>
            </data>
        </segment>
    </body>
</opm>"#;

    const MINIMAL_OEM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<oem id="CCSDS_OEM_VERS" version="3.0">
    <header>
        <CREATION_DATE>2024-01-01T00:00:00</CREATION_DATE>
        <ORIGINATOR>TEST</ORIGINATOR>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>SAT</OBJECT_NAME>
                <OBJECT_ID>2024-000A</OBJECT_ID>
                <CENTER_NAME>EARTH</CENTER_NAME>
                <REF_FRAME>ICRF</REF_FRAME>
                <TIME_SYSTEM>TAI</TIME_SYSTEM>
                <START_TIME>2024-01-01T00:00:00</START_TIME>
                <STOP_TIME>2024-01-01T00:01:00</STOP_TIME>
            </metadata>
            <data>
                <stateVector>
                    <EPOCH>2024-01-01T00:00:00</EPOCH>
                    <X units="km">7000.0</X>
                    <Y units="km">0.0</Y>
                    <Z units="km">0.0</Z>
                    <X_DOT units="km/s">0.0</X_DOT>
                    <Y_DOT units="km/s">7.5</Y_DOT>
                    <Z_DOT units="km/s">0.0</Z_DOT>
                </stateVector>
                <stateVector>
                    <EPOCH>2024-01-01T00:01:00</EPOCH>
                    <X units="km">7001.0</X>
                    <Y units="km">0.0</Y>
                    <Z units="km">0.0</Z>
                    <X_DOT units="km/s">0.0</X_DOT>
                    <Y_DOT units="km/s">7.5</Y_DOT>
                    <Z_DOT units="km/s">0.0</Z_DOT>
                </stateVector>
            </data>
        </segment>
    </body>
</oem>"#;

    const MINIMAL_OMM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<omm id="CCSDS_OMM_VERS" version="3.0">
    <header>
        <CREATION_DATE>2024-01-01T00:00:00</CREATION_DATE>
        <ORIGINATOR>TEST</ORIGINATOR>
    </header>
    <body>
        <segment>
            <metadata>
                <OBJECT_NAME>SAT</OBJECT_NAME>
                <OBJECT_ID>2024-000A</OBJECT_ID>
                <CENTER_NAME>EARTH</CENTER_NAME>
                <REF_FRAME>TEME</REF_FRAME>
                <TIME_SYSTEM>UTC</TIME_SYSTEM>
                <MEAN_ELEMENT_THEORY>SGP/SGP4</MEAN_ELEMENT_THEORY>
            </metadata>
            <data>
                <meanElements>
                    <EPOCH>2024-01-01T00:00:00</EPOCH>
                    <SEMI_MAJOR_AXIS units="km">7000.0</SEMI_MAJOR_AXIS>
                    <ECCENTRICITY>0.001</ECCENTRICITY>
                    <INCLINATION units="deg">51.6</INCLINATION>
                    <RA_OF_ASC_NODE units="deg">247.5</RA_OF_ASC_NODE>
                    <ARG_OF_PERICENTER units="deg">130.5</ARG_OF_PERICENTER>
                    <MEAN_ANOMALY units="deg">325.0</MEAN_ANOMALY>
                </meanElements>
            </data>
        </segment>
    </body>
</omm>"#;

    #[test]
    fn read_ci_dispatches_to_opm() {
        let ci = read_ci(MINIMAL_OPM).expect("read");
        assert!(matches!(ci, OdmCi::Opm(_)));
        assert_eq!(ci.kind(), MessageKind::Opm);
    }

    #[test]
    fn read_ci_dispatches_to_oem() {
        let ci = read_ci(MINIMAL_OEM).expect("read");
        assert!(matches!(ci, OdmCi::Oem(_)));
        assert_eq!(ci.kind(), MessageKind::Oem);
    }

    #[test]
    fn read_ci_dispatches_to_omm() {
        let ci = read_ci(MINIMAL_OMM).expect("read");
        assert!(matches!(ci, OdmCi::Omm(_)));
        assert_eq!(ci.kind(), MessageKind::Omm);
    }

    #[test]
    fn write_ci_emits_opm_root() {
        let ci = read_ci(MINIMAL_OPM).expect("read");
        let written = write_ci(&ci);
        assert!(written.contains("<opm"));
    }

    #[test]
    fn write_ci_emits_oem_root() {
        let ci = read_ci(MINIMAL_OEM).expect("read");
        let written = write_ci(&ci);
        assert!(written.contains("<oem"));
    }

    #[test]
    fn write_ci_emits_omm_root() {
        let ci = read_ci(MINIMAL_OMM).expect("read");
        let written = write_ci(&ci);
        assert!(written.contains("<omm"));
    }

    #[test]
    fn round_trip_opm_via_ci() {
        let ci = read_ci(MINIMAL_OPM).expect("read");
        let written = write_ci(&ci);
        let ci2 = read_ci(&written).expect("re-read");
        assert_eq!(ci.kind(), ci2.kind());
    }

    #[test]
    fn round_trip_oem_via_ci() {
        let ci = read_ci(MINIMAL_OEM).expect("read");
        let written = write_ci(&ci);
        let ci2 = read_ci(&written).expect("re-read");
        assert_eq!(ci.kind(), ci2.kind());
    }

    #[test]
    fn round_trip_omm_via_ci() {
        let ci = read_ci(MINIMAL_OMM).expect("read");
        let written = write_ci(&ci);
        let ci2 = read_ci(&written).expect("re-read");
        assert_eq!(ci.kind(), ci2.kind());
    }

    #[test]
    fn read_ci_rejects_ndm_wrapper() {
        let input = r#"<?xml version="1.0"?><ndm><opm/></ndm>"#;
        let err = read_ci(input).expect_err("NDM wrapper not supported");
        assert!(matches!(err, XmlError::UnsupportedCiRoot(ref s) if s == "ndm"));
    }
}
