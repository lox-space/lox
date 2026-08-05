// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Access-analysis traits: payload metric and payload accessor.

use lox_core::glam::DVec3;
use thiserror::Error;

use crate::events::{DetectError, DetectFn};
use lox_bodies::{CoordinateOrigin, TryMeanRadius, TrySpheroid};
use lox_core::coords::LonLatAlt;
use lox_core::units::Angle;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_frames::{Frame, ReferenceFrame};
use lox_orbits::orbits::Trajectory;
use lox_time::Time;
use lox_time::time_scales::TimeScale;

use crate::imaging::aoi::Aoi;
use crate::imaging::results::PassDirection;
use crate::visibility::EvalError;

/// Returns the per-sample access metric for an AOI.
///
/// Sign convention: positive when the AOI is accessible at this geometry,
/// negative when not. Continuous across the access boundary so that a
/// root finder can locate entry/exit times. Infallible.
//
// TODO(refactor): the current trait shape (driver eagerly computes everything
// any sensor might need, then passes it through) is patchwork. The
// `needs_ground_track_azimuth` opt-out and `Aoi::nearest_point_and_distance`
// helper exist to keep per-sample cost down without changing this API. Replace
// with a pull-based `&AccessContext` carrying memoised accessors for sub-sat,
// altitude, mean radius, ground-track azimuth, AOI distance/nearest point,
// etc. — each sensor pulls only what it needs and any future derived quantity
// goes in one place. See the spec/plan for he/sar (deferred for prototype).
pub trait AccessPayload {
    /// Returns the access metric for the given sub-satellite point and AOI.
    fn access_metric(
        &self,
        sub_sat: LonLatAlt,
        ground_track_az: Angle,
        aoi: &Aoi,
        mean_radius_m: f64,
    ) -> f64;

    /// Returns `true` if [`Self::access_metric`] depends on the ground-track
    /// azimuth. When `false`, the driver skips the per-sample azimuth
    /// computation and passes a zero placeholder. Defaults to `true` for
    /// safety; sensors that only depend on sub-satellite geometry (e.g. a
    /// nadir-centred disk) should override to `false`.
    fn needs_ground_track_azimuth(&self) -> bool {
        true
    }
}

/// Extension trait letting a generic access analysis fetch a payload of type
/// `P` from any type that may carry one.
pub trait PayloadAccessor<P>
where
    P: Copy,
{
    /// Returns the payload, or `None` if no payload of type `P` is installed.
    fn extract(&self) -> Option<P>;
}

// ---------------------------------------------------------------------------
// AccessError
// ---------------------------------------------------------------------------

/// Errors from a generic access analysis run.
#[derive(Debug, Error)]
pub enum AccessError {
    /// Event detection failed.
    #[error(transparent)]
    Detect(#[from] DetectError),
    /// Pass-direction sampling failed (state interpolation / frame rotation).
    #[error("pass-direction sampling failed: {0}")]
    PassDirection(#[from] EvalError),
}

// ---------------------------------------------------------------------------
// ground_track_azimuth helper
// ---------------------------------------------------------------------------

/// Ground-track azimuth (from north, clockwise, in [0, 2π)) of a body-fixed
/// velocity vector at a sub-satellite point.
fn ground_track_azimuth(sub_sat: LonLatAlt, vel_bf: DVec3) -> Angle {
    // SEZ frame from `rotation_to_topocentric()`: x = south, y = east, z = zenith.
    let r_to_sez = sub_sat.rotation_to_topocentric();
    let v_sez = r_to_sez * vel_bf;
    // Azimuth from north, clockwise — north component = -south component.
    let azimuth_rad = v_sez.y.atan2(-v_sez.x);
    let two_pi = core::f64::consts::TAU;
    let normalized = ((azimuth_rad % two_pi) + two_pi) % two_pi;
    Angle::radians(normalized)
}

// ---------------------------------------------------------------------------
// SubSatSample helper
// ---------------------------------------------------------------------------

/// A single per-time sample of the spacecraft state in the body-fixed frame,
/// pre-resolved into the quantities every per-sample computation needs.
pub(crate) struct SubSatSample {
    lla: LonLatAlt,
    vel_bf: DVec3,
    mean_radius_m: f64,
}

/// Computes a [`SubSatSample`] from a trajectory at a given time. Centralises
/// the state-interpolation → body-fixed-rotation → LLA pipeline used by both
/// [`AccessDetectFn::eval`] (per-sample detection) and pass-direction sampling
/// (per-window post-detection).
pub(crate) fn sub_sat_sample<O, R>(
    trajectory: &Trajectory<O, R>,
    time: Time,
    origin: O,
    body_fixed_frame: Frame,
) -> Result<SubSatSample, EvalError>
where
    O: TrySpheroid + TryMeanRadius + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        core::error::Error + Send + Sync + 'static,
{
    let state = trajectory.at(time.into_dynamic());
    let state_bf = state
        .try_to_frame(body_fixed_frame, &DefaultRotationProvider)
        .map_err(|e| EvalError::Rotation(Box::new(e)))?;
    let pos = state_bf.position();
    let vel_bf = state_bf.velocity();
    let ellipsoid = origin.try_ellipsoid().map_err(EvalError::from)?;
    let mean_radius_m = origin
        .try_mean_radius()
        .map_err(EvalError::from)?
        .to_meters();
    let lla = LonLatAlt::from_body_fixed(pos, &ellipsoid)
        .map_err(|e| EvalError::Rotation(Box::new(e)))?;
    Ok(SubSatSample {
        lla,
        vel_bf,
        mean_radius_m,
    })
}

/// Classifies the orbital motion at the given sub-satellite sample as
/// [`PassDirection::Ascending`] (moving northward) or
/// [`PassDirection::Descending`] (moving southward).
///
/// Uses the sign of the SEZ-north component of the body-fixed velocity. Ties
/// (zero north-component — measure-zero in practice) resolve to `Ascending`.
pub(crate) fn pass_direction_of(sample: &SubSatSample) -> PassDirection {
    let r_to_sez = sample.lla.rotation_to_topocentric();
    let v_sez = r_to_sez * sample.vel_bf;
    // SEZ.x is south; north component = -SEZ.x. Strict positive → Ascending.
    if -v_sez.x >= 0.0 {
        PassDirection::Ascending
    } else {
        PassDirection::Descending
    }
}

// ---------------------------------------------------------------------------
// AccessDetectFn
// ---------------------------------------------------------------------------

pub(crate) struct AccessDetectFn<'a, P: AccessPayload, O: CoordinateOrigin, R: ReferenceFrame> {
    pub(crate) payload: P,
    pub(crate) aoi: &'a Aoi,
    pub(crate) trajectory: &'a Trajectory<O, R>,
    pub(crate) origin: O,
    pub(crate) body_fixed_frame: Frame,
}

impl<P, O, R> DetectFn for AccessDetectFn<'_, P, O, R>
where
    P: AccessPayload + Copy,
    O: TrySpheroid + TryMeanRadius + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, Frame, TimeScale>,
    <DefaultRotationProvider as TryRotation<R, Frame, TimeScale>>::Error:
        core::error::Error + Send + Sync + 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time) -> Result<f64, Self::Error> {
        let sample = sub_sat_sample(self.trajectory, time, self.origin, self.body_fixed_frame)?;
        let az = if self.payload.needs_ground_track_azimuth() {
            ground_track_azimuth(sample.lla, sample.vel_bf)
        } else {
            Angle::default()
        };
        Ok(self
            .payload
            .access_metric(sample.lla, az, self.aoi, sample.mean_radius_m))
    }
}

// The eager `AccessAnalysis` and its aggregate `AccessResults` are gone; this is
// the pipeline-backed replacement at the canonical path.
pub use crate::pipeline::analyses::{AccessAnalysis, OpticalAccessAnalysis, SarAccessAnalysis};

#[cfg(test)]
mod tests {

    use super::*;

    use geo::{LineString, Polygon};

    #[derive(Copy, Clone)]
    struct ConstPayload(f64);

    impl AccessPayload for ConstPayload {
        fn access_metric(
            &self,
            _sub_sat: LonLatAlt,
            _ground_track_az: Angle,
            _aoi: &Aoi,
            _mean_radius_m: f64,
        ) -> f64 {
            self.0
        }
    }

    #[test]
    fn const_payload_returns_constant_metric() {
        let aoi = Aoi::new(Polygon::new(
            LineString::from(vec![
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 1.0),
                (0.0, 1.0),
                (0.0, 0.0),
            ]),
            vec![],
        ));
        let lla = LonLatAlt::from_degrees(0.0, 0.0, 500_000.0).unwrap();
        let p = ConstPayload(42.0);
        assert_eq!(
            p.access_metric(lla, Angle::degrees(0.0), &aoi, 6_371_000.0),
            42.0,
        );
    }

    // At sub-sat (lon=0, lat=0), the body-fixed frame aligns with:
    //   ECEF X → up (zenith);  ECEF Y → east;  ECEF Z → north
    // so a body-fixed velocity in the +Y direction is purely eastward,
    // and +Z is purely northward.
    #[test]
    fn ground_track_azimuth_northward_velocity_is_zero() {
        let sub_sat = LonLatAlt::from_degrees(0.0, 0.0, 500_000.0).unwrap();
        let v_north = DVec3::new(0.0, 0.0, 1.0);
        let az = ground_track_azimuth(sub_sat, v_north);
        assert!(az.to_radians().abs() < 1e-9, "expected ≈ 0, got {az}");
    }

    #[test]
    fn ground_track_azimuth_eastward_velocity_is_pi_over_two() {
        let sub_sat = LonLatAlt::from_degrees(0.0, 0.0, 500_000.0).unwrap();
        let v_east = DVec3::new(0.0, 1.0, 0.0);
        let az = ground_track_azimuth(sub_sat, v_east);
        let expected = core::f64::consts::FRAC_PI_2;
        assert!(
            (az.to_radians() - expected).abs() < 1e-9,
            "expected π/2, got {az}",
        );
    }

    #[test]
    fn ground_track_azimuth_normalised_to_positive_range() {
        // Southward velocity → azimuth π (180°), well inside [0, 2π).
        let sub_sat = LonLatAlt::from_degrees(0.0, 0.0, 500_000.0).unwrap();
        let v_south = DVec3::new(0.0, 0.0, -1.0);
        let az = ground_track_azimuth(sub_sat, v_south);
        let expected = core::f64::consts::PI;
        assert!(
            (az.to_radians() - expected).abs() < 1e-9,
            "expected π, got {az}",
        );
    }

    #[test]
    fn pass_direction_ascending_for_northward_velocity() {
        let sample = SubSatSample {
            lla: LonLatAlt::from_degrees(0.0, 0.0, 500_000.0).unwrap(),
            vel_bf: DVec3::new(0.0, 0.0, 1.0), // ECEF +Z is north at the equator
            mean_radius_m: 6_371_000.0,
        };
        assert_eq!(pass_direction_of(&sample), PassDirection::Ascending);
    }

    #[test]
    fn pass_direction_descending_for_southward_velocity() {
        let sample = SubSatSample {
            lla: LonLatAlt::from_degrees(0.0, 0.0, 500_000.0).unwrap(),
            vel_bf: DVec3::new(0.0, 0.0, -1.0),
            mean_radius_m: 6_371_000.0,
        };
        assert_eq!(pass_direction_of(&sample), PassDirection::Descending);
    }

    #[test]
    fn pass_direction_pure_eastward_is_ascending_by_tiebreak() {
        let sample = SubSatSample {
            lla: LonLatAlt::from_degrees(0.0, 0.0, 500_000.0).unwrap(),
            vel_bf: DVec3::new(0.0, 1.0, 0.0), // pure east, zero north
            mean_radius_m: 6_371_000.0,
        };
        assert_eq!(pass_direction_of(&sample), PassDirection::Ascending);
    }
}
