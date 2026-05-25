// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Optical (passive imaging) payload: nadir-centred disk access geometry.

use lox_core::coords::LonLatAlt;
use lox_core::units::{Angle, Distance};

use crate::imaging::analysis::AccessPayload;
use crate::imaging::aoi::Aoi;

/// Optical (passive) imaging payload — describes a nadir-centred disk
/// access region defined by a fixed swath width and a maximum off-nadir
/// pointing angle.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpticalPayload {
    half_swath_ground_range: f64,
    max_off_nadir: f64,
}

impl OpticalPayload {
    /// Creates a nadir-only optical payload with the given full swath width.
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

    /// Returns the total accessible ground-range radius for the disk
    /// access region at the given altitude (m) and body mean radius (m).
    pub fn max_accessible_ground_range(&self, altitude_m: f64, mean_radius_m: f64) -> f64 {
        let off_nadir_range = if self.max_off_nadir > 0.0 {
            let theta = self.max_off_nadir;
            let sin_arg = theta.sin() * (mean_radius_m + altitude_m) / mean_radius_m;
            if sin_arg >= 1.0 {
                core::f64::consts::FRAC_PI_2 * mean_radius_m
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

impl AccessPayload for OpticalPayload {
    fn access_metric(
        &self,
        sub_sat: LonLatAlt,
        _ground_track_az: Angle,
        aoi: &Aoi,
        mean_radius_m: f64,
    ) -> f64 {
        let altitude_m = sub_sat.alt().to_meters();
        let sub_sat_point = geo::Point::new(sub_sat.lon().to_degrees(), sub_sat.lat().to_degrees());
        let max_range = self.max_accessible_ground_range(altitude_m, mean_radius_m);
        let distance = aoi.distance_to(&sub_sat_point, mean_radius_m);
        max_range - distance
    }

    fn needs_ground_track_azimuth(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use geo::{LineString, Polygon};

    #[test]
    fn nadir_only_constructor() {
        let p = OpticalPayload::nadir_only(Distance::kilometers(20.0));
        assert_eq!(p.half_swath_ground_range, 10_000.0);
        assert_eq!(p.max_off_nadir, 0.0);
        let range = p.max_accessible_ground_range(500_000.0, 6_371_000.0);
        assert!((range - 10_000.0).abs() < 1e-6);
    }

    #[test]
    fn off_nadir_constructor() {
        let p = OpticalPayload::off_nadir(Distance::kilometers(20.0), Angle::degrees(30.0));
        let range = p.max_accessible_ground_range(500_000.0, 6_371_000.0);
        assert!(range > 10_000.0);
        assert!(range > 200_000.0);
    }

    #[test]
    fn access_metric_positive_when_inside_disk() {
        let aoi = Aoi::new(Polygon::new(
            LineString::from(vec![
                (0.1, 0.0),
                (0.10001, 0.0),
                (0.10001, 0.00001),
                (0.1, 0.00001),
                (0.1, 0.0),
            ]),
            vec![],
        ));
        let lla = LonLatAlt::from_degrees(0.0, 0.0, 500_000.0).unwrap();
        let p = OpticalPayload::nadir_only(Distance::kilometers(200.0));
        let m = p.access_metric(lla, Angle::degrees(0.0), &aoi, 6_371_000.0);
        assert!(m > 0.0, "expected positive metric inside disk, got {m}");
    }
}
