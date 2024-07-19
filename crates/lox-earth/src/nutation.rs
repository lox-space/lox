/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module nutation exposes a function for calculating Earth nutation using a number of IAU nutation
//! models.

use std::ops::Add;

use lox_math::math::RADIANS_IN_ARCSECOND;
use lox_math::types::units::Radians;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;
use lox_time::Time;

use crate::nutation::iau1980::nutation_iau1980;
use crate::nutation::iau2000::nutation_iau2000a;
use crate::nutation::iau2000::nutation_iau2000b;
use crate::nutation::iau2006::nutation_iau2006a;

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

/// Calculate nutation coefficients at `time` using the given [Model].
pub fn nutation(model: Model, time: Time<Tdb>) -> Nutation {
    let t = time.centuries_since_j2000();
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

    use super::*;

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_nutation_iau1980() {
        let time = Time::j2000(Tdb);
        let expected = Nutation {
            longitude: -0.00006750247617532478,
            obliquity: -0.00002799221238377013,
        };
        let actual = nutation(Model::IAU1980, time);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }
    #[test]
    fn test_nutation_iau2000a() {
        let time = Time::j2000(Tdb);
        let expected = Nutation {
            longitude: -0.00006754422426417299,
            obliquity: -0.00002797083119237414,
        };
        let actual = nutation(Model::IAU2000A, time);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2000b() {
        let time = Time::j2000(Tdb);
        let expected = Nutation {
            longitude: -0.00006754261253992235,
            obliquity: -0.00002797092331098565,
        };
        let actual = nutation(Model::IAU2000B, time);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2006a() {
        let time = Time::j2000(Tdb);
        let expected = Nutation {
            longitude: -0.00006754425598969513,
            obliquity: -0.00002797083119237414,
        };
        let actual = nutation(Model::IAU2006A, time);
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
