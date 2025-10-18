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

use lox_time::Time;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;
use lox_units::Angle;

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
    pub longitude: Angle,
    /// δε
    pub obliquity: Angle,
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
        Model::IAU1980 => Nutation::iau1980(t),
        Model::IAU2000A => nutation_iau2000a(t),
        Model::IAU2000B => nutation_iau2000b(t),
        Model::IAU2006A => nutation_iau2006a(t),
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use lox_units::AngleUnits;

    use super::*;

    const TOLERANCE: Angle = Angle::rad(1e-12);

    #[test]
    fn test_nutation_iau1980() {
        let time = Time::j2000(Tdb);
        let expected = Nutation {
            longitude: -0.00006750247617532478.rad(),
            obliquity: -0.00002799221238377013.rad(),
        };
        let actual = nutation(Model::IAU1980, time);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }
    #[test]
    fn test_nutation_iau2000a() {
        let time = Time::j2000(Tdb);
        let expected = Nutation {
            longitude: -0.00006754422426417299.rad(),
            obliquity: -0.00002797083119237414.rad(),
        };
        let actual = nutation(Model::IAU2000A, time);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2000b() {
        let time = Time::j2000(Tdb);
        let expected = Nutation {
            longitude: -0.00006754261253992235.rad(),
            obliquity: -0.00002797092331098565.rad(),
        };
        let actual = nutation(Model::IAU2000B, time);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }

    #[test]
    fn test_nutation_iau2006a() {
        let time = Time::j2000(Tdb);
        let expected = Nutation {
            longitude: -0.00006754425598969513.rad(),
            obliquity: -0.00002797083119237414.rad(),
        };
        let actual = nutation(Model::IAU2006A, time);
        assert_float_eq!(expected.longitude, actual.longitude, rel <= TOLERANCE);
        assert_float_eq!(expected.obliquity, actual.obliquity, rel <= TOLERANCE);
    }
}
