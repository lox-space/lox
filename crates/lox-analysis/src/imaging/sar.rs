// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! SAR (Synthetic Aperture Radar) payload: side-looking annular access geometry.

use geo::Point;
use thiserror::Error;

use lox_core::coords::LonLatAlt;
use lox_core::units::Angle;

use crate::imaging::analysis::AccessPayload;
use crate::imaging::aoi::Aoi;

/// Which side of the ground track a SAR payload can image.
///
/// `Left` and `Right` are defined relative to the spacecraft's instantaneous
/// body-fixed velocity direction at the sub-satellite point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LookSide {
    /// Image only on the left side of the ground track.
    Left,
    /// Image only on the right side of the ground track.
    Right,
    /// Image on either side (roll-agile platform).
    Either,
}

/// Angular envelope of the SAR field of regard, stored in the convention
/// the caller constructed it with.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum AngleEnvelope {
    /// Look angle (off-nadir, measured at the satellite) in radians.
    Look { min_rad: f64, max_rad: f64 },
    /// Incidence angle (off-vertical, measured at the ground point) in radians.
    Incidence { min_rad: f64, max_rad: f64 },
}

/// Errors from constructing a [`SarPayload`].
#[derive(Debug, Error)]
pub enum SarPayloadError {
    /// Returned when the `min` argument is not strictly less than `max`.
    #[error("invalid angle range: min ({min}°) must be less than max ({max}°)")]
    InvalidAngleRange {
        /// The offending minimum angle in degrees.
        min: f64,
        /// The offending maximum angle in degrees.
        max: f64,
    },
    /// Returned when an angle is outside the valid `[0°, 90°)` range.
    #[error("angle must lie in [0°, 90°), got {0}°")]
    AngleOutOfRange(f64),
}

/// A SAR payload describing a side-looking annular access region.
///
/// Construct via [`SarPayload::with_look_angles`] (look angle at the satellite)
/// or [`SarPayload::with_incidence_angles`] (incidence angle at the ground point).
/// The chosen convention is preserved internally and converted at evaluation
/// time using the actual instantaneous altitude.
///
/// # Limitations
///
/// - **Spherical Earth.** Both look↔incidence conversion and ground-range
///   geometry use the body's mean radius. The error is negligible below ~30°
///   incidence and grows slowly at higher angles.
/// - **Large AOIs straddling the ground track.** The access metric evaluates
///   distance to the *nearest* AOI point. For an AOI wider than the inner
///   annulus radius that straddles the ground track, the nearest point sits
///   inside the inner forbidden ring → metric goes negative → no access is
///   reported, even though the far edge of the AOI may genuinely lie in the
///   annulus on one or both sides. Split such AOIs into smaller polygons or
///   use point targets.
/// - **No squint, modes, or acquisition-quality outputs.** A single envelope
///   per payload; no per-mode swath, NESZ, or resolution. See the project
///   docs for the planned future scope.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SarPayload {
    envelope: AngleEnvelope,
    side: LookSide,
}

impl SarPayload {
    /// Constructs a SAR payload from a look-angle envelope (off-nadir, at the satellite).
    pub fn with_look_angles(
        min: Angle,
        max: Angle,
        side: LookSide,
    ) -> Result<Self, SarPayloadError> {
        let (min_rad, max_rad) = validate_range(min, max)?;
        Ok(Self {
            envelope: AngleEnvelope::Look { min_rad, max_rad },
            side,
        })
    }

    /// Constructs a SAR payload from an incidence-angle envelope (off-vertical, at the ground).
    pub fn with_incidence_angles(
        min: Angle,
        max: Angle,
        side: LookSide,
    ) -> Result<Self, SarPayloadError> {
        let (min_rad, max_rad) = validate_range(min, max)?;
        Ok(Self {
            envelope: AngleEnvelope::Incidence { min_rad, max_rad },
            side,
        })
    }

    /// Returns the configured looking side.
    pub fn side(&self) -> LookSide {
        self.side
    }

    /// Returns `(r_min, r_max)` — the ground-range bounds of the access annulus,
    /// in metres, for the given altitude and mean radius.
    fn ground_range_bounds(&self, altitude_m: f64, mean_radius_m: f64) -> (f64, f64) {
        let (min_look_rad, max_look_rad) = match self.envelope {
            AngleEnvelope::Look { min_rad, max_rad } => (min_rad, max_rad),
            AngleEnvelope::Incidence { min_rad, max_rad } => (
                incidence_to_look(min_rad, altitude_m, mean_radius_m),
                incidence_to_look(max_rad, altitude_m, mean_radius_m),
            ),
        };
        (
            look_to_ground_range(min_look_rad, altitude_m, mean_radius_m),
            look_to_ground_range(max_look_rad, altitude_m, mean_radius_m),
        )
    }
}

impl AccessPayload for SarPayload {
    fn access_metric(
        &self,
        sub_sat: LonLatAlt,
        ground_track_az: Angle,
        aoi: &Aoi,
        mean_radius_m: f64,
    ) -> f64 {
        let altitude_m = sub_sat.alt().to_meters();
        let (r_min, r_max) = self.ground_range_bounds(altitude_m, mean_radius_m);

        let sub_sat_lon = sub_sat.lon().to_degrees();
        let sub_sat_lat = sub_sat.lat().to_degrees();
        let sub_sat_point = Point::new(sub_sat_lon, sub_sat_lat);

        let (nearest, r) = aoi.nearest_point_and_distance(&sub_sat_point, mean_radius_m);

        let annulus_marg = (r - r_min).min(r_max - r);

        let side_marg = match self.side {
            LookSide::Either => f64::INFINITY,
            LookSide::Left | LookSide::Right => {
                let bearing = bearing_from_to(sub_sat_lon, sub_sat_lat, nearest.x(), nearest.y());
                let diff = bearing - ground_track_az.to_radians();
                // sin(diff) > 0 → target on right of ground track; < 0 → on left.
                // sin(diff) == 0 in two cases, both correctly handled by sign = 0
                // → side_marg = 0 → target excluded from Left- or Right-only payloads:
                //   * diff = 0:   target directly ahead on the ground track.
                //   * diff = ±π:  target directly behind on the ground track.
                let sign = diff.sin().signum();
                let signed_r = r * sign;
                match self.side {
                    LookSide::Right => signed_r,
                    LookSide::Left => -signed_r,
                    LookSide::Either => unreachable!(),
                }
            }
        };

        annulus_marg.min(side_marg)
    }
}

/// γ(θ) = arcsin(sin(θ) · (R + h) / R) − θ; ground_range = R · γ.
/// Clamps to a hemisphere if the look ray geometrically misses the body.
fn look_to_ground_range(look_rad: f64, altitude_m: f64, mean_radius_m: f64) -> f64 {
    let s = look_rad.sin() * (mean_radius_m + altitude_m) / mean_radius_m;
    if s >= 1.0 {
        return core::f64::consts::FRAC_PI_2 * mean_radius_m;
    }
    let gamma = s.asin() - look_rad;
    mean_radius_m * gamma
}

/// sin(i) = sin(θ) · (R + h) / R → θ = asin(sin(i) · R / (R + h)).
fn incidence_to_look(incidence_rad: f64, altitude_m: f64, mean_radius_m: f64) -> f64 {
    let s = incidence_rad.sin() * mean_radius_m / (mean_radius_m + altitude_m);
    s.clamp(-1.0, 1.0).asin()
}

/// Great-circle initial bearing from `from` to `to`, both lon/lat degrees.
/// Returns radians measured from north, clockwise, normalised to [0, 2π).
fn bearing_from_to(from_lon_deg: f64, from_lat_deg: f64, to_lon_deg: f64, to_lat_deg: f64) -> f64 {
    let lat1 = from_lat_deg.to_radians();
    let lat2 = to_lat_deg.to_radians();
    let dlon = (to_lon_deg - from_lon_deg).to_radians();
    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    let raw = y.atan2(x);
    let two_pi = core::f64::consts::TAU;
    ((raw % two_pi) + two_pi) % two_pi
}

fn validate_range(min: Angle, max: Angle) -> Result<(f64, f64), SarPayloadError> {
    let min_deg = min.to_degrees();
    let max_deg = max.to_degrees();
    for &deg in &[min_deg, max_deg] {
        if !(0.0..90.0).contains(&deg) {
            return Err(SarPayloadError::AngleOutOfRange(deg));
        }
    }
    if min_deg >= max_deg {
        return Err(SarPayloadError::InvalidAngleRange {
            min: min_deg,
            max: max_deg,
        });
    }
    Ok((min.to_radians(), max.to_radians()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_look_angles_valid() {
        let p = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(45.0),
            LookSide::Right,
        )
        .unwrap();
        assert_eq!(p.side(), LookSide::Right);
    }

    #[test]
    fn with_incidence_angles_valid() {
        let p = SarPayload::with_incidence_angles(
            Angle::degrees(22.0),
            Angle::degrees(46.0),
            LookSide::Either,
        )
        .unwrap();
        assert_eq!(p.side(), LookSide::Either);
    }

    #[test]
    fn rejects_inverted_range() {
        let err = SarPayload::with_look_angles(
            Angle::degrees(45.0),
            Angle::degrees(20.0),
            LookSide::Left,
        )
        .unwrap_err();
        assert!(matches!(err, SarPayloadError::InvalidAngleRange { .. }));
    }

    #[test]
    fn rejects_negative_angle() {
        let err = SarPayload::with_incidence_angles(
            Angle::degrees(-5.0),
            Angle::degrees(45.0),
            LookSide::Right,
        )
        .unwrap_err();
        assert!(matches!(err, SarPayloadError::AngleOutOfRange(_)));
    }

    #[test]
    fn rejects_at_or_above_90() {
        let err = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(90.0),
            LookSide::Right,
        )
        .unwrap_err();
        assert!(matches!(err, SarPayloadError::AngleOutOfRange(_)));
    }

    #[test]
    fn rejects_equal_min_and_max() {
        let err = SarPayload::with_look_angles(
            Angle::degrees(30.0),
            Angle::degrees(30.0),
            LookSide::Right,
        )
        .unwrap_err();
        assert!(matches!(err, SarPayloadError::InvalidAngleRange { .. }));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    use std::collections::HashMap;

    use geo::{LineString, Polygon};

    use lox_bodies::DynOrigin;
    use lox_frames::DynFrame;
    use lox_orbits::orbits::{DynTrajectory, Ensemble};
    use lox_orbits::propagators::OrbitSource;
    use lox_orbits::propagators::Propagator;
    use lox_orbits::propagators::sgp4::{Elements, Sgp4};
    use lox_time::deltas::TimeDelta;
    use lox_time::intervals::{Interval, TimeInterval};
    use lox_time::time_scales::{DynTimeScale, Tai};

    use crate::assets::{AssetId, DynScenario, Spacecraft};
    use crate::imaging::analysis::SarAccessAnalysis;
    use crate::imaging::aoi::{Aoi, AoiId};

    // Sentinel-1A TLE — epoch 2026-079 (20 March 2026), consistent with the
    // Sentinel-2 TLEs used in the optical integration tests.
    const S1A_NAME: &str = "SENTINEL-1A";
    const S1A_LINE1: &[u8] =
        b"1 39634U 14016A   26079.20000000  .00000050  00000+0  37000-4 0  9991";
    const S1A_LINE2: &[u8] =
        b"2 39634  98.1817 105.0000 0001300  90.0000 270.0000 14.59197557600008";

    fn s1a_trajectory() -> DynTrajectory {
        let tle = Elements::from_tle(Some(S1A_NAME.to_string()), S1A_LINE1, S1A_LINE2).unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let t0 = sgp4.time();
        let t1 = t0 + TimeDelta::from_hours(6);
        sgp4.with_step(TimeDelta::from_seconds(10))
            .propagate(Interval::new(t0, t1))
            .unwrap()
            .into_dyn()
    }

    fn western_europe_aoi() -> Aoi {
        Aoi::new(Polygon::new(
            LineString::from(vec![
                (-10.0, 35.0),
                (20.0, 35.0),
                (20.0, 60.0),
                (-10.0, 60.0),
                (-10.0, 35.0),
            ]),
            vec![],
        ))
    }

    fn make_scenario(
        spacecraft: &[Spacecraft],
        interval: TimeInterval<DynTimeScale>,
    ) -> (DynScenario, Ensemble<AssetId, Tai, DynOrigin, DynFrame>) {
        let tai_interval =
            TimeInterval::new(interval.start().to_scale(Tai), interval.end().to_scale(Tai));
        let scenario = DynScenario::with_interval(tai_interval, DynOrigin::Earth, DynFrame::Icrf)
            .with_spacecraft(spacecraft);
        let mut map = HashMap::new();
        for sc in spacecraft {
            if let OrbitSource::Trajectory(traj) = sc.orbit() {
                let (epoch, origin, frame, data) = traj.clone().into_parts();
                let typed = lox_orbits::orbits::Trajectory::from_parts(
                    epoch.with_scale(Tai),
                    origin,
                    frame,
                    data,
                );
                map.insert(sc.id().clone(), typed);
            }
        }
        (scenario, Ensemble::new(map))
    }

    #[test]
    fn sentinel1_over_europe_produces_windows() {
        let traj = s1a_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());

        // Sentinel-1 IW mode: incidence ~29°–46°, right-looking.
        let payload = SarPayload::with_incidence_angles(
            Angle::degrees(29.0),
            Angle::degrees(46.0),
            LookSide::Right,
        )
        .unwrap();

        let sc = Spacecraft::new("s1a", OrbitSource::Trajectory(traj)).with_sar_payload(payload);

        let (scenario, ensemble) = make_scenario(std::slice::from_ref(&sc), interval);
        let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

        let results = SarAccessAnalysis::new(&scenario, &ensemble, aois)
            .with_step(TimeDelta::from_seconds(30))
            .compute()
            .expect("SAR access analysis failed");

        let windows = results.intervals(&AssetId::new("s1a"), &AoiId::new("europe"));
        assert!(
            !windows.is_empty(),
            "expected at least one access window over Western Europe in 6h",
        );
        for w in windows {
            let dur = (w.end() - w.start()).to_seconds().to_f64();
            assert!(dur > 0.0, "zero-length window");
            assert!(
                dur < 600.0,
                "SAR access window {dur:.0}s exceeds plausible 600s LEO pass",
            );
        }
    }

    #[test]
    fn left_vs_right_side_differ_over_asymmetric_aoi() {
        let traj = s1a_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());

        let right = SarPayload::with_incidence_angles(
            Angle::degrees(29.0),
            Angle::degrees(46.0),
            LookSide::Right,
        )
        .unwrap();
        let left = SarPayload::with_incidence_angles(
            Angle::degrees(29.0),
            Angle::degrees(46.0),
            LookSide::Left,
        )
        .unwrap();

        let sc_r =
            Spacecraft::new("s1a_r", OrbitSource::Trajectory(traj.clone())).with_sar_payload(right);
        let sc_l = Spacecraft::new("s1a_l", OrbitSource::Trajectory(traj)).with_sar_payload(left);

        let (scenario, ensemble) = make_scenario(&[sc_r, sc_l], interval);
        let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

        let results = SarAccessAnalysis::new(&scenario, &ensemble, aois)
            .with_step(TimeDelta::from_seconds(30))
            .compute()
            .expect("SAR access analysis failed");

        let r_windows = results.intervals(&AssetId::new("s1a_r"), &AoiId::new("europe"));
        let l_windows = results.intervals(&AssetId::new("s1a_l"), &AoiId::new("europe"));

        assert!(
            !r_windows.is_empty() && !l_windows.is_empty(),
            "expected non-empty access on both sides over Europe",
        );

        // Sides should see different opportunities: at least one window on one
        // side must not overlap any window on the other. Robust to TLE refreshes
        // (unlike a sum-of-durations check, which can coincidentally match).
        let overlaps = |a: &TimeInterval<Tai>, b: &TimeInterval<Tai>| -> bool {
            a.start() < b.end() && b.start() < a.end()
        };
        let left_has_unique = l_windows
            .iter()
            .any(|l| !r_windows.iter().any(|r| overlaps(l, r)));
        let right_has_unique = r_windows
            .iter()
            .any(|r| !l_windows.iter().any(|l| overlaps(r, l)));
        assert!(
            left_has_unique || right_has_unique,
            "every Left window overlaps a Right window and vice versa — sides not differentiated",
        );
    }
}

#[cfg(test)]
mod metric_tests {
    use super::*;

    use geo::{LineString, Polygon};

    use crate::imaging::analysis::AccessPayload;
    use crate::imaging::aoi::Aoi;

    const EARTH_R_M: f64 = 6_371_000.0;

    /// Tiny ~10-m polygon around (lon_deg, lat_deg) — treated as a point target.
    fn point_aoi(lon_deg: f64, lat_deg: f64) -> Aoi {
        let d = 1e-4;
        Aoi::new(Polygon::new(
            LineString::from(vec![
                (lon_deg - d, lat_deg - d),
                (lon_deg + d, lat_deg - d),
                (lon_deg + d, lat_deg + d),
                (lon_deg - d, lat_deg + d),
                (lon_deg - d, lat_deg - d),
            ]),
            vec![],
        ))
    }

    fn sub_sat() -> LonLatAlt {
        LonLatAlt::from_degrees(0.0, 0.0, 500_000.0).unwrap()
    }

    #[test]
    fn either_side_target_in_annulus_yields_positive_metric() {
        // Heading east; target due east at ~400 km ground range (inside [≈200,≈700]).
        let payload = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(45.0),
            LookSide::Either,
        )
        .unwrap();
        let target = point_aoi(3.6, 0.0);
        let m = payload.access_metric(sub_sat(), Angle::degrees(90.0), &target, EARTH_R_M);
        assert!(m > 0.0, "expected positive metric (in annulus), got {m}");
    }

    #[test]
    fn either_side_target_too_close_yields_negative_metric() {
        // Target ~30 km east — inside the inner forbidden ring.
        let payload = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(45.0),
            LookSide::Either,
        )
        .unwrap();
        let target = point_aoi(0.27, 0.0);
        let m = payload.access_metric(sub_sat(), Angle::degrees(90.0), &target, EARTH_R_M);
        assert!(m < 0.0, "expected negative metric (too close), got {m}");
    }

    #[test]
    fn either_side_target_too_far_yields_negative_metric() {
        // Target ~1110 km east — outside the outer ring.
        let payload = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(45.0),
            LookSide::Either,
        )
        .unwrap();
        let target = point_aoi(10.0, 0.0);
        let m = payload.access_metric(sub_sat(), Angle::degrees(90.0), &target, EARTH_R_M);
        assert!(m < 0.0, "expected negative metric (too far), got {m}");
    }

    #[test]
    fn right_side_target_on_wrong_side_yields_negative_metric() {
        // Heading east → "right" is south; target due NORTH is on the wrong side.
        let payload = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(45.0),
            LookSide::Right,
        )
        .unwrap();
        let target = point_aoi(0.0, 3.6);
        let m = payload.access_metric(sub_sat(), Angle::degrees(90.0), &target, EARTH_R_M);
        assert!(m < 0.0, "expected negative metric (wrong side), got {m}");
    }

    #[test]
    fn right_side_target_on_correct_side_yields_positive_metric() {
        // Heading east → "right" is south; target due SOUTH, inside annulus.
        let payload = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(45.0),
            LookSide::Right,
        )
        .unwrap();
        let target = point_aoi(0.0, -3.6);
        let m = payload.access_metric(sub_sat(), Angle::degrees(90.0), &target, EARTH_R_M);
        assert!(
            m > 0.0,
            "expected positive metric (right side, in annulus), got {m}"
        );
    }

    #[test]
    fn incidence_envelope_agrees_with_equivalent_look_envelope() {
        // h=500 km, R=6371 km: look=20° ≈ incidence 21.6°; look=45° ≈ incidence 50.4°.
        let look_pl = SarPayload::with_look_angles(
            Angle::degrees(20.0),
            Angle::degrees(45.0),
            LookSide::Either,
        )
        .unwrap();
        let inc_pl = SarPayload::with_incidence_angles(
            Angle::degrees(21.6),
            Angle::degrees(50.4),
            LookSide::Either,
        )
        .unwrap();
        for target_lon_deg in [3.0, 4.0, 5.0, 6.0] {
            let target = point_aoi(target_lon_deg, 0.0);
            let m_look = look_pl.access_metric(sub_sat(), Angle::degrees(90.0), &target, EARTH_R_M);
            let m_inc = inc_pl.access_metric(sub_sat(), Angle::degrees(90.0), &target, EARTH_R_M);
            assert_eq!(
                m_look.signum(),
                m_inc.signum(),
                "sign mismatch at lon={target_lon_deg}°: look={m_look}, inc={m_inc}",
            );
        }
    }
}
