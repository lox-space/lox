// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Orbit Mean Elements Message (OMM) typed data model.
//!
//! Mirrors CCSDS 502.0-B-3 §4 — mean Keplerian elements at an epoch,
//! tuned for a particular propagation theory (typically SGP4). OMM is
//! the CCSDS-blessed XML/KVN/JSON envelope around TLE data.

use std::collections::BTreeMap;

use lox_core::elements::MeanElements;
use lox_core::units::AreaToMass;
use lox_time::time::DynTime;

use crate::types::common::{Covariance, OdmCenter, OdmFrame, OdmHeader, SpacecraftParameters};

/// Per-message metadata for the OMM (CCSDS 502.0-B-3 §4.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OmmMetadata {
    /// `OBJECT_NAME` — human-readable spacecraft name.
    pub object_name: String,
    /// `OBJECT_ID` — international designator.
    pub object_id: String,
    /// `CENTER_NAME` — center body of the reference frame.
    pub center: OdmCenter,
    /// `REF_FRAME` — reference frame of the mean elements.
    pub frame: OdmFrame,
    /// `REF_FRAME_EPOCH` — optional rotating-frame realisation epoch.
    pub frame_epoch: Option<DynTime>,
    /// `MEAN_ELEMENT_THEORY` — propagation theory the elements are
    /// tuned for. CCSDS does not enumerate values; preserved verbatim
    /// (e.g. `"SGP/SGP4"`, `"DSST"`, `"USM"`).
    pub mean_element_theory: String,
}

/// Optional SGP4-specific element-set parameters (CCSDS 502.0-B-3 §4.4,
/// "TLE-related parameters").
///
/// All fields are optional per CCSDS; they're present when the OMM is
/// tuned for SGP/SGP4 propagation. Dimensionless and unitless fields
/// (`bstar`, `mean_motion_dot`, etc.) stay as `f64`; `BTERM` and `AGOM`
/// use the typed `AreaToMass` newtype.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TleParameters {
    /// `EPHEMERIS_TYPE` — SGP4 model variant code (typically `0`).
    pub ephemeris_type: Option<i32>,
    /// `CLASSIFICATION_TYPE` — single-character classification marker
    /// (e.g. `"U"` for unclassified).
    pub classification_type: Option<String>,
    /// `NORAD_CAT_ID` — NORAD catalog number.
    pub norad_cat_id: Option<i32>,
    /// `ELEMENT_SET_NO` — element-set number assigned by the originator.
    pub element_set_no: Option<i64>,
    /// `REV_AT_EPOCH` — revolution number at the elements' epoch.
    pub rev_at_epoch: Option<u64>,
    /// `BSTAR` — SGP4 ballistic coefficient (Earth-radii⁻¹, unitless in
    /// SGP4's internal model). Stays as `f64` because it's not a
    /// canonical SI quantity.
    pub bstar: Option<f64>,
    /// `BTERM` — ballistic coefficient `Cd · A / m` (m²/kg).
    pub bterm: Option<AreaToMass>,
    /// `MEAN_MOTION_DOT` — first derivative of mean motion (rev/day²).
    /// Stays as `f64`; CCSDS does not require a typed unit here.
    pub mean_motion_dot: Option<f64>,
    /// `MEAN_MOTION_DDOT` — second derivative of mean motion (rev/day³).
    pub mean_motion_ddot: Option<f64>,
    /// `AGOM` — radiation-pressure coefficient `Cr · A / m` (m²/kg).
    pub agom: Option<AreaToMass>,
}

/// The Orbit Mean Elements Message (OMM, CCSDS 502.0-B-3 §4).
///
/// Mean Keplerian elements at an epoch, tuned for a particular
/// propagation theory (typically SGP4 — see
/// [`OmmMetadata::mean_element_theory`]). SGP4 callers will find the
/// [`TleParameters`] block populated with the
/// classification/NORAD-id/BSTAR/etc. metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct Omm {
    /// Common ODM header.
    pub header: OdmHeader,
    /// OMM-specific metadata (object id, center, frame, theory).
    pub metadata: OmmMetadata,
    /// Mean-elements epoch.
    pub epoch: DynTime,
    /// Mean Keplerian elements.
    ///
    /// On the wire, the size element may appear as either
    /// `SEMI_MAJOR_AXIS` (km) or `MEAN_MOTION` (rev/day) per
    /// CCSDS §4.4; the parser normalises to semi-major axis using
    /// the center body's `GM` (or the optional wire `GM` if present).
    /// The choice of wire form is preserved at the AST layer, not here.
    pub mean_elements: MeanElements,
    /// Optional SGP4-specific element-set parameters.
    pub tle_parameters: Option<TleParameters>,
    /// Optional spacecraft physical properties.
    pub spacecraft: Option<SpacecraftParameters>,
    /// Optional 6×6 state covariance.
    pub covariance: Option<Covariance>,
    /// User-defined parameters (preserved verbatim for round-trip).
    pub user_defined: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_bodies::DynOrigin;
    use lox_frames::DynFrame;
    use lox_time::time::Time;
    use lox_time::time_scales::DynTimeScale;

    #[test]
    fn omm_metadata_construction() {
        let m = OmmMetadata {
            object_name: "ISS".to_string(),
            object_id: "1998-067A".to_string(),
            center: OdmCenter::Known(DynOrigin::Earth),
            frame: OdmFrame::Known(DynFrame::Teme),
            frame_epoch: None,
            mean_element_theory: "SGP/SGP4".to_string(),
        };
        assert_eq!(m.object_name, "ISS");
        assert_eq!(m.frame, OdmFrame::Known(DynFrame::Teme));
        assert_eq!(m.mean_element_theory, "SGP/SGP4");
    }

    #[test]
    fn tle_parameters_default_all_none() {
        let p = TleParameters::default();
        assert!(p.ephemeris_type.is_none());
        assert!(p.norad_cat_id.is_none());
        assert!(p.bstar.is_none());
        assert!(p.bterm.is_none());
    }

    #[test]
    fn tle_parameters_typical_sgp4() {
        let p = TleParameters {
            ephemeris_type: Some(0),
            classification_type: Some("U".to_string()),
            norad_cat_id: Some(45018),
            element_set_no: Some(999),
            rev_at_epoch: Some(5327),
            bstar: Some(8.4553e-5),
            bterm: None,
            mean_motion_dot: Some(2.241e-5),
            mean_motion_ddot: Some(0.0),
            agom: None,
        };
        assert_eq!(p.norad_cat_id, Some(45018));
        assert_eq!(p.classification_type.as_deref(), Some("U"));
        assert_eq!(p.bstar, Some(8.4553e-5));
    }

    fn sample_epoch() -> DynTime {
        Time::j2000(DynTimeScale::Tai)
    }

    fn sample_header() -> OdmHeader {
        OdmHeader {
            comments: Vec::new(),
            classification: None,
            creation_date: sample_epoch(),
            originator: "TEST".to_string(),
            message_id: None,
        }
    }

    fn sample_metadata() -> OmmMetadata {
        OmmMetadata {
            object_name: "TEST-SAT".to_string(),
            object_id: "2024-000A".to_string(),
            center: OdmCenter::Known(DynOrigin::Earth),
            frame: OdmFrame::Known(DynFrame::Teme),
            frame_epoch: None,
            mean_element_theory: "SGP/SGP4".to_string(),
        }
    }

    fn sample_mean_elements() -> MeanElements {
        MeanElements {
            a: 6_859_961.0, // m (~ 482 km altitude)
            e: 0.001_335_6,
            i: 1.697_775,    // rad (~ 97.297 deg)
            raan: 1.159_523, // rad
            aop: 1.931_018,  // rad
            m: 5.842_034,    // rad
        }
    }

    #[test]
    fn omm_construction_minimal() {
        let omm = Omm {
            header: sample_header(),
            metadata: sample_metadata(),
            epoch: sample_epoch(),
            mean_elements: sample_mean_elements(),
            tle_parameters: None,
            spacecraft: None,
            covariance: None,
            user_defined: BTreeMap::new(),
        };
        assert_eq!(omm.metadata.mean_element_theory, "SGP/SGP4");
        assert!(omm.tle_parameters.is_none());
        assert!(omm.user_defined.is_empty());
    }

    #[test]
    fn omm_construction_with_tle_params() {
        let omm = Omm {
            header: sample_header(),
            metadata: sample_metadata(),
            epoch: sample_epoch(),
            mean_elements: sample_mean_elements(),
            tle_parameters: Some(TleParameters {
                norad_cat_id: Some(45018),
                ..TleParameters::default()
            }),
            spacecraft: None,
            covariance: None,
            user_defined: BTreeMap::new(),
        };
        assert_eq!(omm.tle_parameters.and_then(|p| p.norad_cat_id), Some(45018));
    }
}
