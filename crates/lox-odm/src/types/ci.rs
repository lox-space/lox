// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Combined-Instantiation wrapper for the ODM family.
//!
//! `OdmCi` carries one of the implemented ODM message types — OPM, OEM,
//! or OMM. CCSDS NDM-CI (Combined Instantiation) allows a single
//! file/document to bundle multiple messages of any NDM family; for the
//! ODM-only scope of this crate, that means a mix of OPM/OEM/OMM.
//!
//! OCM is intentionally not included — see the spec's "Out of scope"
//! section. When a typed `Ocm` lands, an `Ocm(Ocm)` variant is an
//! additive enum extension.

use std::fmt::{self, Display, Formatter};

use crate::types::common::MessageKind;
use crate::types::oem::Oem;
use crate::types::omm::Omm;
use crate::types::opm::Opm;

/// One ODM message of any implemented type.
///
/// Variants mirror the three implemented message types. The wire-format
/// readers (Phase 2b/2c) produce values of this type when parsing
/// NDM-CI envelopes that bundle multiple ODM messages.
#[derive(Clone, Debug, PartialEq)]
pub enum OdmCi {
    /// Orbit Parameter Message.
    Opm(Opm),
    /// Orbit Ephemeris Message.
    Oem(Oem),
    /// Orbit Mean Elements Message.
    Omm(Omm),
}

impl OdmCi {
    /// Returns the [`MessageKind`] discriminator for this variant.
    pub fn kind(&self) -> MessageKind {
        match self {
            OdmCi::Opm(_) => MessageKind::Opm,
            OdmCi::Oem(_) => MessageKind::Oem,
            OdmCi::Omm(_) => MessageKind::Omm,
        }
    }
}

impl Display for OdmCi {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.kind(), f)
    }
}

impl From<Opm> for OdmCi {
    fn from(opm: Opm) -> Self {
        OdmCi::Opm(opm)
    }
}

impl From<Oem> for OdmCi {
    fn from(oem: Oem) -> Self {
        OdmCi::Oem(oem)
    }
}

impl From<Omm> for OdmCi {
    fn from(omm: Omm) -> Self {
        OdmCi::Omm(omm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{OdmCenter, OdmFrame, OdmHeader, OdmTime};
    use crate::types::omm::{OmmMeanElements, OmmMetadata};
    use lox_bodies::DynOrigin;
    use lox_frames::DynFrame;
    use lox_time::time::Time;
    use lox_time::time_scales::DynTimeScale;
    use std::collections::BTreeMap;

    fn sample_epoch() -> OdmTime {
        OdmTime::Time(Time::j2000(DynTimeScale::Tai))
    }

    fn sample_omm() -> Omm {
        Omm {
            header: OdmHeader {
                comments: Vec::new(),
                classification: None,
                creation_date: sample_epoch(),
                originator: "TEST".to_string(),
                message_id: None,
            },
            metadata: OmmMetadata {
                comments: Vec::new(),
                object_name: "TEST-SAT".to_string(),
                object_id: "2024-000A".to_string(),
                center: OdmCenter::Known(DynOrigin::Earth),
                frame: OdmFrame::Known(DynFrame::Teme),
                frame_epoch: None,
                mean_element_theory: "SGP/SGP4".to_string(),
            },
            epoch: sample_epoch(),
            mean_elements: OmmMeanElements::default(),
            tle_parameters: None,
            spacecraft: None,
            covariance: None,
            user_defined: BTreeMap::new(),
        }
    }

    #[test]
    fn kind_returns_correct_discriminator() {
        let ci = OdmCi::Omm(sample_omm());
        assert_eq!(ci.kind(), MessageKind::Omm);
    }

    #[test]
    fn display_emits_kind_name() {
        let ci = OdmCi::Omm(sample_omm());
        assert_eq!(format!("{ci}"), "OMM");
    }

    #[test]
    fn from_omm_lifts_to_variant() {
        let omm = sample_omm();
        let ci: OdmCi = omm.into();
        assert_eq!(ci.kind(), MessageKind::Omm);
    }
}
