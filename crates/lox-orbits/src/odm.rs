// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! ODM integration: builders and conversion helpers for OPM/OEM messages.
//!
//! Requires the `odm` feature flag.

use std::collections::BTreeMap;
use std::path::Path;

use std::f64::consts::PI;

use chrono::{DateTime, Utc as ChronoUtc};
use lox_bodies::{DynOrigin, Origin};
use lox_core::anomalies::{AnomalyError, MeanAnomaly};
use lox_core::coords::Cartesian;
use lox_core::elements::{
    Keplerian, MeanElements,
    keplerian::{
        ArgumentOfPeriapsis, Eccentricity, Inclination, LongitudeOfAscendingNode, SemiMajorAxis,
    },
};
use lox_core::f64::consts::SECONDS_PER_DAY;
use lox_core::units::Angle;
use lox_frames::{DynFrame, ReferenceFrame};
use lox_odm::Format;
use lox_odm::OdmError;
use lox_odm::types::common::{
    OdmCenter, OdmFrame, OdmHeader, OdmTime, OdmTimeError, OdmTimeScale, SpacecraftParameters,
};
use lox_odm::types::oem::{Oem, OemMetadata, OemSegment};
use lox_odm::types::omm::{Omm, OmmMeanElements, OmmMetadata};
use lox_odm::types::opm::{Opm, OpmMetadata};
use lox_time::offsets::{DefaultOffsetProvider, OffsetProvider};
use lox_time::time::DynTime;
use lox_time::time_scales::{DynTimeScale, TimeScale};

use crate::orbits::{
    CartesianOrbit, DynCartesianOrbit, DynKeplerianOrbit, DynTrajectory, KeplerianOrbit, Orbit,
    Trajectory,
};
use crate::propagators::sgp4::{Sgp4, Sgp4Error};

/// Default value for the CCSDS `ORIGINATOR` header field used by all
/// ODM builders unless overridden via `.originator(...)`.
const DEFAULT_ORIGINATOR: &str = "Lox (https://lox.rs)";

// ----------------------------------------------------------------------------
// Helper: convert a `Time<T>` (where T: Into<DynTimeScale>) to `OdmTime`
// ----------------------------------------------------------------------------

fn orbit_epoch_to_odm_time<T>(time: lox_time::Time<T>) -> OdmTime
where
    T: TimeScale + Copy + Into<DynTimeScale>,
{
    OdmTime::Time(time.into_dyn())
}

/// Apply an optional [`OdmTimeScale`] override to an epoch derived from the
/// orbit/trajectory. Used by the builders so that a UTC-origin OPM round-
/// trip (read → orbit → write) can re-emit `TIME_SYSTEM = UTC` instead of
/// silently widening to TAI.
fn apply_time_system<P: OffsetProvider>(
    epoch: OdmTime,
    target: Option<OdmTimeScale>,
    provider: &P,
) -> Result<OdmTime, OdmTimeError> {
    match target {
        Some(scale) => epoch.try_in_scale_with(scale, provider),
        None => Ok(epoch),
    }
}

// ----------------------------------------------------------------------------
// Conversion errors
// ----------------------------------------------------------------------------

/// Error when converting an OPM to a [`DynCartesianOrbit`].
#[derive(Debug, thiserror::Error)]
pub enum OpmFromOdmError {
    /// The OPM center body is a free-form custom name, not a known body.
    #[error("OPM center `{0}` is not a known body — cannot construct a typed Orbit")]
    CustomCenter(String),
    /// The OPM reference frame is a free-form custom name, not a known frame.
    #[error("OPM frame `{0}` is not a known reference frame")]
    CustomFrame(String),
}

/// Composite error for `DynCartesianOrbit::from_opm_str` / `from_opm_file`.
#[derive(Debug, thiserror::Error)]
pub enum OpmReadError {
    /// The underlying ODM parsing or I/O failed.
    #[error(transparent)]
    Odm(#[from] OdmError),
    /// The parsed OPM could not be converted to a typed orbit.
    #[error(transparent)]
    Convert(#[from] OpmFromOdmError),
}

/// Error when building an [`Opm`] from a typed orbit.
#[derive(Debug, thiserror::Error)]
pub enum OpmBuildError {
    /// An epoch could not be re-expressed in the requested `TIME_SYSTEM`
    /// because the offset table does not cover it (currently only UT1).
    #[error(transparent)]
    TimeSystem(#[from] OdmTimeError),
}

/// Composite error for [`OpmBuilder::write_str`] / [`OpmBuilder::write_file`].
#[derive(Debug, thiserror::Error)]
pub enum OpmWriteError {
    /// Constructing the typed [`Opm`] from the orbit failed.
    #[error(transparent)]
    Build(#[from] OpmBuildError),
    /// Serialising the typed [`Opm`] to the wire format failed.
    #[error(transparent)]
    Odm(#[from] OdmError),
}

/// Error when converting an OEM segment to a [`DynTrajectory`].
#[derive(Debug, thiserror::Error)]
pub enum OemFromOdmError {
    /// The OEM center body is a free-form custom name.
    #[error("OEM center `{0}` is not a known body — cannot construct a typed Trajectory")]
    CustomCenter(String),
    /// The OEM reference frame is a free-form custom name.
    #[error("OEM frame `{0}` is not a known reference frame")]
    CustomFrame(String),
    /// The OEM has no segments.
    #[error("OEM contains no segments")]
    NoSegments,
    /// The segment contains fewer than 2 state vectors.
    #[error("OEM segment has {0} state vector(s); at least 2 are required")]
    InsufficientStates(usize),
}

/// Composite error for `DynTrajectory::from_oem_str` / `from_oem_file`.
#[derive(Debug, thiserror::Error)]
pub enum OemReadError {
    /// The underlying ODM parsing or I/O failed.
    #[error(transparent)]
    Odm(#[from] OdmError),
    /// The parsed OEM could not be converted to a typed trajectory.
    #[error(transparent)]
    Convert(#[from] OemFromOdmError),
}

/// Error when building an [`Oem`] from a typed trajectory.
#[derive(Debug, thiserror::Error)]
pub enum OemBuildError {
    /// An epoch (segment start/stop, useable times, frame epoch, or a
    /// per-state-vector timestamp) could not be re-expressed in the
    /// requested `TIME_SYSTEM`.
    #[error(transparent)]
    TimeSystem(#[from] OdmTimeError),
}

/// Composite error for [`OemBuilder::write_str`] / [`OemBuilder::write_file`].
#[derive(Debug, thiserror::Error)]
pub enum OemWriteError {
    /// Constructing the typed [`Oem`] from the trajectory failed.
    #[error(transparent)]
    Build(#[from] OemBuildError),
    /// Serialising the typed [`Oem`] to the wire format failed.
    #[error(transparent)]
    Odm(#[from] OdmError),
}

// ----------------------------------------------------------------------------
// OpmBuilder
// ----------------------------------------------------------------------------

/// Fluent builder for constructing an [`Opm`] from a typed orbit.
///
/// Obtained via [`Orbit::build_opm`]. The orbit's `object_name` and
/// `object_id` are taken at construction; all other fields have
/// defaults that can be overridden via chained method calls. Call
/// [`build`](OpmBuilder::build) (or a `write_*` variant) to consume
/// the builder.
///
/// The builder is generic over an [`OffsetProvider`] used by any
/// [`time_system`](Self::time_system) conversion. The default is
/// [`DefaultOffsetProvider`]; swap it via
/// [`offset_provider`](Self::offset_provider) if you need a custom UT1
/// table or a stricter error policy.
pub struct OpmBuilder<T: TimeScale, O: Origin, R: ReferenceFrame, P = DefaultOffsetProvider> {
    orbit: Orbit<Cartesian, T, O, R>,
    originator: String,
    object_name: String,
    object_id: String,
    creation_date: Option<OdmTime>,
    message_id: Option<String>,
    classification: Option<String>,
    header_comments: Vec<String>,
    metadata_comments: Vec<String>,
    state_comments: Vec<String>,
    frame_epoch: Option<OdmTime>,
    spacecraft: Option<SpacecraftParameters>,
    user_defined: BTreeMap<String, String>,
    time_system: Option<OdmTimeScale>,
    provider: P,
}

impl<T, O, R> OpmBuilder<T, O, R, DefaultOffsetProvider>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    fn new(
        orbit: Orbit<Cartesian, T, O, R>,
        object_name: impl Into<String>,
        object_id: impl Into<String>,
    ) -> Self {
        Self {
            orbit,
            originator: DEFAULT_ORIGINATOR.to_string(),
            object_name: object_name.into(),
            object_id: object_id.into(),
            creation_date: None,
            message_id: None,
            classification: None,
            header_comments: Vec::new(),
            metadata_comments: Vec::new(),
            state_comments: Vec::new(),
            frame_epoch: None,
            spacecraft: None,
            user_defined: BTreeMap::new(),
            time_system: None,
            provider: DefaultOffsetProvider,
        }
    }
}

impl<T, O, R, P> OpmBuilder<T, O, R, P>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    /// Replaces the [`OffsetProvider`] used by [`time_system`](Self::time_system).
    pub fn offset_provider<P2>(self, provider: P2) -> OpmBuilder<T, O, R, P2> {
        OpmBuilder {
            orbit: self.orbit,
            originator: self.originator,
            object_name: self.object_name,
            object_id: self.object_id,
            creation_date: self.creation_date,
            message_id: self.message_id,
            classification: self.classification,
            header_comments: self.header_comments,
            metadata_comments: self.metadata_comments,
            state_comments: self.state_comments,
            frame_epoch: self.frame_epoch,
            spacecraft: self.spacecraft,
            user_defined: self.user_defined,
            time_system: self.time_system,
            provider,
        }
    }

    /// Overrides `ORIGINATOR`. Defaults to `"Lox (https://lox.rs)"`.
    pub fn originator(mut self, s: impl Into<String>) -> Self {
        self.originator = s.into();
        self
    }

    /// Overrides `OBJECT_NAME` (initially set by [`Orbit::build_opm`]).
    pub fn object_name(mut self, s: impl Into<String>) -> Self {
        self.object_name = s.into();
        self
    }

    /// Overrides `OBJECT_ID` (initially set by [`Orbit::build_opm`]).
    pub fn object_id(mut self, s: impl Into<String>) -> Self {
        self.object_id = s.into();
        self
    }

    /// Sets `CREATION_DATE`. Defaults to the orbit's epoch when not set.
    pub fn creation_date(mut self, t: OdmTime) -> Self {
        self.creation_date = Some(t);
        self
    }

    /// Forces the wire-format `TIME_SYSTEM` keyword and re-expresses every
    /// orbit-derived epoch in that scale.
    ///
    /// Use this to preserve the original time system across a round-trip
    /// through a typed orbit: e.g. an OPM read with `TIME_SYSTEM = UTC`
    /// becomes a [`DynCartesianOrbit`] in TAI (because [`DynTimeScale`]
    /// has no UTC variant), and setting `.time_system(OdmTimeScale::Utc)`
    /// on the builder restores UTC on output.
    ///
    /// When unset, the builder uses the orbit's native scale. Explicitly-set
    /// epochs (e.g. via [`Self::creation_date`]) are left untouched.
    pub fn time_system(mut self, scale: OdmTimeScale) -> Self {
        self.time_system = Some(scale);
        self
    }

    /// Sets `MESSAGE_ID`.
    pub fn message_id(mut self, s: impl Into<String>) -> Self {
        self.message_id = Some(s.into());
        self
    }

    /// Sets the header `CLASSIFICATION` marker.
    pub fn classification(mut self, s: impl Into<String>) -> Self {
        self.classification = Some(s.into());
        self
    }

    /// Appends a `COMMENT` line to the header block.
    pub fn header_comment(mut self, s: impl Into<String>) -> Self {
        self.header_comments.push(s.into());
        self
    }

    /// Appends a `COMMENT` line to the metadata block.
    pub fn metadata_comment(mut self, s: impl Into<String>) -> Self {
        self.metadata_comments.push(s.into());
        self
    }

    /// Appends a `COMMENT` line to the state-vector block.
    pub fn state_comment(mut self, s: impl Into<String>) -> Self {
        self.state_comments.push(s.into());
        self
    }

    /// Sets `REF_FRAME_EPOCH`.
    pub fn frame_epoch(mut self, t: OdmTime) -> Self {
        self.frame_epoch = Some(t);
        self
    }

    /// Sets the spacecraft physical-properties block.
    pub fn spacecraft(mut self, sp: SpacecraftParameters) -> Self {
        self.spacecraft = Some(sp);
        self
    }

    /// Inserts a user-defined key/value pair.
    pub fn user_defined(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.user_defined.insert(k.into(), v.into());
        self
    }
}

impl<T, O, R, P> OpmBuilder<T, O, R, P>
where
    T: TimeScale + Copy + Into<DynTimeScale>,
    O: Origin + Copy + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Into<DynFrame>,
    P: OffsetProvider,
{
    /// Builds the [`Opm`], consuming the builder.
    ///
    /// The orbit's epoch, state, center, and frame populate the
    /// message. If `creation_date` was not set, it defaults to the
    /// orbit's epoch.
    ///
    /// **Lossy boundary**: a [`Cartesian`] orbit carries only the
    /// 6-element state. CCSDS-optional sub-blocks that the source OPM may
    /// have included — `keplerian`, `covariance`, and the `maneuvers` list —
    /// are not represented in the orbit and are therefore emitted as
    /// empty/`None` regardless of what the original message contained.
    /// Round-tripping `OPM(with maneuvers) → DynCartesianOrbit → OpmBuilder`
    /// drops those sections.
    pub fn build(self) -> Result<Opm, OpmBuildError> {
        let epoch = apply_time_system(
            orbit_epoch_to_odm_time(self.orbit.time()),
            self.time_system,
            &self.provider,
        )?;
        let creation_date = self.creation_date.unwrap_or(epoch);
        let center = OdmCenter::from_wire(self.orbit.origin().name());
        let frame = OdmFrame::from_wire(&self.orbit.reference_frame().abbreviation());
        let frame_epoch = self
            .frame_epoch
            .map(|e| apply_time_system(e, self.time_system, &self.provider))
            .transpose()?;

        Ok(Opm {
            header: OdmHeader {
                comments: self.header_comments,
                classification: self.classification,
                creation_date,
                originator: self.originator,
                message_id: self.message_id,
            },
            metadata: OpmMetadata {
                comments: self.metadata_comments,
                object_name: self.object_name,
                object_id: self.object_id,
                center,
                frame,
                frame_epoch,
            },
            epoch,
            state: self.orbit.state(),
            state_comments: self.state_comments,
            keplerian: None,
            spacecraft: self.spacecraft,
            covariance: None,
            maneuvers: Vec::new(),
            user_defined: self.user_defined,
        })
    }

    /// Builds the OPM and serializes it to the requested wire format.
    pub fn write_str(self, format: Format) -> Result<String, OpmWriteError> {
        let opm = self.build()?;
        Ok(lox_odm::write_opm(&opm, format)?)
    }

    /// Builds the OPM and writes it to a file in the requested format.
    pub fn write_file(self, path: impl AsRef<Path>, format: Format) -> Result<(), OpmWriteError> {
        let opm = self.build()?;
        Ok(lox_odm::write_opm_file(&opm, path, format)?)
    }
}

// ----------------------------------------------------------------------------
// Orbit::build_opm
// ----------------------------------------------------------------------------

impl<T, O, R> Orbit<Cartesian, T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    /// Opens an [`OpmBuilder`] for this orbit. `object_name` and
    /// `object_id` are CCSDS-mandatory and have no canonical default,
    /// so they're supplied here; everything else has a default and can
    /// be overridden fluently before [`build`](OpmBuilder::build) or a
    /// `write_*` variant is called.
    pub fn build_opm(
        self,
        object_name: impl Into<String>,
        object_id: impl Into<String>,
    ) -> OpmBuilder<T, O, R, DefaultOffsetProvider> {
        OpmBuilder::new(self, object_name, object_id)
    }
}

// ----------------------------------------------------------------------------
// DynCartesianOrbit from_opm methods
// ----------------------------------------------------------------------------

impl DynCartesianOrbit {
    /// Constructs a [`DynCartesianOrbit`] from a typed OPM.
    ///
    /// Returns an error if the OPM's center or frame is `Custom` (free-form)
    /// because such names cannot be mapped to a `DynOrigin` / `DynFrame`.
    pub fn from_opm(opm: &Opm) -> Result<Self, OpmFromOdmError> {
        let origin = opm.metadata.center.known().ok_or_else(|| {
            OpmFromOdmError::CustomCenter(opm.metadata.center.name().into_owned())
        })?;
        let frame =
            opm.metadata.frame.known().ok_or_else(|| {
                OpmFromOdmError::CustomFrame(opm.metadata.frame.name().into_owned())
            })?;
        let epoch = opm.epoch.to_dyn_time();
        Ok(Orbit::from_state(opm.state, epoch, origin, frame))
    }

    /// Parses an OPM from `input` (auto-detecting wire format) and converts it
    /// to a [`DynCartesianOrbit`].
    pub fn from_opm_str(input: &str) -> Result<Self, OpmReadError> {
        let opm = lox_odm::read_opm(input)?;
        Ok(Self::from_opm(&opm)?)
    }

    /// Reads an OPM from a file (auto-detecting format) and converts it to a
    /// [`DynCartesianOrbit`].
    pub fn from_opm_file(path: impl AsRef<Path>) -> Result<Self, OpmReadError> {
        let opm = lox_odm::read_opm_file(path)?;
        Ok(Self::from_opm(&opm)?)
    }
}

// ----------------------------------------------------------------------------
// OemBuilder
// ----------------------------------------------------------------------------

/// Fluent builder for constructing an [`Oem`] from a typed trajectory.
///
/// Obtained via [`Trajectory::build_oem`]. The trajectory's
/// `object_name` and `object_id` are taken at construction; all other
/// fields have defaults that can be overridden via chained method
/// calls. Call [`build`](OemBuilder::build) (or a `write_*` variant)
/// to consume the builder. The builder is generic over an
/// [`OffsetProvider`] used by
/// [`time_system`](Self::time_system) conversions; see
/// [`offset_provider`](Self::offset_provider).
pub struct OemBuilder<T: TimeScale, O: Origin, R: ReferenceFrame, P = DefaultOffsetProvider> {
    trajectory: Trajectory<T, O, R>,
    originator: String,
    object_name: String,
    object_id: String,
    creation_date: Option<OdmTime>,
    message_id: Option<String>,
    classification: Option<String>,
    header_comments: Vec<String>,
    metadata_comments: Vec<String>,
    data_comments: Vec<String>,
    frame_epoch: Option<OdmTime>,
    useable_start_time: Option<OdmTime>,
    useable_stop_time: Option<OdmTime>,
    interpolation: Option<String>,
    interpolation_degree: Option<u64>,
    user_defined: BTreeMap<String, String>,
    time_system: Option<OdmTimeScale>,
    provider: P,
}

impl<T, O, R> OemBuilder<T, O, R, DefaultOffsetProvider>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    fn new(
        trajectory: Trajectory<T, O, R>,
        object_name: impl Into<String>,
        object_id: impl Into<String>,
    ) -> Self {
        Self {
            trajectory,
            originator: DEFAULT_ORIGINATOR.to_string(),
            object_name: object_name.into(),
            object_id: object_id.into(),
            creation_date: None,
            message_id: None,
            classification: None,
            header_comments: Vec::new(),
            metadata_comments: Vec::new(),
            data_comments: Vec::new(),
            frame_epoch: None,
            useable_start_time: None,
            useable_stop_time: None,
            interpolation: None,
            interpolation_degree: None,
            user_defined: BTreeMap::new(),
            time_system: None,
            provider: DefaultOffsetProvider,
        }
    }
}

impl<T, O, R, P> OemBuilder<T, O, R, P>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    /// Replaces the [`OffsetProvider`] used by [`time_system`](Self::time_system).
    pub fn offset_provider<P2>(self, provider: P2) -> OemBuilder<T, O, R, P2> {
        OemBuilder {
            trajectory: self.trajectory,
            originator: self.originator,
            object_name: self.object_name,
            object_id: self.object_id,
            creation_date: self.creation_date,
            message_id: self.message_id,
            classification: self.classification,
            header_comments: self.header_comments,
            metadata_comments: self.metadata_comments,
            data_comments: self.data_comments,
            frame_epoch: self.frame_epoch,
            useable_start_time: self.useable_start_time,
            useable_stop_time: self.useable_stop_time,
            interpolation: self.interpolation,
            interpolation_degree: self.interpolation_degree,
            user_defined: self.user_defined,
            time_system: self.time_system,
            provider,
        }
    }

    /// Overrides `ORIGINATOR`. Defaults to `"Lox (https://lox.rs)"`.
    pub fn originator(mut self, s: impl Into<String>) -> Self {
        self.originator = s.into();
        self
    }

    /// Overrides `OBJECT_NAME` (initially set by [`Trajectory::build_oem`]).
    pub fn object_name(mut self, s: impl Into<String>) -> Self {
        self.object_name = s.into();
        self
    }

    /// Overrides `OBJECT_ID` (initially set by [`Trajectory::build_oem`]).
    pub fn object_id(mut self, s: impl Into<String>) -> Self {
        self.object_id = s.into();
        self
    }

    /// Sets `CREATION_DATE`. Defaults to the trajectory's start epoch when not set.
    pub fn creation_date(mut self, t: OdmTime) -> Self {
        self.creation_date = Some(t);
        self
    }

    /// Sets the header `CLASSIFICATION` marker.
    pub fn classification(mut self, s: impl Into<String>) -> Self {
        self.classification = Some(s.into());
        self
    }

    /// Forces the segment's wire-format `TIME_SYSTEM` keyword and
    /// re-expresses every trajectory-derived epoch (`START_TIME`,
    /// `STOP_TIME`, state-vector epochs, optional useable times and
    /// `REF_FRAME_EPOCH`) in that scale.
    ///
    /// See [`OpmBuilder::time_system`] for the motivating round-trip case.
    pub fn time_system(mut self, scale: OdmTimeScale) -> Self {
        self.time_system = Some(scale);
        self
    }

    /// Sets `MESSAGE_ID`.
    pub fn message_id(mut self, s: impl Into<String>) -> Self {
        self.message_id = Some(s.into());
        self
    }

    /// Appends a `COMMENT` line to the header block.
    pub fn header_comment(mut self, s: impl Into<String>) -> Self {
        self.header_comments.push(s.into());
        self
    }

    /// Appends a `COMMENT` line to the metadata block.
    pub fn metadata_comment(mut self, s: impl Into<String>) -> Self {
        self.metadata_comments.push(s.into());
        self
    }

    /// Appends a `COMMENT` line to the data block.
    pub fn data_comment(mut self, s: impl Into<String>) -> Self {
        self.data_comments.push(s.into());
        self
    }

    /// Sets `REF_FRAME_EPOCH`.
    pub fn frame_epoch(mut self, t: OdmTime) -> Self {
        self.frame_epoch = Some(t);
        self
    }

    /// Sets `USEABLE_START_TIME`.
    pub fn useable_start_time(mut self, t: OdmTime) -> Self {
        self.useable_start_time = Some(t);
        self
    }

    /// Sets `USEABLE_STOP_TIME`.
    pub fn useable_stop_time(mut self, t: OdmTime) -> Self {
        self.useable_stop_time = Some(t);
        self
    }

    /// Sets the interpolation hint fields (`INTERPOLATION` and
    /// `INTERPOLATION_DEGREE`). Both are set or neither is.
    pub fn interpolation(mut self, method: impl Into<String>, degree: u64) -> Self {
        self.interpolation = Some(method.into());
        self.interpolation_degree = Some(degree);
        self
    }

    /// Inserts a user-defined key/value pair.
    pub fn user_defined(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.user_defined.insert(k.into(), v.into());
        self
    }
}

impl<T, O, R, P> OemBuilder<T, O, R, P>
where
    T: TimeScale + Copy + Into<DynTimeScale>,
    O: Origin + Copy + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Into<DynFrame>,
    P: OffsetProvider,
{
    /// Builds the [`Oem`], consuming the builder.
    ///
    /// The trajectory's epoch (start/stop), origin, and frame populate the
    /// single segment's metadata. State vectors are taken from the
    /// trajectory's knot points via [`Trajectory::states`].
    pub fn build(self) -> Result<Oem, OemBuildError> {
        let ts = self.time_system;
        let provider = &self.provider;
        let start_time = apply_time_system(
            orbit_epoch_to_odm_time(self.trajectory.start_time()),
            ts,
            provider,
        )?;
        let stop_time = apply_time_system(
            orbit_epoch_to_odm_time(self.trajectory.end_time()),
            ts,
            provider,
        )?;
        let creation_date = self.creation_date.unwrap_or(start_time);
        let center = OdmCenter::from_wire(self.trajectory.origin().name());
        let frame = OdmFrame::from_wire(&self.trajectory.reference_frame().abbreviation());

        let states: Vec<(OdmTime, Cartesian)> = self
            .trajectory
            .states()
            .into_iter()
            .map(|s| {
                apply_time_system(orbit_epoch_to_odm_time(s.time()), ts, provider)
                    .map(|t| (t, s.state()))
            })
            .collect::<Result<_, _>>()?;

        let frame_epoch = self
            .frame_epoch
            .map(|e| apply_time_system(e, ts, provider))
            .transpose()?;
        let useable_start_time = self
            .useable_start_time
            .map(|t| apply_time_system(t, ts, provider))
            .transpose()?;
        let useable_stop_time = self
            .useable_stop_time
            .map(|t| apply_time_system(t, ts, provider))
            .transpose()?;

        let segment = OemSegment {
            metadata: OemMetadata {
                comments: self.metadata_comments,
                object_name: self.object_name,
                object_id: self.object_id,
                center,
                frame,
                frame_epoch,
                start_time,
                useable_start_time,
                useable_stop_time,
                stop_time,
                interpolation: self.interpolation,
                interpolation_degree: self.interpolation_degree,
            },
            data_comments: self.data_comments,
            states,
            covariance_history: Vec::new(),
        };

        Ok(Oem {
            header: OdmHeader {
                comments: self.header_comments,
                classification: self.classification,
                creation_date,
                originator: self.originator,
                message_id: self.message_id,
            },
            segments: vec![segment],
            user_defined: self.user_defined,
        })
    }

    /// Builds the OEM and serializes it to the requested wire format.
    pub fn write_str(self, format: Format) -> Result<String, OemWriteError> {
        let oem = self.build()?;
        Ok(lox_odm::write_oem(&oem, format)?)
    }

    /// Builds the OEM and writes it to a file in the requested format.
    pub fn write_file(self, path: impl AsRef<Path>, format: Format) -> Result<(), OemWriteError> {
        let oem = self.build()?;
        Ok(lox_odm::write_oem_file(&oem, path, format)?)
    }
}

// ----------------------------------------------------------------------------
// Trajectory::build_oem
// ----------------------------------------------------------------------------

impl<T, O, R> Trajectory<T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    /// Opens an [`OemBuilder`] for this trajectory. `object_name` and
    /// `object_id` are CCSDS-mandatory and have no canonical default,
    /// so they're supplied here; everything else has a default and can
    /// be overridden fluently before [`build`](OemBuilder::build) or a
    /// `write_*` variant is called.
    pub fn build_oem(
        self,
        object_name: impl Into<String>,
        object_id: impl Into<String>,
    ) -> OemBuilder<T, O, R, DefaultOffsetProvider> {
        OemBuilder::new(self, object_name, object_id)
    }
}

// ----------------------------------------------------------------------------
// DynTrajectory from_oem methods
// ----------------------------------------------------------------------------

impl DynTrajectory {
    /// Constructs a [`DynTrajectory`] from a typed OEM (first segment).
    ///
    /// Returns an error if the OEM has no segments or fewer than 2 state
    /// vectors in the first segment.
    pub fn from_oem(oem: &Oem) -> Result<Self, OemFromOdmError> {
        let segment = oem.segments.first().ok_or(OemFromOdmError::NoSegments)?;
        Self::from_oem_segment(segment)
    }

    /// Constructs a [`DynTrajectory`] from a specific OEM segment.
    pub fn from_oem_segment(segment: &OemSegment) -> Result<Self, OemFromOdmError> {
        let origin = segment.metadata.center.known().ok_or_else(|| {
            OemFromOdmError::CustomCenter(segment.metadata.center.name().into_owned())
        })?;
        let frame = segment.metadata.frame.known().ok_or_else(|| {
            OemFromOdmError::CustomFrame(segment.metadata.frame.name().into_owned())
        })?;

        if segment.states.len() < 2 {
            return Err(OemFromOdmError::InsufficientStates(segment.states.len()));
        }

        let orbits: Vec<CartesianOrbit<DynTimeScale, DynOrigin, DynFrame>> = segment
            .states
            .iter()
            .map(|(odm_time, state)| {
                let epoch: DynTime = odm_time.to_dyn_time();
                Orbit::from_state(*state, epoch, origin, frame)
            })
            .collect();

        Ok(Trajectory::new(orbits))
    }

    /// Parses an OEM from `input` (auto-detecting wire format) and converts it
    /// to a [`DynTrajectory`] (first segment).
    pub fn from_oem_str(input: &str) -> Result<Self, OemReadError> {
        let oem = lox_odm::read_oem(input)?;
        Ok(Self::from_oem(&oem)?)
    }

    /// Reads an OEM from a file (auto-detecting format) and converts it to a
    /// [`DynTrajectory`] (first segment).
    pub fn from_oem_file(path: impl AsRef<Path>) -> Result<Self, OemReadError> {
        let oem = lox_odm::read_oem_file(path)?;
        Ok(Self::from_oem(&oem)?)
    }
}

// ============================================================================
// OMM ↔ SGP4 / KeplerianOrbit
// ============================================================================

/// Error when converting an OMM to a typed propagator/orbit.
#[derive(Debug, thiserror::Error)]
pub enum OmmFromOdmError {
    /// The OMM center is a free-form custom name.
    #[error("OMM center `{0}` is not a known body")]
    CustomCenter(String),
    /// The OMM reference frame is a free-form custom name.
    #[error("OMM frame `{0}` is not a known reference frame")]
    CustomFrame(String),
    /// No wire `GM` and the centre body has no canonical gravitational parameter.
    #[error("OMM has no `GM` and centre `{0}` has no canonical gravitational parameter")]
    MissingGm(String),
    /// The epoch could not be converted to UTC (required by `sgp4::Elements`).
    #[error("failed to convert OMM epoch to UTC: {0}")]
    EpochConversion(String),
    /// SGP4 rejected the elements (e.g. unphysical mean motion).
    #[error(transparent)]
    Sgp4(#[from] Sgp4Error),
    /// An element-validation error from the typed Keplerian model.
    #[error("invalid Keplerian element: {0}")]
    InvalidElement(String),
}

/// Composite error for `Sgp4::from_omm_str` / `from_omm_file` and the
/// analogous `DynKeplerianOrbit::from_omm_*` constructors.
#[derive(Debug, thiserror::Error)]
pub enum OmmReadError {
    /// The underlying ODM parsing or I/O failed.
    #[error(transparent)]
    Odm(#[from] OdmError),
    /// The parsed OMM could not be converted to the target type.
    #[error(transparent)]
    Convert(#[from] OmmFromOdmError),
}

/// Error when building an [`Omm`] from a [`KeplerianOrbit`].
#[derive(Debug, thiserror::Error)]
pub enum OmmBuildError {
    /// The orbit's true anomaly could not be converted to a mean anomaly.
    /// This happens only for hyperbolic orbits whose true anomaly is
    /// outside the asymptote limits, or when an iterative solver fails to
    /// converge. CCSDS OMMs are typically used for periodic (Earth) orbits
    /// where this conversion is total.
    #[error("failed to convert true anomaly to mean anomaly: {0}")]
    Anomaly(#[from] AnomalyError),
    /// An epoch could not be re-expressed in the requested `TIME_SYSTEM`.
    #[error(transparent)]
    TimeSystem(#[from] OdmTimeError),
}

/// Composite error for [`OmmBuilder::write_str`] / [`OmmBuilder::write_file`].
#[derive(Debug, thiserror::Error)]
pub enum OmmWriteError {
    /// Constructing the typed [`Omm`] from the orbit failed.
    #[error(transparent)]
    Build(#[from] OmmBuildError),
    /// Serialising the typed [`Omm`] to the wire format failed.
    #[error(transparent)]
    Odm(#[from] OdmError),
}

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

/// Convert an [`OdmTime`] to a UTC-naive timestamp for `sgp4::Elements`.
fn omm_epoch_to_naive_utc(epoch: OdmTime) -> Result<chrono::NaiveDateTime, OmmFromOdmError> {
    let utc_epoch = epoch
        .try_in_scale(OdmTimeScale::Utc)
        .map_err(|e| OmmFromOdmError::EpochConversion(e.to_string()))?;
    // `OdmTime::Utc(_)` is the only variant we expect after `try_in_scale(Utc)`.
    let utc = match utc_epoch {
        OdmTime::Utc(u) => u,
        OdmTime::Time(_) => {
            return Err(OmmFromOdmError::EpochConversion(
                "expected UTC after in_scale; got continuous scale".to_string(),
            ));
        }
    };
    let dt: DateTime<ChronoUtc> = utc.try_into().map_err(|e: lox_time::chrono::ChronoError| {
        OmmFromOdmError::EpochConversion(e.to_string())
    })?;
    Ok(dt.naive_utc())
}

/// Decode the single-letter TLE `CLASSIFICATION_TYPE` from the OMM
/// TLE-parameters block (not the free-form `OdmHeader::classification`
/// marker, which uses CCSDS labels like `UNCLASSIFIED`/`SECRET`).
fn classification_from_str(s: Option<&str>) -> sgp4::Classification {
    match s.map(str::trim) {
        Some("C") => sgp4::Classification::Classified,
        Some("S") => sgp4::Classification::Secret,
        _ => sgp4::Classification::Unclassified,
    }
}

/// Compute SGP4's `mean_motion` (rev/day) from semi-major axis (m). Inverse
/// of [`mean_motion_to_sma`].
fn sma_to_mean_motion(sma_m: f64, gm_m3_per_s2: f64) -> f64 {
    let n_rad_per_s = (gm_m3_per_s2 / sma_m.powi(3)).sqrt();
    n_rad_per_s * SECONDS_PER_DAY / (2.0 * PI)
}

// ----------------------------------------------------------------------------
// Sgp4::from_omm / to_omm
// ----------------------------------------------------------------------------

impl Sgp4 {
    /// Constructs an [`Sgp4`] propagator from a typed [`Omm`].
    ///
    /// The OMM's mean elements (`SEMI_MAJOR_AXIS` or `MEAN_MOTION`),
    /// `BSTAR`, `MEAN_MOTION_DOT`/`DDOT`, and TLE-parameter block are
    /// translated into an [`sgp4::Elements`]. The epoch is converted to UTC
    /// (SGP4's wire format). Missing TLE parameters default to `0`.
    ///
    /// # Errors
    ///
    /// - [`OmmFromOdmError::CustomCenter`] if the OMM centre is not Earth.
    /// - [`OmmFromOdmError::MissingGm`] if no `GM` is available to derive
    ///   the SGP4 `MEAN_MOTION` from `SEMI_MAJOR_AXIS`.
    /// - [`OmmFromOdmError::Sgp4`] if SGP4 itself rejects the elements.
    pub fn from_omm(omm: &Omm) -> Result<Self, OmmFromOdmError> {
        let centre = omm.metadata.center.known().ok_or_else(|| {
            OmmFromOdmError::CustomCenter(omm.metadata.center.name().into_owned())
        })?;
        if centre != DynOrigin::Earth {
            return Err(OmmFromOdmError::CustomCenter(format!(
                "SGP4 only supports Earth; got {}",
                omm.metadata.center.name()
            )));
        }

        let gm = omm
            .gm()
            .ok_or_else(|| OmmFromOdmError::MissingGm(omm.metadata.center.name().into_owned()))?;

        let me = &omm.mean_elements.elements;
        let datetime = omm_epoch_to_naive_utc(omm.epoch)?;

        let tle = omm.tle_parameters.as_ref();
        let elements = sgp4::Elements {
            object_name: Some(omm.metadata.object_name.clone()),
            international_designator: Some(omm.metadata.object_id.clone()),
            norad_id: tle
                .and_then(|t| t.norad_cat_id)
                .map(|i| i.max(0) as u64)
                .unwrap_or(0),
            classification: classification_from_str(
                tle.and_then(|t| t.classification_type.as_deref()),
            ),
            datetime,
            mean_motion_dot: tle.and_then(|t| t.mean_motion_dot).unwrap_or(0.0),
            mean_motion_ddot: tle.and_then(|t| t.mean_motion_ddot).unwrap_or(0.0),
            drag_term: tle.and_then(|t| t.bstar).unwrap_or(0.0),
            element_set_number: tle
                .and_then(|t| t.element_set_no)
                .map(|i| i.max(0) as u64)
                .unwrap_or(0),
            inclination: me.i.to_degrees(),
            right_ascension: me.raan.to_degrees(),
            eccentricity: me.e,
            argument_of_perigee: me.aop.to_degrees(),
            mean_anomaly: me.m.to_degrees(),
            mean_motion: sma_to_mean_motion(me.a, gm.as_f64()),
            revolution_number: tle.and_then(|t| t.rev_at_epoch).unwrap_or(0),
            ephemeris_type: tle
                .and_then(|t| t.ephemeris_type)
                .map(|i| i.max(0) as u8)
                .unwrap_or(0),
        };

        Ok(Self::new(elements)?)
    }

    /// Parses an OMM from `input` (auto-detecting wire format) and
    /// constructs an [`Sgp4`] propagator.
    pub fn from_omm_str(input: &str) -> Result<Self, OmmReadError> {
        let omm = lox_odm::read_omm(input)?;
        Ok(Self::from_omm(&omm)?)
    }

    /// Reads an OMM from a file (auto-detecting format) and constructs an
    /// [`Sgp4`] propagator.
    pub fn from_omm_file(path: impl AsRef<Path>) -> Result<Self, OmmReadError> {
        let omm = lox_odm::read_omm_file(path)?;
        Ok(Self::from_omm(&omm)?)
    }
}

// ----------------------------------------------------------------------------
// OmmBuilder (KeplerianOrbit → OMM)
// ----------------------------------------------------------------------------

/// Fluent builder for constructing an [`Omm`] from a [`KeplerianOrbit`].
///
/// The orbit's Keplerian elements populate the OMM mean-elements block
/// (true anomaly → mean anomaly is a geometric conversion, independent of
/// the chosen `MEAN_ELEMENT_THEORY`). No TLE-parameters block is emitted —
/// those are SGP4-specific data that a generic [`KeplerianOrbit`] does not
/// carry. Callers needing an SGP4-tuned OMM with `BSTAR`/`MEAN_MOTION_DOT`
/// etc. should construct the typed [`Omm`] directly.
///
/// The builder is generic over an [`OffsetProvider`] used by
/// [`time_system`](Self::time_system) conversions; see
/// [`offset_provider`](Self::offset_provider).
pub struct OmmBuilder<T: TimeScale, O: Origin, R: ReferenceFrame, P = DefaultOffsetProvider> {
    orbit: KeplerianOrbit<T, O, R>,
    originator: String,
    object_name: String,
    object_id: String,
    creation_date: Option<OdmTime>,
    message_id: Option<String>,
    classification: Option<String>,
    header_comments: Vec<String>,
    metadata_comments: Vec<String>,
    data_comments: Vec<String>,
    frame_epoch: Option<OdmTime>,
    mean_element_theory: String,
    spacecraft: Option<SpacecraftParameters>,
    user_defined: BTreeMap<String, String>,
    time_system: Option<OdmTimeScale>,
    provider: P,
}

impl<T, O, R> OmmBuilder<T, O, R, DefaultOffsetProvider>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    fn new(
        orbit: KeplerianOrbit<T, O, R>,
        object_name: impl Into<String>,
        object_id: impl Into<String>,
    ) -> Self {
        Self {
            orbit,
            originator: DEFAULT_ORIGINATOR.to_string(),
            object_name: object_name.into(),
            object_id: object_id.into(),
            creation_date: None,
            message_id: None,
            classification: None,
            header_comments: Vec::new(),
            metadata_comments: Vec::new(),
            data_comments: Vec::new(),
            frame_epoch: None,
            mean_element_theory: "SGP/SGP4".to_string(),
            spacecraft: None,
            user_defined: BTreeMap::new(),
            time_system: None,
            provider: DefaultOffsetProvider,
        }
    }
}

impl<T, O, R, P> OmmBuilder<T, O, R, P>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    /// Replaces the [`OffsetProvider`] used by [`time_system`](Self::time_system).
    pub fn offset_provider<P2>(self, provider: P2) -> OmmBuilder<T, O, R, P2> {
        OmmBuilder {
            orbit: self.orbit,
            originator: self.originator,
            object_name: self.object_name,
            object_id: self.object_id,
            creation_date: self.creation_date,
            message_id: self.message_id,
            classification: self.classification,
            header_comments: self.header_comments,
            metadata_comments: self.metadata_comments,
            data_comments: self.data_comments,
            frame_epoch: self.frame_epoch,
            mean_element_theory: self.mean_element_theory,
            spacecraft: self.spacecraft,
            user_defined: self.user_defined,
            time_system: self.time_system,
            provider,
        }
    }

    /// Overrides `ORIGINATOR`. Defaults to `"Lox (https://lox.rs)"`.
    pub fn originator(mut self, s: impl Into<String>) -> Self {
        self.originator = s.into();
        self
    }

    /// Overrides `OBJECT_NAME` (initially set by [`KeplerianOrbit::build_omm`]).
    pub fn object_name(mut self, s: impl Into<String>) -> Self {
        self.object_name = s.into();
        self
    }

    /// Overrides `OBJECT_ID` (initially set by [`KeplerianOrbit::build_omm`]).
    pub fn object_id(mut self, s: impl Into<String>) -> Self {
        self.object_id = s.into();
        self
    }

    /// Sets `CREATION_DATE`. Defaults to the orbit's epoch when not set.
    pub fn creation_date(mut self, t: OdmTime) -> Self {
        self.creation_date = Some(t);
        self
    }

    /// Forces the wire-format `TIME_SYSTEM` keyword. See
    /// [`OpmBuilder::time_system`] for the round-trip rationale.
    pub fn time_system(mut self, scale: OdmTimeScale) -> Self {
        self.time_system = Some(scale);
        self
    }

    /// Sets `MESSAGE_ID`.
    pub fn message_id(mut self, s: impl Into<String>) -> Self {
        self.message_id = Some(s.into());
        self
    }

    /// Sets the header `CLASSIFICATION` marker.
    pub fn classification(mut self, s: impl Into<String>) -> Self {
        self.classification = Some(s.into());
        self
    }

    /// Appends a header `COMMENT`.
    pub fn header_comment(mut self, s: impl Into<String>) -> Self {
        self.header_comments.push(s.into());
        self
    }

    /// Appends a metadata `COMMENT`.
    pub fn metadata_comment(mut self, s: impl Into<String>) -> Self {
        self.metadata_comments.push(s.into());
        self
    }

    /// Appends a mean-elements `COMMENT`.
    pub fn data_comment(mut self, s: impl Into<String>) -> Self {
        self.data_comments.push(s.into());
        self
    }

    /// Sets `REF_FRAME_EPOCH`.
    pub fn frame_epoch(mut self, t: OdmTime) -> Self {
        self.frame_epoch = Some(t);
        self
    }

    /// Overrides `MEAN_ELEMENT_THEORY` (default `"SGP/SGP4"`).
    pub fn mean_element_theory(mut self, s: impl Into<String>) -> Self {
        self.mean_element_theory = s.into();
        self
    }

    /// Attaches the spacecraft physical-properties block.
    pub fn spacecraft(mut self, sp: SpacecraftParameters) -> Self {
        self.spacecraft = Some(sp);
        self
    }

    /// Inserts a user-defined key/value pair.
    pub fn user_defined(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.user_defined.insert(k.into(), v.into());
        self
    }
}

impl<T, O, R, P> OmmBuilder<T, O, R, P>
where
    T: TimeScale + Copy + Into<DynTimeScale>,
    O: Origin + Copy + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Into<DynFrame>,
    P: OffsetProvider,
{
    /// Builds the [`Omm`], consuming the builder.
    ///
    /// # Errors
    ///
    /// - [`OmmBuildError::Anomaly`] if the orbit's true anomaly cannot be
    ///   converted to a mean anomaly — only possible for hyperbolic
    ///   orbits whose true anomaly is outside the asymptote limits.
    /// - [`OmmBuildError::TimeSystem`] if a [`time_system`](Self::time_system)
    ///   override requires an offset that the builder's
    ///   [`OffsetProvider`] can't resolve for the orbit's epoch.
    pub fn build(self) -> Result<Omm, OmmBuildError> {
        let kep = self.orbit.state();
        let ecc = kep.eccentricity();
        // True → mean is a geometric conversion. For hyperbolic orbits
        // it can fail when the true anomaly exceeds the asymptote limit;
        // propagate that as a typed error instead of panicking.
        let mean_anomaly_rad = kep.true_anomaly().to_mean(ecc)?.as_f64();

        let mean_elements = MeanElements {
            a: kep.semi_major_axis().to_meters(),
            e: ecc.as_f64(),
            i: kep.inclination().as_f64(),
            raan: kep.longitude_of_ascending_node().as_f64(),
            aop: kep.argument_of_periapsis().as_f64(),
            m: mean_anomaly_rad,
        };

        let raw_epoch = orbit_epoch_to_odm_time(self.orbit.time());
        let epoch = apply_time_system(raw_epoch, self.time_system, &self.provider)?;
        let creation_date = self.creation_date.unwrap_or(epoch);
        let frame_epoch = self
            .frame_epoch
            .map(|e| apply_time_system(e, self.time_system, &self.provider))
            .transpose()?;
        let center = OdmCenter::from_wire(self.orbit.origin().name());
        let frame = OdmFrame::from_wire(&self.orbit.reference_frame().abbreviation());

        Ok(Omm {
            header: OdmHeader {
                comments: self.header_comments,
                classification: self.classification,
                creation_date,
                originator: self.originator,
                message_id: self.message_id,
            },
            metadata: OmmMetadata {
                comments: self.metadata_comments,
                object_name: self.object_name,
                object_id: self.object_id,
                center,
                frame,
                frame_epoch,
                mean_element_theory: self.mean_element_theory,
            },
            epoch,
            mean_elements: OmmMeanElements {
                comments: self.data_comments,
                elements: mean_elements,
                gm: None,
            },
            tle_parameters: None,
            spacecraft: self.spacecraft,
            covariance: None,
            user_defined: self.user_defined,
            provider_extras: BTreeMap::new(),
        })
    }

    /// Builds the OMM and serialises it to the requested wire format.
    pub fn write_str(self, format: Format) -> Result<String, OmmWriteError> {
        let omm = self.build()?;
        Ok(lox_odm::write_omm(&omm, format)?)
    }

    /// Builds the OMM and writes it to a file in the requested format.
    pub fn write_file(self, path: impl AsRef<Path>, format: Format) -> Result<(), OmmWriteError> {
        let omm = self.build()?;
        Ok(lox_odm::write_omm_file(&omm, path, format)?)
    }
}

// ----------------------------------------------------------------------------
// KeplerianOrbit::build_omm
// ----------------------------------------------------------------------------

impl<T, O, R> KeplerianOrbit<T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    /// Opens an [`OmmBuilder`] for this orbit. `object_name` and
    /// `object_id` are CCSDS-mandatory and have no canonical default,
    /// so they're supplied here; everything else has a default and can
    /// be overridden fluently before [`build`](OmmBuilder::build) or a
    /// `write_*` variant is called.
    pub fn build_omm(
        self,
        object_name: impl Into<String>,
        object_id: impl Into<String>,
    ) -> OmmBuilder<T, O, R, DefaultOffsetProvider> {
        OmmBuilder::new(self, object_name, object_id)
    }
}

// ----------------------------------------------------------------------------
// DynKeplerianOrbit::from_omm
// ----------------------------------------------------------------------------

impl DynKeplerianOrbit {
    /// Constructs a [`DynKeplerianOrbit`] from a typed OMM by treating the
    /// stored mean elements as Keplerian orbital elements.
    ///
    /// **Caveat:** OMM mean elements are tuned for a specific propagation
    /// theory (see [`OmmMetadata::mean_element_theory`]). For SGP4-tuned
    /// OMMs they live in TEME and are *not* osculating — converting them
    /// to a Cartesian state via [`KeplerianOrbit::to_cartesian`] will not
    /// match the SGP4 prediction. Use [`Sgp4::from_omm`] for actual
    /// propagation of SGP4-tuned OMMs.
    pub fn from_omm(omm: &Omm) -> Result<Self, OmmFromOdmError> {
        let origin = omm.metadata.center.known().ok_or_else(|| {
            OmmFromOdmError::CustomCenter(omm.metadata.center.name().into_owned())
        })?;
        let frame =
            omm.metadata.frame.known().ok_or_else(|| {
                OmmFromOdmError::CustomFrame(omm.metadata.frame.name().into_owned())
            })?;
        let me = &omm.mean_elements.elements;

        let eccentricity = Eccentricity::try_new(me.e)
            .map_err(|e| OmmFromOdmError::InvalidElement(e.to_string()))?;
        let mean_anomaly = MeanAnomaly::new(Angle::radians(me.m));
        let true_anomaly = mean_anomaly
            .to_true(eccentricity)
            .map_err(|e| OmmFromOdmError::InvalidElement(e.to_string()))?;

        let kep = Keplerian::new(
            SemiMajorAxis::meters(me.a),
            eccentricity,
            Inclination::try_new(Angle::radians(me.i))
                .map_err(|e| OmmFromOdmError::InvalidElement(e.to_string()))?,
            LongitudeOfAscendingNode::try_new(Angle::radians(me.raan))
                .map_err(|e| OmmFromOdmError::InvalidElement(e.to_string()))?,
            ArgumentOfPeriapsis::try_new(Angle::radians(me.aop))
                .map_err(|e| OmmFromOdmError::InvalidElement(e.to_string()))?,
            true_anomaly,
        );

        // Use raw `Orbit::from_state` because TEME is not in
        // `DynFrame::TryQuasiInertial`'s permitted set, but for typed-view
        // purposes we still want to allow construction.
        let time = omm.epoch.to_dyn_time();
        Ok(Orbit::from_state(kep, time, origin, frame))
    }

    /// Parses an OMM from `input` (auto-detecting wire format) and converts
    /// it to a [`DynKeplerianOrbit`].
    pub fn from_omm_str(input: &str) -> Result<Self, OmmReadError> {
        let omm = lox_odm::read_omm(input)?;
        Ok(Self::from_omm(&omm)?)
    }

    /// Reads an OMM from a file (auto-detecting format) and converts it to
    /// a [`DynKeplerianOrbit`].
    pub fn from_omm_file(path: impl AsRef<Path>) -> Result<Self, OmmReadError> {
        let omm = lox_odm::read_omm_file(path)?;
        Ok(Self::from_omm(&omm)?)
    }
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use lox_bodies::{DynOrigin, Earth};
    use lox_core::coords::Cartesian;
    use lox_core::units::{Distance, Velocity};
    use lox_frames::{DynFrame, Icrf};
    use lox_odm::Format;
    use lox_odm::types::common::{OdmCenter, OdmFrame, OdmTime};
    use lox_time::deltas::TimeDelta;
    use lox_time::time_scales::Tai;
    use lox_time::{Time, time};

    /// Offset provider that errors on every UT1 conversion. Used to
    /// exercise the [`OdmTimeError::OffsetUnavailable`] path end-to-end
    /// through the builders.
    #[derive(Debug, Default)]
    struct FailingUt1Provider;

    #[derive(Debug)]
    struct FailingUt1Error;

    impl std::fmt::Display for FailingUt1Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    fn sample_cartesian() -> Cartesian {
        Cartesian::new(
            Distance::kilometers(7000.0),
            Distance::kilometers(0.0),
            Distance::kilometers(0.0),
            Velocity::kilometers_per_second(0.0),
            Velocity::kilometers_per_second(7.5),
            Velocity::kilometers_per_second(0.0),
        )
    }

    fn sample_epoch_tai() -> Time<Tai> {
        time!(Tai, 2024, 1, 1, 0, 0, 0.0).unwrap()
    }

    fn sample_dyn_orbit() -> DynCartesianOrbit {
        Orbit::from_state(
            sample_cartesian(),
            sample_epoch_tai().into_dyn(),
            DynOrigin::Earth,
            DynFrame::Icrf,
        )
    }

    fn sample_static_orbit() -> CartesianOrbit<Tai, Earth, Icrf> {
        Orbit::from_state(sample_cartesian(), sample_epoch_tai(), Earth, Icrf)
    }

    fn sample_dyn_trajectory() -> DynTrajectory {
        use lox_time::deltas::TimeDelta;
        let t0 = sample_epoch_tai().into_dyn();
        let t1 = t0 + TimeDelta::from_seconds(60);
        let t2 = t0 + TimeDelta::from_seconds(120);
        let s0 = Orbit::from_state(sample_cartesian(), t0, DynOrigin::Earth, DynFrame::Icrf);
        let s1 = Orbit::from_state(
            Cartesian::new(
                Distance::kilometers(6999.0),
                Distance::kilometers(100.0),
                Distance::kilometers(0.0),
                Velocity::kilometers_per_second(-0.01),
                Velocity::kilometers_per_second(7.5),
                Velocity::kilometers_per_second(0.0),
            ),
            t1,
            DynOrigin::Earth,
            DynFrame::Icrf,
        );
        let s2 = Orbit::from_state(
            Cartesian::new(
                Distance::kilometers(6996.0),
                Distance::kilometers(200.0),
                Distance::kilometers(0.0),
                Velocity::kilometers_per_second(-0.02),
                Velocity::kilometers_per_second(7.5),
                Velocity::kilometers_per_second(0.0),
            ),
            t2,
            DynOrigin::Earth,
            DynFrame::Icrf,
        );
        Trajectory::new(vec![s0, s1, s2])
    }

    // -------------------------------------------------------------------------
    // OPM tests
    // -------------------------------------------------------------------------

    #[test]
    fn to_opm_from_dyn_orbit() {
        let orbit = sample_dyn_orbit();
        let opm = orbit
            .build_opm("TEST-SAT", "2024-000A")
            .originator("TEST_ORG")
            .build()
            .unwrap();
        assert_eq!(opm.metadata.object_name, "TEST-SAT");
        assert_eq!(opm.metadata.object_id, "2024-000A");
        assert_eq!(opm.header.originator, "TEST_ORG");
        assert_eq!(opm.metadata.center, OdmCenter::Known(DynOrigin::Earth));
        assert_eq!(opm.metadata.frame, OdmFrame::Known(DynFrame::Icrf));
        assert_eq!(opm.state, sample_cartesian());
    }

    #[test]
    fn opm_builder_uses_default_originator() {
        let opm = sample_dyn_orbit()
            .build_opm("TEST-SAT", "2024-000A")
            .build()
            .unwrap();
        assert_eq!(opm.header.originator, "Lox (https://lox.rs)");
    }

    #[test]
    fn to_opm_from_static_orbit() {
        let orbit = sample_static_orbit();
        let opm = orbit
            .build_opm("STATIC-SAT", "2024-001A")
            .originator("STATIC_ORG")
            .build()
            .unwrap();
        assert_eq!(opm.metadata.center, OdmCenter::Known(DynOrigin::Earth));
        assert_eq!(opm.metadata.frame, OdmFrame::Known(DynFrame::Icrf));
        assert_eq!(opm.state, sample_cartesian());
    }

    #[test]
    fn to_opm_str_round_trip_kvn() {
        let orbit = sample_dyn_orbit();
        let kvn = orbit
            .build_opm("TEST-SAT", "2024-000A")
            .originator("TEST_ORG")
            .write_str(Format::Kvn)
            .unwrap();
        let parsed = lox_odm::read_opm(&kvn).unwrap();
        assert_eq!(parsed.metadata.object_name, "TEST-SAT");
        assert_eq!(parsed.state, orbit.state());
    }

    #[test]
    fn from_opm_reconstructs_dyn_orbit() {
        let orbit = sample_dyn_orbit();
        let opm = orbit
            .build_opm("TEST-SAT", "2024-000A")
            .originator("TEST_ORG")
            .build()
            .unwrap();
        let reconstructed = DynCartesianOrbit::from_opm(&opm).unwrap();
        assert_eq!(reconstructed.origin(), DynOrigin::Earth);
        assert_eq!(reconstructed.reference_frame(), DynFrame::Icrf);
        assert_eq!(reconstructed.state(), orbit.state());
    }

    #[test]
    fn from_opm_str_kvn() {
        let orbit = sample_dyn_orbit();
        let kvn = orbit
            .build_opm("TEST-SAT", "2024-000A")
            .originator("TEST_ORG")
            .write_str(Format::Kvn)
            .unwrap();
        let reconstructed = DynCartesianOrbit::from_opm_str(&kvn).unwrap();
        assert_eq!(reconstructed.state(), orbit.state());
    }

    #[test]
    fn opm_builder_propagates_offset_provider_failure() {
        let err = sample_dyn_orbit()
            .build_opm("SAT", "2024-000A")
            .offset_provider(FailingUt1Provider)
            .time_system(OdmTimeScale::Ut1)
            .build()
            .unwrap_err();
        assert!(
            matches!(
                err,
                OpmBuildError::TimeSystem(OdmTimeError::OffsetUnavailable { to: "UT1", .. })
            ),
            "expected TimeSystem(OffsetUnavailable {{ to: UT1, .. }}), got {err:?}"
        );
    }

    #[test]
    fn opm_builder_write_str_propagates_offset_provider_failure() {
        let err = sample_dyn_orbit()
            .build_opm("SAT", "2024-000A")
            .offset_provider(FailingUt1Provider)
            .time_system(OdmTimeScale::Ut1)
            .write_str(Format::Kvn)
            .unwrap_err();
        assert!(matches!(
            err,
            OpmWriteError::Build(OpmBuildError::TimeSystem(_))
        ));
    }

    #[test]
    fn from_opm_errors_on_custom_center() {
        use lox_odm::types::opm::OpmMetadata;
        use std::collections::BTreeMap;
        let time = sample_epoch_tai().into_dyn();
        let epoch = OdmTime::Time(time);
        let opm = Opm {
            header: OdmHeader {
                comments: Vec::new(),
                classification: None,
                creation_date: epoch,
                originator: "TEST".to_string(),
                message_id: None,
            },
            metadata: OpmMetadata {
                comments: Vec::new(),
                object_name: "APOPHIS".to_string(),
                object_id: "2004-XY".to_string(),
                center: OdmCenter::Custom("APOPHIS".to_string()),
                frame: OdmFrame::Known(DynFrame::Icrf),
                frame_epoch: None,
            },
            epoch,
            state: sample_cartesian(),
            state_comments: Vec::new(),
            keplerian: None,
            spacecraft: None,
            covariance: None,
            maneuvers: Vec::new(),
            user_defined: BTreeMap::new(),
        };
        let err = DynCartesianOrbit::from_opm(&opm).unwrap_err();
        assert!(matches!(err, OpmFromOdmError::CustomCenter(_)));
    }

    // -------------------------------------------------------------------------
    // OEM tests
    // -------------------------------------------------------------------------

    #[test]
    fn to_oem_from_dyn_trajectory() {
        let oem = sample_dyn_trajectory()
            .build_oem("TEST-SAT", "2024-000A")
            .originator("TEST_ORG")
            .build()
            .unwrap();
        assert_eq!(oem.segments.len(), 1);
        let seg = &oem.segments[0];
        assert_eq!(seg.metadata.center, OdmCenter::Known(DynOrigin::Earth));
        assert_eq!(seg.metadata.frame, OdmFrame::Known(DynFrame::Icrf));
        assert_eq!(seg.states.len(), 3);
    }

    #[test]
    fn from_oem_reconstructs_dyn_trajectory() {
        let traj = sample_dyn_trajectory();
        let expected_len = traj.states().len();
        let oem = traj
            .build_oem("TEST-SAT", "2024-000A")
            .originator("TEST_ORG")
            .build()
            .unwrap();
        let reconstructed = DynTrajectory::from_oem(&oem).unwrap();
        assert_eq!(reconstructed.origin(), DynOrigin::Earth);
        assert_eq!(reconstructed.reference_frame(), DynFrame::Icrf);
        assert_eq!(reconstructed.states().len(), expected_len);
    }

    #[test]
    fn oem_builder_propagates_offset_provider_failure() {
        let err = sample_dyn_trajectory()
            .build_oem("SAT", "2024-000A")
            .offset_provider(FailingUt1Provider)
            .time_system(OdmTimeScale::Ut1)
            .build()
            .unwrap_err();
        assert!(matches!(
            err,
            OemBuildError::TimeSystem(OdmTimeError::OffsetUnavailable { to: "UT1", .. })
        ));
    }

    #[test]
    fn from_oem_segment_specific_segment() {
        use lox_odm::types::oem::OemMetadata;
        use lox_time::deltas::TimeDelta;
        let t0 = sample_epoch_tai().into_dyn();
        let t1 = t0 + TimeDelta::from_seconds(60);
        let epoch0 = OdmTime::Time(t0);
        let epoch1 = OdmTime::Time(t1);
        let state = sample_cartesian();
        let seg = OemSegment {
            metadata: OemMetadata {
                comments: Vec::new(),
                object_name: "SAT".to_string(),
                object_id: "2024-000A".to_string(),
                center: OdmCenter::Known(DynOrigin::Earth),
                frame: OdmFrame::Known(DynFrame::Icrf),
                frame_epoch: None,
                start_time: epoch0,
                useable_start_time: None,
                useable_stop_time: None,
                stop_time: epoch1,
                interpolation: None,
                interpolation_degree: None,
            },
            data_comments: Vec::new(),
            states: vec![(epoch0, state), (epoch1, state)],
            covariance_history: Vec::new(),
        };
        let traj = DynTrajectory::from_oem_segment(&seg).unwrap();
        assert_eq!(traj.states().len(), 2);
        assert_eq!(traj.origin(), DynOrigin::Earth);
    }

    #[test]
    fn oem_builder_fluent_setters() {
        let traj = sample_dyn_trajectory();
        let start_time = orbit_epoch_to_odm_time(traj.start_time());
        let stop_time = orbit_epoch_to_odm_time(traj.end_time());
        let oem = traj
            .build_oem("SAT", "2024-000A")
            .originator("ORG")
            .header_comment("generated by lox")
            .metadata_comment("segment 1")
            .interpolation("HERMITE", 7)
            .useable_start_time(start_time)
            .useable_stop_time(stop_time)
            .build()
            .unwrap();
        let seg = &oem.segments[0];
        assert_eq!(seg.metadata.interpolation.as_deref(), Some("HERMITE"));
        assert_eq!(seg.metadata.interpolation_degree, Some(7));
        assert!(seg.metadata.useable_start_time.is_some());
    }
}
