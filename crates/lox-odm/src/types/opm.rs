// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Orbit Parameter Message (OPM) typed data model.
//!
//! Mirrors CCSDS 502.0-B-3 §3 — a single Cartesian state vector at one
//! epoch, optionally extended with osculating Keplerian elements,
//! spacecraft physical properties, a covariance matrix, and a list of
//! maneuvers.

use std::collections::BTreeMap;

use lox_core::coords::Cartesian;
use lox_core::elements::Keplerian;
use lox_core::time::deltas::TimeDelta;
use lox_core::units::{Mass, Velocity};
use lox_orbits::orbits::DynCartesianOrbit;
use lox_time::time::DynTime;
use nalgebra::Matrix6;

use crate::types::common::{
    CustomBodyOrFrameError, OdmCenter, OdmFrame, OdmHeader, SpacecraftParameters,
};

/// Per-message metadata for the OPM (CCSDS 502.0-B-3 §3.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpmMetadata {
    /// `OBJECT_NAME` — human-readable spacecraft name.
    pub object_name: String,
    /// `OBJECT_ID` — international designator (e.g. `2020-003C`).
    pub object_id: String,
    /// `CENTER_NAME` — center body of the reference frame.
    pub center: OdmCenter,
    /// `REF_FRAME` — reference frame of the state vector.
    pub frame: OdmFrame,
    /// `REF_FRAME_EPOCH` — optional epoch at which the (rotating) frame
    /// is realised. Required only for rotating frames per §3.3.1.5.
    pub frame_epoch: Option<DynTime>,
}

/// Optional 6×6 Cartesian-state covariance carried by an OPM (CCSDS
/// 502.0-B-3 §3.5).
///
/// The matrix layout is the standard (X, Y, Z, X_DOT, Y_DOT, Z_DOT)
/// covariance; the 21 unique upper-triangle values from the wire
/// (`CX_X`, `CY_X`, `CY_Y`, …) populate a full symmetric `Matrix6<f64>`
/// when read.
#[derive(Clone, Debug, PartialEq)]
pub struct OpmCovariance {
    /// `COV_REF_FRAME` — optional frame override; when `None` the
    /// covariance is in the same frame as the state vector.
    pub frame: Option<OdmFrame>,
    /// The 6×6 covariance matrix.
    pub matrix: Matrix6<f64>,
}

/// A single orbital maneuver carried by an OPM (CCSDS 502.0-B-3 §3.6).
///
/// Either impulsive (`duration == TimeDelta::from_seconds(0)`) or
/// finite-burn. The delta-v vector is expressed in the frame named by
/// `frame` if present, else in the OPM's state-vector frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Maneuver {
    /// `MAN_EPOCH_IGNITION` — epoch at which the maneuver starts.
    pub ignition_epoch: DynTime,
    /// `MAN_DURATION` — burn duration; zero for impulsive maneuvers.
    pub duration: TimeDelta,
    /// `MAN_DELTA_MASS` — change in spacecraft mass (typically negative
    /// for propellant expenditure).
    pub delta_mass: Mass,
    /// `MAN_REF_FRAME` — optional frame in which the delta-v is
    /// expressed; when `None`, the OPM state-vector frame is used.
    pub frame: Option<OdmFrame>,
    /// `MAN_DV_1`/`MAN_DV_2`/`MAN_DV_3` — delta-v components.
    pub delta_v: [Velocity; 3],
}

/// The Orbit Parameter Message (OPM, CCSDS 502.0-B-3 §3).
///
/// A single Cartesian state vector at one epoch, optionally extended with
/// osculating Keplerian elements, spacecraft physical properties, a 6×6
/// covariance, and a list of maneuvers. Round-trip-faithful to KVN, XML,
/// and (planned) wire-format adapters.
#[derive(Clone, Debug, PartialEq)]
pub struct Opm {
    /// Common ODM header.
    pub header: OdmHeader,
    /// OPM-specific metadata (object id, center, frame).
    pub metadata: OpmMetadata,
    /// State-vector epoch.
    pub epoch: DynTime,
    /// Cartesian state (position and velocity in the metadata frame).
    pub state: Cartesian,
    /// Optional osculating Keplerian element section (CCSDS §3.4.2).
    pub keplerian: Option<Keplerian>,
    /// Optional spacecraft physical properties.
    pub spacecraft: Option<SpacecraftParameters>,
    /// Optional 6×6 state covariance.
    pub covariance: Option<OpmCovariance>,
    /// Zero or more maneuvers.
    pub maneuvers: Vec<Maneuver>,
    /// User-defined parameters (preserved verbatim for round-trip).
    pub user_defined: BTreeMap<String, String>,
}

impl Opm {
    /// Upgrades the OPM to a fully-typed [`DynCartesianOrbit`].
    ///
    /// Fails with [`CustomBodyOrFrameError`] when the message's center
    /// body or reference frame is `Custom(_)` and therefore not
    /// representable as `DynOrigin` / `DynFrame`.
    pub fn try_into_orbit(&self) -> Result<DynCartesianOrbit, CustomBodyOrFrameError> {
        let origin = self.metadata.center.known().ok_or_else(|| {
            CustomBodyOrFrameError::Body(self.metadata.center.name().into_owned())
        })?;
        let frame = self.metadata.frame.known().ok_or_else(|| {
            CustomBodyOrFrameError::Frame(self.metadata.frame.name().into_owned())
        })?;
        Ok(DynCartesianOrbit::from_state(
            self.state, self.epoch, origin, frame,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_bodies::DynOrigin;
    use lox_core::units::Distance;
    use lox_frames::DynFrame;

    #[test]
    fn maneuver_impulsive() {
        let epoch = lox_time::time::Time::j2000(lox_time::time_scales::DynTimeScale::Tai);
        let m = Maneuver {
            ignition_epoch: epoch,
            duration: TimeDelta::from_seconds(0),
            delta_mass: Mass::kilograms(-1.5),
            frame: None,
            delta_v: [
                Velocity::meters_per_second(10.0),
                Velocity::meters_per_second(0.0),
                Velocity::meters_per_second(-5.0),
            ],
        };
        assert_eq!(m.duration.seconds(), Some(0));
        assert_eq!(m.delta_mass.to_kilograms(), -1.5);
        assert_eq!(m.delta_v[0].to_meters_per_second(), 10.0);
    }

    #[test]
    fn opm_metadata_construction() {
        let m = OpmMetadata {
            object_name: "ISS".to_string(),
            object_id: "1998-067A".to_string(),
            center: OdmCenter::Known(DynOrigin::Earth),
            frame: OdmFrame::Known(DynFrame::Icrf),
            frame_epoch: None,
        };
        assert_eq!(m.object_name, "ISS");
        assert_eq!(m.center, OdmCenter::Known(DynOrigin::Earth));
        assert!(m.frame_epoch.is_none());
    }

    #[test]
    fn opm_covariance_construction() {
        let cov = OpmCovariance {
            frame: None,
            matrix: Matrix6::identity(),
        };
        assert!(cov.frame.is_none());
        assert_eq!(cov.matrix[(0, 0)], 1.0);
        assert_eq!(cov.matrix[(0, 1)], 0.0);
    }

    #[test]
    fn opm_covariance_with_frame_override() {
        let cov = OpmCovariance {
            frame: Some(OdmFrame::Known(DynFrame::Itrf)),
            matrix: Matrix6::zeros(),
        };
        assert_eq!(cov.frame, Some(OdmFrame::Known(DynFrame::Itrf)));
        assert_eq!(cov.matrix[(5, 5)], 0.0);
    }

    fn sample_opm(center: OdmCenter, frame: OdmFrame) -> Opm {
        let epoch = lox_time::time::Time::j2000(lox_time::time_scales::DynTimeScale::Tai);
        Opm {
            header: crate::types::common::OdmHeader {
                comments: Vec::new(),
                classification: None,
                creation_date: epoch,
                originator: "TEST".to_string(),
                message_id: None,
            },
            metadata: OpmMetadata {
                object_name: "TEST-SAT".to_string(),
                object_id: "2024-000A".to_string(),
                center,
                frame,
                frame_epoch: None,
            },
            epoch,
            state: Cartesian::new(
                Distance::kilometers(7000.0),
                Distance::kilometers(0.0),
                Distance::kilometers(0.0),
                Velocity::kilometers_per_second(0.0),
                Velocity::kilometers_per_second(7.5),
                Velocity::kilometers_per_second(0.0),
            ),
            keplerian: None,
            spacecraft: None,
            covariance: None,
            maneuvers: Vec::new(),
            user_defined: BTreeMap::new(),
        }
    }

    #[test]
    fn opm_construction() {
        let opm = sample_opm(
            OdmCenter::Known(DynOrigin::Earth),
            OdmFrame::Known(DynFrame::Icrf),
        );
        assert_eq!(opm.metadata.object_name, "TEST-SAT");
        assert!(opm.maneuvers.is_empty());
        assert!(opm.user_defined.is_empty());
    }

    #[test]
    fn opm_try_into_orbit_succeeds_for_known() {
        let opm = sample_opm(
            OdmCenter::Known(DynOrigin::Earth),
            OdmFrame::Known(DynFrame::Icrf),
        );
        let orbit = opm.try_into_orbit().expect("known body and frame");
        assert_eq!(orbit.origin(), DynOrigin::Earth);
        assert_eq!(orbit.reference_frame(), DynFrame::Icrf);
    }

    #[test]
    fn opm_try_into_orbit_fails_for_custom_body() {
        let opm = sample_opm(
            OdmCenter::Custom("APOPHIS".to_string()),
            OdmFrame::Known(DynFrame::Icrf),
        );
        let err = opm.try_into_orbit().expect_err("custom body should fail");
        assert!(matches!(err, CustomBodyOrFrameError::Body(ref s) if s == "APOPHIS"));
    }

    #[test]
    fn opm_try_into_orbit_fails_for_custom_frame() {
        let opm = sample_opm(
            OdmCenter::Known(DynOrigin::Earth),
            OdmFrame::Custom("OPERATOR_LVLH".to_string()),
        );
        let err = opm.try_into_orbit().expect_err("custom frame should fail");
        assert!(matches!(err, CustomBodyOrFrameError::Frame(ref s) if s == "OPERATOR_LVLH"));
    }
}
