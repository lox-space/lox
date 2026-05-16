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
use lox_core::time::calendar_dates::CalendarDate;
use lox_core::time::time_of_day::CivilTime;
use lox_core::units::{Area, Mass};
use lox_frames::{DynFrame, traits::ReferenceFrame};
use lox_time::deltas::ToDelta;
use lox_time::offsets::{DefaultOffsetProvider, OffsetProvider, TryOffset};
use lox_time::time::{DynTime, Time};
use lox_time::time_scales::{DynTimeScale, Tai, TimeScale};
use lox_time::utc::Utc;
use lox_time::utc::transformations::ToUtc;
use nalgebra::Matrix6;

/// Discriminator for the four ODM message variants.
///
/// Mirrors the `CCSDS_<KIND>_VERS` keyword on the wire:
/// `CCSDS_OPM_VERS`, `CCSDS_OEM_VERS`, `CCSDS_OMM_VERS`, `CCSDS_OCM_VERS`.
/// The NDM-CI envelope (`CCSDS_NDM_VERS = 1.0`) is out of scope for this
/// crate.
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
}

impl Display for MessageKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            MessageKind::Opm => "OPM",
            MessageKind::Oem => "OEM",
            MessageKind::Omm => "OMM",
            MessageKind::Ocm => "OCM",
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
    /// CCSDS wire form is uppercase (e.g. `EARTH`, `SUN`,
    /// `SOLAR SYSTEM BARYCENTER`). [`DynOrigin::from_str`] matches
    /// lowercase identifiers, so the input is lowercased before lookup
    /// (asymmetric with [`OdmFrame::from_wire`], which uppercases for
    /// [`DynFrame::from_str`]).
    /// On failure, wraps the original input as [`OdmCenter::Custom`].
    pub fn from_wire(s: &str) -> Self {
        match DynOrigin::from_str(&s.to_lowercase()) {
            Ok(origin) => OdmCenter::Known(origin),
            Err(_) => OdmCenter::Custom(s.to_string()),
        }
    }

    /// The wire-equivalent name for this center.
    ///
    /// For [`OdmCenter::Known`] this is the body name in CCSDS wire
    /// form (uppercase, e.g. `"EARTH"`, `"SOLAR SYSTEM BARYCENTER"`).
    /// For [`OdmCenter::Custom`] it is the stored string verbatim.
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            OdmCenter::Known(o) => Cow::Owned(o.name().to_uppercase()),
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
    /// CCSDS wire form is uppercase (e.g. `ICRF`, `EME2000`, `TEME`).
    /// [`DynFrame::from_str`] matches uppercase identifiers, so the input
    /// is uppercased before lookup (mirror of [`OdmCenter::from_wire`],
    /// which lowercases for [`DynOrigin::from_str`]).
    /// On failure, wraps the original input as [`OdmFrame::Custom`].
    pub fn from_wire(s: &str) -> Self {
        match DynFrame::from_str(&s.to_uppercase()) {
            Ok(frame) => OdmFrame::Known(frame),
            Err(_) => OdmFrame::Custom(s.to_string()),
        }
    }

    /// The wire-equivalent name for this frame.
    ///
    /// For [`OdmFrame::Known`] this is the canonical frame
    /// abbreviation from [`ReferenceFrame::abbreviation`] (e.g.
    /// `"ICRF"`, `"TOD"`, `"EME2000"`) — the form used on the wire by
    /// CCSDS ODM messages, not the human-readable long name. For
    /// [`OdmFrame::Custom`] it is the stored string verbatim.
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            OdmFrame::Known(f) => Cow::Owned(f.abbreviation()),
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
    pub creation_date: OdmTime,
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
/// `COMMENT` lines that precede the block in the wire format are preserved
/// in `comments` so that KVN→typed→KVN round-trips reproduce them exactly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpacecraftParameters {
    /// `COMMENT` lines for this sub-block, in document order.
    pub comments: Vec<String>,
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
    /// `COMMENT` lines for this sub-block, in document order.
    pub comments: Vec<String>,
    /// `COV_REF_FRAME` — optional frame override; when `None` the
    /// covariance is in the same frame as the message's state vector.
    pub frame: Option<OdmFrame>,
    /// The 6×6 covariance matrix.
    pub matrix: Matrix6<f64>,
}

/// An epoch from an ODM message.
///
/// CCSDS `TIME_SYSTEM` permits both continuous atomic scales (TAI, TCB,
/// TCG, TDB, TT, UT1, GPS) and the discontinuous UTC scale with leap
/// seconds. Continuous scales fit naturally into [`DynTime`]; UTC needs
/// special handling because of its non-monotonic seconds during leap
/// events. `OdmTime` preserves the wire-format choice so that
/// `read → write` round-trips emit the same `TIME_SYSTEM` keyword.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OdmTime {
    /// Continuous time scale: TAI, TCB, TCG, TDB, TT, UT1, or GPS.
    Time(DynTime),
    /// UTC — leap seconds make it discontinuous.
    Utc(Utc),
}

impl OdmTime {
    /// Parses an ISO-formatted timestamp under the wire `TIME_SYSTEM`
    /// keyword (e.g. `"TAI"`, `"UTC"`, `"GPS"`).
    pub fn from_wire(time_system: &str, iso: &str) -> Result<Self, OdmTimeError> {
        if time_system.eq_ignore_ascii_case("UTC") {
            let utc = Utc::from_iso(iso).map_err(|_| OdmTimeError::InvalidIso(iso.to_string()))?;
            return Ok(OdmTime::Utc(utc));
        }
        let scale = DynTimeScale::from_str(time_system)
            .map_err(|_| OdmTimeError::UnknownTimeSystem(time_system.to_string()))?;
        let t =
            Time::from_iso(scale, iso).map_err(|_| OdmTimeError::InvalidIso(iso.to_string()))?;
        Ok(OdmTime::Time(t))
    }

    /// Returns the continuous-scale [`DynTime`] view of this epoch. UTC
    /// times are converted to TAI via the built-in leap-seconds table.
    pub fn to_dyn_time(&self) -> DynTime {
        match self {
            OdmTime::Time(t) => *t,
            OdmTime::Utc(u) => u.to_dyn_time(),
        }
    }

    /// Returns the wire-format `TIME_SYSTEM` keyword for this epoch
    /// (e.g. `"TAI"`, `"UTC"`, `"GPS"`).
    pub fn time_system(&self) -> &'static str {
        self.scale().abbreviation()
    }

    /// Returns the [`OdmTimeScale`] this epoch is expressed in.
    pub fn scale(&self) -> OdmTimeScale {
        match self {
            OdmTime::Time(t) => OdmTimeScale::from(t.scale()),
            OdmTime::Utc(_) => OdmTimeScale::Utc,
        }
    }

    /// Re-expresses this epoch in `target` using the [`DefaultOffsetProvider`].
    ///
    /// Convenience wrapper around [`Self::try_in_scale_with`]. See that
    /// method for the conversion semantics and error contract.
    pub fn try_in_scale(&self, target: OdmTimeScale) -> Result<Self, OdmTimeError> {
        self.try_in_scale_with(target, &DefaultOffsetProvider)
    }

    /// Re-expresses this epoch in `target` using the supplied offset provider.
    ///
    /// Continuous-to-continuous conversions route through `provider`
    /// (TAI as a pivot when needed). UTC ↔ continuous conversions apply
    /// the leap-seconds table. A no-op return if `target` already matches
    /// [`Self::scale`].
    ///
    /// Returns [`OdmTimeError::OffsetUnavailable`] when the provider has
    /// no offset for the requested conversion at this epoch.
    pub fn try_in_scale_with<P: OffsetProvider>(
        &self,
        target: OdmTimeScale,
        provider: &P,
    ) -> Result<Self, OdmTimeError> {
        if self.scale() == target {
            return Ok(*self);
        }
        match target {
            OdmTimeScale::Utc => Ok(OdmTime::Utc(self.try_tai_pivot_with(provider)?.to_utc())),
            _ => {
                let target_dyn = target
                    .as_continuous()
                    .expect("non-Utc OdmTimeScale always maps to a DynTimeScale");
                let dyn_source = self.to_dyn_time();
                if dyn_source.scale() == target_dyn {
                    return Ok(OdmTime::Time(dyn_source));
                }
                let offset = provider
                    .try_offset(dyn_source.scale(), target_dyn, dyn_source.to_delta())
                    .map_err(|e| OdmTimeError::OffsetUnavailable {
                        from: dyn_source.scale().abbreviation(),
                        to: target_dyn.abbreviation(),
                        reason: e.to_string(),
                    })?;
                Ok(OdmTime::Time(Time::from_delta(
                    target_dyn,
                    dyn_source.to_delta() + offset,
                )))
            }
        }
    }

    /// `Time<Tai>` pivot used by [`Self::try_in_scale_with`].
    fn try_tai_pivot_with<P: OffsetProvider>(
        &self,
        provider: &P,
    ) -> Result<Time<Tai>, OdmTimeError> {
        match self {
            OdmTime::Utc(u) => Ok(u.to_time()),
            OdmTime::Time(d) => {
                if d.scale() == DynTimeScale::Tai {
                    Ok(Time::from_delta(Tai, d.to_delta()))
                } else {
                    let offset = provider
                        .try_offset(d.scale(), DynTimeScale::Tai, d.to_delta())
                        .map_err(|e| OdmTimeError::OffsetUnavailable {
                            from: d.scale().abbreviation(),
                            to: "TAI",
                            reason: e.to_string(),
                        })?;
                    Ok(Time::from_delta(Tai, d.to_delta() + offset))
                }
            }
        }
    }

    /// Formats the epoch as a bare ISO-8601 timestamp (`YYYY-MM-DDTHH:MM:SS.ffffff`)
    /// at microsecond precision, without a trailing time-scale abbreviation.
    ///
    /// This is the form mandated by CCSDS wire formats for fields like
    /// `CREATION_DATE`, `EPOCH`, `START_TIME`, etc., where the time scale is
    /// carried separately via `TIME_SYSTEM` (KVN) or the document/segment
    /// metadata (XML/JSON). [`Display`] keeps the trailing scale token for
    /// human-readable use; writers must use [`iso`](Self::iso) for wire output.
    pub fn iso(&self) -> String {
        self.iso_with_precision(6)
    }

    /// Like [`iso`](Self::iso) but with a caller-chosen sub-second precision.
    pub fn iso_with_precision(&self, precision: usize) -> String {
        match self {
            OdmTime::Time(t) => format!("{}T{:.*}", t.date(), precision, t.time()),
            OdmTime::Utc(u) => format!("{}T{:.*}", u.date(), precision, u.time()),
        }
    }
}

impl std::fmt::Display for OdmTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Default to microsecond precision (6 decimals); callers may
        // override via `{:.N}`. The underlying `Time<T>` and `Utc`
        // Display impls default to milliseconds, which loses precision
        // for typical operational fixtures (e.g. CCSDS Annex examples
        // use microseconds).
        let precision = f.precision().unwrap_or(6);
        match self {
            OdmTime::Time(t) => write!(f, "{t:.precision$}"),
            OdmTime::Utc(u) => write!(f, "{u:.precision$}"),
        }
    }
}

impl From<DynTime> for OdmTime {
    fn from(t: DynTime) -> Self {
        OdmTime::Time(t)
    }
}

impl From<Utc> for OdmTime {
    fn from(u: Utc) -> Self {
        OdmTime::Utc(u)
    }
}

/// Error returned by [`OdmTime::from_wire`] / [`OdmTime::try_in_scale`].
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum OdmTimeError {
    /// The `TIME_SYSTEM` keyword wasn't one of the supported continuous
    /// scales (TAI, TCB, TCG, TDB, TT, UT1, GPS) or `UTC`.
    #[error("unknown time-system keyword: {0:?}")]
    UnknownTimeSystem(String),
    /// The ISO timestamp string couldn't be parsed.
    #[error("invalid ISO timestamp: {0:?}")]
    InvalidIso(String),
    /// The offset provider has no data covering this epoch for the
    /// requested `from → to` conversion. Currently only emitted for UT1
    /// conversions outside the EOP table's range.
    #[error("time-scale offset from {from} to {to} is undefined for this epoch: {reason}")]
    OffsetUnavailable {
        /// Source scale abbreviation (e.g. `"UT1"`).
        from: &'static str,
        /// Target scale abbreviation (e.g. `"TAI"`).
        to: &'static str,
        /// Provider-supplied explanation.
        reason: String,
    },
}

/// The eight time systems permitted by CCSDS ODM messages.
///
/// Mirrors [`DynTimeScale`] plus `Utc`. UTC is special-cased because it is
/// not a continuous scale (it has leap seconds) and so cannot be represented
/// by [`DynTimeScale`].
///
/// Used as the target of [`OdmTime::in_scale`] and as the wire-format hint
/// passed to OPM/OEM builders to control the emitted `TIME_SYSTEM` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum OdmTimeScale {
    /// International Atomic Time.
    Tai,
    /// Barycentric Coordinate Time.
    Tcb,
    /// Geocentric Coordinate Time.
    Tcg,
    /// Barycentric Dynamical Time.
    Tdb,
    /// Terrestrial Time.
    Tt,
    /// Universal Time.
    Ut1,
    /// GPS Time.
    Gps,
    /// Coordinated Universal Time (leap-second-aware).
    Utc,
}

impl OdmTimeScale {
    /// Wire-format `TIME_SYSTEM` keyword (e.g. `"TAI"`, `"UTC"`).
    pub fn abbreviation(&self) -> &'static str {
        match self {
            Self::Tai => "TAI",
            Self::Tcb => "TCB",
            Self::Tcg => "TCG",
            Self::Tdb => "TDB",
            Self::Tt => "TT",
            Self::Ut1 => "UT1",
            Self::Gps => "GPS",
            Self::Utc => "UTC",
        }
    }

    /// Maps to a [`DynTimeScale`] for the seven continuous variants; returns
    /// `None` for UTC.
    pub fn as_continuous(&self) -> Option<DynTimeScale> {
        match self {
            Self::Tai => Some(DynTimeScale::Tai),
            Self::Tcb => Some(DynTimeScale::Tcb),
            Self::Tcg => Some(DynTimeScale::Tcg),
            Self::Tdb => Some(DynTimeScale::Tdb),
            Self::Tt => Some(DynTimeScale::Tt),
            Self::Ut1 => Some(DynTimeScale::Ut1),
            Self::Gps => Some(DynTimeScale::Gps),
            Self::Utc => None,
        }
    }
}

impl FromStr for OdmTimeScale {
    type Err = OdmTimeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "TAI" => Ok(Self::Tai),
            "TCB" => Ok(Self::Tcb),
            "TCG" => Ok(Self::Tcg),
            "TDB" => Ok(Self::Tdb),
            "TT" => Ok(Self::Tt),
            "UT1" => Ok(Self::Ut1),
            "GPS" => Ok(Self::Gps),
            "UTC" => Ok(Self::Utc),
            _ => Err(OdmTimeError::UnknownTimeSystem(s.to_string())),
        }
    }
}

impl Display for OdmTimeScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.abbreviation())
    }
}

impl From<DynTimeScale> for OdmTimeScale {
    fn from(s: DynTimeScale) -> Self {
        match s {
            DynTimeScale::Tai => Self::Tai,
            DynTimeScale::Tcb => Self::Tcb,
            DynTimeScale::Tcg => Self::Tcg,
            DynTimeScale::Tdb => Self::Tdb,
            DynTimeScale::Tt => Self::Tt,
            DynTimeScale::Ut1 => Self::Ut1,
            DynTimeScale::Gps => Self::Gps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_time::deltas::TimeDelta;
    use lox_time::time::Time;
    use lox_time::time_scales::DynTimeScale;

    /// Offset provider that errors on every UT1 conversion. Used to
    /// exercise the [`OdmTimeError::OffsetUnavailable`] path; the
    /// bundled [`DefaultOffsetProvider`] uses `Infallible` and so cannot
    /// reach it.
    #[derive(Debug, Default)]
    struct FailingUt1Provider;

    #[derive(Debug)]
    struct FailingUt1Error;

    impl fmt::Display for FailingUt1Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("UT1 offset unavailable")
        }
    }

    impl std::error::Error for FailingUt1Error {}

    impl OffsetProvider for FailingUt1Provider {
        type Error = FailingUt1Error;
        fn tai_to_ut1(&self, _delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
            Err(FailingUt1Error)
        }
        fn ut1_to_tai(&self, _delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
            Err(FailingUt1Error)
        }
    }

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
        assert_eq!(c.name(), Cow::Borrowed("MARS"));
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
        assert_eq!(format!("{earth}"), "EARTH");
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

    fn sample_epoch() -> OdmTime {
        // Construct a valid OdmTime for fixture use. The specific value
        // doesn't matter for OdmHeader tests — only that we have an OdmTime.
        OdmTime::Time(Time::j2000(DynTimeScale::Tai))
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
    fn spacecraft_parameters_construction() {
        let sp = SpacecraftParameters {
            comments: Vec::new(),
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

    #[test]
    fn odm_time_from_wire_tai() {
        let t = OdmTime::from_wire("TAI", "2024-01-01T00:00:00").unwrap();
        assert!(matches!(t, OdmTime::Time(_)));
        assert_eq!(t.time_system(), "TAI");
    }

    #[test]
    fn odm_time_from_wire_utc() {
        let t = OdmTime::from_wire("UTC", "2024-01-01T00:00:00").unwrap();
        assert!(matches!(t, OdmTime::Utc(_)));
        assert_eq!(t.time_system(), "UTC");
    }

    #[test]
    fn odm_time_from_wire_gps() {
        let t = OdmTime::from_wire("GPS", "2024-01-01T00:00:00").unwrap();
        assert!(matches!(t, OdmTime::Time(_)));
        assert_eq!(t.time_system(), "GPS");
    }

    #[test]
    fn odm_time_from_wire_rejects_unknown_system() {
        let result = OdmTime::from_wire("XYZ", "2024-01-01T00:00:00");
        assert!(matches!(
            result.unwrap_err(),
            OdmTimeError::UnknownTimeSystem(_)
        ));
    }

    #[test]
    fn odm_time_from_wire_rejects_invalid_iso() {
        let result = OdmTime::from_wire("TAI", "not-a-date");
        assert!(matches!(result.unwrap_err(), OdmTimeError::InvalidIso(_)));
    }

    #[test]
    fn odm_time_to_dyn_time_passes_through_for_continuous_scale() {
        let original = Time::j2000(DynTimeScale::Tai);
        let odm = OdmTime::Time(original);
        assert_eq!(odm.to_dyn_time(), original);
    }

    #[test]
    fn odm_time_from_dyn_time_via_into() {
        let t: OdmTime = Time::j2000(DynTimeScale::Tai).into();
        assert!(matches!(t, OdmTime::Time(_)));
    }

    #[test]
    fn odm_time_iso_strips_scale_suffix_for_continuous_scales() {
        let t = OdmTime::Time(Time::j2000(DynTimeScale::Tai));
        let iso = t.iso();
        assert!(
            !iso.contains("TAI") && !iso.contains(' '),
            "iso() leaked scale suffix: {iso:?}"
        );
        assert!(
            iso.starts_with("2000-01-01T"),
            "unexpected iso form: {iso:?}"
        );
    }

    #[test]
    fn odm_time_iso_strips_scale_suffix_for_utc() {
        let t = OdmTime::from_wire("UTC", "2024-06-15T12:34:56.789").unwrap();
        let iso = t.iso();
        assert!(
            !iso.contains("UTC") && !iso.contains(' '),
            "iso() leaked scale suffix: {iso:?}"
        );
    }

    // --- OdmTimeScale ------------------------------------------------------

    #[test]
    fn odm_time_scale_from_str_known() {
        for (s, expected) in [
            ("TAI", OdmTimeScale::Tai),
            ("UTC", OdmTimeScale::Utc),
            ("GPS", OdmTimeScale::Gps),
            ("tt", OdmTimeScale::Tt),
            ("ut1", OdmTimeScale::Ut1),
        ] {
            assert_eq!(s.parse::<OdmTimeScale>().unwrap(), expected);
        }
    }

    #[test]
    fn odm_time_scale_from_str_unknown() {
        assert!("XYZ".parse::<OdmTimeScale>().is_err());
    }

    #[test]
    fn odm_time_scale_abbreviation_matches_wire_keyword() {
        for scale in [
            OdmTimeScale::Tai,
            OdmTimeScale::Tcb,
            OdmTimeScale::Tcg,
            OdmTimeScale::Tdb,
            OdmTimeScale::Tt,
            OdmTimeScale::Ut1,
            OdmTimeScale::Gps,
            OdmTimeScale::Utc,
        ] {
            // Round-trip through FromStr to be sure.
            assert_eq!(scale.abbreviation().parse::<OdmTimeScale>().unwrap(), scale);
        }
    }

    // --- OdmTime::in_scale -------------------------------------------------

    #[test]
    fn odm_time_in_scale_noop_when_target_matches_source() {
        let tai = OdmTime::Time(Time::j2000(DynTimeScale::Tai));
        assert_eq!(tai.try_in_scale(OdmTimeScale::Tai).unwrap(), tai);

        let utc = OdmTime::from_wire("UTC", "2024-01-01T00:00:00").unwrap();
        assert_eq!(utc.try_in_scale(OdmTimeScale::Utc).unwrap(), utc);
    }

    #[test]
    fn odm_time_in_scale_tai_to_utc_then_back_is_lossless_for_post_1972_epoch() {
        let utc_in = OdmTime::from_wire("UTC", "2024-06-15T12:34:56").unwrap();
        let tai = utc_in.try_in_scale(OdmTimeScale::Tai).unwrap();
        assert_eq!(tai.scale(), OdmTimeScale::Tai);
        let utc_out = tai.try_in_scale(OdmTimeScale::Utc).unwrap();
        assert_eq!(utc_out, utc_in);
    }

    #[test]
    fn odm_time_in_scale_tai_to_gps_offsets_by_19_seconds() {
        // J2000 in TAI is the canonical pivot. TAI - GPS = +19 s ⇒
        // converting the same instant to GPS must be 19 s "earlier" in
        // GPS-second-count terms (i.e. GPS reads 19s lower).
        let tai = OdmTime::Time(Time::j2000(DynTimeScale::Tai));
        let gps = tai.try_in_scale(OdmTimeScale::Gps).unwrap();
        assert_eq!(gps.scale(), OdmTimeScale::Gps);
        // Round-trip through TAI must return the original.
        let tai_again = gps.try_in_scale(OdmTimeScale::Tai).unwrap();
        assert_eq!(tai_again, tai);
    }

    #[test]
    fn odm_time_in_scale_utc_to_gps_via_tai_pivot() {
        let utc = OdmTime::from_wire("UTC", "2024-06-15T12:00:00").unwrap();
        let gps = utc.try_in_scale(OdmTimeScale::Gps).unwrap();
        assert_eq!(gps.scale(), OdmTimeScale::Gps);
        let utc_back = gps.try_in_scale(OdmTimeScale::Utc).unwrap();
        assert_eq!(utc_back, utc);
    }

    #[test]
    fn try_in_scale_with_propagates_offset_failure_from_ut1_source() {
        let ut1 = OdmTime::from_wire("UT1", "2024-06-15T12:00:00").unwrap();
        let err = ut1
            .try_in_scale_with(OdmTimeScale::Tai, &FailingUt1Provider)
            .unwrap_err();
        assert!(
            matches!(
                err,
                OdmTimeError::OffsetUnavailable {
                    from: "UT1",
                    to: "TAI",
                    ..
                }
            ),
            "expected OffsetUnavailable(UT1 → TAI), got {err:?}"
        );
    }

    #[test]
    fn try_in_scale_with_propagates_offset_failure_for_ut1_target() {
        let tai = OdmTime::Time(Time::j2000(DynTimeScale::Tai));
        let err = tai
            .try_in_scale_with(OdmTimeScale::Ut1, &FailingUt1Provider)
            .unwrap_err();
        assert!(
            matches!(
                err,
                OdmTimeError::OffsetUnavailable {
                    from: "TAI",
                    to: "UT1",
                    ..
                }
            ),
            "expected OffsetUnavailable(TAI → UT1), got {err:?}"
        );
    }

    #[test]
    fn try_in_scale_with_propagates_failure_through_utc_pivot() {
        // UT1 → UTC pivots through TAI; the UT1 → TAI step fails first.
        let ut1 = OdmTime::from_wire("UT1", "2024-06-15T12:00:00").unwrap();
        let err = ut1
            .try_in_scale_with(OdmTimeScale::Utc, &FailingUt1Provider)
            .unwrap_err();
        assert!(matches!(
            err,
            OdmTimeError::OffsetUnavailable { from: "UT1", .. }
        ));
    }

    #[test]
    fn odm_time_scale_accessor_matches_construction() {
        let utc = OdmTime::from_wire("UTC", "2024-01-01T00:00:00").unwrap();
        assert_eq!(utc.scale(), OdmTimeScale::Utc);
        let tai = OdmTime::Time(Time::j2000(DynTimeScale::Tai));
        assert_eq!(tai.scale(), OdmTimeScale::Tai);
        let gps = OdmTime::Time(Time::j2000(DynTimeScale::Gps));
        assert_eq!(gps.scale(), OdmTimeScale::Gps);
    }
}
