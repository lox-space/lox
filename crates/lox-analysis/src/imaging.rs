// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;
use std::fmt;

use geo::prelude::Contains;
use geo::{Closest, HaversineClosestPoint};

use lox_bodies::{DynOrigin, Origin, TryMeanRadius, TrySpheroid};
use lox_core::coords::LonLatAlt;
use lox_core::units::{Angle, Distance};
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::TryRotation;
use lox_frames::{DynFrame, ReferenceFrame};
use lox_orbits::events::{
    DetectError, DetectFn, EventsToIntervals, IntervalDetector, RootFindingDetector,
};
use lox_orbits::orbits::{Ensemble, Trajectory};
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::Tai;
use rayon::prelude::*;
use thiserror::Error;

use crate::assets::{AssetId, Scenario, Spacecraft};
use crate::visibility::EvalError;

/// Haversine great-circle distance between two lon/lat points (degrees) on a
/// sphere of the given radius (meters).
fn haversine_distance(a: geo::Point<f64>, b: geo::Point<f64>, radius_m: f64) -> f64 {
    let (lon1, lat1) = (a.x().to_radians(), a.y().to_radians());
    let (lon2, lat2) = (b.x().to_radians(), b.y().to_radians());
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;
    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * radius_m * h.sqrt().asin()
}

// ---------------------------------------------------------------------------
// AoiId
// ---------------------------------------------------------------------------

/// Unique identifier for an area of interest.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AoiId(String);

impl AoiId {
    /// Creates a new AOI identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AoiId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Aoi — Area of Interest
// ---------------------------------------------------------------------------

/// An area of interest defined as a geographic polygon.
///
/// Coordinates follow GeoJSON convention: longitude/latitude in degrees.
#[derive(Debug, Clone)]
pub struct Aoi {
    polygon: geo::Polygon<f64>,
}

/// Errors from AOI construction.
#[cfg(feature = "geojson")]
#[derive(Debug, Error)]
pub enum AoiError {
    /// Invalid GeoJSON string.
    #[error("invalid GeoJSON: {0}")]
    InvalidGeoJson(String),
    /// No polygon found in the GeoJSON.
    #[error("no polygon found in GeoJSON")]
    NoPolygon,
}

impl Aoi {
    /// Creates a new AOI from a `geo::Polygon` (lon/lat in degrees).
    pub fn new(polygon: geo::Polygon<f64>) -> Self {
        Self { polygon }
    }

    /// Parses an AOI from a GeoJSON string.
    ///
    /// Expects a GeoJSON Polygon geometry (or a Feature containing one).
    #[cfg(feature = "geojson")]
    pub fn from_geojson(geojson: &str) -> Result<Self, AoiError> {
        use geojson::GeoJson;
        use std::convert::TryInto;

        let gj: GeoJson = geojson
            .parse()
            .map_err(|e: geojson::Error| AoiError::InvalidGeoJson(e.to_string()))?;

        let geometry = match gj {
            GeoJson::Geometry(g) => g,
            GeoJson::Feature(f) => f.geometry.ok_or(AoiError::NoPolygon)?,
            GeoJson::FeatureCollection(fc) => fc
                .features
                .into_iter()
                .find_map(|f| f.geometry)
                .ok_or(AoiError::NoPolygon)?,
        };

        let polygon: geo::Polygon<f64> =
            geometry.value.try_into().map_err(|_| AoiError::NoPolygon)?;

        Ok(Self { polygon })
    }

    /// Returns the great-circle distance in meters from a point to the AOI polygon.
    ///
    /// Uses the haversine formula with the given `mean_radius_m` so that the
    /// computation is valid for any spherical body, not just Earth.
    /// Returns 0.0 if the point is inside the polygon.
    pub fn distance_to(&self, point: &geo::Point<f64>, mean_radius_m: f64) -> f64 {
        if self.polygon.contains(point) {
            0.0
        } else {
            match self.polygon.haversine_closest_point(point) {
                Closest::Intersection(_) => 0.0,
                Closest::SinglePoint(closest) => haversine_distance(*point, closest, mean_radius_m),
                Closest::Indeterminate => unreachable!("degenerate polygon geometry"),
            }
        }
    }

    /// Returns a reference to the underlying polygon.
    pub fn polygon(&self) -> &geo::Polygon<f64> {
        &self.polygon
    }
}

// ---------------------------------------------------------------------------
// ImagingPayload — Imaging sensor payload
// ---------------------------------------------------------------------------

/// Imaging sensor payload describing a spacecraft's ground coverage capability.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImagingPayload {
    half_swath_ground_range: f64,
    max_off_nadir: f64,
}

impl ImagingPayload {
    /// Creates imaging parameters for a nadir-only sensor.
    ///
    /// `swath_width` is the full swath width; internally stored as half.
    pub fn nadir_only(swath_width: Distance) -> Self {
        Self {
            half_swath_ground_range: swath_width.to_meters() / 2.0,
            max_off_nadir: 0.0,
        }
    }

    /// Creates imaging parameters for a sensor with off-nadir pointing.
    ///
    /// `swath_width` is the full swath width; `max_off_nadir` is the maximum
    /// off-nadir angle.
    pub fn off_nadir(swath_width: Distance, max_off_nadir: Angle) -> Self {
        Self {
            half_swath_ground_range: swath_width.to_meters() / 2.0,
            max_off_nadir: max_off_nadir.to_radians(),
        }
    }

    /// Computes the total accessible ground range in meters.
    ///
    /// Given altitude `h`, max off-nadir angle `θ`, Earth radius `R`:
    ///   γ = arcsin(sin(θ)·(R+h)/R) - θ
    ///   off-nadir ground range = R·γ
    ///
    /// Total range = off-nadir ground range + half_swath_ground_range.
    pub fn max_accessible_ground_range(&self, altitude_m: f64, mean_radius_m: f64) -> f64 {
        let off_nadir_range = if self.max_off_nadir > 0.0 {
            let theta = self.max_off_nadir;
            let sin_arg = theta.sin() * (mean_radius_m + altitude_m) / mean_radius_m;
            // Clamp to valid asin domain — if sin_arg >= 1.0, the satellite
            // can see the entire hemisphere.
            if sin_arg >= 1.0 {
                std::f64::consts::FRAC_PI_2 * mean_radius_m
            } else {
                let gamma = sin_arg.asin() - theta;
                mean_radius_m * gamma
            }
        } else {
            0.0
        };
        off_nadir_range + self.half_swath_ground_range
    }
}

// ---------------------------------------------------------------------------
// AoiImagingDetectFn — DetectFn implementation
// ---------------------------------------------------------------------------

/// Detection function for AOI imaging events.
///
/// Returns `max_accessible_ground_range - geodesic_distance(sub_sat, AOI)`.
/// Positive means the AOI is within the sensor's accessible area.
struct AoiImagingDetectFn<'a, O: Origin, R: ReferenceFrame> {
    aoi: &'a Aoi,
    params: ImagingPayload,
    trajectory: &'a Trajectory<Tai, O, R>,
    origin: O,
    body_fixed_frame: DynFrame,
}

impl<O, R> DetectFn<Tai> for AoiImagingDetectFn<'_, O, R>
where
    O: TrySpheroid + TryMeanRadius + Copy,
    R: ReferenceFrame + Copy,
    DefaultRotationProvider: TryRotation<R, DynFrame, Tai>,
    <DefaultRotationProvider as TryRotation<R, DynFrame, Tai>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    type Error = EvalError;

    fn eval(&self, time: Time<Tai>) -> Result<f64, Self::Error> {
        let state = self.trajectory.interpolate_at(time);
        let state_bf = state
            .try_to_frame(self.body_fixed_frame, &DefaultRotationProvider)
            .map_err(|e| EvalError::Rotation(Box::new(e)))?;

        let pos = state_bf.position();

        let eq_radius = self
            .origin
            .try_equatorial_radius()
            .map_err(EvalError::from)?;
        let flattening = self.origin.try_flattening().map_err(EvalError::from)?;
        let mean_radius = self
            .origin
            .try_mean_radius()
            .map_err(EvalError::from)?
            .to_meters();

        let lla = LonLatAlt::from_body_fixed(pos, eq_radius, flattening)
            .map_err(|e| EvalError::Rotation(Box::new(e)))?;

        let sub_sat_point = geo::Point::new(lla.lon().to_degrees(), lla.lat().to_degrees());

        let altitude_m = lla.alt().to_meters();
        let max_range = self
            .params
            .max_accessible_ground_range(altitude_m, mean_radius);
        let distance = self.aoi.distance_to(&sub_sat_point, mean_radius);

        Ok(max_range - distance)
    }
}

// ---------------------------------------------------------------------------
// ImagingError
// ---------------------------------------------------------------------------

/// Errors from imaging analysis computation.
#[derive(Debug, Error)]
pub enum ImagingError {
    /// Event detection failed.
    #[error(transparent)]
    Detect(#[from] DetectError),
}

// ---------------------------------------------------------------------------
// ImagingResults
// ---------------------------------------------------------------------------

type ImagingIntervalMap = HashMap<(AssetId, AoiId), Vec<TimeInterval<Tai>>>;

/// Results of an imaging analysis.
pub struct ImagingResults {
    intervals: ImagingIntervalMap,
}

impl ImagingResults {
    /// Returns imaging intervals for a specific (spacecraft, AOI) pair.
    pub fn intervals(&self, sc_id: &AssetId, aoi_id: &AoiId) -> &[TimeInterval<Tai>] {
        self.intervals
            .get(&(sc_id.clone(), aoi_id.clone()))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns an iterator over all (spacecraft, AOI) pairs and their intervals.
    pub fn all_intervals(&self) -> &ImagingIntervalMap {
        &self.intervals
    }

    /// Returns `true` if no imaging intervals were found.
    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    /// Returns the number of (spacecraft, AOI) pairs.
    pub fn num_pairs(&self) -> usize {
        self.intervals.len()
    }
}

// ---------------------------------------------------------------------------
// ImagingAnalysis — orchestration
// ---------------------------------------------------------------------------

/// AOI imaging analysis: computes imaging windows for spacecraft over AOIs.
///
/// Only spacecraft that have an [`ImagingPayload`] assigned are considered.
/// Generic over origin `O` and reference frame `R`.
pub struct ImagingAnalysis<'a, O: Origin, R: ReferenceFrame> {
    scenario: &'a Scenario<O, R>,
    ensemble: &'a Ensemble<AssetId, Tai, O, R>,
    aois: Vec<(AoiId, Aoi)>,
    step: TimeDelta,
    body_fixed_frame: DynFrame,
}

impl<'a, O, R> ImagingAnalysis<'a, O, R>
where
    O: TrySpheroid + TryMeanRadius + Copy + Send + Sync + Into<DynOrigin>,
    R: ReferenceFrame + Copy + Send + Sync + Into<DynFrame>,
    DefaultRotationProvider: TryRotation<R, DynFrame, Tai>,
    <DefaultRotationProvider as TryRotation<R, DynFrame, Tai>>::Error:
        std::error::Error + Send + Sync + 'static,
{
    /// Creates a new imaging analysis.
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
        }
    }

    /// Sets the time step for event detection.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = step;
        self
    }

    /// Overrides the body-fixed frame (defaults to IAU frame of the origin).
    pub fn with_body_fixed_frame(mut self, frame: DynFrame) -> Self {
        self.body_fixed_frame = frame;
        self
    }

    /// Compute imaging intervals for all (spacecraft, AOI) pairs.
    ///
    /// Only spacecraft with an [`ImagingPayload`] are considered; spacecraft
    /// without a payload are silently skipped.
    pub fn compute(&self) -> Result<ImagingResults, ImagingError> {
        let interval = *self.scenario.interval();

        // Only include spacecraft that carry an imaging payload.
        let spacecraft_with_payload: Vec<_> = self
            .scenario
            .spacecraft()
            .iter()
            .filter_map(|sc| sc.imaging_payload().map(|p| (sc, p)))
            .collect();

        let pairs: Vec<_> = spacecraft_with_payload
            .iter()
            .flat_map(|&(sc, payload)| self.aois.iter().map(move |aoi| (sc, payload, aoi)))
            .collect();

        let compute_one =
            |&(sc, payload, (aoi_id, aoi)): &(&Spacecraft, ImagingPayload, &(AoiId, Aoi))| {
                let key = (sc.id().clone(), aoi_id.clone());
                let traj = self.ensemble.get(sc.id()).expect(
                "trajectory not found in ensemble; did you forget to propagate this spacecraft?",
            );

                let detect_fn = AoiImagingDetectFn {
                    aoi,
                    params: payload,
                    trajectory: traj,
                    origin: self.scenario.origin(),
                    body_fixed_frame: self.body_fixed_frame,
                };
                let detector = RootFindingDetector::new(detect_fn, self.step);
                let windows = EventsToIntervals::new(detector).detect(interval)?;
                Ok((key, windows))
            };

        let results: Result<Vec<_>, ImagingError> = if pairs.len() > 100 {
            pairs.par_iter().map(compute_one).collect()
        } else {
            pairs.iter().map(compute_one).collect()
        };

        let intervals: HashMap<_, _> = results?.into_iter().collect();
        Ok(ImagingResults { intervals })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{LineString, Polygon, point};
    use lox_bodies::{Earth, MeanRadius};

    fn earth_mean_radius_m() -> f64 {
        Earth.mean_radius().to_meters()
    }

    #[test]
    fn test_aoi_id() {
        let id = AoiId::new("rome");
        assert_eq!(id.as_str(), "rome");
        assert_eq!(format!("{id}"), "rome");
    }

    #[test]
    fn test_aoi_distance_inside() {
        let polygon = Polygon::new(
            LineString::from(vec![
                (10.0, 45.0),
                (11.0, 45.0),
                (11.0, 46.0),
                (10.0, 46.0),
                (10.0, 45.0),
            ]),
            vec![],
        );
        let aoi = Aoi::new(polygon);
        let inside = point!(x: 10.5, y: 45.5);
        assert_eq!(aoi.distance_to(&inside, earth_mean_radius_m()), 0.0);
    }

    #[test]
    fn test_aoi_distance_outside() {
        let polygon = Polygon::new(
            LineString::from(vec![
                (10.0, 45.0),
                (11.0, 45.0),
                (11.0, 46.0),
                (10.0, 46.0),
                (10.0, 45.0),
            ]),
            vec![],
        );
        let aoi = Aoi::new(polygon);
        let outside = point!(x: 12.0, y: 45.5);
        let d = aoi.distance_to(&outside, earth_mean_radius_m());
        assert!(d > 0.0);
        // ~78 km from edge
        assert!(d > 70_000.0 && d < 90_000.0);
    }

    #[test]
    fn test_haversine_matches_geodesic_for_earth() {
        use geo::{Distance as GeoDistance, Geodesic};

        // Two points ~78 km apart (edge of AOI to a point outside)
        let a = point!(x: 11.0, y: 45.5); // on polygon edge
        let b = point!(x: 12.0, y: 45.5); // 1° east

        let geodesic = Geodesic::distance(a, b);
        let haversine = haversine_distance(a, b, earth_mean_radius_m());

        // Haversine on a sphere vs Vincenty on WGS-84 ellipsoid: expect < 0.5% error
        let rel_err = (haversine - geodesic).abs() / geodesic;
        assert!(
            rel_err < 0.005,
            "haversine ({haversine:.1}) vs geodesic ({geodesic:.1}): {:.2}% error",
            rel_err * 100.0,
        );
    }

    #[test]
    fn test_distance_scales_with_body_radius() {
        let polygon = Polygon::new(
            LineString::from(vec![
                (10.0, 45.0),
                (11.0, 45.0),
                (11.0, 46.0),
                (10.0, 46.0),
                (10.0, 45.0),
            ]),
            vec![],
        );
        let aoi = Aoi::new(polygon);
        let outside = point!(x: 12.0, y: 45.5);

        let earth_d = aoi.distance_to(&outside, earth_mean_radius_m());
        // Mars mean radius ~3389.5 km
        let mars_d = aoi.distance_to(&outside, 3_389_500.0);

        // Same angular separation, smaller body → shorter distance
        assert!(mars_d < earth_d);
        let ratio = earth_d / mars_d;
        let expected_ratio = earth_mean_radius_m() / 3_389_500.0;
        assert!(
            (ratio - expected_ratio).abs() < 0.01,
            "distance should scale linearly with radius"
        );
    }

    #[test]
    fn test_imaging_params_nadir_only() {
        let params = ImagingPayload::nadir_only(Distance::kilometers(20.0));
        assert_eq!(params.half_swath_ground_range, 10_000.0);
        assert_eq!(params.max_off_nadir, 0.0);
        // Nadir-only: range is just half swath
        let range = params.max_accessible_ground_range(500_000.0, 6_371_000.0);
        assert!((range - 10_000.0).abs() < 1e-6);
    }

    #[test]
    fn test_imaging_params_off_nadir() {
        let params = ImagingPayload::off_nadir(Distance::kilometers(20.0), Angle::degrees(30.0));
        let range = params.max_accessible_ground_range(500_000.0, 6_371_000.0);
        // Should be > half swath (10 km) due to off-nadir contribution
        assert!(range > 10_000.0);
        // For 30° off-nadir at 500 km altitude, expect ~300-400 km total
        assert!(range > 200_000.0);
    }

    #[cfg(feature = "geojson")]
    #[test]
    fn test_aoi_from_geojson() {
        let geojson =
            r#"{"type":"Polygon","coordinates":[[[10,45],[11,45],[11,46],[10,46],[10,45]]]}"#;
        let aoi = Aoi::from_geojson(geojson).unwrap();
        let inside = point!(x: 10.5, y: 45.5);
        assert_eq!(aoi.distance_to(&inside, earth_mean_radius_m()), 0.0);
    }

    #[cfg(feature = "geojson")]
    #[test]
    fn test_aoi_from_geojson_feature() {
        let geojson = r#"{"type":"Feature","geometry":{"type":"Polygon","coordinates":[[[10,45],[11,45],[11,46],[10,46],[10,45]]]},"properties":{}}"#;
        let aoi = Aoi::from_geojson(geojson).unwrap();
        let inside = point!(x: 10.5, y: 45.5);
        assert_eq!(aoi.distance_to(&inside, earth_mean_radius_m()), 0.0);
    }

    #[cfg(feature = "geojson")]
    #[test]
    fn test_aoi_from_geojson_invalid() {
        let result = Aoi::from_geojson("not json");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Integration tests — full ImagingAnalysis pipeline with Sentinel-2 TLEs
    // -----------------------------------------------------------------------

    use lox_bodies::DynOrigin;
    use lox_frames::DynFrame;
    use lox_orbits::orbits::{DynTrajectory, Ensemble};
    use lox_orbits::propagators::OrbitSource;
    use lox_time::time_scales::{DynTimeScale, Tai};

    /// Propagate a Sentinel-2 TLE into a DynTrajectory over a 6-hour window.
    fn sentinel2_trajectory(name: &str, line1: &[u8], line2: &[u8]) -> DynTrajectory {
        use lox_orbits::propagators::Propagator;
        use lox_orbits::propagators::sgp4::{Elements, Sgp4};
        use lox_time::intervals::Interval;

        let tle = Elements::from_tle(Some(name.to_string()), line1, line2).unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let t0 = sgp4.time();
        let t1 = t0 + TimeDelta::from_hours(6);
        sgp4.with_step(TimeDelta::from_seconds(10))
            .propagate(Interval::new(t0, t1))
            .unwrap()
            .into_dyn()
    }

    fn sentinel2a_trajectory() -> DynTrajectory {
        sentinel2_trajectory(
            "SENTINEL-2A",
            b"1 40697U 15028A   26079.19377485 -.00000072  00000+0 -11026-4 0  9994",
            b"2 40697  98.5642 155.3327 0001269  98.1407 261.9920 14.30816376561005",
        )
    }

    fn sentinel2b_trajectory() -> DynTrajectory {
        sentinel2_trajectory(
            "SENTINEL-2B",
            b"1 42063U 17013A   26079.18648189  .00000015  00000+0  22231-4 0  9995",
            b"2 42063  98.5694 155.2271 0001161  93.5553 266.5763 14.30810963471912",
        )
    }

    /// Build a Scenario + Ensemble from spacecraft with pre-computed trajectories.
    fn make_imaging_scenario(
        space_assets: &[Spacecraft],
        interval: TimeInterval<DynTimeScale>,
    ) -> (
        Scenario<DynOrigin, DynFrame>,
        Ensemble<AssetId, Tai, DynOrigin, DynFrame>,
    ) {
        let tai_interval =
            TimeInterval::new(interval.start().to_scale(Tai), interval.end().to_scale(Tai));
        let scenario = Scenario::with_interval(tai_interval, DynOrigin::Earth, DynFrame::Icrf)
            .with_spacecraft(space_assets);
        let mut map = std::collections::HashMap::new();
        for sc in space_assets {
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

    /// A large AOI covering most of Western Europe — Sentinel-2 SSO orbits
    /// will definitely overfly this within 6 hours.
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

    /// A small AOI in the middle of the Pacific — unlikely to be imaged
    /// from a sun-synchronous orbit in a short window.
    fn pacific_aoi() -> Aoi {
        Aoi::new(Polygon::new(
            LineString::from(vec![
                (-175.0, -5.0),
                (-174.0, -5.0),
                (-174.0, -4.0),
                (-175.0, -4.0),
                (-175.0, -5.0),
            ]),
            vec![],
        ))
    }

    #[test]
    fn test_imaging_analysis_sentinel2_over_europe() {
        let traj = sentinel2a_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());

        // Sentinel-2: 290 km swath, nadir-only
        let payload = ImagingPayload::nadir_only(Distance::kilometers(290.0));
        let sc =
            Spacecraft::new("s2a", OrbitSource::Trajectory(traj)).with_imaging_payload(payload);

        let (scenario, ensemble) = make_imaging_scenario(std::slice::from_ref(&sc), interval);

        let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

        let analysis =
            ImagingAnalysis::new(&scenario, &ensemble, aois).with_step(TimeDelta::from_seconds(30));
        let results = analysis.compute().expect("imaging analysis failed");

        let intervals = results.intervals(&AssetId::new("s2a"), &AoiId::new("europe"));
        // A sun-synchronous LEO satellite should overfly Western Europe
        // at least once in 6 hours.
        assert!(
            !intervals.is_empty(),
            "expected at least one imaging window over Western Europe"
        );

        // Each window should be short (a few minutes at most for a LEO pass).
        for iv in intervals {
            let duration_s = (iv.end() - iv.start()).to_seconds().to_f64();
            assert!(duration_s > 0.0, "zero-length interval");
            assert!(
                duration_s < 600.0,
                "imaging window too long ({duration_s:.0}s) — expected < 600s for a LEO pass"
            );
        }
    }

    #[test]
    fn test_imaging_analysis_no_payload_skips_spacecraft() {
        let traj = sentinel2a_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());

        // Spacecraft without an imaging payload
        let sc = Spacecraft::new("s2a", OrbitSource::Trajectory(traj));

        let (scenario, ensemble) = make_imaging_scenario(&[sc], interval);

        let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

        let analysis = ImagingAnalysis::new(&scenario, &ensemble, aois);
        let results = analysis.compute().expect("imaging analysis failed");

        assert!(
            results.is_empty(),
            "expected no results when spacecraft has no payload"
        );
    }

    #[test]
    fn test_imaging_analysis_multiple_spacecraft_and_aois() {
        let traj_a = sentinel2a_trajectory();
        let traj_b = sentinel2b_trajectory();
        let interval = TimeInterval::new(traj_a.start_time(), traj_a.end_time());

        let payload = ImagingPayload::nadir_only(Distance::kilometers(290.0));

        let sc_a =
            Spacecraft::new("s2a", OrbitSource::Trajectory(traj_a)).with_imaging_payload(payload);
        let sc_b =
            Spacecraft::new("s2b", OrbitSource::Trajectory(traj_b)).with_imaging_payload(payload);

        let (scenario, ensemble) = make_imaging_scenario(&[sc_a.clone(), sc_b.clone()], interval);

        let aois = vec![
            (AoiId::new("europe"), western_europe_aoi()),
            (AoiId::new("pacific"), pacific_aoi()),
        ];

        let analysis =
            ImagingAnalysis::new(&scenario, &ensemble, aois).with_step(TimeDelta::from_seconds(30));
        let results = analysis.compute().expect("imaging analysis failed");

        // Both spacecraft should have windows over the large European AOI.
        let s2a_europe = results.intervals(&AssetId::new("s2a"), &AoiId::new("europe"));
        let s2b_europe = results.intervals(&AssetId::new("s2b"), &AoiId::new("europe"));
        assert!(!s2a_europe.is_empty(), "S2A should image Europe");
        assert!(!s2b_europe.is_empty(), "S2B should image Europe");

        // The small Pacific AOI may or may not be hit. But the total result
        // count should cover all 4 pairs.
        assert_eq!(results.num_pairs(), 4);
    }

    #[test]
    fn test_imaging_off_nadir_wider_than_nadir() {
        let traj = sentinel2a_trajectory();
        let interval = TimeInterval::new(traj.start_time(), traj.end_time());

        let nadir_payload = ImagingPayload::nadir_only(Distance::kilometers(290.0));
        let off_nadir_payload =
            ImagingPayload::off_nadir(Distance::kilometers(290.0), Angle::degrees(30.0));

        // Same trajectory, different payloads
        let sc_nadir = Spacecraft::new("nadir", OrbitSource::Trajectory(traj.clone()))
            .with_imaging_payload(nadir_payload);
        let sc_off_nadir = Spacecraft::new("off_nadir", OrbitSource::Trajectory(traj))
            .with_imaging_payload(off_nadir_payload);

        let (scenario, ensemble) = make_imaging_scenario(&[sc_nadir, sc_off_nadir], interval);

        let aois = vec![(AoiId::new("europe"), western_europe_aoi())];

        let analysis =
            ImagingAnalysis::new(&scenario, &ensemble, aois).with_step(TimeDelta::from_seconds(30));
        let results = analysis.compute().expect("imaging analysis failed");

        let nadir_intervals = results.intervals(&AssetId::new("nadir"), &AoiId::new("europe"));
        let off_nadir_intervals =
            results.intervals(&AssetId::new("off_nadir"), &AoiId::new("europe"));

        // Agile satellite should have at least as much total coverage time.
        let nadir_total: f64 = nadir_intervals
            .iter()
            .map(|iv| (iv.end() - iv.start()).to_seconds().to_f64())
            .sum();
        let off_nadir_total: f64 = off_nadir_intervals
            .iter()
            .map(|iv| (iv.end() - iv.start()).to_seconds().to_f64())
            .sum();
        assert!(
            off_nadir_total >= nadir_total - 1.0,
            "off-nadir ({off_nadir_total:.0}s) should have >= nadir ({nadir_total:.0}s) coverage"
        );
    }
}
