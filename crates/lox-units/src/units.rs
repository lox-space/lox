use std::fmt::{Display, Formatter, Result};

/// An angle in radians
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Angle(pub f64);

impl Display for Angle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.to_degrees().fmt(f)?;
        write!(f, "º")
    }
}

pub trait AngleUnits {
    fn deg(&self) -> Angle;
    fn rad(&self) -> Angle;
}

impl AngleUnits for f64 {
    fn deg(&self) -> Angle {
        Angle(self.to_radians())
    }

    fn rad(&self) -> Angle {
        Angle(*self)
    }
}

const ASTRONOMICAL_UNIT: f64 = 1.495978707e11;

/// A distance in meters
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Distance(pub f64);

impl Display for Distance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        (1e-3 * self.0).fmt(f)?;
        write!(f, " km")
    }
}

pub trait DistanceUnits {
    fn m(&self) -> Distance;
    fn km(&self) -> Distance;
    fn au(&self) -> Distance;
}

impl DistanceUnits for f64 {
    fn m(&self) -> Distance {
        Distance(*self)
    }

    fn km(&self) -> Distance {
        Distance(1e3 * self)
    }

    fn au(&self) -> Distance {
        Distance(ASTRONOMICAL_UNIT * self)
    }
}

/// A velocity in meters per second
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Velocity(pub f64);

pub trait VelocityUnits {
    fn mps(&self) -> Velocity;
    fn kps(&self) -> Velocity;
}

impl VelocityUnits for f64 {
    fn mps(&self) -> Velocity {
        Velocity(*self)
    }

    fn kps(&self) -> Velocity {
        Velocity(1e3 * self)
    }
}

impl Display for Velocity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        (1e-3 * self.0).fmt(f)?;
        write!(f, " km/s")
    }
}

#[cfg(test)]
mod tests {
    use crate::{AngleUnits, DistanceUnits, VelocityUnits};

    #[test]
    fn test_angle_display() {
        let angle = 90.123456.deg();
        assert_eq!(format!("{:.2}", angle), "90.12º")
    }

    #[test]
    fn test_distance_display() {
        let distance = 9.123456.km();
        assert_eq!(format!("{:.2}", distance), "9.12 km")
    }

    #[test]
    fn test_velocity_display() {
        let velocity = 9.123456.kps();
        assert_eq!(format!("{:.2}", velocity), "9.12 km/s")
    }
}
