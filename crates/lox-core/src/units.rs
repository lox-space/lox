// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Newtype wrappers for unitful [`f64`] double precision values

use std::{
    f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU},
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

use glam::DMat3;
use lox_test_utils::ApproxEq;

/// Degrees in full circle
pub const DEGREES_IN_CIRCLE: f64 = 360.0;

/// Arcseconds in full circle
pub const ARCSECONDS_IN_CIRCLE: f64 = DEGREES_IN_CIRCLE * 60.0 * 60.0;

/// Radians per arcsecond
pub const RADIANS_IN_ARCSECOND: f64 = TAU / ARCSECONDS_IN_CIRCLE;

type Radians = f64;

/// Angle in radians.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[repr(transparent)]
pub struct Angle(Radians);

impl Angle {
    /// An angle equal to zero.
    pub const ZERO: Self = Self(0.0);
    /// An angle equal to π.
    pub const PI: Self = Self(PI);
    /// An angle equal to τ = 2π.
    pub const TAU: Self = Self(TAU);
    /// An angle equal to π/2.
    pub const FRAC_PI_2: Self = Self(FRAC_PI_2);
    /// An angle equal to π/4.
    pub const FRAC_PI_4: Self = Self(FRAC_PI_4);

    /// Creates a new angle from an `f64` value in radians.
    pub const fn new(rad: f64) -> Self {
        Self(rad)
    }

    /// Creates a new angle from an `f64` value in radians.
    pub const fn radians(rad: f64) -> Self {
        Self(rad)
    }

    /// Creates a new angle from an `f64` value in radians and normalize the angle
    /// to the interval [0.0, 2π).
    pub const fn radians_normalized(rad: f64) -> Self {
        Self(rad).mod_two_pi()
    }

    /// Creates a new angle from an `f64` value in radians and normalize the angle
    /// to the interval (-2π, 2π).
    pub const fn radians_normalized_signed(rad: f64) -> Self {
        Self(rad).mod_two_pi_signed()
    }

    /// Creates a new angle from an `f64` value in degrees.
    pub const fn degrees(deg: f64) -> Self {
        Self(deg.to_radians())
    }

    pub const fn from_hms(hours: i64, minutes: u8, seconds: f64) -> Self {
        Self::degrees(15.0 * (hours as f64 + minutes as f64 / 60.0 + seconds / 3600.0))
    }

    /// Creates a new angle from an `f64` value in degrees and normalize the angle
    /// to the interval [0.0, 2π).
    pub const fn degrees_normalized(deg: f64) -> Self {
        Self((deg % DEGREES_IN_CIRCLE).to_radians()).mod_two_pi()
    }

    /// Creates a new angle from an `f64` value in degrees and normalize the angle
    /// to the interval (-2π, 2π).
    pub const fn degrees_normalized_signed(deg: f64) -> Self {
        Self((deg % DEGREES_IN_CIRCLE).to_radians())
    }

    /// Creates a new angle from an `f64` value in arcseconds.
    pub const fn arcseconds(asec: f64) -> Self {
        Self(asec * RADIANS_IN_ARCSECOND)
    }

    /// Creates a new angle from an `f64` value in arcseconds and normalize the angle
    /// to the interval [0.0, 2π).
    pub const fn arcseconds_normalized(asec: f64) -> Self {
        Self((asec % ARCSECONDS_IN_CIRCLE) * RADIANS_IN_ARCSECOND).mod_two_pi()
    }

    /// Creates a new angle from an `f64` value in arcseconds and normalize the angle
    /// to the interval (-2π, 2π).
    pub const fn arcseconds_normalized_signed(asec: f64) -> Self {
        Self((asec % ARCSECONDS_IN_CIRCLE) * RADIANS_IN_ARCSECOND)
    }

    pub const fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    pub fn from_asin(value: f64) -> Self {
        Self(value.asin())
    }

    pub fn from_asinh(value: f64) -> Self {
        Self(value.asinh())
    }

    pub fn from_acos(value: f64) -> Self {
        Self(value.acos())
    }

    pub fn from_acosh(value: f64) -> Self {
        Self(value.acosh())
    }

    pub fn from_atan(value: f64) -> Self {
        Self(value.atan())
    }

    pub fn from_atanh(value: f64) -> Self {
        Self(value.atanh())
    }

    pub fn from_atan2(y: f64, x: f64) -> Self {
        Self(y.atan2(x))
    }

    /// Returns the cosine of the angle.
    pub fn cos(&self) -> f64 {
        self.0.cos()
    }

    /// Returns the hyperbolic cosine of the angle.
    pub fn cosh(&self) -> f64 {
        self.0.cosh()
    }

    /// Returns the sine of the angle.
    pub fn sin(&self) -> f64 {
        self.0.sin()
    }

    /// Returns the hyperbolic sine of the angle.
    pub fn sinh(&self) -> f64 {
        self.0.sinh()
    }

    /// Returns sine and cosine of the angle.
    pub fn sin_cos(&self) -> (f64, f64) {
        self.0.sin_cos()
    }

    /// Returns the tangent of the angle.
    pub fn tan(&self) -> f64 {
        self.0.tan()
    }

    /// Returns the hyperbolic tangent of the angle.
    pub fn tanh(&self) -> f64 {
        self.0.tanh()
    }

    /// Returns a new angle that is normalized to the interval [0.0, 2π).
    pub const fn mod_two_pi(&self) -> Self {
        let mut a = self.0 % TAU;
        if a < 0.0 {
            a += TAU
        }
        Self(a)
    }

    /// Returns a new angle that is normalized to the interval (-2π, 2π).
    pub const fn mod_two_pi_signed(&self) -> Self {
        Self(self.0 % TAU)
    }

    /// Returns a new angle that is normalized to a (-π, π) interval
    /// centered around `center`.
    pub const fn normalize_two_pi(&self, center: Self) -> Self {
        Self(self.0 - TAU * ((self.0 + PI - center.0) / TAU).floor())
    }

    /// Returns the value of the angle in radians.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the value of the angle in radians.
    pub const fn to_radians(&self) -> f64 {
        self.0
    }

    /// Returns the value of the angle in degrees.
    pub const fn to_degrees(&self) -> f64 {
        self.0.to_degrees()
    }

    /// Returns the value of the angle in arcseconds.
    pub const fn to_arcseconds(&self) -> f64 {
        self.0 / RADIANS_IN_ARCSECOND
    }

    pub fn rotation_x(&self) -> DMat3 {
        DMat3::from_rotation_x(-self.to_radians())
    }

    pub fn rotation_y(&self) -> DMat3 {
        DMat3::from_rotation_y(-self.to_radians())
    }

    pub fn rotation_z(&self) -> DMat3 {
        DMat3::from_rotation_z(-self.to_radians())
    }
}

impl Display for Angle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.to_degrees().fmt(f)?;
        write!(f, " deg")
    }
}

/// A trait for creating [`Angle`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::AngleUnits;
///
/// let angle = 360.deg();
/// assert_eq!(angle.to_radians(), std::f64::consts::TAU);
/// ```
pub trait AngleUnits {
    /// Creates an angle from a value in radians.
    fn rad(&self) -> Angle;
    /// Creates an angle from a value in degrees.
    fn deg(&self) -> Angle;
    /// Creates an angle from a value in arcseconds.
    fn arcsec(&self) -> Angle;
    /// Creates an angle from a value in milliarcseconds.
    fn mas(&self) -> Angle;
    /// Creates an angle from a value in microarcseconds.
    fn uas(&self) -> Angle;
}

impl AngleUnits for f64 {
    fn rad(&self) -> Angle {
        Angle::radians(*self)
    }

    fn deg(&self) -> Angle {
        Angle::degrees(*self)
    }

    fn arcsec(&self) -> Angle {
        Angle::arcseconds(*self)
    }

    fn mas(&self) -> Angle {
        Angle::arcseconds(self * 1e-3)
    }

    fn uas(&self) -> Angle {
        Angle::arcseconds(self * 1e-6)
    }
}

impl AngleUnits for i64 {
    fn rad(&self) -> Angle {
        Angle::radians(*self as f64)
    }

    fn deg(&self) -> Angle {
        Angle::degrees(*self as f64)
    }

    fn arcsec(&self) -> Angle {
        Angle::arcseconds(*self as f64)
    }

    fn mas(&self) -> Angle {
        Angle::arcseconds(*self as f64 * 1e-3)
    }

    fn uas(&self) -> Angle {
        Angle::arcseconds(*self as f64 * 1e-6)
    }
}

/// The astronomical unit in meters.
pub const ASTRONOMICAL_UNIT: f64 = 1.495978707e11;

type Meters = f64;

/// Distance in meters.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[repr(transparent)]
pub struct Distance(Meters);

impl Distance {
    /// Create a new distance from an `f64` value in meters.
    pub const fn new(m: f64) -> Self {
        Self(m)
    }

    /// Create a new distance from an `f64` value in meters.
    pub const fn meters(m: f64) -> Self {
        Self(m)
    }

    /// Create a new distance from an `f64` value in kilometers.
    pub const fn kilometers(m: f64) -> Self {
        Self(m * 1e3)
    }

    /// Create a new distance from an `f64` value in astronomical units.
    pub const fn astronomical_units(m: f64) -> Self {
        Self(m * ASTRONOMICAL_UNIT)
    }

    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the value of the distance in meters.
    pub const fn to_meters(&self) -> f64 {
        self.0
    }

    /// Returns the value of the distance in kilometers.
    pub const fn to_kilometers(&self) -> f64 {
        self.0 * 1e-3
    }

    /// Returns the value of the distance in astronomical units.
    pub const fn to_astronomical_units(&self) -> f64 {
        self.0 / ASTRONOMICAL_UNIT
    }
}

impl Display for Distance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        (1e-3 * self.0).fmt(f)?;
        write!(f, " km")
    }
}

/// A trait for creating [`Distance`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::DistanceUnits;
///
/// let d = 1.km();
/// assert_eq!(d.to_meters(), 1e3);
/// ```
pub trait DistanceUnits {
    /// Creates a distance from a value in meters.
    fn m(&self) -> Distance;
    /// Creates a distance from a value in kilometers.
    fn km(&self) -> Distance;
    /// Creates a distance from a value in astronomical units.
    fn au(&self) -> Distance;
}

impl DistanceUnits for f64 {
    fn m(&self) -> Distance {
        Distance::meters(*self)
    }

    fn km(&self) -> Distance {
        Distance::kilometers(*self)
    }

    fn au(&self) -> Distance {
        Distance::astronomical_units(*self)
    }
}

impl DistanceUnits for i64 {
    fn m(&self) -> Distance {
        Distance::meters(*self as f64)
    }

    fn km(&self) -> Distance {
        Distance::kilometers(*self as f64)
    }

    fn au(&self) -> Distance {
        Distance::astronomical_units(*self as f64)
    }
}

type MetersPerSecond = f64;

/// Velocity in meters per second.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[repr(transparent)]
pub struct Velocity(MetersPerSecond);

impl Velocity {
    /// Creates a new velocity from an `f64` value in m/s.
    pub const fn new(mps: f64) -> Self {
        Self(mps)
    }

    /// Creates a new velocity from an `f64` value in m/s.
    pub const fn meters_per_second(mps: f64) -> Self {
        Self(mps)
    }

    /// Creates a new velocity from an `f64` value in km/s.
    pub const fn kilometers_per_second(mps: f64) -> Self {
        Self(mps * 1e3)
    }

    /// Returns the value of the velocity in m/s.
    pub const fn to_meters_per_second(&self) -> f64 {
        self.0
    }

    /// Returns the value of the velocity in km/s.
    pub const fn to_kilometers_per_second(&self) -> f64 {
        self.0 * 1e-3
    }
}

impl Display for Velocity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        (1e-3 * self.0).fmt(f)?;
        write!(f, " km/s")
    }
}

/// A trait for creating [`Velocity`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::VelocityUnits;
///
/// let v = 1.kps();
/// assert_eq!(v.to_meters_per_second(), 1e3);
/// ```
pub trait VelocityUnits {
    /// Creates a velocity from a value in m/s.
    fn mps(&self) -> Velocity;
    /// Creates a velocity from a value in km/s.
    fn kps(&self) -> Velocity;
}

impl VelocityUnits for f64 {
    fn mps(&self) -> Velocity {
        Velocity::meters_per_second(*self)
    }

    fn kps(&self) -> Velocity {
        Velocity::kilometers_per_second(*self)
    }
}

impl VelocityUnits for i64 {
    fn mps(&self) -> Velocity {
        Velocity::meters_per_second(*self as f64)
    }

    fn kps(&self) -> Velocity {
        Velocity::kilometers_per_second(*self as f64)
    }
}

/// The speed of light in vacuum in m/s.
pub const SPEED_OF_LIGHT: f64 = 299792458.0;

/// IEEE letter codes for frequency bands commonly used for satellite communications.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum FrequencyBand {
    /// HF (High Frequency) – 3 to 30 MHz
    HF,
    /// VHF (Very High Frequency) – 30 to 300 MHz
    VHF,
    /// UHF (Ultra-High Frequency) – 0.3 to 1 GHz
    UHF,
    /// L – 1 to 2 GHz
    L,
    /// S – 2 to 4 GHz
    S,
    /// C – 4 to 8 GHz
    C,
    /// X – 8 to 12 GHz
    X,
    /// Kᵤ – 12 to 18 GHz
    Ku,
    /// K – 18 to 27 GHz
    K,
    /// Kₐ – 27 to 40 GHz
    Ka,
    /// V – 40 to 75 GHz
    V,
    /// W – 75 to 110 GHz
    W,
    /// G – 110 to 300 GHz
    G,
}

type Hertz = f64;

/// Frequency in Hertz
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[repr(transparent)]
pub struct Frequency(Hertz);

impl Frequency {
    /// Creates a new frequency from an `f64` value in Hz.
    pub const fn new(hz: Hertz) -> Self {
        Self(hz)
    }

    /// Creates a new frequency from an `f64` value in Hz.
    pub const fn hertz(hz: Hertz) -> Self {
        Self(hz)
    }

    /// Creates a new frequency from an `f64` value in KHz.
    pub const fn kilohertz(hz: Hertz) -> Self {
        Self(hz * 1e3)
    }

    /// Creates a new frequency from an `f64` value in MHz.
    pub const fn megahertz(hz: Hertz) -> Self {
        Self(hz * 1e6)
    }

    /// Creates a new frequency from an `f64` value in GHz.
    pub const fn gigahertz(hz: Hertz) -> Self {
        Self(hz * 1e9)
    }

    /// Creates a new frequency from an `f64` value in THz.
    pub const fn terahertz(hz: Hertz) -> Self {
        Self(hz * 1e12)
    }

    /// Returns the value of the frequency in Hz.
    pub const fn to_hertz(&self) -> f64 {
        self.0
    }

    /// Returns the value of the frequency in KHz.
    pub const fn to_kilohertz(&self) -> f64 {
        self.0 * 1e-3
    }

    /// Returns the value of the frequency in MHz.
    pub const fn to_megahertz(&self) -> f64 {
        self.0 * 1e-6
    }

    /// Returns the value of the frequency in GHz.
    pub const fn to_gigahertz(&self) -> f64 {
        self.0 * 1e-9
    }

    /// Returns the value of the frequency in THz.
    pub const fn to_terahertz(&self) -> f64 {
        self.0 * 1e-12
    }

    /// Returns the wavelength.
    pub fn wavelength(&self) -> Distance {
        Distance(SPEED_OF_LIGHT / self.0)
    }

    /// Returns the IEEE letter code if the frequency matches one of the bands.
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

/// A trait for creating [`Frequency`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::FrequencyUnits;
///
/// let f = 1.ghz();
/// assert_eq!(f.to_hertz(), 1e9);
/// ```
pub trait FrequencyUnits {
    /// Creates a frequency from a value in Hz.
    fn hz(&self) -> Frequency;
    /// Creates a frequency from a value in KHz.
    fn khz(&self) -> Frequency;
    /// Creates a frequency from a value in MHz.
    fn mhz(&self) -> Frequency;
    /// Creates a frequency from a value in GHz.
    fn ghz(&self) -> Frequency;
    /// Creates a frequency from a value in THz.
    fn thz(&self) -> Frequency;
}

impl FrequencyUnits for f64 {
    fn hz(&self) -> Frequency {
        Frequency::hertz(*self)
    }

    fn khz(&self) -> Frequency {
        Frequency::kilohertz(*self)
    }

    fn mhz(&self) -> Frequency {
        Frequency::megahertz(*self)
    }

    fn ghz(&self) -> Frequency {
        Frequency::gigahertz(*self)
    }

    fn thz(&self) -> Frequency {
        Frequency::terahertz(*self)
    }
}

impl FrequencyUnits for i64 {
    fn hz(&self) -> Frequency {
        Frequency::hertz(*self as f64)
    }

    fn khz(&self) -> Frequency {
        Frequency::kilohertz(*self as f64)
    }

    fn mhz(&self) -> Frequency {
        Frequency::megahertz(*self as f64)
    }

    fn ghz(&self) -> Frequency {
        Frequency::gigahertz(*self as f64)
    }

    fn thz(&self) -> Frequency {
        Frequency::terahertz(*self as f64)
    }
}

macro_rules! trait_impls {
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

            impl From<$unit> for f64 {
                fn from(val: $unit) -> Self {
                    val.0
                }
            }
        )*
    };
}

trait_impls!(Angle, Distance, Frequency, Velocity);

#[cfg(test)]
mod tests {
    use alloc::format;
    use std::f64::consts::{FRAC_PI_2, PI};

    use lox_test_utils::assert_approx_eq;
    use rstest::rstest;

    extern crate alloc;

    use super::*;

    #[test]
    fn test_angle_deg() {
        let angle = 90.0.deg();
        assert_approx_eq!(angle.0, FRAC_PI_2, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_rad() {
        let angle = PI.rad();
        assert_approx_eq!(angle.0, PI, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_conversions() {
        let angle_deg = 180.0.deg();
        let angle_rad = PI.rad();
        assert_approx_eq!(angle_deg.0, angle_rad.0, rtol <= 1e-10);
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
        // atol is preferred to rtol for floating-point comparisons with 0.0. See
        // https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/#inferna
        if exp == 0.0 {
            assert_approx_eq!(angle.normalize_two_pi(center).0, exp, atol <= TOLERANCE);
        } else {
            assert_approx_eq!(angle.normalize_two_pi(center).0, exp, rtol <= TOLERANCE);
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
        assert_approx_eq!(d1.0, d2.0, rtol <= 1e-9);
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
        assert_approx_eq!(wavelength.0, 0.299792458, rtol <= 1e-9);
    }

    #[test]
    fn test_frequency_wavelength_speed_of_light() {
        let f = 299792458.0.hz(); // 1 Hz at speed of light
        let wavelength = f.wavelength();
        assert_approx_eq!(wavelength.0, 1.0, rtol <= 1e-10);
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
