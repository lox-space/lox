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

const C_0: f64 = 299792458.0;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum FrequencyBand {
    HF,
    VHF,
    UHF,
    L,
    S,
    C,
    X,
    Ku,
    K,
    Ka,
    V,
    W,
    G,
}

/// A frequency in Hertz
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Frequency(pub f64);

impl Frequency {
    pub fn wavelength(&self) -> Distance {
        Distance(C_0 / self.0)
    }

    pub fn band(&self) -> Option<FrequencyBand> {
        match self.0 {
            f if f < 3e6 => None,
            f if f < 30e6 => Some(FrequencyBand::HF),
            f if f < 300e6 => Some(FrequencyBand::VHF),
            f if f < 1e9 => Some(FrequencyBand::UHF),
            f if f < 2e9 => Some(FrequencyBand::L),
            f if f < 4e9 => Some(FrequencyBand::S),
            f if f < 8e9 => Some(FrequencyBand::C),
            f if f < 12e9 => Some(FrequencyBand::X),
            f if f < 18e9 => Some(FrequencyBand::Ku),
            f if f < 27e9 => Some(FrequencyBand::K),
            f if f < 40e9 => Some(FrequencyBand::Ka),
            f if f < 75e9 => Some(FrequencyBand::V),
            f if f < 110e9 => Some(FrequencyBand::W),
            f if f < 300e9 => Some(FrequencyBand::G),
            _ => None,
        }
    }
}

pub trait FrequencyUnits {
    fn hz(&self) -> Frequency;
    fn khz(&self) -> Frequency;
    fn mhz(&self) -> Frequency;
    fn ghz(&self) -> Frequency;
    fn thz(&self) -> Frequency;
}

impl FrequencyUnits for f64 {
    fn hz(&self) -> Frequency {
        Frequency(*self)
    }

    fn khz(&self) -> Frequency {
        Frequency(1e3 * self)
    }

    fn mhz(&self) -> Frequency {
        Frequency(1e6 * self)
    }

    fn ghz(&self) -> Frequency {
        Frequency(1e9 * self)
    }

    fn thz(&self) -> Frequency {
        Frequency(1e12 * self)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        AngleUnits, DistanceUnits, Frequency, FrequencyBand, FrequencyUnits, VelocityUnits,
    };

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

    #[rstest]
    #[case(0.0.hz(), None)]
    #[case(3.0.mhz(), Some(FrequencyBand::HF))]
    #[case(30.0.mhz(), Some(FrequencyBand::VHF))]
    #[case(300.0.mhz(), Some(FrequencyBand::UHF))]
    #[case(1.0.ghz(), Some(FrequencyBand::L))]
    #[case(2.0.ghz(), Some(FrequencyBand::S))]
    #[case(4.0.ghz(), Some(FrequencyBand::C))]
    #[case(8.0.ghz(), Some(FrequencyBand::X))]
    #[case(12.0.ghz(), Some(FrequencyBand::Ku))]
    #[case(18.0.ghz(), Some(FrequencyBand::K))]
    #[case(27.0.ghz(), Some(FrequencyBand::Ka))]
    #[case(40.0.ghz(), Some(FrequencyBand::V))]
    #[case(75.0.ghz(), Some(FrequencyBand::W))]
    #[case(110.0.ghz(), Some(FrequencyBand::G))]
    #[case(1.0.thz(), None)]
    fn test_frequency_band(#[case] f: Frequency, #[case] exp: Option<FrequencyBand>) {
        dbg!(f);
        assert_eq!(f.band(), exp)
    }
}
