// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Access-analysis traits: payload metric and payload accessor.

use lox_core::coords::LonLatAlt;
use lox_core::units::Angle;

use crate::imaging::aoi::Aoi;

/// Returns the per-sample access metric for an AOI.
///
/// Sign convention: positive when the AOI is accessible at this geometry,
/// negative when not. Continuous across the access boundary so that a
/// root finder can locate entry/exit times. Infallible.
pub trait AccessPayload {
    /// Returns the access metric for the given sub-satellite point and AOI.
    fn access_metric(
        &self,
        sub_sat: LonLatAlt,
        ground_track_az: Angle,
        aoi: &Aoi,
        mean_radius_m: f64,
    ) -> f64;
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
}
