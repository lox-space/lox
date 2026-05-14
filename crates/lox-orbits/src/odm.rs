// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! ODM integration: builders and conversion helpers for OPM/OEM messages.
//!
//! Requires the `odm` feature flag.

use std::collections::BTreeMap;
use std::path::Path;

use lox_bodies::{DynOrigin, Origin};
use lox_core::coords::Cartesian;
use lox_frames::{DynFrame, ReferenceFrame};
use lox_odm::Format;
use lox_odm::OdmError;
use lox_odm::types::common::{OdmCenter, OdmFrame, OdmHeader, OdmTime, SpacecraftParameters};
use lox_odm::types::oem::{Oem, OemMetadata, OemSegment};
use lox_odm::types::opm::{Opm, OpmMetadata};
use lox_time::time::DynTime;
use lox_time::time_scales::{DynTimeScale, TimeScale};

use crate::orbits::{CartesianOrbit, DynCartesianOrbit, DynTrajectory, Orbit, Trajectory};

// ----------------------------------------------------------------------------
// Helper: convert a `Time<T>` (where T: Into<DynTimeScale>) to `OdmTime`
// ----------------------------------------------------------------------------

fn orbit_epoch_to_odm_time<T>(time: lox_time::Time<T>) -> OdmTime
where
    T: TimeScale + Copy + Into<DynTimeScale>,
{
    OdmTime::Time(time.into_dyn())
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

// ----------------------------------------------------------------------------
// OpmBuilder
// ----------------------------------------------------------------------------

/// Fluent builder for constructing an [`Opm`] from a typed orbit.
///
/// Required fields (`originator`, `object_name`, `object_id`) are provided up
/// front; all optional CCSDS header/metadata fields are set via chained
/// method calls. Call [`build`](OpmBuilder::build) (or a `write_*` variant)
/// to consume the builder.
pub struct OpmBuilder {
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
}

impl OpmBuilder {
    /// Creates a new builder with the three CCSDS-mandatory identification
    /// fields. All other fields default to `None` / empty.
    pub fn new(
        originator: impl Into<String>,
        object_name: impl Into<String>,
        object_id: impl Into<String>,
    ) -> Self {
        Self {
            originator: originator.into(),
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
        }
    }

    /// Sets `CREATION_DATE`. Defaults to the orbit's epoch when not set.
    pub fn creation_date(mut self, t: OdmTime) -> Self {
        self.creation_date = Some(t);
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

    /// Builds the [`Opm`], consuming the builder.
    ///
    /// `T`/`O`/`R` are inferred from the orbit. The orbit's epoch, state,
    /// center, and frame are used to populate the message. If
    /// `creation_date` was not set, it defaults to the orbit's epoch.
    pub fn build<T, O, R>(self, orbit: &Orbit<Cartesian, T, O, R>) -> Opm
    where
        T: TimeScale + Copy + Into<DynTimeScale>,
        O: Origin + Copy + Into<DynOrigin>,
        R: ReferenceFrame + Copy + Into<DynFrame>,
    {
        let epoch = orbit_epoch_to_odm_time(orbit.time());
        let creation_date = self.creation_date.unwrap_or(epoch);
        let center = OdmCenter::from_wire(orbit.origin().name());
        let frame = OdmFrame::from_wire(&orbit.reference_frame().abbreviation());

        Opm {
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
                frame_epoch: self.frame_epoch,
            },
            epoch,
            state: orbit.state(),
            state_comments: self.state_comments,
            keplerian: None,
            spacecraft: self.spacecraft,
            covariance: None,
            maneuvers: Vec::new(),
            user_defined: self.user_defined,
        }
    }

    /// Builds the OPM and serializes it to the requested wire format.
    pub fn write_str<T, O, R>(
        self,
        orbit: &Orbit<Cartesian, T, O, R>,
        format: Format,
    ) -> Result<String, OdmError>
    where
        T: TimeScale + Copy + Into<DynTimeScale>,
        O: Origin + Copy + Into<DynOrigin>,
        R: ReferenceFrame + Copy + Into<DynFrame>,
    {
        let opm = self.build(orbit);
        lox_odm::write_opm(&opm, format)
    }

    /// Builds the OPM and writes it to a file in the requested format.
    pub fn write_file<T, O, R>(
        self,
        orbit: &Orbit<Cartesian, T, O, R>,
        path: impl AsRef<Path>,
        format: Format,
    ) -> Result<(), OdmError>
    where
        T: TimeScale + Copy + Into<DynTimeScale>,
        O: Origin + Copy + Into<DynOrigin>,
        R: ReferenceFrame + Copy + Into<DynFrame>,
    {
        let opm = self.build(orbit);
        lox_odm::write_opm_file(&opm, path, format)
    }
}

// ----------------------------------------------------------------------------
// Orbit inherent methods (generic blanket — works for both static and Dyn)
// ----------------------------------------------------------------------------

impl<T, O, R> Orbit<Cartesian, T, O, R>
where
    T: TimeScale + Copy + Into<DynTimeScale>,
    O: Origin + Copy + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Into<DynFrame>,
{
    /// Converts this orbit into an OPM using the provided builder configuration.
    pub fn to_opm(&self, builder: OpmBuilder) -> Opm {
        builder.build(self)
    }

    /// Builds the OPM and serializes it to the requested wire format.
    pub fn to_opm_str(&self, builder: OpmBuilder, format: Format) -> Result<String, OdmError> {
        builder.write_str(self, format)
    }

    /// Builds the OPM and writes it to a file in the requested format.
    pub fn to_opm_file(
        &self,
        builder: OpmBuilder,
        path: impl AsRef<Path>,
        format: Format,
    ) -> Result<(), OdmError> {
        builder.write_file(self, path, format)
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
/// Required fields (`originator`, `object_name`, `object_id`) are provided
/// up front. All optional CCSDS fields are set via chained method calls.
pub struct OemBuilder {
    originator: String,
    object_name: String,
    object_id: String,
    creation_date: Option<OdmTime>,
    message_id: Option<String>,
    header_comments: Vec<String>,
    metadata_comments: Vec<String>,
    data_comments: Vec<String>,
    frame_epoch: Option<OdmTime>,
    useable_start_time: Option<OdmTime>,
    useable_stop_time: Option<OdmTime>,
    interpolation: Option<String>,
    interpolation_degree: Option<u64>,
    user_defined: BTreeMap<String, String>,
}

impl OemBuilder {
    /// Creates a new builder with the three CCSDS-mandatory identification
    /// fields.
    pub fn new(
        originator: impl Into<String>,
        object_name: impl Into<String>,
        object_id: impl Into<String>,
    ) -> Self {
        Self {
            originator: originator.into(),
            object_name: object_name.into(),
            object_id: object_id.into(),
            creation_date: None,
            message_id: None,
            header_comments: Vec::new(),
            metadata_comments: Vec::new(),
            data_comments: Vec::new(),
            frame_epoch: None,
            useable_start_time: None,
            useable_stop_time: None,
            interpolation: None,
            interpolation_degree: None,
            user_defined: BTreeMap::new(),
        }
    }

    /// Sets `CREATION_DATE`. Defaults to the trajectory's start epoch when not set.
    pub fn creation_date(mut self, t: OdmTime) -> Self {
        self.creation_date = Some(t);
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

    /// Builds the [`Oem`], consuming the builder.
    ///
    /// The trajectory's epoch (start/stop), origin, and frame populate the
    /// single segment's metadata. State vectors are taken from the
    /// trajectory's knot points via [`Trajectory::states`].
    pub fn build<T, O, R>(self, trajectory: &Trajectory<T, O, R>) -> Oem
    where
        T: TimeScale + Copy + Into<DynTimeScale>,
        O: Origin + Copy + Into<DynOrigin>,
        R: ReferenceFrame + Copy + Into<DynFrame>,
    {
        let start_time = orbit_epoch_to_odm_time(trajectory.start_time());
        let stop_time = orbit_epoch_to_odm_time(trajectory.end_time());
        let creation_date = self.creation_date.unwrap_or(start_time);
        let center = OdmCenter::from_wire(trajectory.origin().name());
        let frame = OdmFrame::from_wire(&trajectory.reference_frame().abbreviation());

        let states: Vec<(OdmTime, Cartesian)> = trajectory
            .states()
            .into_iter()
            .map(|s| (orbit_epoch_to_odm_time(s.time()), s.state()))
            .collect();

        let segment = OemSegment {
            metadata: OemMetadata {
                comments: self.metadata_comments,
                object_name: self.object_name,
                object_id: self.object_id,
                center,
                frame,
                frame_epoch: self.frame_epoch,
                start_time,
                useable_start_time: self.useable_start_time,
                useable_stop_time: self.useable_stop_time,
                stop_time,
                interpolation: self.interpolation,
                interpolation_degree: self.interpolation_degree,
            },
            data_comments: self.data_comments,
            states,
            covariance_history: Vec::new(),
        };

        Oem {
            header: OdmHeader {
                comments: self.header_comments,
                classification: None,
                creation_date,
                originator: self.originator,
                message_id: self.message_id,
            },
            segments: vec![segment],
            user_defined: self.user_defined,
        }
    }

    /// Builds the OEM and serializes it to the requested wire format.
    pub fn write_str<T, O, R>(
        self,
        trajectory: &Trajectory<T, O, R>,
        format: Format,
    ) -> Result<String, OdmError>
    where
        T: TimeScale + Copy + Into<DynTimeScale>,
        O: Origin + Copy + Into<DynOrigin>,
        R: ReferenceFrame + Copy + Into<DynFrame>,
    {
        let oem = self.build(trajectory);
        lox_odm::write_oem(&oem, format)
    }

    /// Builds the OEM and writes it to a file in the requested format.
    pub fn write_file<T, O, R>(
        self,
        trajectory: &Trajectory<T, O, R>,
        path: impl AsRef<Path>,
        format: Format,
    ) -> Result<(), OdmError>
    where
        T: TimeScale + Copy + Into<DynTimeScale>,
        O: Origin + Copy + Into<DynOrigin>,
        R: ReferenceFrame + Copy + Into<DynFrame>,
    {
        let oem = self.build(trajectory);
        lox_odm::write_oem_file(&oem, path, format)
    }
}

// ----------------------------------------------------------------------------
// Trajectory inherent methods (generic — static or Dyn)
// ----------------------------------------------------------------------------

impl<T, O, R> Trajectory<T, O, R>
where
    T: TimeScale + Copy + Into<DynTimeScale>,
    O: Origin + Copy + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Into<DynFrame>,
{
    /// Converts this trajectory into an OEM using the provided builder
    /// configuration.
    pub fn to_oem(&self, builder: OemBuilder) -> Oem {
        builder.build(self)
    }

    /// Builds the OEM and serializes it to the requested wire format.
    pub fn to_oem_str(&self, builder: OemBuilder, format: Format) -> Result<String, OdmError> {
        builder.write_str(self, format)
    }

    /// Builds the OEM and writes it to a file in the requested format.
    pub fn to_oem_file(
        &self,
        builder: OemBuilder,
        path: impl AsRef<Path>,
        format: Format,
    ) -> Result<(), OdmError> {
        builder.write_file(self, path, format)
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
    use lox_time::time_scales::Tai;
    use lox_time::{Time, time};

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
        let builder = OpmBuilder::new("TEST_ORG", "TEST-SAT", "2024-000A");
        let opm = orbit.to_opm(builder);
        assert_eq!(opm.metadata.object_name, "TEST-SAT");
        assert_eq!(opm.metadata.object_id, "2024-000A");
        assert_eq!(opm.header.originator, "TEST_ORG");
        assert_eq!(opm.metadata.center, OdmCenter::Known(DynOrigin::Earth));
        assert_eq!(opm.metadata.frame, OdmFrame::Known(DynFrame::Icrf));
        assert_eq!(opm.state, sample_cartesian());
    }

    #[test]
    fn to_opm_from_static_orbit() {
        let orbit = sample_static_orbit();
        let builder = OpmBuilder::new("STATIC_ORG", "STATIC-SAT", "2024-001A");
        let opm = orbit.to_opm(builder);
        assert_eq!(opm.metadata.center, OdmCenter::Known(DynOrigin::Earth));
        assert_eq!(opm.metadata.frame, OdmFrame::Known(DynFrame::Icrf));
        assert_eq!(opm.state, sample_cartesian());
    }

    #[test]
    fn to_opm_str_round_trip_kvn() {
        let orbit = sample_dyn_orbit();
        let builder = OpmBuilder::new("TEST_ORG", "TEST-SAT", "2024-000A");
        let kvn = orbit.to_opm_str(builder, Format::Kvn).unwrap();
        let parsed = lox_odm::read_opm(&kvn).unwrap();
        assert_eq!(parsed.metadata.object_name, "TEST-SAT");
        assert_eq!(parsed.state, orbit.state());
    }

    #[test]
    fn from_opm_reconstructs_dyn_orbit() {
        let orbit = sample_dyn_orbit();
        let builder = OpmBuilder::new("TEST_ORG", "TEST-SAT", "2024-000A");
        let opm = orbit.to_opm(builder);
        let reconstructed = DynCartesianOrbit::from_opm(&opm).unwrap();
        assert_eq!(reconstructed.origin(), DynOrigin::Earth);
        assert_eq!(reconstructed.reference_frame(), DynFrame::Icrf);
        assert_eq!(reconstructed.state(), orbit.state());
    }

    #[test]
    fn from_opm_str_kvn() {
        let orbit = sample_dyn_orbit();
        let builder = OpmBuilder::new("TEST_ORG", "TEST-SAT", "2024-000A");
        let kvn = orbit.to_opm_str(builder, Format::Kvn).unwrap();
        let reconstructed = DynCartesianOrbit::from_opm_str(&kvn).unwrap();
        assert_eq!(reconstructed.state(), orbit.state());
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
        let traj = sample_dyn_trajectory();
        let builder = OemBuilder::new("TEST_ORG", "TEST-SAT", "2024-000A");
        let oem = traj.to_oem(builder);
        assert_eq!(oem.segments.len(), 1);
        let seg = &oem.segments[0];
        assert_eq!(seg.metadata.center, OdmCenter::Known(DynOrigin::Earth));
        assert_eq!(seg.metadata.frame, OdmFrame::Known(DynFrame::Icrf));
        assert_eq!(seg.states.len(), 3);
    }

    #[test]
    fn from_oem_reconstructs_dyn_trajectory() {
        let traj = sample_dyn_trajectory();
        let builder = OemBuilder::new("TEST_ORG", "TEST-SAT", "2024-000A");
        let oem = traj.to_oem(builder);
        let reconstructed = DynTrajectory::from_oem(&oem).unwrap();
        assert_eq!(reconstructed.origin(), DynOrigin::Earth);
        assert_eq!(reconstructed.reference_frame(), DynFrame::Icrf);
        assert_eq!(reconstructed.states().len(), traj.states().len());
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
        let builder = OemBuilder::new("ORG", "SAT", "2024-000A")
            .header_comment("generated by lox")
            .metadata_comment("segment 1")
            .interpolation("HERMITE", 7)
            .useable_start_time(start_time)
            .useable_stop_time(stop_time);
        let oem = traj.to_oem(builder);
        let seg = &oem.segments[0];
        assert_eq!(seg.metadata.interpolation.as_deref(), Some("HERMITE"));
        assert_eq!(seg.metadata.interpolation_degree, Some(7));
        assert!(seg.metadata.useable_start_time.is_some());
    }
}
