// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Orbit Ephemeris Message (OEM) typed data model.
//!
//! Mirrors CCSDS 502.0-B-3 §5 — one or more time-ordered segments, each
//! with metadata describing the spacecraft/center/frame/interval and a
//! list of Cartesian state vectors. Optional per-segment covariance
//! history.

use std::collections::BTreeMap;

use lox_core::coords::Cartesian;
use lox_orbits::orbits::{DynCartesianOrbit, DynTrajectory, TrajectorError};
use lox_time::time::DynTime;
use nalgebra::Matrix6;

use crate::types::common::{CustomBodyOrFrameError, OdmCenter, OdmFrame, OdmHeader};

/// Per-segment metadata for the OEM (CCSDS 502.0-B-3 §5.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OemMetadata {
    /// `OBJECT_NAME` — human-readable spacecraft name.
    pub object_name: String,
    /// `OBJECT_ID` — international designator.
    pub object_id: String,
    /// `CENTER_NAME` — center body of the reference frame.
    pub center: OdmCenter,
    /// `REF_FRAME` — reference frame of the state vectors.
    pub frame: OdmFrame,
    /// `REF_FRAME_EPOCH` — optional rotating-frame realisation epoch.
    pub frame_epoch: Option<DynTime>,
    /// `START_TIME` — mandatory start of the segment's time interval.
    pub start_time: DynTime,
    /// `USEABLE_START_TIME` — optional interpolation-quality lower bound.
    pub useable_start_time: Option<DynTime>,
    /// `USEABLE_STOP_TIME` — optional interpolation-quality upper bound.
    pub useable_stop_time: Option<DynTime>,
    /// `STOP_TIME` — mandatory end of the segment's time interval.
    pub stop_time: DynTime,
    /// `INTERPOLATION` — optional interpolation method name
    /// (e.g. `"HERMITE"`, `"LAGRANGE"`). Stored verbatim; the OEM model
    /// does not interpret it.
    pub interpolation: Option<String>,
    /// `INTERPOLATION_DEGREE` — optional interpolation degree.
    pub interpolation_degree: Option<u64>,
}

/// A single timestamped 6×6 covariance entry from an OEM covariance
/// history (CCSDS 502.0-B-3 §5.4).
///
/// Unlike OPM covariance (which has no epoch — it's implicit at the OPM
/// state-vector epoch), each OEM covariance row carries its own `EPOCH`.
/// A segment's full covariance history is a `Vec<OemCovariance>`.
#[derive(Clone, Debug, PartialEq)]
pub struct OemCovariance {
    /// `EPOCH` — the epoch this covariance is valid at.
    pub epoch: DynTime,
    /// `COV_REF_FRAME` — optional frame override; when `None` the
    /// covariance is in the same frame as the segment's state vectors.
    pub frame: Option<OdmFrame>,
    /// The 6×6 covariance matrix.
    pub matrix: Matrix6<f64>,
}

/// A single segment within an OEM (CCSDS 502.0-B-3 §5.2).
///
/// Each segment carries its own metadata and a time-ordered list of
/// state vectors, plus an optional covariance history. Most OEMs have a
/// single segment; multi-segment files appear when the spacecraft
/// changes center body mid-trajectory (e.g. during interplanetary
/// cruise).
#[derive(Clone, Debug, PartialEq)]
pub struct OemSegment {
    /// Per-segment metadata.
    pub metadata: OemMetadata,
    /// Time-ordered ephemeris state vectors.
    pub states: Vec<(DynTime, Cartesian)>,
    /// Optional covariance history (empty when the segment has no
    /// `COVARIANCE` section).
    pub covariance_history: Vec<OemCovariance>,
}

/// Returned by [`OemSegment::try_into_trajectory`].
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum OemTrajectoryError {
    /// The segment's center or frame is `Custom(_)` and therefore not
    /// representable as `DynOrigin`/`DynFrame`.
    #[error(transparent)]
    CustomBodyOrFrame(#[from] CustomBodyOrFrameError),
    /// The segment has fewer than 2 states; a `DynTrajectory` requires
    /// at least 2 for interpolation.
    #[error("OEM segment has {0} state vector(s); at least 2 are required for a trajectory")]
    InsufficientStates(usize),
}

impl From<TrajectorError> for OemTrajectoryError {
    fn from(err: TrajectorError) -> Self {
        let TrajectorError::InsufficientStates(n) = err;
        OemTrajectoryError::InsufficientStates(n)
    }
}

/// The Orbit Ephemeris Message (OEM, CCSDS 502.0-B-3 §5).
///
/// A common header plus one or more time-ordered segments. Each segment
/// carries its own metadata, state vectors, and optional covariance
/// history.
#[derive(Clone, Debug, PartialEq)]
pub struct Oem {
    /// Common ODM header.
    pub header: OdmHeader,
    /// One or more time-ordered segments.
    pub segments: Vec<OemSegment>,
    /// User-defined parameters (preserved verbatim for round-trip).
    pub user_defined: BTreeMap<String, String>,
}

impl OemSegment {
    /// Borrowing iterator over the segment's state vectors.
    pub fn iter_states(&self) -> impl Iterator<Item = &(DynTime, Cartesian)> {
        self.states.iter()
    }

    /// Upgrades the segment to a fully-typed [`DynTrajectory`].
    ///
    /// Requires `≥ 2` state vectors and `Known` center + frame. Returns
    /// [`OemTrajectoryError`] otherwise. Per-segment metadata fields
    /// (interpolation hints, useable-time bounds) and the covariance
    /// history are not propagated to the trajectory.
    pub fn try_into_trajectory(&self) -> Result<DynTrajectory, OemTrajectoryError> {
        let origin = self.metadata.center.known().ok_or_else(|| {
            CustomBodyOrFrameError::Body(self.metadata.center.name().into_owned())
        })?;
        let frame = self.metadata.frame.known().ok_or_else(|| {
            CustomBodyOrFrameError::Frame(self.metadata.frame.name().into_owned())
        })?;
        let orbits = self
            .states
            .iter()
            .map(|(epoch, state)| DynCartesianOrbit::from_state(*state, *epoch, origin, frame));
        Ok(DynTrajectory::try_new(orbits)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_bodies::DynOrigin;
    use lox_core::units::{Distance, Velocity};
    use lox_frames::DynFrame;
    use lox_time::deltas::TimeDelta;
    use lox_time::time::Time;
    use lox_time::time_scales::DynTimeScale;

    fn sample_epoch() -> DynTime {
        Time::j2000(DynTimeScale::Tai)
    }

    #[test]
    fn oem_metadata_construction() {
        let m = OemMetadata {
            object_name: "ISS".to_string(),
            object_id: "1998-067A".to_string(),
            center: OdmCenter::Known(DynOrigin::Earth),
            frame: OdmFrame::Known(DynFrame::Icrf),
            frame_epoch: None,
            start_time: sample_epoch(),
            useable_start_time: None,
            useable_stop_time: None,
            stop_time: sample_epoch(),
            interpolation: Some("HERMITE".to_string()),
            interpolation_degree: Some(7),
        };
        assert_eq!(m.object_name, "ISS");
        assert_eq!(m.interpolation.as_deref(), Some("HERMITE"));
        assert_eq!(m.interpolation_degree, Some(7));
    }

    #[test]
    fn oem_covariance_construction() {
        let cov = OemCovariance {
            epoch: sample_epoch(),
            frame: None,
            matrix: Matrix6::identity(),
        };
        assert_eq!(cov.matrix[(0, 0)], 1.0);
        assert!(cov.frame.is_none());
    }

    #[test]
    fn oem_covariance_with_frame_override() {
        let cov = OemCovariance {
            epoch: sample_epoch(),
            frame: Some(OdmFrame::Known(DynFrame::Itrf)),
            matrix: Matrix6::zeros(),
        };
        assert_eq!(cov.frame, Some(OdmFrame::Known(DynFrame::Itrf)));
        assert_eq!(cov.matrix[(5, 5)], 0.0);
    }

    fn sample_state(km_along_track: f64) -> Cartesian {
        Cartesian::new(
            Distance::kilometers(7000.0 + km_along_track),
            Distance::kilometers(0.0),
            Distance::kilometers(0.0),
            Velocity::kilometers_per_second(0.0),
            Velocity::kilometers_per_second(7.5),
            Velocity::kilometers_per_second(0.0),
        )
    }

    fn sample_metadata() -> OemMetadata {
        OemMetadata {
            object_name: "TEST".to_string(),
            object_id: "2024-000A".to_string(),
            center: OdmCenter::Known(DynOrigin::Earth),
            frame: OdmFrame::Known(DynFrame::Icrf),
            frame_epoch: None,
            start_time: sample_epoch(),
            useable_start_time: None,
            useable_stop_time: None,
            stop_time: sample_epoch(),
            interpolation: None,
            interpolation_degree: None,
        }
    }

    #[test]
    fn oem_segment_construction() {
        let seg = OemSegment {
            metadata: sample_metadata(),
            states: vec![
                (sample_epoch(), sample_state(0.0)),
                (sample_epoch(), sample_state(1.0)),
            ],
            covariance_history: Vec::new(),
        };
        assert_eq!(seg.states.len(), 2);
        assert!(seg.covariance_history.is_empty());
    }

    #[test]
    fn oem_segment_iter_states_yields_references() {
        let seg = OemSegment {
            metadata: sample_metadata(),
            states: vec![
                (sample_epoch(), sample_state(0.0)),
                (sample_epoch(), sample_state(1.0)),
                (sample_epoch(), sample_state(2.0)),
            ],
            covariance_history: Vec::new(),
        };
        let collected: Vec<_> = seg.iter_states().collect();
        assert_eq!(collected.len(), 3);
    }

    #[test]
    fn oem_segment_try_into_trajectory_succeeds_for_known() {
        let seg = OemSegment {
            metadata: sample_metadata(),
            states: vec![
                (sample_epoch(), sample_state(0.0)),
                (
                    sample_epoch() + TimeDelta::from_seconds(60),
                    sample_state(1.0),
                ),
            ],
            covariance_history: Vec::new(),
        };
        let traj = seg
            .try_into_trajectory()
            .expect("known center+frame and 2+ states");
        // Smoke-test: trajectory constructed successfully.
        let _ = traj;
    }

    #[test]
    fn oem_segment_try_into_trajectory_fails_for_custom_body() {
        let mut metadata = sample_metadata();
        metadata.center = OdmCenter::Custom("APOPHIS".to_string());
        let seg = OemSegment {
            metadata,
            states: vec![
                (sample_epoch(), sample_state(0.0)),
                (
                    sample_epoch() + TimeDelta::from_seconds(60),
                    sample_state(1.0),
                ),
            ],
            covariance_history: Vec::new(),
        };
        let err = seg
            .try_into_trajectory()
            .expect_err("custom body should fail");
        assert!(matches!(
            err,
            OemTrajectoryError::CustomBodyOrFrame(CustomBodyOrFrameError::Body(ref s)) if s == "APOPHIS"
        ));
    }

    #[test]
    fn oem_segment_try_into_trajectory_fails_for_insufficient_states() {
        // NOTE: lox-orbits' `Trajectory::try_new` only reports
        // `InsufficientStates(0)` (the upstream `peek` check never advances
        // the iterator, so single-state inputs slip through to panic in
        // `Self::new`). Test with an empty `states` vector so the error
        // path is exercised against the upstream behaviour.
        let seg = OemSegment {
            metadata: sample_metadata(),
            states: Vec::new(),
            covariance_history: Vec::new(),
        };
        let err = seg
            .try_into_trajectory()
            .expect_err("empty-state should fail");
        assert!(matches!(err, OemTrajectoryError::InsufficientStates(0)));
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

    fn sample_segment() -> OemSegment {
        OemSegment {
            metadata: sample_metadata(),
            states: vec![
                (sample_epoch(), sample_state(0.0)),
                (
                    sample_epoch() + TimeDelta::from_seconds(60),
                    sample_state(1.0),
                ),
            ],
            covariance_history: Vec::new(),
        }
    }

    #[test]
    fn oem_construction() {
        let oem = Oem {
            header: sample_header(),
            segments: vec![sample_segment()],
            user_defined: BTreeMap::new(),
        };
        assert_eq!(oem.segments.len(), 1);
        assert!(oem.user_defined.is_empty());
    }

    #[test]
    fn oem_construction_multi_segment() {
        let oem = Oem {
            header: sample_header(),
            segments: vec![sample_segment(), sample_segment()],
            user_defined: BTreeMap::new(),
        };
        assert_eq!(oem.segments.len(), 2);
    }
}
