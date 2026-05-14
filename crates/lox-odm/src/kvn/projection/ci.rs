// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! KVN ↔ typed [`OdmCi`] dispatch.
//!
//! [`OdmCi`] is the message-kind-agnostic wrapper around OPM, OEM, and
//! OMM. [`read_ci`] inspects the version header to determine which kind
//! a KVN input represents and dispatches to the per-kind reader.
//! [`write_ci`] pattern-matches the variant and delegates to the per-kind
//! writer.

use crate::kvn::ast::KvnDocument;
use crate::kvn::error::{KvnError, KvnErrorKind, Span};
use crate::kvn::parser::parse;
use crate::kvn::projection::{oem, omm, opm};
use crate::types::ci::OdmCi;
use crate::types::common::MessageKind;
use crate::types::oem::Oem;
use crate::types::omm::Omm;
use crate::types::opm::Opm;

/// Parse a KVN-encoded ODM message of any kind, returning the
/// appropriate [`OdmCi`] variant.
///
/// The wire-format kind token (`CCSDS_OPM_VERS`, `CCSDS_OEM_VERS`,
/// `CCSDS_OMM_VERS`) determines the variant. Inputs whose first line is
/// `CCSDS_NDM_VERS = ...` (the multi-message NDM-CI wrapper) are not
/// currently supported and return `UnknownMessageKind("NDM")`.
pub fn read_ci(input: &str) -> Result<OdmCi, KvnError> {
    let doc = parse(input)?;
    OdmCi::try_from(doc)
}

/// Emit the KVN-encoded form of an [`OdmCi`], delegating to the
/// per-kind writer.
pub fn write_ci(ci: &OdmCi) -> String {
    match ci {
        OdmCi::Opm(o) => opm::write_opm(o),
        OdmCi::Oem(o) => oem::write_oem(o),
        OdmCi::Omm(o) => omm::write_omm(o),
    }
}

impl TryFrom<KvnDocument> for OdmCi {
    type Error = KvnError;

    fn try_from(doc: KvnDocument) -> Result<Self, Self::Error> {
        match doc.message_kind {
            MessageKind::Opm => Opm::try_from(doc).map(OdmCi::Opm),
            MessageKind::Oem => Oem::try_from(doc).map(OdmCi::Oem),
            MessageKind::Omm => Omm::try_from(doc).map(OdmCi::Omm),
            MessageKind::Ocm => Err(KvnError {
                span: Span::default(),
                kind: KvnErrorKind::UnknownMessageKind("OCM".to_string()),
            }),
            MessageKind::Ci => Err(KvnError {
                span: Span::default(),
                kind: KvnErrorKind::UnknownMessageKind("NDM".to_string()),
            }),
        }
    }
}

impl From<&OdmCi> for KvnDocument {
    fn from(ci: &OdmCi) -> Self {
        match ci {
            OdmCi::Opm(o) => o.into(),
            OdmCi::Oem(o) => o.into(),
            OdmCi::Omm(o) => o.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_OPM: &str = "\
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

    const MINIMAL_OEM: &str = "\
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

    const MINIMAL_OMM: &str = "\
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
    fn write_ci_emits_opm() {
        let ci = read_ci(MINIMAL_OPM).expect("read");
        let written = write_ci(&ci);
        assert!(written.starts_with("CCSDS_OPM_VERS = 3.0"));
    }

    #[test]
    fn write_ci_emits_oem() {
        let ci = read_ci(MINIMAL_OEM).expect("read");
        let written = write_ci(&ci);
        assert!(written.starts_with("CCSDS_OEM_VERS = 3.0"));
    }

    #[test]
    fn write_ci_emits_omm() {
        let ci = read_ci(MINIMAL_OMM).expect("read");
        let written = write_ci(&ci);
        assert!(written.starts_with("CCSDS_OMM_VERS = 3.0"));
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
        let input =
            "CCSDS_NDM_VERS = 1.0\nCREATION_DATE = 2024-01-01T00:00:00\nORIGINATOR = TEST\n";
        let err = read_ci(input).expect_err("NDM wrapper not supported");
        assert!(matches!(err.kind, KvnErrorKind::UnknownMessageKind(ref s) if s == "NDM"));
    }
}
