use std::{
    f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU},
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

pub const DEGREES_IN_CIRCLE: f64 = 360.0;

pub const ARCSECONDS_IN_CIRCLE: f64 = DEGREES_IN_CIRCLE * 60.0 * 60.0;

pub const RADIANS_IN_ARCSECOND: f64 = TAU / ARCSECONDS_IN_CIRCLE;

pub type Radians = f64;

/// An angle in radians
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Angle(pub Radians);

impl Angle {
    pub const ZERO: Self = Self(0.0);
    pub const PI: Self = Self(PI);
    pub const TAU: Self = Self(TAU);
    pub const FRAC_PI_2: Self = Self(FRAC_PI_2);
    pub const FRAC_PI_4: Self = Self(FRAC_PI_4);

    pub fn rad(rad: f64) -> Self {
        Self(rad)
    }

    pub fn rad_normalized(rad: f64) -> Self {
        Self(rad).mod_two_pi()
    }

    pub fn rad_normalized_signed(rad: f64) -> Self {
        Self(rad).mod_two_pi_signed()
    }

    pub fn deg(deg: f64) -> Self {
        Self(deg.to_radians())
    }

    pub fn deg_normalized(deg: f64) -> Self {
        Self((deg % DEGREES_IN_CIRCLE).to_radians()).mod_two_pi()
    }

    pub fn deg_normalized_signed(deg: f64) -> Self {
        Self((deg % DEGREES_IN_CIRCLE).to_radians())
    }

    pub fn asec(asec: f64) -> Self {
        Self(asec * RADIANS_IN_ARCSECOND)
    }

    pub fn asec_normalized(asec: f64) -> Self {
        Self((asec % ARCSECONDS_IN_CIRCLE) * RADIANS_IN_ARCSECOND).mod_two_pi()
    }

    pub fn asec_normalized_signed(asec: f64) -> Self {
        Self((asec % ARCSECONDS_IN_CIRCLE) * RADIANS_IN_ARCSECOND)
    }

    pub fn cos(&self) -> f64 {
        self.0.cos()
    }

    pub fn cosh(&self) -> f64 {
        self.0.cosh()
    }

    pub fn sin(&self) -> f64 {
        self.0.sin()
    }

    pub fn sinh(&self) -> f64 {
        self.0.sinh()
    }

    pub fn sin_cos(&self) -> (f64, f64) {
        self.0.sin_cos()
    }

    pub fn tan(&self) -> f64 {
        self.0.tan()
    }

    pub fn tanh(&self) -> f64 {
        self.0.tanh()
    }

    pub fn mod_two_pi(&self) -> Self {
        let mut a = self.0 % TAU;
        if a < 0.0 {
            a += TAU
        }
        Self(a)
    }

    pub fn mod_two_pi_signed(&self) -> Self {
        Self(self.0 % TAU)
    }

    pub fn normalize_two_pi(&self, center: Self) -> Self {
        Self(self.0 - TAU * ((self.0 + PI - center.0) / TAU).floor())
    }
}

impl Display for Angle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.to_degrees().fmt(f)?;
        write!(f, " deg")
    }
}

pub trait AngleUnits {
    fn deg(&self) -> Angle;
    fn rad(&self) -> Angle;
    fn asec(&self) -> Angle;
    fn mas(&self) -> Angle;
    fn uas(&self) -> Angle;
}

impl AngleUnits for f64 {
    fn deg(&self) -> Angle {
        Angle::deg(*self)
    }

    fn rad(&self) -> Angle {
        Angle::rad(*self)
    }

    fn asec(&self) -> Angle {
        Angle::asec(*self)
    }

    fn mas(&self) -> Angle {
        Angle::asec(self * 1e-3)
    }

    fn uas(&self) -> Angle {
        Angle::asec(self * 1e-6)
    }
}

pub const ASTRONOMICAL_UNIT: f64 = 1.495978707e11;

type Meters = f64;

/// A distance in meters
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Distance(pub Meters);

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

type MetersPerSecond = f64;

/// A velocity in meters per second
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Velocity(pub MetersPerSecond);

impl Display for Velocity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        (1e-3 * self.0).fmt(f)?;
        write!(f, " km/s")
    }
}

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

impl Display for Frequency {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        (1e-9 * self.0).fmt(f)?;
        write!(f, " GHz")
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

macro_rules! impl_ops {
    ($($unit:ident),*) => {
        $(
            impl Neg for $unit {
                type Output = Self;

                fn neg(self) -> Self::Output {
                    Self(-self.0)
                }
            }

            impl Add for $unit {
                type Output = Self;

                fn add(self, rhs: Self) -> Self::Output {
                    Self(self.0 + rhs.0)
                }
            }

            impl AddAssign for $unit {
                fn add_assign(&mut self, rhs: Self) {
                    self.0 = self.0 + rhs.0;
                }
            }

            impl Sub for $unit {
                type Output = Self;

                fn sub(self, rhs: Self) -> Self::Output {
                    Self(self.0 - rhs.0)
                }
            }

            impl SubAssign for $unit {
                fn sub_assign(&mut self, rhs: Self) {
                    self.0 = self.0 - rhs.0
                }
            }

            impl Mul<$unit> for f64 {
                type Output = $unit;

                fn mul(self, rhs: $unit) -> Self::Output {
                    $unit(self * rhs.0)
                }
            }
        )*
    };
}

impl_ops!(Angle, Distance, Velocity);

#[cfg(test)]
mod tests {
    use alloc::format;
    use core::f64::consts::{FRAC_PI_2, PI};

    use float_eq::assert_float_eq;
    use rstest::rstest;

    extern crate alloc;

    use super::*;

    #[test]
    fn test_angle_deg() {
        let angle = 90.0.deg();
        assert_float_eq!(angle.0, FRAC_PI_2, rel <= 1e-10);
    }

    #[test]
    fn test_angle_rad() {
        let angle = PI.rad();
        assert_float_eq!(angle.0, PI, rel <= 1e-10);
    }

    #[test]
    fn test_angle_conversions() {
        let angle_deg = 180.0.deg();
        let angle_rad = PI.rad();
        assert_float_eq!(angle_deg.0, angle_rad.0, rel <= 1e-10);
    }

    #[test]
    fn test_angle_display() {
        let angle = 90.123456.deg();
        assert_eq!(format!("{:.2}", angle), "90.12 deg")
    }

    #[test]
    fn test_angle_neg() {
        assert_eq!(Angle(-1.0), -1.0.rad())
    }

    const TOLERANCE: f64 = f64::EPSILON;

    #[rstest]
    // Center 0.0 – expected range [-π, π).
    #[case(Angle::ZERO, Angle::ZERO, 0.0)]
    #[case(Angle::PI, Angle::ZERO, -PI)]
    #[case(-Angle::PI, Angle::ZERO, -PI)]
    #[case(Angle::TAU, Angle::ZERO, 0.0)]
    #[case(Angle::FRAC_PI_2, Angle::ZERO, FRAC_PI_2)]
    #[case(-Angle::FRAC_PI_2, Angle::ZERO, -FRAC_PI_2)]
    // Center π – expected range [0, 2π).
    #[case(Angle::ZERO, Angle::PI, 0.0)]
    #[case(Angle::PI, Angle::PI, PI)]
    #[case(-Angle::PI, Angle::PI, PI)]
    #[case(Angle::TAU, Angle::PI, 0.0)]
    #[case(Angle::FRAC_PI_2, Angle::PI, FRAC_PI_2)]
    #[case(-Angle::FRAC_PI_2, Angle::PI, 3.0 * PI / 2.0)]
    // Center -π – expected range [-2π, 0).
    #[case(Angle::ZERO, -Angle::PI, -TAU)]
    #[case(Angle::PI, -Angle::PI, -PI)]
    #[case(-Angle::PI, -Angle::PI, -PI)]
    #[case(Angle::TAU, -Angle::PI, -TAU)]
    #[case(Angle::FRAC_PI_2, -Angle::PI, -3.0 * PI / 2.0)]
    #[case(-Angle::FRAC_PI_2, -Angle::PI, -FRAC_PI_2)]
    fn test_angle_normalize_two_pi(#[case] angle: Angle, #[case] center: Angle, #[case] exp: f64) {
        // abs is preferred to rel for floating-point comparisons with 0.0. See
        // https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/#inferna
        if exp == 0.0 {
            assert_float_eq!(angle.normalize_two_pi(center).0, exp, abs <= TOLERANCE);
        } else {
            assert_float_eq!(angle.normalize_two_pi(center).0, exp, rel <= TOLERANCE);
        }
    }

    #[test]
    fn test_distance_m() {
        let distance = 1000.0.m();
        assert_eq!(distance.0, 1000.0);
    }

    #[test]
    fn test_distance_km() {
        let distance = 1.0.km();
        assert_eq!(distance.0, 1000.0);
    }

    #[test]
    fn test_distance_au() {
        let distance = 1.0.au();
        assert_eq!(distance.0, ASTRONOMICAL_UNIT);
    }

    #[test]
    fn test_distance_conversions() {
        let d1 = 1.5e11.m();
        let d2 = (1.5e11 / ASTRONOMICAL_UNIT).au();
        assert_float_eq!(d1.0, d2.0, rel <= 1e-9);
    }

    #[test]
    fn test_distance_display() {
        let distance = 9.123456.km();
        assert_eq!(format!("{:.2}", distance), "9.12 km")
    }

    #[test]
    fn test_distance_neg() {
        assert_eq!(Distance(-1.0), -1.0.m())
    }

    #[test]
    fn test_velocity_mps() {
        let velocity = 1000.0.mps();
        assert_eq!(velocity.0, 1000.0);
    }

    #[test]
    fn test_velocity_kps() {
        let velocity = 1.0.kps();
        assert_eq!(velocity.0, 1000.0);
    }

    #[test]
    fn test_velocity_conversions() {
        let v1 = 7500.0.mps();
        let v2 = 7.5.kps();
        assert_eq!(v1.0, v2.0);
    }

    #[test]
    fn test_velocity_display() {
        let velocity = 9.123456.kps();
        assert_eq!(format!("{:.2}", velocity), "9.12 km/s")
    }

    #[test]
    fn test_velocity_neg() {
        assert_eq!(Velocity(-1.0), -1.0.mps())
    }

    #[test]
    fn test_frequency_hz() {
        let frequency = 1000.0.hz();
        assert_eq!(frequency.0, 1000.0);
    }

    #[test]
    fn test_frequency_khz() {
        let frequency = 1.0.khz();
        assert_eq!(frequency.0, 1000.0);
    }

    #[test]
    fn test_frequency_mhz() {
        let frequency = 1.0.mhz();
        assert_eq!(frequency.0, 1_000_000.0);
    }

    #[test]
    fn test_frequency_ghz() {
        let frequency = 1.0.ghz();
        assert_eq!(frequency.0, 1_000_000_000.0);
    }

    #[test]
    fn test_frequency_thz() {
        let frequency = 1.0.thz();
        assert_eq!(frequency.0, 1_000_000_000_000.0);
    }

    #[test]
    fn test_frequency_conversions() {
        let f1 = 2.4.ghz();
        let f2 = 2400.0.mhz();
        assert_eq!(f1.0, f2.0);
    }

    #[test]
    fn test_frequency_wavelength() {
        let f = 1.0.ghz();
        let wavelength = f.wavelength();
        assert_float_eq!(wavelength.0, 0.299792458, rel <= 1e-9);
    }

    #[test]
    fn test_frequency_wavelength_speed_of_light() {
        let f = 299792458.0.hz(); // 1 Hz at speed of light
        let wavelength = f.wavelength();
        assert_float_eq!(wavelength.0, 1.0, rel <= 1e-10);
    }

    #[test]
    fn test_frequency_display() {
        let frequency = 2.4123456.ghz();
        assert_eq!(format!("{:.2}", frequency), "2.41 GHz");
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
        assert_eq!(f.band(), exp)
    }
}
