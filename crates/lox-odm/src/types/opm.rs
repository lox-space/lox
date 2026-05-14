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

use crate::types::common::{
    Covariance, OdmCenter, OdmFrame, OdmHeader, OdmTime, SpacecraftParameters,
};
use lox_bodies::TryPointMass;
use lox_core::coords::Cartesian;
use lox_core::elements::{GravitationalParameter, Keplerian};
use lox_core::time::deltas::TimeDelta;
use lox_core::units::{Mass, Velocity};

/// Per-message metadata for the OPM (CCSDS 502.0-B-3 §3.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpmMetadata {
    /// `COMMENT` lines for this sub-block, in document order.
    pub comments: Vec<String>,
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
    pub frame_epoch: Option<OdmTime>,
}

/// A single orbital maneuver carried by an OPM (CCSDS 502.0-B-3 §3.6).
///
/// Either impulsive (`duration == TimeDelta::from_seconds(0)`) or
/// finite-burn. The delta-v vector is expressed in the frame named by
/// `frame` if present, else in the OPM's state-vector frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Maneuver {
    /// `COMMENT` lines for this sub-block, in document order.
    pub comments: Vec<String>,
    /// `MAN_EPOCH_IGNITION` — epoch at which the maneuver starts.
    pub ignition_epoch: OdmTime,
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

/// OPM Keplerian-element block (CCSDS 502.0-B-3 §3.4.2).
///
/// Wraps the pure-physics [`Keplerian`] elements with the two
/// wire-specific decorations that appear in the Keplerian block on
/// the wire: an optional `GM` value and any `COMMENT` lines that
/// precede the block. Both decorations round-trip losslessly.
///
/// The `gm` field captures the operator's chosen gravitational
/// parameter exactly as written on the wire. It is preserved
/// regardless of whether the message's center is `Known` (where a
/// canonical GM is available via [`lox_bodies::Origin::gravitational_parameter`])
/// or `Custom`. When `None`, the message did not include a `GM`
/// field on the wire.
#[derive(Clone, Debug, PartialEq)]
pub struct OpmKeplerian {
    /// `COMMENT` lines for this sub-block, in document order.
    pub comments: Vec<String>,
    /// Osculating Keplerian elements (semi-major axis in meters,
    /// angles in radians).
    pub elements: Keplerian,
    /// `GM [km**3/s**2]` from the wire, stored in canonical m³/s².
    /// `None` when the wire did not specify GM.
    pub gm: Option<GravitationalParameter>,
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
    pub epoch: OdmTime,
    /// Cartesian state (position and velocity in the metadata frame).
    pub state: Cartesian,
    /// `COMMENT` lines that appear immediately before the state vector
    /// block (between metadata and `EPOCH`).
    pub state_comments: Vec<String>,
    /// Optional osculating Keplerian element section (CCSDS §3.4.2).
    pub keplerian: Option<OpmKeplerian>,
    /// Optional spacecraft physical properties.
    pub spacecraft: Option<SpacecraftParameters>,
    /// Optional 6×6 state covariance.
    pub covariance: Option<Covariance>,
    /// Zero or more maneuvers.
    pub maneuvers: Vec<Maneuver>,
    /// User-defined parameters (preserved verbatim for round-trip).
    pub user_defined: BTreeMap<String, String>,
}

impl Opm {
    /// Returns a gravitational parameter for this OPM, preferring the
    /// operator's wire `GM` (stored on [`OpmKeplerian::gm`]) and
    /// falling back to the canonical body GM via
    /// [`TryPointMass::try_gravitational_parameter`].
    ///
    /// Returns `None` only when both are unavailable — i.e. the wire
    /// did not include `GM` *and* the center is [`OdmCenter::Custom`]
    /// or the body has no defined gravitational parameter.
    pub fn gm(&self) -> Option<GravitationalParameter> {
        self.keplerian.as_ref().and_then(|k| k.gm).or_else(|| {
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
    use lox_core::elements::GravitationalParameter;
    use lox_core::units::{Angle, Distance};
    use lox_frames::DynFrame;
    use nalgebra::Matrix6;

    #[test]
    fn maneuver_impulsive() {
        let epoch = OdmTime::Time(lox_time::time::Time::j2000(
            lox_time::time_scales::DynTimeScale::Tai,
        ));
        let m = Maneuver {
            comments: Vec::new(),
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
            comments: Vec::new(),
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
        let cov = Covariance {
            comments: Vec::new(),
            frame: None,
            matrix: Matrix6::identity(),
        };
        assert!(cov.frame.is_none());
        assert_eq!(cov.matrix[(0, 0)], 1.0);
        assert_eq!(cov.matrix[(0, 1)], 0.0);
    }

    #[test]
    fn opm_covariance_with_frame_override() {
        let cov = Covariance {
            comments: Vec::new(),
            frame: Some(OdmFrame::Known(DynFrame::Itrf)),
            matrix: Matrix6::zeros(),
        };
        assert_eq!(cov.frame, Some(OdmFrame::Known(DynFrame::Itrf)));
        assert_eq!(cov.matrix[(5, 5)], 0.0);
    }

    fn sample_opm(center: OdmCenter, frame: OdmFrame) -> Opm {
        let time = lox_time::time::Time::j2000(lox_time::time_scales::DynTimeScale::Tai);
        let epoch = OdmTime::Time(time);
        Opm {
            header: crate::types::common::OdmHeader {
                comments: Vec::new(),
                classification: None,
                creation_date: epoch,
                originator: "TEST".to_string(),
                message_id: None,
            },
            metadata: OpmMetadata {
                comments: Vec::new(),
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
            state_comments: Vec::new(),
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
    fn opm_gm_prefers_wire_value() {
        let mut opm = sample_opm(
            OdmCenter::Known(DynOrigin::Earth),
            OdmFrame::Known(DynFrame::Icrf),
        );
        let wire_gm = GravitationalParameter::km3_per_s2(398600.4415);
        let elements = Keplerian::builder()
            .with_semi_major_axis(Distance::kilometers(7000.0), 0.001)
            .with_inclination(Angle::radians(0.9))
            .with_longitude_of_ascending_node(Angle::ZERO)
            .with_argument_of_periapsis(Angle::ZERO)
            .with_true_anomaly(Angle::ZERO)
            .build()
            .expect("valid Keplerian elements");
        opm.keplerian = Some(OpmKeplerian {
            comments: Vec::new(),
            elements,
            gm: Some(wire_gm),
        });
        assert_eq!(opm.gm(), Some(wire_gm));
    }

    #[test]
    fn opm_gm_falls_back_to_canonical_for_known_center() {
        use lox_bodies::TryPointMass;
        let opm = sample_opm(
            OdmCenter::Known(DynOrigin::Earth),
            OdmFrame::Known(DynFrame::Icrf),
        );
        let expected = DynOrigin::Earth.try_gravitational_parameter().ok();
        assert_eq!(opm.gm(), expected);
        assert!(opm.gm().is_some());
    }

    #[test]
    fn opm_gm_returns_none_for_custom_center_without_wire_gm() {
        let opm = sample_opm(
            OdmCenter::Custom("APOPHIS".to_string()),
            OdmFrame::Known(DynFrame::Icrf),
        );
        assert_eq!(opm.gm(), None);
    }
}
