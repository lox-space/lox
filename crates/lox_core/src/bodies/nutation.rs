use std::ops::Add;

use crate::bodies::nutation::iau1980::nutation_iau1980;
use crate::math::RADIANS_IN_MILLIARCSECOND;
use crate::time::epochs::Epoch;
use crate::time::intervals::{tdb_julian_centuries_since_j2000, TDBJulianCenturiesSinceJ2000};
use crate::types::Radians;

mod iau1980;
mod iau2000a;

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

fn nutation_iau2000a(_t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    todo!()
}

fn nutation_iau2000b(_t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    todo!()
}

fn nutation_iau2006a(_t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    todo!()
}

pub(crate) const RADIANS_IN_POINT_ONE_MILLIARCSECOND: Radians = RADIANS_IN_MILLIARCSECOND / 10.0;

/// Units of 0.1 mas are returned by nutation calculations before being converted to radians.
pub(crate) type Point1Milliarcsec = f64;

#[inline]
pub(crate) fn point1_milliarcsec_to_rad(p1_mas: Point1Milliarcsec) -> Radians {
    p1_mas * RADIANS_IN_POINT_ONE_MILLIARCSECOND
}

#[cfg(test)]
mod tests {
    use crate::time::epochs::{Epoch, TimeScale};
    use float_eq::assert_float_eq;

    use super::{nutation, point1_milliarcsec_to_rad, Model, Nutation};

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
    fn test_point1_milliarcsec_to_rad() {
        assert_float_eq!(point1_milliarcsec_to_rad(0.0), 0.0, rel <= TOLERANCE);
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
            1.7938106201052832e-8,
            rel <= TOLERANCE
        );
    }
}
