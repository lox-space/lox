/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub enum JulianEpoch {
    JulianDate,
    ModifiedJulianDate,
    J1950,
    J2000,
}

pub enum Unit {
    Seconds,
    Days,
    Centuries,
}

pub trait JulianDate {
    fn julian_date(&self, epoch: JulianEpoch, unit: Unit) -> f64;

    fn two_part_julian_date(&self) -> (f64, f64);

    fn seconds_since_julian_epoch(&self) -> f64 {
        self.julian_date(JulianEpoch::JulianDate, Unit::Seconds)
    }

    fn seconds_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(JulianEpoch::ModifiedJulianDate, Unit::Seconds)
    }

    fn seconds_since_j1950(&self) -> f64 {
        self.julian_date(JulianEpoch::J1950, Unit::Seconds)
    }

    fn seconds_since_j2000(&self) -> f64 {
        self.julian_date(JulianEpoch::J2000, Unit::Seconds)
    }

    fn days_since_julian_epoch(&self) -> f64 {
        self.julian_date(JulianEpoch::JulianDate, Unit::Days)
    }

    fn days_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(JulianEpoch::ModifiedJulianDate, Unit::Days)
    }

    fn days_since_j1950(&self) -> f64 {
        self.julian_date(JulianEpoch::J1950, Unit::Days)
    }

    fn days_since_j2000(&self) -> f64 {
        self.julian_date(JulianEpoch::J2000, Unit::Days)
    }

    fn centuries_since_julian_epoch(&self) -> f64 {
        self.julian_date(JulianEpoch::JulianDate, Unit::Centuries)
    }

    fn centuries_since_modified_julian_epoch(&self) -> f64 {
        self.julian_date(JulianEpoch::ModifiedJulianDate, Unit::Centuries)
    }

    fn centuries_since_j1950(&self) -> f64 {
        self.julian_date(JulianEpoch::J1950, Unit::Centuries)
    }

    fn centuries_since_j2000(&self) -> f64 {
        self.julian_date(JulianEpoch::J2000, Unit::Centuries)
    }
}
