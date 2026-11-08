// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Operators that relate quantities of different dimensions.

use core::ops::{Div, Mul};

use crate::time::TimeDelta;

use super::{Angle, AngularRate, Area, AreaToMass, DataRate, DataVolume, Distance, Mass, Velocity};

impl Div<TimeDelta> for Distance {
    type Output = Velocity;

    fn div(self, rhs: TimeDelta) -> Velocity {
        Velocity(self.0 / rhs.as_seconds_f64())
    }
}

impl Mul<TimeDelta> for Velocity {
    type Output = Distance;

    fn mul(self, rhs: TimeDelta) -> Distance {
        Distance(self.0 * rhs.as_seconds_f64())
    }
}

impl Mul<Velocity> for TimeDelta {
    type Output = Distance;

    fn mul(self, rhs: Velocity) -> Distance {
        rhs * self
    }
}

impl Div<TimeDelta> for Angle {
    type Output = AngularRate;

    fn div(self, rhs: TimeDelta) -> AngularRate {
        AngularRate(self.0 / rhs.as_seconds_f64())
    }
}

impl Mul<TimeDelta> for AngularRate {
    type Output = Angle;

    fn mul(self, rhs: TimeDelta) -> Angle {
        Angle(self.0 * rhs.as_seconds_f64())
    }
}

impl Mul<AngularRate> for TimeDelta {
    type Output = Angle;

    fn mul(self, rhs: AngularRate) -> Angle {
        rhs * self
    }
}

impl Mul for Distance {
    type Output = Area;

    fn mul(self, rhs: Distance) -> Area {
        Area(self.0 * rhs.0)
    }
}

impl Div<Mass> for Area {
    type Output = AreaToMass;

    fn div(self, rhs: Mass) -> AreaToMass {
        AreaToMass(self.0 / rhs.0)
    }
}

impl Mul<TimeDelta> for DataRate {
    type Output = DataVolume;

    fn mul(self, rhs: TimeDelta) -> DataVolume {
        DataVolume(self.0 * rhs.as_seconds_f64())
    }
}

impl Mul<DataRate> for TimeDelta {
    type Output = DataVolume;

    fn mul(self, rhs: DataRate) -> DataVolume {
        rhs * self
    }
}

impl Div<TimeDelta> for DataVolume {
    type Output = DataRate;

    fn div(self, rhs: TimeDelta) -> DataRate {
        DataRate(self.0 / rhs.as_seconds_f64())
    }
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;

    use super::*;

    #[test]
    fn test_kinematics() {
        let dt = TimeDelta::from_seconds(100);
        assert_eq!(
            Distance::kilometers(750.0) / dt,
            Velocity::meters_per_second(7500.0)
        );
        assert_eq!(
            Velocity::meters_per_second(7500.0) * dt,
            Distance::kilometers(750.0)
        );
        assert_eq!(
            dt * Velocity::meters_per_second(7500.0),
            Distance::kilometers(750.0)
        );
    }

    #[test]
    fn test_rotation() {
        let day = TimeDelta::from_seconds(86400);
        let rate = Angle::TAU / day;
        assert_approx_eq!(
            rate.to_radians_per_second(),
            7.27220521664304e-5,
            rtol <= 1e-14
        );
        assert_approx_eq!(rate * day, Angle::TAU, rtol <= 1e-15);
        assert_approx_eq!(day * rate, Angle::TAU, rtol <= 1e-15);
    }

    #[test]
    fn test_area_to_mass() {
        let side = Distance::meters(2.0);
        let area = side * side;
        assert_eq!(area, Area::square_meters(4.0));
        assert_eq!(
            area / Mass::kilograms(80.0),
            AreaToMass::square_meters_per_kilogram(0.05)
        );
    }

    #[test]
    fn test_data_transfer() {
        let pass = TimeDelta::from_seconds(600);
        let rate = DataRate::megabits_per_second(150.0);
        let volume = rate * pass;
        assert_eq!(volume, DataVolume::gigabits(90.0));
        assert_eq!(pass * rate, volume);
        assert_eq!(volume / pass, rate);
    }
}
