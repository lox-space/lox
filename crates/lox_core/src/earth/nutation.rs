use std::ops::Add;

use crate::earth::nutation::iau1980::nutation_iau1980;
use crate::earth::nutation::iau2000::nutation_iau2000a;
use crate::earth::nutation::iau2000::nutation_iau2000b;
use crate::earth::nutation::iau2006::nutation_iau2006a;
use crate::math::RADIANS_IN_ARCSECOND;
use crate::time::epochs::Epoch;
use crate::time::intervals::tdb_julian_centuries_since_j2000;
use crate::types::Radians;

mod iau1980;
mod iau2000;
mod iau2006;

/// The supported IAU nutation models.
pub enum Model {
    IAU1980,
    IAU2000A,
    IAU2000B,
    IAU2006A,
}

/// Nutation components with respect to some ecliptic of date.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Nutation {
    /// δψ
    pub longitude: Radians,
    /// δε
    pub obliquity: Radians,
}

impl Add for Nutation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            longitude: self.longitude + rhs.longitude,
            obliquity: self.obliquity + rhs.obliquity,
        }
    }
}

impl Add<&Self> for Nutation {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        Nutation {
            longitude: self.longitude + rhs.longitude,
            obliquity: self.obliquity + rhs.obliquity,
        }
    }
}

/// Calculate nutation coefficients at `epoch` using the given [Model].
pub fn nutation(model: Model, epoch: Epoch) -> Nutation {
    let t = tdb_julian_centuries_since_j2000(epoch);
    match model {
        Model::IAU1980 => nutation_iau1980(t),
        Model::IAU2000A => nutation_iau2000a(t),
        Model::IAU2000B => nutation_iau2000b(t),
        Model::IAU2006A => nutation_iau2006a(t),
    }
}

const RADIANS_IN_POINT_ONE_MILLIARCSECOND: Radians = RADIANS_IN_ARCSECOND / 1e4;

/// Units of 0.1 mas are returned by certain nutation calculations before being converted to
/// radians.
type Point1Milliarcsec = f64;

#[inline]
fn point1_milliarcsec_to_rad(p1_mas: Point1Milliarcsec) -> Radians {
    p1_mas * RADIANS_IN_POINT_ONE_MILLIARCSECOND
}

const RADIANS_IN_POINT_ONE_MICROARCSECOND: Radians = RADIANS_IN_ARCSECOND / 1e7;

/// Units of 0.1 μas are returned by certain nutation calculations before being converted to
/// radians.
type Point1Microarcsec = f64;

#[inline]
fn point1_microarcsec_to_rad(p1_uas: Point1Microarcsec) -> Radians {
    p1_uas * RADIANS_IN_POINT_ONE_MICROARCSECOND
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use crate::time::epochs::{Epoch, TimeScale};

    use super::*;

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_nutation_iau1980() {
        let epoch = Epoch::j2000(TimeScale::TT);
        let expected = Nutation {
            longitude: -0.00006750247617532478,
            obliquity: -0.00002799221238377013,
        };
        let actual = nutation(Model::IAU1980, epoch);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }
    #[test]
    fn test_nutation_iau2000a() {
        let epoch = Epoch::j2000(TimeScale::TT);
        let expected = Nutation {
            longitude: -0.00006754422426417299,
            obliquity: -0.00002797083119237414,
        };
        let actual = nutation(Model::IAU2000A, epoch);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2000b() {
        let epoch = Epoch::j2000(TimeScale::TT);
        let expected = Nutation {
            longitude: -0.00006754261253992235,
            obliquity: -0.00002797092331098565,
        };
        let actual = nutation(Model::IAU2000B, epoch);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2006a() {
        let epoch = Epoch::j2000(TimeScale::TT);
        let expected = Nutation {
            longitude: -0.00006754425598969513,
            obliquity: -0.00002797083119237414,
        };
        let actual = nutation(Model::IAU2006A, epoch);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_point1_milliarcsec_to_rad() {
        assert_float_eq!(point1_milliarcsec_to_rad(0.0), 0.0, abs <= TOLERANCE);
        assert_float_eq!(
            point1_milliarcsec_to_rad(1.0),
            4.84813681109536e-10,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            point1_milliarcsec_to_rad(-1.0),
            -4.84813681109536e-10,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            point1_milliarcsec_to_rad(37.0),
            1.793810620105283e-8,
            rel <= TOLERANCE
        );
    }
    #[test]
    fn test_point1_microarcsec_to_rad() {
        assert_float_eq!(point1_microarcsec_to_rad(0.0), 0.0, abs <= TOLERANCE);
        assert_float_eq!(
            point1_microarcsec_to_rad(1.0),
            4.84813681109536e-13,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            point1_microarcsec_to_rad(-1.0),
            -4.84813681109536e-13,
            rel <= TOLERANCE
        );
        assert_float_eq!(
            point1_microarcsec_to_rad(37.0),
            1.793810620105283e-11,
            rel <= TOLERANCE
        );
    }
}
