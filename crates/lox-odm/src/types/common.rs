// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Types shared across all CCSDS Orbit Data Messages.
//!
//! - [`MessageKind`] — discriminator enum for the five message variants
//!   (OPM, OEM, OMM, OCM, ODM-CI).
//! - [`OdmCenter`] / [`OdmFrame`] — wrappers around [`lox_bodies::DynOrigin`]
//!   / [`lox_frames::DynFrame`] that admit free-form names appearing in
//!   CCSDS messages.
//! - [`OdmHeader`] — common header carried by every ODM message.

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use lox_bodies::{DynOrigin, Origin};
use lox_core::units::{Area, Mass};
use lox_frames::{DynFrame, traits::ReferenceFrame};
use lox_time::time::DynTime;
use nalgebra::Matrix6;

/// Discriminator for the five ODM message variants.
///
/// Mirrors the `CCSDS_<KIND>_VERS` keyword on the wire:
/// `CCSDS_OPM_VERS`, `CCSDS_OEM_VERS`, `CCSDS_OMM_VERS`, `CCSDS_OCM_VERS`,
/// and the combined-instantiation `CCSDS_NDM_VERS`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MessageKind {
    /// Orbit Parameter Message.
    Opm,
    /// Orbit Ephemeris Message.
    Oem,
    /// Orbit Mean Elements Message.
    Omm,
    /// Orbit Comprehensive Message.
    Ocm,
    /// ODM Combined Instantiation — a wrapper containing one or more
    /// of the four message types above.
    Ci,
}

impl Display for MessageKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            MessageKind::Opm => "OPM",
            MessageKind::Oem => "OEM",
            MessageKind::Omm => "OMM",
            MessageKind::Ocm => "OCM",
            MessageKind::Ci => "ODM-CI",
        };
        f.write_str(s)
    }
}

/// Center body as it appears in an ODM `CENTER_NAME` field.
///
/// Either a body lox knows about (Earth, Moon, …) or a free-form name
/// (asteroids, custom barycenters, mission-specific identifiers) that
/// is preserved verbatim for lossless round-trip.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OdmCenter {
    /// A body recognised by [`DynOrigin`].
    Known(DynOrigin),
    /// A free-form name not recognised by [`DynOrigin`].
    /// Preserved verbatim on write.
    Custom(String),
}

impl OdmCenter {
    /// Parses a wire-format `CENTER_NAME` string.
    ///
    /// Tries [`DynOrigin::from_str`] first (which accepts both the
    /// canonical capitalised form and the all-lowercase form). On
    /// failure, wraps the original input as [`OdmCenter::Custom`].
    pub fn from_wire(s: &str) -> Self {
        match DynOrigin::from_str(s) {
            Ok(origin) => OdmCenter::Known(origin),
            Err(_) => OdmCenter::Custom(s.to_string()),
        }
    }

    /// The wire-equivalent name for this center.
    ///
    /// For [`OdmCenter::Known`] this is the canonical body name from
    /// [`Origin::name`]; for [`OdmCenter::Custom`] it is the stored
    /// string.
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            OdmCenter::Known(o) => Cow::Borrowed(o.name()),
            OdmCenter::Custom(s) => Cow::Borrowed(s.as_str()),
        }
    }

    /// Returns the underlying [`DynOrigin`] if this is a known body,
    /// or [`None`] for [`OdmCenter::Custom`].
    pub fn known(&self) -> Option<DynOrigin> {
        match self {
            OdmCenter::Known(o) => Some(*o),
            OdmCenter::Custom(_) => None,
        }
    }
}

impl Display for OdmCenter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

impl From<DynOrigin> for OdmCenter {
    fn from(origin: DynOrigin) -> Self {
        OdmCenter::Known(origin)
    }
}

/// Reference frame as it appears in an ODM `REF_FRAME` field.
///
/// Either a frame lox knows about (ICRF, EME2000, TEME, …) or a free-form
/// name (operator-specific frames, mission-specific identifiers). Custom
/// names are preserved verbatim for lossless round-trip.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OdmFrame {
    /// A frame recognised by [`DynFrame`].
    Known(DynFrame),
    /// A free-form name not recognised by [`DynFrame`].
    /// Preserved verbatim on write.
    Custom(String),
}

impl OdmFrame {
    /// Parses a wire-format `REF_FRAME` string.
    ///
    /// Tries [`DynFrame::from_str`] first (which accepts mixed case via
    /// uppercase folding). On failure, wraps the original input as
    /// [`OdmFrame::Custom`].
    pub fn from_wire(s: &str) -> Self {
        match DynFrame::from_str(s) {
            Ok(frame) => OdmFrame::Known(frame),
            Err(_) => OdmFrame::Custom(s.to_string()),
        }
    }

    /// The wire-equivalent name for this frame.
    ///
    /// For [`OdmFrame::Known`] this is the canonical frame name from
    /// [`ReferenceFrame::name`] (returned as [`Cow::Owned`] because the
    /// underlying call returns `String`). For [`OdmFrame::Custom`] it
    /// is the stored string as [`Cow::Borrowed`].
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            OdmFrame::Known(f) => Cow::Owned(f.name()),
            OdmFrame::Custom(s) => Cow::Borrowed(s.as_str()),
        }
    }

    /// Returns the underlying [`DynFrame`] if this is a known frame,
    /// or [`None`] for [`OdmFrame::Custom`].
    pub fn known(&self) -> Option<DynFrame> {
        match self {
            OdmFrame::Known(f) => Some(*f),
            OdmFrame::Custom(_) => None,
        }
    }
}

impl Display for OdmFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

impl From<DynFrame> for OdmFrame {
    fn from(frame: DynFrame) -> Self {
        OdmFrame::Known(frame)
    }
}

/// Common header carried by every CCSDS ODM message (OPM, OEM, OMM, OCM,
/// ODM-CI).
///
/// Mirrors the header section of CCSDS 502.0-B-3: zero or more `COMMENT`
/// lines, an optional classification marker, a mandatory `CREATION_DATE`
/// and `ORIGINATOR`, and an optional `MESSAGE_ID`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OdmHeader {
    /// Header `COMMENT` lines, in document order.
    pub comments: Vec<String>,
    /// Optional `CLASSIFICATION` marker (e.g. `UNCLASSIFIED`, `SECRET`).
    pub classification: Option<String>,
    /// Mandatory `CREATION_DATE` of the message.
    pub creation_date: DynTime,
    /// Mandatory `ORIGINATOR` (the organisation that produced the message).
    pub originator: String,
    /// Optional `MESSAGE_ID` (unique identifier assigned by the originator).
    pub message_id: Option<String>,
}

/// Optional spacecraft physical-properties block carried by OPM and OMM
/// messages.
///
/// All fields are optional per CCSDS 502.0-B-3 §3.4 (OPM) and §4.4 (OMM).
/// `SOLAR_RAD_COEFF` and `DRAG_COEFF` are dimensionless and stay as `f64`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpacecraftParameters {
    /// `MASS` — spacecraft mass.
    pub mass: Option<Mass>,
    /// `SOLAR_RAD_AREA` — area exposed to solar radiation pressure.
    pub solar_rad_area: Option<Area>,
    /// `SOLAR_RAD_COEFF` — dimensionless radiation-pressure coefficient.
    pub solar_rad_coeff: Option<f64>,
    /// `DRAG_AREA` — area exposed to atmospheric drag.
    pub drag_area: Option<Area>,
    /// `DRAG_COEFF` — dimensionless drag coefficient.
    pub drag_coeff: Option<f64>,
}

/// Single-snapshot 6×6 Cartesian-state covariance carried by OPM
/// (CCSDS 502.0-B-3 §3.5) and OMM (§4.6).
///
/// The matrix layout is the standard (X, Y, Z, X_DOT, Y_DOT, Z_DOT)
/// covariance; the 21 unique upper-triangle values from the wire
/// (`CX_X`, `CY_X`, `CY_Y`, …) populate a full symmetric `Matrix6<f64>`
/// when read.
///
/// OEM has a structurally different covariance (timestamped, in a list);
/// see [`crate::types::oem::OemCovariance`].
#[derive(Clone, Debug, PartialEq)]
pub struct Covariance {
    /// `COV_REF_FRAME` — optional frame override; when `None` the
    /// covariance is in the same frame as the message's state vector.
    pub frame: Option<OdmFrame>,
    /// The 6×6 covariance matrix.
    pub matrix: Matrix6<f64>,
}

/// Returned by the `try_into_*` upgrade methods on ODM message types when
/// the message's [`OdmCenter`] or [`OdmFrame`] is `Custom(_)` and therefore
/// cannot be converted to the typed `DynOrigin` / `DynFrame` required by
/// `lox_orbits::Orbit` / `Trajectory`.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CustomBodyOrFrameError {
    /// The message's center body is a custom name not recognised by
    /// `DynOrigin`.
    #[error("custom body `{0}` cannot be upgraded to DynOrigin")]
    Body(String),
    /// The message's reference frame is a custom name not recognised by
    /// `DynFrame`.
    #[error("custom frame `{0}` cannot be upgraded to DynFrame")]
    Frame(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_time::time::Time;
    use lox_time::time_scales::DynTimeScale;

    #[test]
    fn message_kind_is_copy_clone_eq() {
        let kind = MessageKind::Opm;
        let copied = kind;
        let cloned = Clone::clone(&kind);
        assert_eq!(kind, copied);
        assert_eq!(kind, cloned);
    }

    #[test]
    fn message_kind_display() {
        assert_eq!(format!("{}", MessageKind::Opm), "OPM");
        assert_eq!(format!("{}", MessageKind::Oem), "OEM");
        assert_eq!(format!("{}", MessageKind::Omm), "OMM");
        assert_eq!(format!("{}", MessageKind::Ocm), "OCM");
        assert_eq!(format!("{}", MessageKind::Ci), "ODM-CI");
    }

    #[test]
    fn odm_center_from_wire_known_capitalised() {
        let c = OdmCenter::from_wire("Earth");
        assert_eq!(c, OdmCenter::Known(DynOrigin::Earth));
    }

    #[test]
    fn odm_center_from_wire_known_lowercase() {
        // DynOrigin::from_str accepts both "earth" and "Earth"
        let c = OdmCenter::from_wire("earth");
        assert_eq!(c, OdmCenter::Known(DynOrigin::Earth));
    }

    #[test]
    fn odm_center_from_wire_custom() {
        let c = OdmCenter::from_wire("APOPHIS");
        assert_eq!(c, OdmCenter::Custom("APOPHIS".to_string()));
    }

    #[test]
    fn odm_center_name_known() {
        let c = OdmCenter::Known(DynOrigin::Mars);
        assert_eq!(c.name(), Cow::Borrowed("Mars"));
    }

    #[test]
    fn odm_center_name_custom() {
        let c = OdmCenter::Custom("BENNU".to_string());
        assert_eq!(c.name(), Cow::Borrowed("BENNU"));
    }

    #[test]
    fn odm_center_known_returns_dyn_origin() {
        let c = OdmCenter::Known(DynOrigin::Moon);
        assert_eq!(c.known(), Some(DynOrigin::Moon));
    }

    #[test]
    fn odm_center_known_returns_none_for_custom() {
        let c = OdmCenter::Custom("SOME_ASTEROID".to_string());
        assert_eq!(c.known(), None);
    }

    #[test]
    fn odm_center_display() {
        let earth = OdmCenter::Known(DynOrigin::Earth);
        assert_eq!(format!("{earth}"), "Earth");
        let asteroid = OdmCenter::Custom("APOPHIS".to_string());
        assert_eq!(format!("{asteroid}"), "APOPHIS");
    }

    #[test]
    fn odm_center_from_dyn_origin() {
        let c: OdmCenter = DynOrigin::Venus.into();
        assert_eq!(c, OdmCenter::Known(DynOrigin::Venus));
    }

    #[test]
    fn odm_frame_from_wire_known() {
        let f = OdmFrame::from_wire("ICRF");
        assert_eq!(f, OdmFrame::Known(DynFrame::Icrf));
    }

    #[test]
    fn odm_frame_from_wire_known_lowercase() {
        // DynFrame::from_str does a to_uppercase() on input.
        let f = OdmFrame::from_wire("teme");
        assert_eq!(f, OdmFrame::Known(DynFrame::Teme));
    }

    #[test]
    fn odm_frame_from_wire_custom() {
        let f = OdmFrame::from_wire("OPERATOR_LVLH");
        assert_eq!(f, OdmFrame::Custom("OPERATOR_LVLH".to_string()));
    }

    #[test]
    fn odm_frame_name_known() {
        // DynFrame::name returns an owned String; OdmFrame::name returns
        // Cow::Owned for known frames and Cow::Borrowed for custom.
        let f = OdmFrame::Known(DynFrame::Icrf);
        let n = f.name();
        assert!(!n.is_empty());
        // The exact wire-canonical form for ICRF lives in DynFrame's name()
        // — we just smoke-test that something non-empty is produced.
    }

    #[test]
    fn odm_frame_name_custom() {
        let f = OdmFrame::Custom("OPERATOR_LVLH".to_string());
        assert_eq!(f.name(), Cow::Borrowed("OPERATOR_LVLH"));
    }

    #[test]
    fn odm_frame_known_returns_dyn_frame() {
        let f = OdmFrame::Known(DynFrame::Teme);
        assert_eq!(f.known(), Some(DynFrame::Teme));
    }

    #[test]
    fn odm_frame_known_returns_none_for_custom() {
        let f = OdmFrame::Custom("X".to_string());
        assert_eq!(f.known(), None);
    }

    #[test]
    fn odm_frame_from_dyn_frame() {
        let f: OdmFrame = DynFrame::J2000.into();
        assert_eq!(f, OdmFrame::Known(DynFrame::J2000));
    }

    #[test]
    fn odm_frame_display() {
        let known = OdmFrame::Known(DynFrame::Icrf);
        // Smoke-test: Display delegates to name(), which is non-empty for known frames.
        assert!(!format!("{known}").is_empty());
        let custom = OdmFrame::Custom("OPERATOR_LVLH".to_string());
        assert_eq!(format!("{custom}"), "OPERATOR_LVLH");
    }

    fn sample_epoch() -> DynTime {
        // Construct a valid DynTime for fixture use. The specific value
        // doesn't matter for OdmHeader tests — only that we have a DynTime.
        Time::j2000(DynTimeScale::Tai)
    }

    #[test]
    fn odm_header_construction() {
        let h = OdmHeader {
            comments: vec!["a comment".to_string()],
            classification: Some("UNCLASSIFIED".to_string()),
            creation_date: sample_epoch(),
            originator: "TEST".to_string(),
            message_id: Some("msg-123".to_string()),
        };
        assert_eq!(h.comments.len(), 1);
        assert_eq!(h.originator, "TEST");
        assert_eq!(h.message_id.as_deref(), Some("msg-123"));
    }

    #[test]
    fn odm_header_default_minimal_fields() {
        let h = OdmHeader {
            comments: Vec::new(),
            classification: None,
            creation_date: sample_epoch(),
            originator: "TEST".to_string(),
            message_id: None,
        };
        assert!(h.comments.is_empty());
        assert!(h.classification.is_none());
        assert!(h.message_id.is_none());
    }

    #[test]
    fn custom_body_or_frame_error_displays_name() {
        let e = CustomBodyOrFrameError::Body("APOPHIS".to_string());
        assert!(format!("{e}").contains("APOPHIS"));
        let e = CustomBodyOrFrameError::Frame("OPERATOR_LVLH".to_string());
        assert!(format!("{e}").contains("OPERATOR_LVLH"));
    }

    #[test]
    fn spacecraft_parameters_construction() {
        let sp = SpacecraftParameters {
            mass: Some(Mass::kilograms(120.5)),
            solar_rad_area: Some(Area::square_meters(2.5)),
            solar_rad_coeff: Some(1.2),
            drag_area: Some(Area::square_meters(2.0)),
            drag_coeff: Some(2.2),
        };
        assert_eq!(sp.mass.map(|m| m.to_kilograms()), Some(120.5));
        assert_eq!(sp.solar_rad_coeff, Some(1.2));
    }

    #[test]
    fn spacecraft_parameters_all_none() {
        let sp = SpacecraftParameters::default();
        assert!(sp.mass.is_none());
        assert!(sp.solar_rad_area.is_none());
        assert!(sp.solar_rad_coeff.is_none());
        assert!(sp.drag_area.is_none());
        assert!(sp.drag_coeff.is_none());
    }
}
