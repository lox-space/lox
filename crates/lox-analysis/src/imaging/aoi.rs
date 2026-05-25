// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Area-of-interest primitives shared by all access-analysis sensors.

use std::fmt;

use geo::prelude::Contains;
use geo::{Closest, HaversineClosestPoint};

#[cfg(feature = "geojson")]
use geojson::GeoJson;
#[cfg(feature = "geojson")]
use std::convert::TryInto;

use thiserror::Error;

/// Densify a polygon by inserting intermediate vertices via linear lon/lat
/// interpolation so that no sub-segment exceeds `max_deg` degrees (Euclidean
/// distance in lon/lat coordinate space).
///
/// `geo::HaversineClosestPoint` treats each polygon edge as a great-circle arc.
/// For long edges (e.g. 100° of longitude at mid-latitudes) that arc can curve
/// many degrees away from the intended constant-latitude boundary, producing
/// near-zero cross-track distances for satellites that are in fact far outside
/// the AOI. By keeping sub-segments ≤ 0.5° long the great-circle arc and the
/// straight lon/lat line are indistinguishable for practical sensor ranges.
fn densify_polygon_linear(polygon: geo::Polygon<f64>, max_deg: f64) -> geo::Polygon<f64> {
    let exterior = densify_ring_linear(polygon.exterior(), max_deg);
    let interiors: Vec<_> = polygon
        .interiors()
        .iter()
        .map(|ring| densify_ring_linear(ring, max_deg))
        .collect();
    geo::Polygon::new(exterior, interiors)
}

fn densify_ring_linear(ring: &geo::LineString<f64>, max_deg: f64) -> geo::LineString<f64> {
    let coords: Vec<geo::Coord<f64>> = ring.coords().copied().collect();
    let mut out: Vec<geo::Coord<f64>> = Vec::with_capacity(coords.len() * 4);
    for pair in coords.windows(2) {
        let (x0, y0) = (pair[0].x, pair[0].y);
        let (x1, y1) = (pair[1].x, pair[1].y);
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        let n = ((len / max_deg).ceil() as usize).max(1);
        out.push(geo::Coord { x: x0, y: y0 });
        for i in 1..n {
            let t = i as f64 / n as f64;
            out.push(geo::Coord {
                x: x0 + t * dx,
                y: y0 + t * dy,
            });
        }
    }
    if let Some(&last) = coords.last() {
        out.push(last);
    }
    geo::LineString::new(out)
}

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
        let polygon = densify_polygon_linear(polygon, 0.5);
        Self { polygon }
    }

    /// Parses an AOI from a GeoJSON string.
    ///
    /// Expects a GeoJSON Polygon geometry (or a Feature containing one).
    #[cfg(feature = "geojson")]
    pub fn from_geojson(geojson: &str) -> Result<Self, AoiError> {
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

        Ok(Self::new(polygon))
    }

    /// Returns the great-circle distance in meters from a point to the AOI polygon.
    ///
    /// Uses the haversine formula with the given `mean_radius_m` so that the
    /// computation is valid for any spherical body, not just Earth.
    /// Returns 0.0 if the point is inside the polygon. For a degenerate polygon
    /// (NaN coords or zero-length edges) returns [`f64::INFINITY`] so that
    /// callers in the access-analysis pipeline get a "never accessible" metric
    /// rather than a panic — preserving the infallible `AccessPayload` contract.
    pub fn distance_to(&self, point: &geo::Point<f64>, mean_radius_m: f64) -> f64 {
        if self.polygon.contains(point) {
            0.0
        } else {
            match self.polygon.haversine_closest_point(point) {
                Closest::Intersection(_) => 0.0,
                Closest::SinglePoint(closest) => haversine_distance(*point, closest, mean_radius_m),
                Closest::Indeterminate => f64::INFINITY,
            }
        }
    }

    /// Returns the great-circle-nearest point of the polygon to `point`.
    /// If `point` lies inside the polygon, returns `point` itself. For a
    /// degenerate polygon returns `point` itself as a benign sentinel (paired
    /// with the [`f64::INFINITY`] distance from [`Aoi::nearest_point_and_distance`]).
    /// The returned point is in lon/lat degrees, matching the polygon's
    /// coordinate convention.
    pub fn nearest_point(&self, point: &geo::Point<f64>) -> geo::Point<f64> {
        if self.polygon.contains(point) {
            return *point;
        }
        match self.polygon.haversine_closest_point(point) {
            geo::Closest::Intersection(p) | geo::Closest::SinglePoint(p) => p,
            geo::Closest::Indeterminate => *point,
        }
    }

    /// Returns both the nearest polygon point and its great-circle distance in
    /// metres from `point`. If `point` lies inside the polygon, returns
    /// `(*point, 0.0)`. For a degenerate polygon returns `(*point, f64::INFINITY)`
    /// so the access-analysis pipeline degrades to "never accessible" rather
    /// than panicking. Folds the work of [`Aoi::nearest_point`] and
    /// [`Aoi::distance_to`] into a single polygon traversal.
    pub fn nearest_point_and_distance(
        &self,
        point: &geo::Point<f64>,
        mean_radius_m: f64,
    ) -> (geo::Point<f64>, f64) {
        if self.polygon.contains(point) {
            return (*point, 0.0);
        }
        match self.polygon.haversine_closest_point(point) {
            Closest::Intersection(p) => (p, 0.0),
            Closest::SinglePoint(p) => (p, haversine_distance(*point, p, mean_radius_m)),
            Closest::Indeterminate => (*point, f64::INFINITY),
        }
    }

    /// Returns a reference to the underlying polygon.
    pub fn polygon(&self) -> &geo::Polygon<f64> {
        &self.polygon
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{Distance as GeoDistance, Geodesic, LineString, Polygon, point};
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

    // Regression test for issue #401: large AOI at high latitudes produced false
    // imaging intervals because geo::HaversineClosestPoint treats polygon edges as
    // great-circle arcs. The great circle connecting (-45°,50°) to (55°,50°)
    // curves up to ~61.7°N at its midpoint (lon≈5°). A satellite at (5°,62°N)
    // therefore appeared to be only ~38 km from the polygon boundary, even though
    // the true distance to the lat=50° edge is ~1334 km. With a sensor range of
    // ~290 km this produced a false imaging detection.
    #[test]
    fn test_aoi_distance_large_polygon_near_great_circle_arc_peak() {
        let polygon = Polygon::new(
            LineString::from(vec![
                (-45.0, 30.0),
                (55.0, 30.0),
                (55.0, 50.0),
                (-45.0, 50.0),
                (-45.0, 30.0),
            ]),
            vec![],
        );
        let aoi = Aoi::new(polygon);
        // 12° north of the top edge – the great-circle arc of that edge peaks at
        // ~61.7°N, so (5°,62°) sits almost on the arc, yielding a near-zero
        // computed distance before the fix.
        let near_arc_peak = point!(x: 5.0, y: 62.0);
        let d = aoi.distance_to(&near_arc_peak, earth_mean_radius_m());
        // True distance to the 50°N boundary is ~1334 km.
        // Before the fix this returned ~38 km (false positive).
        assert!(
            d > 1_000_000.0,
            "expected distance > 1000 km for point 12° north of AOI boundary, got {:.0} m",
            d
        );
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

    #[test]
    fn nearest_point_inside_polygon_returns_query_point() {
        let aoi = Aoi::new(Polygon::new(
            LineString::from(vec![
                (10.0, 45.0),
                (11.0, 45.0),
                (11.0, 46.0),
                (10.0, 46.0),
                (10.0, 45.0),
            ]),
            vec![],
        ));
        let inside = point!(x: 10.5, y: 45.5);
        let np = aoi.nearest_point(&inside);
        assert!((np.x() - 10.5).abs() < 1e-12 && (np.y() - 45.5).abs() < 1e-12);
    }

    #[test]
    fn nearest_point_outside_polygon_lies_on_boundary() {
        let aoi = Aoi::new(Polygon::new(
            LineString::from(vec![
                (10.0, 45.0),
                (11.0, 45.0),
                (11.0, 46.0),
                (10.0, 46.0),
                (10.0, 45.0),
            ]),
            vec![],
        ));
        let outside = point!(x: 12.0, y: 45.5);
        let np = aoi.nearest_point(&outside);
        // Closest boundary point sits on the lon=11 edge near lat≈45.5.
        assert!((np.x() - 11.0).abs() < 0.01);
        assert!((np.y() - 45.5).abs() < 0.05);
    }
}
