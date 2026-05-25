// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Access-analysis traits: payload metric and payload accessor.

use core::marker::PhantomData;
use std::collections::HashMap;

use lox_core::glam::DVec3;
use rayon::prelude::*;
use thiserror::Error;

use lox_bodies::{DynOrigin, Origin, TryMeanRadius, TrySpheroid};
use lox_core::coords::LonLatAlt;
use lox_core::units::Angle;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_frames::{DynFrame, ReferenceFrame};
use lox_orbits::events::{
    DetectError, DetectFn, EventsToIntervals, IntervalDetector, RootFindingDetector,
};
use lox_orbits::orbits::{Ensemble, Trajectory};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::time_scales::Tai;

use crate::assets::{AssetId, Scenario, Spacecraft};
use crate::imaging::aoi::{Aoi, AoiId};
use crate::imaging::optical::OpticalPayload;
use crate::imaging::results::{AccessResults, AccessWindow, PassDirection};
use crate::imaging::sar::SarPayload;
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
struct SubSatSample {
    lla: LonLatAlt,
    vel_bf: DVec3,
    mean_radius_m: f64,
}

/// Computes a [`SubSatSample`] from a trajectory at a given time. Centralises
/// the state-interpolation → body-fixed-rotation → LLA pipeline used by both
/// [`AccessDetectFn::eval`] (per-sample detection) and pass-direction sampling
/// (per-window post-detection).
fn sub_sat_sample<O, R>(
    trajectory: &Trajectory<Tai, O, R>,
    time: Time<Tai>,
    origin: O,
    body_fixed_frame: DynFrame,
) -> Result<SubSatSample, EvalError>
where
    O: TrySpheroid + TryMeanRadius + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, DynFrame, Tai>,
    <DefaultRotationProvider as TryRotation<R, DynFrame, Tai>>::Error:
        core::error::Error + Send + Sync + 'static,
{
    let state = trajectory.interpolate_at(time);
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
fn pass_direction_of(sample: &SubSatSample) -> PassDirection {
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

struct AccessDetectFn<'a, P: AccessPayload, O: Origin, R: ReferenceFrame> {
    payload: P,
    aoi: &'a Aoi,
    trajectory: &'a Trajectory<Tai, O, R>,
    origin: O,
    body_fixed_frame: DynFrame,
}

impl<P, O, R> DetectFn<Tai> for AccessDetectFn<'_, P, O, R>
where
    P: AccessPayload + Copy,
    O: TrySpheroid + TryMeanRadius + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, DynFrame, Tai>,
    <DefaultRotationProvider as TryRotation<R, DynFrame, Tai>>::Error:
        core::error::Error + Send + Sync + 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time<Tai>) -> Result<f64, Self::Error> {
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

// ---------------------------------------------------------------------------
// AccessAnalysis orchestrator
// ---------------------------------------------------------------------------

/// Generic access analysis: computes per-(spacecraft, AOI) windows for spacecraft
/// carrying a payload of type `P`.
pub struct AccessAnalysis<'a, P, O: Origin, R: ReferenceFrame>
where
    P: AccessPayload + Copy + Send + Sync,
    Spacecraft: PayloadAccessor<P>,
{
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, Tai, O, R>,
    aois: Vec<(AoiId, Aoi)>,
    step: TimeDelta,
    body_fixed_frame: DynFrame,
    _marker: PhantomData<P>,
}

impl<'a, P, O, R> AccessAnalysis<'a, P, O, R>
where
    P: AccessPayload + Copy + Send + Sync,
    Spacecraft: PayloadAccessor<P>,
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Send + Sync + Into<DynFrame>,
    DefaultRotationProvider: TryRotation<R, DynFrame, Tai>,
    <DefaultRotationProvider as TryRotation<R, DynFrame, Tai>>::Error:
        core::error::Error + Send + Sync + 'static,
{
    /// Creates a new access analysis. The body-fixed frame defaults to the
    /// scenario origin's IAU frame.
    pub fn new(
        scenario: &'a Scenario<O, R>,
        ensemble: &'a Ensemble<AssetId, Tai, O, R>,
        aois: Vec<(AoiId, Aoi)>,
    ) -> Self {
        let body_fixed_frame = DynFrame::Iau(scenario.origin().into());
        Self {
            scenario,
            ensemble,
            aois,
            step: TimeDelta::from_seconds(60),
            body_fixed_frame,
            _marker: PhantomData,
        }
    }

    /// Overrides the time step for event detection (default 60 s).
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Overrides the body-fixed frame (default IAU of scenario origin).
    pub fn with_body_fixed_frame(mut self, frame: DynFrame) -> Self {
        self.body_fixed_frame = frame;
        self
    }

    /// Computes per-(spacecraft, AOI) access windows.
    pub fn compute(&self) -> Result<AccessResults, AccessError> {
        let interval = *self.scenario.interval();

        let with_payload: Vec<(&Spacecraft, P)> = self
            .scenario
            .spacecraft()
            .iter()
            .filter_map(|sc| <Spacecraft as PayloadAccessor<P>>::extract(sc).map(|p| (sc, p)))
            .collect();

        let pairs: Vec<(&Spacecraft, P, &(AoiId, Aoi))> = with_payload
            .iter()
            .flat_map(|&(sc, p)| self.aois.iter().map(move |aoi| (sc, p, aoi)))
            .collect();

        let compute_one = |&(sc, payload, (aoi_id, aoi)): &(&Spacecraft, P, &(AoiId, Aoi))| {
            let key = (sc.id().clone(), aoi_id.clone());
            let traj = self.ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );
            let detect_fn = AccessDetectFn {
                payload,
                aoi,
                trajectory: traj,
                origin: self.scenario.origin(),
                body_fixed_frame: self.body_fixed_frame,
            };
            let detector = RootFindingDetector::new(detect_fn, self.step);
            let intervals = EventsToIntervals::new(detector).detect(interval)?;
            let windows: Vec<AccessWindow> = intervals
                .into_iter()
                .map(|interval| AccessWindow {
                    interval,
                    // TODO(task-6): replace placeholder with pass_direction_of(...) sampled at midpoint
                    direction: PassDirection::Ascending,
                })
                .collect();
            Ok::<_, AccessError>((key, windows))
        };

        let results: Result<Vec<_>, AccessError> = if pairs.len() > 100 {
            pairs.par_iter().map(compute_one).collect()
        } else {
            pairs.iter().map(compute_one).collect()
        };

        let windows_by_pair: HashMap<_, _> = results?.into_iter().collect();
        Ok(AccessResults::new(windows_by_pair))
    }
}

/// Type alias for the optical access analysis (parameterised by [`OpticalPayload`]).
pub type OpticalAccessAnalysis<'a, O, R> = AccessAnalysis<'a, OpticalPayload, O, R>;

/// Type alias for the SAR access analysis (parameterised by [`SarPayload`]).
pub type SarAccessAnalysis<'a, O, R> = AccessAnalysis<'a, SarPayload, O, R>;

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
