// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Orbit Mean Elements Message (OMM) typed data model.
//!
//! Mirrors CCSDS 502.0-B-3 §4 — mean Keplerian elements at an epoch,
//! tuned for a particular propagation theory (typically SGP4). OMM is
//! the CCSDS-blessed XML/KVN/JSON envelope around TLE data.

use std::collections::BTreeMap;

use crate::types::common::{
    Covariance, OdmCenter, OdmFrame, OdmHeader, OdmTime, SpacecraftParameters,
};
use lox_bodies::TryPointMass;
use lox_core::elements::{GravitationalParameter, MeanElements};
use lox_core::units::AreaToMass;

/// Per-message metadata for the OMM (CCSDS 502.0-B-3 §4.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OmmMetadata {
    /// `COMMENT` lines for the metadata block, in document order.
    pub comments: Vec<String>,
    /// `OBJECT_NAME` — human-readable spacecraft name.
    pub object_name: String,
    /// `OBJECT_ID` — international designator.
    pub object_id: String,
    /// `CENTER_NAME` — center body of the reference frame.
    pub center: OdmCenter,
    /// `REF_FRAME` — reference frame of the mean elements.
    pub frame: OdmFrame,
    /// `REF_FRAME_EPOCH` — optional rotating-frame realisation epoch.
    pub frame_epoch: Option<OdmTime>,
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
    /// `COMMENT` lines for the TLE-parameters block, in document order.
    pub comments: Vec<String>,
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

/// OMM mean-elements block (CCSDS 502.0-B-3 §4.4).
///
/// Wraps the pure-physics [`MeanElements`] with wire-specific
/// decorations: an optional `GM` and any `COMMENT` lines that precede
/// the block on the wire. Mirrors [`crate::types::opm::OpmKeplerian`].
///
/// The `gm` field captures the operator's wire `GM` value exactly.
/// Preserved regardless of whether the center is `Known` or `Custom`.
/// When `None`, the wire did not include `GM`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OmmMeanElements {
    /// `COMMENT` lines for this sub-block, in document order.
    pub comments: Vec<String>,
    /// Mean Keplerian elements (semi-major axis in meters, angles
    /// in radians).
    pub elements: MeanElements,
    /// `GM [km**3/s**2]` from the wire, stored in canonical m³/s².
    /// `None` when the wire did not specify GM.
    pub gm: Option<GravitationalParameter>,
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
    pub epoch: OdmTime,
    /// Mean Keplerian elements plus wire-specific decorations (`GM`,
    /// COMMENTs).
    ///
    /// On the wire, the size element may appear as either
    /// `SEMI_MAJOR_AXIS` (km) or `MEAN_MOTION` (rev/day) per
    /// CCSDS §4.4; the parser normalises to semi-major axis using
    /// the wire `GM` (preferred) or the center body's canonical GM.
    /// The choice of wire form is preserved at the AST layer, not here.
    pub mean_elements: OmmMeanElements,
    /// Optional SGP4-specific element-set parameters.
    pub tle_parameters: Option<TleParameters>,
    /// Optional spacecraft physical properties.
    pub spacecraft: Option<SpacecraftParameters>,
    /// Optional 6×6 state covariance.
    pub covariance: Option<Covariance>,
    /// User-defined parameters (preserved verbatim for round-trip).
    pub user_defined: BTreeMap<String, String>,
    /// Non-CCSDS extras carried by Space-Track / Celestrak JSON OMMs
    /// (`TLE_LINE0`/`1`/`2`, `OBJECT_TYPE`, `RCS_SIZE`, `COUNTRY_CODE`,
    /// `LAUNCH_DATE`, `SITE`, `DECAY_DATE`, `FILE`, `GP_ID`, `PERIOD`,
    /// `APOAPSIS`, `PERIAPSIS`, `SEMIMAJOR_AXIS`, …).
    ///
    /// Populated only by the JSON reader; the KVN and XML wire formats
    /// have no equivalent extension mechanism and leave this empty. The
    /// JSON writer re-emits these keys verbatim, preserving round-trip
    /// for operator-supplied data.
    pub provider_extras: BTreeMap<String, serde_json::Value>,
}

impl Omm {
    /// Returns a gravitational parameter for this OMM, preferring the
    /// operator's wire `GM` (stored on [`OmmMeanElements::gm`]) and
    /// falling back to the canonical body GM via
    /// [`TryPointMass::try_gravitational_parameter`].
    ///
    /// Returns `None` only when both are unavailable — i.e. the wire
    /// did not include `GM` *and* the center is [`OdmCenter::Custom`]
    /// or the body has no defined gravitational parameter.
    pub fn gm(&self) -> Option<GravitationalParameter> {
        self.mean_elements.gm.or_else(|| {
            self.metadata
                .center
                .known()
                .and_then(|o| o.try_gravitational_parameter().ok())
        })
    }
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
            comments: Vec::new(),
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
            comments: Vec::new(),
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

    fn sample_epoch() -> OdmTime {
        OdmTime::Time(Time::j2000(DynTimeScale::Tai))
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
            comments: Vec::new(),
            object_name: "TEST-SAT".to_string(),
            object_id: "2024-000A".to_string(),
            center: OdmCenter::Known(DynOrigin::Earth),
            frame: OdmFrame::Known(DynFrame::Teme),
            frame_epoch: None,
            mean_element_theory: "SGP/SGP4".to_string(),
        }
    }

    fn sample_mean_elements() -> OmmMeanElements {
        OmmMeanElements {
            comments: Vec::new(),
            elements: MeanElements {
                a: 6_859_961.0, // m (~ 482 km altitude)
                e: 0.001_335_6,
                i: 1.697_775,    // rad (~ 97.297 deg)
                raan: 1.159_523, // rad
                aop: 1.931_018,  // rad
                m: 5.842_034,    // rad
            },
            gm: None,
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
            provider_extras: BTreeMap::new(),
        };
        assert_eq!(omm.metadata.mean_element_theory, "SGP/SGP4");
        assert!(omm.tle_parameters.is_none());
        assert!(omm.user_defined.is_empty());
    }

    fn sample_omm() -> Omm {
        Omm {
            header: sample_header(),
            metadata: sample_metadata(),
            epoch: sample_epoch(),
            mean_elements: sample_mean_elements(),
            tle_parameters: None,
            spacecraft: None,
            covariance: None,
            user_defined: BTreeMap::new(),
            provider_extras: BTreeMap::new(),
        }
    }

    #[test]
    fn omm_gm_prefers_wire_value() {
        let mut omm = sample_omm();
        let wire_gm = GravitationalParameter::km3_per_s2(398600.4415);
        omm.mean_elements.gm = Some(wire_gm);
        assert_eq!(omm.gm(), Some(wire_gm));
    }

    #[test]
    fn omm_gm_falls_back_to_canonical_for_known_center() {
        let omm = sample_omm();
        let expected = DynOrigin::Earth.try_gravitational_parameter().ok();
        assert_eq!(omm.gm(), expected);
    }

    #[test]
    fn omm_gm_returns_none_for_custom_center_without_wire_gm() {
        let mut omm = sample_omm();
        omm.metadata.center = OdmCenter::Custom("APOPHIS".to_string());
        assert_eq!(omm.gm(), None);
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
            provider_extras: BTreeMap::new(),
        };
        assert_eq!(omm.tle_parameters.and_then(|p| p.norad_cat_id), Some(45018));
    }
}
