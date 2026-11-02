// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2013-2021 NumFOCUS Foundation
//
// SPDX-License-Identifier: MPL-2.0 AND LicenseRef-ERFA

//! Newtype wrappers for unitful [`f64`] double precision values
//!
//! All quantities store SI units. `Display` prints the SI value with its unit symbol, e.g.
//! `7000 m`; `display_in` formats in another unit, and `FromStr` parses `"<value> <unit>"`
//! strings such as `"7 km"`. With the `serde` feature, quantities serialize and deserialize
//! as plain numbers in SI units.
//!
//! Error messages and the `Display` of domain types such as orbital elements instead use the
//! unit a reader expects in context, e.g. kilometers for orbital distances and degrees for
//! angles.

mod runtime;
mod sexagesimal;

pub use runtime::{
    AngleUnit, AngularRateUnit, AreaToMassUnit, AreaUnit, DataRateUnit, DataVolumeUnit,
    DecibelUnit, DecibelWattsUnit, DisplayIn, DistanceUnit, FrequencyUnit, MassUnit,
    ParseUnitError, PowerUnit, PressureUnit, TemperatureUnit, VelocityUnit,
};
pub use sexagesimal::{Dms, Hms, SexagesimalError};

use core::{
    cmp::Ordering,
    f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU},
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use glam::DMat3;
use lox_approx::ApproxEq;

use crate::f64::consts::SECONDS_PER_DAY;
use crate::math::float::{
    abs, acos, acosh, asin, asinh, atan, atan2, atanh, cos, cosh, log10, powf, sin, sin_cos, sinh,
    tan, tanh, to_degrees, to_radians,
};

/// Degrees in full circle
pub const DEGREES_IN_CIRCLE: f64 = 360.0;

/// Arcseconds in full circle
pub const ARCSECONDS_IN_CIRCLE: f64 = DEGREES_IN_CIRCLE * 60.0 * 60.0;

/// Radians per arcsecond
pub const RADIANS_IN_ARCSECOND: f64 = TAU / ARCSECONDS_IN_CIRCLE;

/// Radians per second of time
const RADIANS_IN_SECOND_OF_TIME: f64 = TAU / SECONDS_PER_DAY;

/// Sign of a signed component. Used for HMS/DMS decomposition.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Sign {
    /// Positive sign (`+`).
    Positive,
    /// Negative sign (`-`).
    Negative,
}

impl Sign {
    /// Returns +1.0 for `Positive` and -1.0 for `Negative`.
    ///
    /// `const` so it can be called from `const fn` composition methods.
    pub const fn as_f64(self) -> f64 {
        match self {
            Sign::Positive => 1.0,
            Sign::Negative => -1.0,
        }
    }
}

impl Display for Sign {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match self {
                Sign::Positive => "+",
                Sign::Negative => "-",
            }
        )
    }
}

impl From<f64> for Sign {
    fn from(x: f64) -> Self {
        // `is_sign_negative()` reads the IEEE sign bit and correctly
        // classifies -0.0 as `Negative`.
        if x.is_sign_negative() {
            Sign::Negative
        } else {
            Sign::Positive
        }
    }
}

impl From<Sign> for f64 {
    fn from(s: Sign) -> f64 {
        s.as_f64()
    }
}

macro_rules! impl_sign_from_signed_int {
    ($($t:ty),+ $(,)?) => {
        $(
            impl From<$t> for Sign {
                fn from(x: $t) -> Self {
                    if x < 0 { Sign::Negative } else { Sign::Positive }
                }
            }
        )+
    };
}
impl_sign_from_signed_int!(i8, i16, i32, i64, isize);

/// Angle in radians.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Angle(f64);

impl Angle {
    /// An angle equal to π.
    pub const PI: Self = Self(PI);
    /// An angle equal to τ = 2π.
    pub const TAU: Self = Self(TAU);
    /// An angle equal to π/2.
    pub const FRAC_PI_2: Self = Self(FRAC_PI_2);
    /// An angle equal to π/4.
    pub const FRAC_PI_4: Self = Self(FRAC_PI_4);

    /// Creates a new angle from an `f64` value in radians.
    pub const fn radians(rad: f64) -> Self {
        Self(rad)
    }

    /// Creates a new angle from an `f64` value in microradians.
    pub const fn microradians(urad: f64) -> Self {
        Self(urad / 1e6)
    }

    /// Creates a new angle from an `f64` value in degrees.
    pub const fn degrees(deg: f64) -> Self {
        Self(to_radians(deg))
    }

    /// Creates a new angle from an `f64` value in arcseconds.
    pub const fn arcseconds(asec: f64) -> Self {
        Self(asec * RADIANS_IN_ARCSECOND)
    }

    /// Creates a new angle from an `f64` value in milliarcseconds.
    pub const fn milliarcseconds(mas: f64) -> Self {
        Self::arcseconds(mas / 1e3)
    }

    /// Creates a new angle from an `f64` value in microarcseconds.
    pub const fn microarcseconds(uas: f64) -> Self {
        Self::arcseconds(uas / 1e6)
    }

    /// Creates a new angle from an `f64` value in arcseconds, reduced to the remainder of a full
    /// circle with the sign of `asec`, in the interval (-2π, 2π).
    ///
    /// Equivalent to `Angle::arcseconds(asec).rem_two_pi()`, but without the loss of precision
    /// for large values.
    pub const fn arcseconds_rem(asec: f64) -> Self {
        Self::arcseconds(asec % ARCSECONDS_IN_CIRCLE)
    }

    /// Creates a new angle from an `f64` value in seconds of time, with 86400 seconds to the full
    /// circle.
    ///
    /// Seconds of time are an angle unit for hour angles and sidereal time, not a duration.
    pub const fn seconds_of_time(s: f64) -> Self {
        Self(s * RADIANS_IN_SECOND_OF_TIME)
    }

    /// Returns `true` if the angle is exactly zero.
    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }

    /// Creates an angle from the arcsine of a value.
    pub fn from_asin(value: f64) -> Self {
        Self(asin(value))
    }

    /// Creates an angle from the inverse hyperbolic sine of a value.
    pub fn from_asinh(value: f64) -> Self {
        Self(asinh(value))
    }

    /// Creates an angle from the arccosine of a value.
    pub fn from_acos(value: f64) -> Self {
        Self(acos(value))
    }

    /// Creates an angle from the inverse hyperbolic cosine of a value.
    pub fn from_acosh(value: f64) -> Self {
        Self(acosh(value))
    }

    /// Creates an angle from the arctangent of a value.
    pub fn from_atan(value: f64) -> Self {
        Self(atan(value))
    }

    /// Creates an angle from the inverse hyperbolic tangent of a value.
    pub fn from_atanh(value: f64) -> Self {
        Self(atanh(value))
    }

    /// Creates an angle from the two-argument arctangent of `y` and `x`.
    pub fn from_atan2(y: f64, x: f64) -> Self {
        Self(atan2(y, x))
    }

    /// Returns the cosine of the angle.
    pub fn cos(self) -> f64 {
        cos(self.0)
    }

    /// Returns the hyperbolic cosine of the angle.
    pub fn cosh(self) -> f64 {
        cosh(self.0)
    }

    /// Returns the sine of the angle.
    pub fn sin(self) -> f64 {
        sin(self.0)
    }

    /// Returns the hyperbolic sine of the angle.
    pub fn sinh(self) -> f64 {
        sinh(self.0)
    }

    /// Returns sine and cosine of the angle.
    pub fn sin_cos(self) -> (f64, f64) {
        sin_cos(self.0)
    }

    /// Returns the tangent of the angle.
    pub fn tan(self) -> f64 {
        tan(self.0)
    }

    /// Returns the hyperbolic tangent of the angle.
    pub fn tanh(self) -> f64 {
        tanh(self.0)
    }

    /// Returns the angle wrapped to the interval [0, 2π).
    pub const fn wrap_unsigned(self) -> Self {
        let mut a = self.0 % TAU;
        if a < 0.0 {
            a += TAU;
            // A tiny negative remainder rounds up to 2π, whose closest representative is 0.
            if a == TAU {
                a = 0.0;
            }
        }
        // Adding zero turns -0.0 into 0.0.
        Self(a + 0.0)
    }

    /// Returns the angle wrapped to the interval [-π, π).
    pub const fn wrap_signed(self) -> Self {
        self.wrap_around(Self::ZERO)
    }

    /// Returns the angle wrapped to the interval [center - π, center + π).
    ///
    /// The result is accurate for centers well below 10¹⁶ rad in magnitude; beyond that, `center`
    /// and `center ± π` are no longer distinct in floating point.
    pub const fn wrap_around(self, center: Self) -> Self {
        // The remainder is exact, and so is the shift by 2π, which leaves `w` in [-π, π).
        let mut w = (self.0 - center.0) % TAU;
        if w >= PI {
            w -= TAU;
        } else if w < -PI {
            w += TAU;
        }
        let mut a = w + center.0;
        // Adding a non-zero center can round onto the open upper bound. The same value one turn
        // down then lies on the closed lower bound, but can round to just below it.
        if a >= center.0 + PI {
            a -= TAU;
            if a < center.0 - PI {
                a = center.0 - PI;
            }
        }
        Self(a)
    }

    /// Returns the remainder of the angle divided by 2π, with the sign of the angle, in the
    /// interval (-2π, 2π).
    pub const fn rem_two_pi(self) -> Self {
        Self(self.0 % TAU)
    }

    /// Returns the value of the angle in radians.
    pub const fn to_radians(self) -> f64 {
        self.0
    }

    /// Returns the value of the angle in microradians.
    pub const fn to_microradians(self) -> f64 {
        self.0 * 1e6
    }

    /// Returns the value of the angle in degrees.
    pub const fn to_degrees(self) -> f64 {
        to_degrees(self.0)
    }

    /// Returns the value of the angle in arcseconds.
    pub const fn to_arcseconds(self) -> f64 {
        self.0 / RADIANS_IN_ARCSECOND
    }

    /// Returns the value of the angle in seconds of time, with 86400 seconds to the full circle.
    pub const fn to_seconds_of_time(self) -> f64 {
        self.0 / RADIANS_IN_SECOND_OF_TIME
    }

    /// Returns the value of the angle in milliarcseconds.
    pub const fn to_milliarcseconds(self) -> f64 {
        self.to_arcseconds() * 1e3
    }

    /// Returns the value of the angle in microarcseconds.
    pub const fn to_microarcseconds(self) -> f64 {
        self.to_arcseconds() * 1e6
    }

    /// Returns the 3×3 rotation matrix for a rotation about the X axis.
    pub fn rotation_x(self) -> DMat3 {
        DMat3::from_rotation_x(-self.to_radians())
    }

    /// Returns the 3×3 rotation matrix for a rotation about the Y axis.
    pub fn rotation_y(self) -> DMat3 {
        DMat3::from_rotation_y(-self.to_radians())
    }

    /// Returns the 3×3 rotation matrix for a rotation about the Z axis.
    pub fn rotation_z(self) -> DMat3 {
        DMat3::from_rotation_z(-self.to_radians())
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
        Angle::milliarcseconds(*self)
    }

    fn uas(&self) -> Angle {
        Angle::microarcseconds(*self)
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
        Angle::milliarcseconds(*self as f64)
    }

    fn uas(&self) -> Angle {
        Angle::microarcseconds(*self as f64)
    }
}

/// The astronomical unit in meters.
pub const ASTRONOMICAL_UNIT: f64 = 1.495978707e11;

/// Distance in meters.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Distance(f64);

impl Distance {
    /// Create a new distance from an `f64` value in meters.
    pub const fn meters(m: f64) -> Self {
        Self(m)
    }

    /// Create a new distance from an `f64` value in kilometers.
    pub const fn kilometers(km: f64) -> Self {
        Self(km * 1e3)
    }

    /// Create a new distance from an `f64` value in astronomical units.
    pub const fn astronomical_units(au: f64) -> Self {
        Self(au * ASTRONOMICAL_UNIT)
    }

    /// Returns the value of the distance in meters.
    pub const fn to_meters(self) -> f64 {
        self.0
    }

    /// Returns the value of the distance in kilometers.
    pub const fn to_kilometers(self) -> f64 {
        self.0 / 1e3
    }

    /// Returns the value of the distance in astronomical units.
    pub const fn to_astronomical_units(self) -> f64 {
        self.0 / ASTRONOMICAL_UNIT
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

/// Velocity in meters per second.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Velocity(f64);

impl Velocity {
    /// Creates a new velocity from an `f64` value in m/s.
    pub const fn meters_per_second(mps: f64) -> Self {
        Self(mps)
    }

    /// Creates a new velocity from an `f64` value in km/s.
    pub const fn kilometers_per_second(kps: f64) -> Self {
        Self(kps * 1e3)
    }

    /// Creates a new velocity from an `f64` value in au/d.
    pub const fn astronomical_units_per_day(aud: f64) -> Self {
        Self(aud * ASTRONOMICAL_UNIT / SECONDS_PER_DAY)
    }

    /// Creates a new velocity from an `f64` value in 1/c.
    pub const fn fraction_of_speed_of_light(c: f64) -> Self {
        Self(c * SPEED_OF_LIGHT)
    }

    /// Returns the value of the velocity in m/s.
    pub const fn to_meters_per_second(self) -> f64 {
        self.0
    }

    /// Returns the value of the velocity in km/s.
    pub const fn to_kilometers_per_second(self) -> f64 {
        self.0 / 1e3
    }

    /// Returns the value of the velocity in au/d.
    pub const fn to_astronomical_units_per_day(self) -> f64 {
        self.0 * SECONDS_PER_DAY / ASTRONOMICAL_UNIT
    }

    /// Returns the value of the velocity in 1/c.
    pub const fn to_fraction_of_speed_of_light(self) -> f64 {
        self.0 / SPEED_OF_LIGHT
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
    /// Creates a velocity from a value in au/d.
    fn aud(&self) -> Velocity;
    /// Crates a velocity from a value in 1/c (fraction of the speed of light).
    fn c(&self) -> Velocity;
}

impl VelocityUnits for f64 {
    fn mps(&self) -> Velocity {
        Velocity::meters_per_second(*self)
    }

    fn kps(&self) -> Velocity {
        Velocity::kilometers_per_second(*self)
    }

    fn aud(&self) -> Velocity {
        Velocity::astronomical_units_per_day(*self)
    }

    fn c(&self) -> Velocity {
        Velocity::fraction_of_speed_of_light(*self)
    }
}

impl VelocityUnits for i64 {
    fn mps(&self) -> Velocity {
        Velocity::meters_per_second(*self as f64)
    }

    fn kps(&self) -> Velocity {
        Velocity::kilometers_per_second(*self as f64)
    }

    fn aud(&self) -> Velocity {
        Velocity::astronomical_units_per_day(*self as f64)
    }

    fn c(&self) -> Velocity {
        Velocity::fraction_of_speed_of_light(*self as f64)
    }
}

/// The speed of light in vacuum in m/s.
pub const SPEED_OF_LIGHT: f64 = 299792458.0;

/// The comoving radius of the observable universe, about 46.5 billion light-years, in m.
pub const OBSERVABLE_UNIVERSE_RADIUS: f64 = 4.4e26;

/// Frequency in Hertz
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Frequency(f64);

impl Frequency {
    /// Creates a new frequency from an `f64` value in Hz.
    pub const fn hertz(hz: f64) -> Self {
        Self(hz)
    }

    /// Creates a new frequency from an `f64` value in KHz.
    pub const fn kilohertz(khz: f64) -> Self {
        Self(khz * 1e3)
    }

    /// Creates a new frequency from an `f64` value in MHz.
    pub const fn megahertz(mhz: f64) -> Self {
        Self(mhz * 1e6)
    }

    /// Creates a new frequency from an `f64` value in GHz.
    pub const fn gigahertz(ghz: f64) -> Self {
        Self(ghz * 1e9)
    }

    /// Creates a new frequency from an `f64` value in THz.
    pub const fn terahertz(thz: f64) -> Self {
        Self(thz * 1e12)
    }

    /// Returns the value of the frequency in Hz.
    pub const fn to_hertz(self) -> f64 {
        self.0
    }

    /// Returns the value of the frequency in KHz.
    pub const fn to_kilohertz(self) -> f64 {
        self.0 / 1e3
    }

    /// Returns the value of the frequency in MHz.
    pub const fn to_megahertz(self) -> f64 {
        self.0 / 1e6
    }

    /// Returns the value of the frequency in GHz.
    pub const fn to_gigahertz(self) -> f64 {
        self.0 / 1e9
    }

    /// Returns the value of the frequency in THz.
    pub const fn to_terahertz(self) -> f64 {
        self.0 / 1e12
    }

    /// Returns the wavelength.
    pub fn wavelength(self) -> Distance {
        Distance(SPEED_OF_LIGHT / self.0)
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

/// Mass in kilograms.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Mass(f64);

impl Mass {
    /// Creates a new mass from an `f64` value in kilograms.
    pub const fn kilograms(kg: f64) -> Self {
        Self(kg)
    }

    /// Creates a new mass from an `f64` value in grams.
    pub const fn grams(g: f64) -> Self {
        Self(g / 1e3)
    }

    /// Creates a new mass from an `f64` value in metric tons (1000 kg).
    pub const fn metric_tons(t: f64) -> Self {
        Self(t * 1e3)
    }

    /// Returns the value of the mass in kilograms.
    pub const fn to_kilograms(self) -> f64 {
        self.0
    }

    /// Returns the value of the mass in grams.
    pub const fn to_grams(self) -> f64 {
        self.0 * 1e3
    }

    /// Returns the value of the mass in metric tons.
    pub const fn to_metric_tons(self) -> f64 {
        self.0 / 1e3
    }
}

/// A trait for creating [`Mass`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::MassUnits;
///
/// let m = 1.kg();
/// assert_eq!(m.to_kilograms(), 1.0);
/// ```
pub trait MassUnits {
    /// Creates a mass from a value in kilograms.
    fn kg(&self) -> Mass;
    /// Creates a mass from a value in grams.
    fn g(&self) -> Mass;
    /// Creates a mass from a value in metric tons.
    fn t(&self) -> Mass;
}

impl MassUnits for f64 {
    fn kg(&self) -> Mass {
        Mass::kilograms(*self)
    }

    fn g(&self) -> Mass {
        Mass::grams(*self)
    }

    fn t(&self) -> Mass {
        Mass::metric_tons(*self)
    }
}

impl MassUnits for i64 {
    fn kg(&self) -> Mass {
        Mass::kilograms(*self as f64)
    }

    fn g(&self) -> Mass {
        Mass::grams(*self as f64)
    }

    fn t(&self) -> Mass {
        Mass::metric_tons(*self as f64)
    }
}

/// Area in square meters.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Area(f64);

impl Area {
    /// Creates a new area from an `f64` value in square meters.
    pub const fn square_meters(m2: f64) -> Self {
        Self(m2)
    }

    /// Creates a new area from an `f64` value in square kilometers.
    pub const fn square_kilometers(km2: f64) -> Self {
        Self(km2 * 1e6)
    }

    /// Returns the value of the area in square meters.
    pub const fn to_square_meters(self) -> f64 {
        self.0
    }

    /// Returns the value of the area in square kilometers.
    pub const fn to_square_kilometers(self) -> f64 {
        self.0 / 1e6
    }
}

/// A trait for creating [`Area`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::AreaUnits;
///
/// let a = 4.m2();
/// assert_eq!(a.to_square_meters(), 4.0);
/// ```
pub trait AreaUnits {
    /// Creates an area from a value in square meters.
    fn m2(&self) -> Area;
    /// Creates an area from a value in square kilometers.
    fn km2(&self) -> Area;
}

impl AreaUnits for f64 {
    fn m2(&self) -> Area {
        Area::square_meters(*self)
    }

    fn km2(&self) -> Area {
        Area::square_kilometers(*self)
    }
}

impl AreaUnits for i64 {
    fn m2(&self) -> Area {
        Area::square_meters(*self as f64)
    }

    fn km2(&self) -> Area {
        Area::square_kilometers(*self as f64)
    }
}

/// Area-to-mass ratio in m²/kg.
///
/// Used for ballistic and radiation-pressure coefficients (e.g. the OMM
/// `BTERM` and `AGOM` fields in CCSDS 502.0-B-3).
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct AreaToMass(f64);

impl AreaToMass {
    /// Creates a new area-to-mass ratio from an `f64` value in m²/kg.
    pub const fn square_meters_per_kilogram(m2_per_kg: f64) -> Self {
        Self(m2_per_kg)
    }

    /// Returns the value in m²/kg.
    pub const fn to_square_meters_per_kilogram(self) -> f64 {
        self.0
    }
}

/// A trait for creating [`AreaToMass`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::AreaToMassUnits;
///
/// let r = 0.05.m2_per_kg();
/// assert_eq!(r.to_square_meters_per_kilogram(), 0.05);
/// ```
pub trait AreaToMassUnits {
    /// Creates an area-to-mass ratio from a value in m²/kg.
    fn m2_per_kg(&self) -> AreaToMass;
}

impl AreaToMassUnits for f64 {
    fn m2_per_kg(&self) -> AreaToMass {
        AreaToMass::square_meters_per_kilogram(*self)
    }
}

impl AreaToMassUnits for i64 {
    fn m2_per_kg(&self) -> AreaToMass {
        AreaToMass::square_meters_per_kilogram(*self as f64)
    }
}

/// Temperature in Kelvin.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Temperature(f64);

impl Temperature {
    /// Creates a new temperature from an `f64` value in Kelvin.
    pub const fn kelvin(k: f64) -> Self {
        Self(k)
    }

    /// Returns the value in Kelvin.
    pub const fn to_kelvin(self) -> f64 {
        self.0
    }
}

/// Extension trait for ergonomic temperature construction.
///
/// # Example
///
/// ```
/// use lox_core::units::TemperatureUnits;
///
/// let t = 290.k();
/// assert_eq!(t.to_kelvin(), 290.0);
/// ```
pub trait TemperatureUnits {
    /// Creates a temperature from a value in Kelvin.
    fn k(&self) -> Temperature;
}

impl TemperatureUnits for f64 {
    fn k(&self) -> Temperature {
        Temperature::kelvin(*self)
    }
}

impl TemperatureUnits for i64 {
    fn k(&self) -> Temperature {
        Temperature::kelvin(*self as f64)
    }
}

/// Pressure in pascals.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Pressure(f64);

impl Pressure {
    /// Creates a new pressure from an `f64` value in pascals.
    pub const fn pascals(pa: f64) -> Self {
        Self(pa)
    }

    /// Creates a new pressure from an `f64` value in hectopascals.
    pub const fn hectopascals(hpa: f64) -> Self {
        Self(hpa * 100.0)
    }

    /// Returns the value in pascals.
    pub const fn to_pascals(self) -> f64 {
        self.0
    }

    /// Returns the value in hectopascals.
    pub const fn to_hectopascals(self) -> f64 {
        self.0 / 100.0
    }
}

/// Power in Watts.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Power(f64);

impl Power {
    /// Creates a new power from an `f64` value in Watts.
    pub const fn watts(w: f64) -> Self {
        Self(w)
    }

    /// Creates a new power from an `f64` value in kilowatts.
    pub const fn kilowatts(kw: f64) -> Self {
        Self(kw * 1e3)
    }

    /// Returns the value in Watts.
    pub const fn to_watts(self) -> f64 {
        self.0
    }

    /// Returns the value in kilowatts.
    pub const fn to_kilowatts(self) -> f64 {
        self.0 / 1e3
    }

    /// Returns the power level in dBW.
    pub fn to_decibel_watts(self) -> DecibelWatts {
        DecibelWatts(10.0 * log10(self.0))
    }
}

/// Extension trait for ergonomic power construction.
///
/// # Example
///
/// ```
/// use lox_core::units::PowerUnits;
///
/// let p = 2.w();
/// assert_eq!(p.to_watts(), 2.0);
/// ```
pub trait PowerUnits {
    /// Creates a power from a value in watts.
    fn w(&self) -> Power;
    /// Creates a power from a value in kilowatts.
    fn kw(&self) -> Power;
}

impl PowerUnits for f64 {
    fn w(&self) -> Power {
        Power::watts(*self)
    }

    fn kw(&self) -> Power {
        Power::kilowatts(*self)
    }
}

impl PowerUnits for i64 {
    fn w(&self) -> Power {
        Power::watts(*self as f64)
    }

    fn kw(&self) -> Power {
        Power::kilowatts(*self as f64)
    }
}

/// Angular rate in radians per second.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct AngularRate(f64);

impl AngularRate {
    /// Creates a new angular rate from an `f64` value in rad/s.
    pub const fn radians_per_second(rps: f64) -> Self {
        Self(rps)
    }

    /// Creates a new angular rate from an `f64` value in deg/s.
    pub const fn degrees_per_second(dps: f64) -> Self {
        Self(to_radians(dps))
    }

    /// Returns the value in rad/s.
    pub const fn to_radians_per_second(self) -> f64 {
        self.0
    }

    /// Returns the value in deg/s.
    pub const fn to_degrees_per_second(self) -> f64 {
        to_degrees(self.0)
    }
}

/// A value in decibels.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Decibel(f64);

impl Decibel {
    /// Creates a new decibel value from an `f64` value in dB.
    pub const fn decibels(db: f64) -> Self {
        Self(db)
    }

    /// Returns the value in dB.
    pub const fn to_decibels(self) -> f64 {
        self.0
    }

    /// Converts a linear power-ratio value to decibels.
    pub fn from_linear(val: f64) -> Self {
        Self(10.0 * log10(val))
    }

    /// Converts this decibel value to a linear power-ratio.
    pub fn to_linear(self) -> f64 {
        powf(10.0_f64, self.0 / 10.0)
    }
}

/// A trait for creating [`Decibel`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::DecibelUnits;
///
/// let d = 3.0.db();
/// assert_eq!(d.to_decibels(), 3.0);
/// ```
pub trait DecibelUnits {
    /// Creates a decibel value.
    fn db(&self) -> Decibel;
}

impl DecibelUnits for f64 {
    fn db(&self) -> Decibel {
        Decibel::decibels(*self)
    }
}

impl DecibelUnits for i64 {
    fn db(&self) -> Decibel {
        Decibel::decibels(*self as f64)
    }
}

/// Bits in a kibibyte (2¹⁰ bytes).
const BITS_PER_KIBIBYTE: f64 = 8.0 * 1024.0;
/// Bits in a mebibyte (2²⁰ bytes).
const BITS_PER_MEBIBYTE: f64 = BITS_PER_KIBIBYTE * 1024.0;
/// Bits in a gibibyte (2³⁰ bytes).
const BITS_PER_GIBIBYTE: f64 = BITS_PER_MEBIBYTE * 1024.0;
/// Bits in a tebibyte (2⁴⁰ bytes).
const BITS_PER_TEBIBYTE: f64 = BITS_PER_GIBIBYTE * 1024.0;

/// Data rate in bits per second.
///
/// Symbol and chip rates are [`Frequency`] values; converting between them and a data rate
/// requires the bits per symbol of the modulation.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct DataRate(f64);

impl DataRate {
    /// Creates a new data rate from an `f64` value in bit/s.
    pub const fn bits_per_second(bps: f64) -> Self {
        Self(bps)
    }

    /// Creates a new data rate from an `f64` value in kbit/s.
    pub const fn kilobits_per_second(kbps: f64) -> Self {
        Self(kbps * 1e3)
    }

    /// Creates a new data rate from an `f64` value in Mbit/s.
    pub const fn megabits_per_second(mbps: f64) -> Self {
        Self(mbps * 1e6)
    }

    /// Creates a new data rate from an `f64` value in Gbit/s.
    pub const fn gigabits_per_second(gbps: f64) -> Self {
        Self(gbps * 1e9)
    }

    /// Returns the value in bit/s.
    pub const fn to_bits_per_second(self) -> f64 {
        self.0
    }

    /// Returns the value in kbit/s.
    pub const fn to_kilobits_per_second(self) -> f64 {
        self.0 / 1e3
    }

    /// Returns the value in Mbit/s.
    pub const fn to_megabits_per_second(self) -> f64 {
        self.0 / 1e6
    }

    /// Returns the value in Gbit/s.
    pub const fn to_gigabits_per_second(self) -> f64 {
        self.0 / 1e9
    }
}

/// A trait for creating [`DataRate`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::DataRateUnits;
///
/// let r = 150.mbps();
/// assert_eq!(r.to_bits_per_second(), 150e6);
/// ```
pub trait DataRateUnits {
    /// Creates a data rate from a value in bit/s.
    fn bps(&self) -> DataRate;
    /// Creates a data rate from a value in kbit/s.
    fn kbps(&self) -> DataRate;
    /// Creates a data rate from a value in Mbit/s.
    fn mbps(&self) -> DataRate;
    /// Creates a data rate from a value in Gbit/s.
    fn gbps(&self) -> DataRate;
}

impl DataRateUnits for f64 {
    fn bps(&self) -> DataRate {
        DataRate::bits_per_second(*self)
    }

    fn kbps(&self) -> DataRate {
        DataRate::kilobits_per_second(*self)
    }

    fn mbps(&self) -> DataRate {
        DataRate::megabits_per_second(*self)
    }

    fn gbps(&self) -> DataRate {
        DataRate::gigabits_per_second(*self)
    }
}

impl DataRateUnits for i64 {
    fn bps(&self) -> DataRate {
        DataRate::bits_per_second(*self as f64)
    }

    fn kbps(&self) -> DataRate {
        DataRate::kilobits_per_second(*self as f64)
    }

    fn mbps(&self) -> DataRate {
        DataRate::megabits_per_second(*self as f64)
    }

    fn gbps(&self) -> DataRate {
        DataRate::gigabits_per_second(*self as f64)
    }
}

/// Data volume in bits.
///
/// Byte units come in decimal (`kilobytes`, 10³ bytes) and binary (`kibibytes`, 2¹⁰ bytes)
/// variants.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct DataVolume(f64);

impl DataVolume {
    /// Creates a new data volume from an `f64` value in bits.
    pub const fn bits(bits: f64) -> Self {
        Self(bits)
    }

    /// Creates a new data volume from an `f64` value in kilobits (10³ bits).
    pub const fn kilobits(kbit: f64) -> Self {
        Self(kbit * 1e3)
    }

    /// Creates a new data volume from an `f64` value in megabits (10⁶ bits).
    pub const fn megabits(mbit: f64) -> Self {
        Self(mbit * 1e6)
    }

    /// Creates a new data volume from an `f64` value in gigabits (10⁹ bits).
    pub const fn gigabits(gbit: f64) -> Self {
        Self(gbit * 1e9)
    }

    /// Creates a new data volume from an `f64` value in terabits (10¹² bits).
    pub const fn terabits(tbit: f64) -> Self {
        Self(tbit * 1e12)
    }

    /// Creates a new data volume from an `f64` value in bytes (8 bits).
    pub const fn bytes(bytes: f64) -> Self {
        Self(bytes * 8.0)
    }

    /// Creates a new data volume from an `f64` value in kilobytes (10³ bytes).
    pub const fn kilobytes(kb: f64) -> Self {
        Self(kb * 8e3)
    }

    /// Creates a new data volume from an `f64` value in megabytes (10⁶ bytes).
    pub const fn megabytes(mb: f64) -> Self {
        Self(mb * 8e6)
    }

    /// Creates a new data volume from an `f64` value in gigabytes (10⁹ bytes).
    pub const fn gigabytes(gb: f64) -> Self {
        Self(gb * 8e9)
    }

    /// Creates a new data volume from an `f64` value in terabytes (10¹² bytes).
    pub const fn terabytes(tb: f64) -> Self {
        Self(tb * 8e12)
    }

    /// Creates a new data volume from an `f64` value in kibibytes (2¹⁰ bytes).
    pub const fn kibibytes(kib: f64) -> Self {
        Self(kib * BITS_PER_KIBIBYTE)
    }

    /// Creates a new data volume from an `f64` value in mebibytes (2²⁰ bytes).
    pub const fn mebibytes(mib: f64) -> Self {
        Self(mib * BITS_PER_MEBIBYTE)
    }

    /// Creates a new data volume from an `f64` value in gibibytes (2³⁰ bytes).
    pub const fn gibibytes(gib: f64) -> Self {
        Self(gib * BITS_PER_GIBIBYTE)
    }

    /// Creates a new data volume from an `f64` value in tebibytes (2⁴⁰ bytes).
    pub const fn tebibytes(tib: f64) -> Self {
        Self(tib * BITS_PER_TEBIBYTE)
    }

    /// Returns the value in bits.
    pub const fn to_bits(self) -> f64 {
        self.0
    }

    /// Returns the value in kilobits (10³ bits).
    pub const fn to_kilobits(self) -> f64 {
        self.0 / 1e3
    }

    /// Returns the value in megabits (10⁶ bits).
    pub const fn to_megabits(self) -> f64 {
        self.0 / 1e6
    }

    /// Returns the value in gigabits (10⁹ bits).
    pub const fn to_gigabits(self) -> f64 {
        self.0 / 1e9
    }

    /// Returns the value in terabits (10¹² bits).
    pub const fn to_terabits(self) -> f64 {
        self.0 / 1e12
    }

    /// Returns the value in bytes (8 bits).
    pub const fn to_bytes(self) -> f64 {
        self.0 / 8.0
    }

    /// Returns the value in kilobytes (10³ bytes).
    pub const fn to_kilobytes(self) -> f64 {
        self.0 / 8e3
    }

    /// Returns the value in megabytes (10⁶ bytes).
    pub const fn to_megabytes(self) -> f64 {
        self.0 / 8e6
    }

    /// Returns the value in gigabytes (10⁹ bytes).
    pub const fn to_gigabytes(self) -> f64 {
        self.0 / 8e9
    }

    /// Returns the value in terabytes (10¹² bytes).
    pub const fn to_terabytes(self) -> f64 {
        self.0 / 8e12
    }

    /// Returns the value in kibibytes (2¹⁰ bytes).
    pub const fn to_kibibytes(self) -> f64 {
        self.0 / BITS_PER_KIBIBYTE
    }

    /// Returns the value in mebibytes (2²⁰ bytes).
    pub const fn to_mebibytes(self) -> f64 {
        self.0 / BITS_PER_MEBIBYTE
    }

    /// Returns the value in gibibytes (2³⁰ bytes).
    pub const fn to_gibibytes(self) -> f64 {
        self.0 / BITS_PER_GIBIBYTE
    }

    /// Returns the value in tebibytes (2⁴⁰ bytes).
    pub const fn to_tebibytes(self) -> f64 {
        self.0 / BITS_PER_TEBIBYTE
    }
}

/// A trait for creating [`DataVolume`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::DataVolumeUnits;
///
/// let v = 2.bytes();
/// assert_eq!(v.to_bits(), 16.0);
/// ```
pub trait DataVolumeUnits {
    /// Creates a data volume from a value in bits.
    fn bits(&self) -> DataVolume;
    /// Creates a data volume from a value in bytes.
    fn bytes(&self) -> DataVolume;
}

impl DataVolumeUnits for f64 {
    fn bits(&self) -> DataVolume {
        DataVolume::bits(*self)
    }

    fn bytes(&self) -> DataVolume {
        DataVolume::bytes(*self)
    }
}

impl DataVolumeUnits for i64 {
    fn bits(&self) -> DataVolume {
        DataVolume::bits(*self as f64)
    }

    fn bytes(&self) -> DataVolume {
        DataVolume::bytes(*self as f64)
    }
}

/// An absolute power level in decibels relative to one watt (dBW).
///
/// Adding or subtracting a [`Decibel`] ratio gives another power level, and the difference of
/// two power levels is a [`Decibel`] ratio. Power levels cannot be added to each other.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct DecibelWatts(f64);

impl DecibelWatts {
    /// Creates a new power level from an `f64` value in dBW.
    // Named after the unit, like all unit constructors; that it matches the type is incidental.
    #[allow(clippy::self_named_constructors)]
    pub const fn decibel_watts(dbw: f64) -> Self {
        Self(dbw)
    }

    /// Creates a new power level from an `f64` value in dBm.
    pub const fn decibel_milliwatts(dbm: f64) -> Self {
        Self(dbm - 30.0)
    }

    /// Returns the value in dBW.
    pub const fn to_decibel_watts(self) -> f64 {
        self.0
    }

    /// Returns the value in dBm.
    pub const fn to_decibel_milliwatts(self) -> f64 {
        self.0 + 30.0
    }

    /// Returns the linear power.
    pub fn to_power(self) -> Power {
        Power(powf(10.0_f64, self.0 / 10.0))
    }

    /// Returns the minimum of the two values, ignoring NaN.
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// Returns the maximum of the two values, ignoring NaN.
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    /// Restricts the value to the interval `[min, max]`.
    ///
    /// # Panics
    ///
    /// Panics if `min > max`, `min` is NaN, or `max` is NaN.
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }

    /// Returns the ordering between `self` and `other` according to the IEEE 754 `totalOrder`
    /// predicate.
    pub fn total_cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }

    /// Returns `true` if the value is neither infinite nor NaN.
    pub const fn is_finite(self) -> bool {
        self.0.is_finite()
    }

    /// Returns `true` if the value is NaN.
    pub const fn is_nan(self) -> bool {
        self.0.is_nan()
    }
}

impl From<Power> for DecibelWatts {
    fn from(power: Power) -> Self {
        power.to_decibel_watts()
    }
}

impl From<DecibelWatts> for Power {
    fn from(level: DecibelWatts) -> Self {
        level.to_power()
    }
}

impl Add<Decibel> for DecibelWatts {
    type Output = Self;

    fn add(self, rhs: Decibel) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<DecibelWatts> for Decibel {
    type Output = DecibelWatts;

    fn add(self, rhs: DecibelWatts) -> Self::Output {
        DecibelWatts(self.0 + rhs.0)
    }
}

impl AddAssign<Decibel> for DecibelWatts {
    fn add_assign(&mut self, rhs: Decibel) {
        self.0 += rhs.0;
    }
}

impl Sub<Decibel> for DecibelWatts {
    type Output = Self;

    fn sub(self, rhs: Decibel) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign<Decibel> for DecibelWatts {
    fn sub_assign(&mut self, rhs: Decibel) {
        self.0 -= rhs.0;
    }
}

impl Sub for DecibelWatts {
    type Output = Decibel;

    fn sub(self, rhs: Self) -> Self::Output {
        Decibel(self.0 - rhs.0)
    }
}

/// A trait for creating [`DecibelWatts`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::DecibelWattsUnits;
///
/// let p = 30.dbm();
/// assert_eq!(p.to_decibel_watts(), 0.0);
/// ```
pub trait DecibelWattsUnits {
    /// Creates a power level from a value in dBW.
    fn dbw(&self) -> DecibelWatts;
    /// Creates a power level from a value in dBm.
    fn dbm(&self) -> DecibelWatts;
}

impl DecibelWattsUnits for f64 {
    fn dbw(&self) -> DecibelWatts {
        DecibelWatts::decibel_watts(*self)
    }

    fn dbm(&self) -> DecibelWatts {
        DecibelWatts::decibel_milliwatts(*self)
    }
}

impl DecibelWattsUnits for i64 {
    fn dbw(&self) -> DecibelWatts {
        DecibelWatts::decibel_watts(*self as f64)
    }

    fn dbm(&self) -> DecibelWatts {
        DecibelWatts::decibel_milliwatts(*self as f64)
    }
}

/// Implements the arithmetic and comparison API shared by all unit types.
macro_rules! quantity_impls {
    ($($unit:ident),*) => {
        $(
            impl $unit {
                /// The zero value.
                pub const ZERO: Self = Self(0.0);

                /// Returns the absolute value.
                pub fn abs(self) -> Self {
                    Self(abs(self.0))
                }

                /// Returns the minimum of the two values, ignoring NaN.
                pub fn min(self, other: Self) -> Self {
                    Self(self.0.min(other.0))
                }

                /// Returns the maximum of the two values, ignoring NaN.
                pub fn max(self, other: Self) -> Self {
                    Self(self.0.max(other.0))
                }

                /// Restricts the value to the interval `[min, max]`.
                ///
                /// # Panics
                ///
                /// Panics if `min > max`, `min` is NaN, or `max` is NaN.
                pub fn clamp(self, min: Self, max: Self) -> Self {
                    Self(self.0.clamp(min.0, max.0))
                }

                /// Returns the ordering between `self` and `other` according to the IEEE 754
                /// `totalOrder` predicate.
                pub fn total_cmp(&self, other: &Self) -> Ordering {
                    self.0.total_cmp(&other.0)
                }

                /// Returns `true` if the value is neither infinite nor NaN.
                pub const fn is_finite(self) -> bool {
                    self.0.is_finite()
                }

                /// Returns `true` if the value is NaN.
                pub const fn is_nan(self) -> bool {
                    self.0.is_nan()
                }
            }

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
                    self.0 += rhs.0;
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
                    self.0 -= rhs.0;
                }
            }

            impl Mul<f64> for $unit {
                type Output = Self;

                fn mul(self, rhs: f64) -> Self::Output {
                    Self(self.0 * rhs)
                }
            }

            impl Mul<$unit> for f64 {
                type Output = $unit;

                fn mul(self, rhs: $unit) -> Self::Output {
                    $unit(self * rhs.0)
                }
            }

            impl MulAssign<f64> for $unit {
                fn mul_assign(&mut self, rhs: f64) {
                    self.0 *= rhs;
                }
            }

            impl Div<f64> for $unit {
                type Output = Self;

                fn div(self, rhs: f64) -> Self::Output {
                    Self(self.0 / rhs)
                }
            }

            impl DivAssign<f64> for $unit {
                fn div_assign(&mut self, rhs: f64) {
                    self.0 /= rhs;
                }
            }

            impl Div for $unit {
                type Output = f64;

                fn div(self, rhs: Self) -> Self::Output {
                    self.0 / rhs.0
                }
            }

            impl Sum for $unit {
                fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                    Self(iter.map(|q| q.0).sum())
                }
            }

            impl<'a> Sum<&'a $unit> for $unit {
                fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                    Self(iter.map(|q| q.0).sum())
                }
            }
        )*
    };
}

quantity_impls!(
    Angle,
    AngularRate,
    Area,
    AreaToMass,
    DataRate,
    DataVolume,
    Decibel,
    Distance,
    Frequency,
    Mass,
    Power,
    Pressure,
    Temperature,
    Velocity
);

#[cfg(test)]
mod tests {
    use alloc::format;
    use core::f64::consts::{FRAC_PI_2, PI};

    use lox_approx::assert_approx_eq;
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
        assert_eq!(format!("{:.2}", angle), "1.57 rad");
        assert_eq!(
            format!("{:.2}", angle.display_in(AngleUnit::Degrees)),
            "90.12 deg"
        );
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
    fn test_angle_wrap_around(#[case] angle: Angle, #[case] center: Angle, #[case] exp: f64) {
        // atol is preferred to rtol for floating-point comparisons with 0.0. See
        // https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/#inferna
        if exp == 0.0 {
            assert_approx_eq!(angle.wrap_around(center).0, exp, atol <= TOLERANCE);
        } else {
            assert_approx_eq!(angle.wrap_around(center).0, exp, rtol <= TOLERANCE);
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
        assert_eq!(format!("{:.2}", distance), "9123.46 m");
        assert_eq!(
            format!("{:.2}", distance.display_in(DistanceUnit::Kilometers)),
            "9.12 km"
        );
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
        assert_eq!(format!("{:.2}", velocity), "9123.46 m/s");
        assert_eq!(
            format!(
                "{:.2}",
                velocity.display_in(VelocityUnit::KilometersPerSecond)
            ),
            "9.12 km/s"
        );
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
    fn test_temperature_units_trait() {
        assert_eq!(290.0.k().to_kelvin(), 290.0);
        assert_eq!(290.k().to_kelvin(), 290.0);
        assert_eq!(290.0.k().to_kelvin(), 290.0);
    }

    #[test]
    fn test_power_units_trait() {
        assert_eq!(2.0.w().to_watts(), 2.0);
        assert_eq!(2.w().to_watts(), 2.0);
        assert_eq!(1.5.kw().to_watts(), 1500.0);
        assert_eq!(2.kw().to_kilowatts(), 2.0);
        assert_eq!(10.0.w().to_decibel_watts().to_decibel_watts(), 10.0);
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
        assert_eq!(format!("{:.2}", frequency), "2412345600.00 Hz");
        assert_eq!(
            format!("{:.2}", frequency.display_in(FrequencyUnit::Gigahertz)),
            "2.41 GHz"
        );
    }

    #[test]
    fn test_decibel_db() {
        let d = 3.0.db();
        assert_eq!(d.to_decibels(), 3.0);
    }

    #[test]
    fn test_decibel_from_linear() {
        let d = Decibel::from_linear(100.0);
        assert_approx_eq!(d.0, 20.0, rtol <= 1e-10);
    }

    #[test]
    fn test_decibel_to_linear() {
        let d = Decibel::decibels(20.0);
        assert_approx_eq!(d.to_linear(), 100.0, rtol <= 1e-10);
    }

    #[test]
    fn test_decibel_roundtrip() {
        let val = 42.5;
        let d = Decibel::decibels(val);
        let roundtripped = Decibel::from_linear(d.to_linear());
        assert_approx_eq!(roundtripped.0, val, rtol <= 1e-10);
    }

    #[test]
    fn test_decibel_add() {
        let sum = 3.0.db() + 3.0.db();
        assert_approx_eq!(sum.0, 6.0, rtol <= 1e-10);
    }

    #[test]
    fn test_decibel_sub() {
        let diff = 6.0.db() - 3.0.db();
        assert_approx_eq!(diff.0, 3.0, rtol <= 1e-10);
    }

    #[test]
    fn test_decibel_neg() {
        assert_eq!(-3.0.db(), Decibel::decibels(-3.0));
    }

    #[test]
    fn test_decibel_display() {
        let d = 3.0.db();
        assert_eq!(format!("{:.1}", d), "3.0 dB");
    }

    #[test]
    fn test_temperature_kelvin() {
        let t = Temperature::kelvin(300.0);
        assert_eq!(t.to_kelvin(), 300.0);
    }

    #[test]
    fn test_temperature_display() {
        let t = Temperature::kelvin(290.0);
        assert_eq!(format!("{}", t), "290 K");
    }

    #[test]
    fn test_temperature_arithmetic() {
        let a = Temperature::kelvin(100.0);
        let b = Temperature::kelvin(200.0);
        assert_eq!((a + b).to_kelvin(), 300.0);
        assert_eq!((b - a).to_kelvin(), 100.0);
        assert_eq!((-a).to_kelvin(), -100.0);
        assert_eq!((2.0 * a).to_kelvin(), 200.0);
    }

    #[test]
    fn test_power_watts() {
        let p = Power::watts(100.0);
        assert_eq!(p.to_watts(), 100.0);
    }

    #[test]
    fn test_power_kilowatts() {
        let p = Power::kilowatts(1.0);
        assert_eq!(p.to_watts(), 1000.0);
        assert_eq!(p.to_kilowatts(), 1.0);
    }

    #[test]
    fn test_power_dbw() {
        let p = Power::watts(100.0);
        assert_approx_eq!(p.to_decibel_watts().to_decibel_watts(), 20.0, rtol <= 1e-10);
    }

    #[test]
    fn test_power_display() {
        let p = Power::watts(100.0);
        assert_eq!(format!("{}", p), "100 W");
    }

    #[test]
    fn test_power_arithmetic() {
        let a = Power::watts(50.0);
        let b = Power::watts(150.0);
        assert_eq!((a + b).to_watts(), 200.0);
        assert_eq!((b - a).to_watts(), 100.0);
        assert_eq!((-a).to_watts(), -50.0);
    }

    #[test]
    fn test_angular_rate_rps() {
        let ar = AngularRate::radians_per_second(1.0);
        assert_eq!(ar.to_radians_per_second(), 1.0);
        assert_approx_eq!(ar.to_degrees_per_second(), 57.29577951308232, rtol <= 1e-10);
    }

    #[test]
    fn test_angular_rate_dps() {
        let ar = AngularRate::degrees_per_second(180.0);
        assert_approx_eq!(
            ar.to_radians_per_second(),
            core::f64::consts::PI,
            rtol <= 1e-10
        );
    }

    #[test]
    fn test_angular_rate_display() {
        let ar = AngularRate::radians_per_second(1.0);
        assert_eq!(format!("{}", ar), "1 rad/s");
        assert_eq!(
            format!("{:.3}", ar.display_in(AngularRateUnit::DegreesPerSecond)),
            "57.296 deg/s"
        );
    }

    #[test]
    fn test_angular_rate_arithmetic() {
        let a = AngularRate::radians_per_second(1.0);
        let b = AngularRate::radians_per_second(2.0);
        assert_eq!((a + b).to_radians_per_second(), 3.0);
        assert_eq!((b - a).to_radians_per_second(), 1.0);
        assert_eq!((-a).to_radians_per_second(), -1.0);
        assert_eq!((3.0 * a).to_radians_per_second(), 3.0);
    }

    #[test]
    fn test_pressure_hpa() {
        let p = Pressure::hectopascals(1013.25);
        assert_eq!(p.to_hectopascals(), 1013.25);
        assert_approx_eq!(p.to_pascals(), 101325.0, rtol <= 1e-10);
    }

    #[test]
    fn test_pressure_pa() {
        let p = Pressure::pascals(101325.0);
        assert_approx_eq!(p.to_hectopascals(), 1013.25, rtol <= 1e-10);
    }

    #[test]
    fn test_pressure_display() {
        let p = Pressure::pascals(101325.0);
        let s = format!("{}", p);
        assert!(s.contains("Pa"));
    }

    #[test]
    fn test_mass_kilograms() {
        let m = Mass::kilograms(1.5);
        assert_eq!(m.to_kilograms(), 1.5);
    }

    #[test]
    fn test_mass_grams() {
        let m = Mass::grams(2500.0);
        assert_approx_eq!(m.to_kilograms(), 2.5, rtol <= 1e-12);
    }

    #[test]
    fn test_mass_metric_tons() {
        let m = Mass::metric_tons(0.5);
        assert_approx_eq!(m.to_kilograms(), 500.0, rtol <= 1e-12);
    }

    #[test]
    fn test_mass_units_kg() {
        let m = 1.5.kg();
        assert_eq!(m.to_kilograms(), 1.5);
    }

    #[test]
    fn test_mass_units_g() {
        let m = 500.0.g();
        assert_approx_eq!(m.to_kilograms(), 0.5, rtol <= 1e-12);
    }

    #[test]
    fn test_mass_display() {
        let m = 12.5.kg();
        assert_eq!(format!("{:.2}", m), "12.50 kg");
    }

    #[test]
    fn test_mass_neg() {
        assert_eq!(Mass(-1.0), -1.0.kg())
    }

    #[test]
    fn test_area_square_meters() {
        let a = Area::square_meters(2.5);
        assert_eq!(a.to_square_meters(), 2.5);
    }

    #[test]
    fn test_area_square_kilometers() {
        let a = Area::square_kilometers(1.0);
        assert_approx_eq!(a.to_square_meters(), 1e6, rtol <= 1e-12);
    }

    #[test]
    fn test_area_units_m2() {
        let a = 9.0.m2();
        assert_eq!(a.to_square_meters(), 9.0);
    }

    #[test]
    fn test_area_units_km2() {
        let a = 2.0.km2();
        assert_approx_eq!(a.to_square_meters(), 2e6, rtol <= 1e-12);
    }

    #[test]
    fn test_area_display() {
        let a = 4.5.m2();
        assert_eq!(format!("{:.2}", a), "4.50 m²");
    }

    #[test]
    fn test_area_neg() {
        assert_eq!(Area(-1.0), -1.0.m2())
    }

    #[test]
    fn test_area_to_mass_square_meters_per_kilogram() {
        let r = AreaToMass::square_meters_per_kilogram(0.025);
        assert_eq!(r.to_square_meters_per_kilogram(), 0.025);
    }

    #[test]
    fn test_area_to_mass_units_shorthand() {
        let r = 0.05.m2_per_kg();
        assert_eq!(r.to_square_meters_per_kilogram(), 0.05);
    }

    #[test]
    fn test_area_to_mass_display() {
        let r = 0.05.m2_per_kg();
        assert_eq!(format!("{:.2}", r), "0.05 m²/kg");
    }

    #[test]
    fn test_area_to_mass_neg() {
        assert_eq!(AreaToMass(-1.0), -1.0.m2_per_kg())
    }

    #[test]
    fn test_angle_arcseconds_roundtrip() {
        let a = Angle::arcseconds(3600.0); // = 1 degree
        assert_approx_eq!(a.to_degrees(), 1.0, rtol <= 1e-10);
        assert_approx_eq!(a.to_arcseconds(), 3600.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_arcseconds_rem() {
        let a = Angle::arcseconds_rem(ARCSECONDS_IN_CIRCLE + 3600.0);
        assert_approx_eq!(a.to_degrees(), 1.0, rtol <= 1e-12);
        let a = Angle::arcseconds_rem(-ARCSECONDS_IN_CIRCLE - 3600.0);
        assert_approx_eq!(a.to_degrees(), -1.0, rtol <= 1e-12);
    }

    #[test]
    fn test_angle_arcseconds_rem_keeps_precision_for_large_values() {
        // Mean anomaly of the Moon (IERS 2003) at t = 0.8 Julian centuries; the expected value is
        // the ERFA t_erfa_c.c::t_fal03 test vector.
        let t = 0.8;
        let asec = 485_868.249_036
            + t * (1_717_915_923.217_8 + t * (31.8792 + t * (0.051_635 + t * -0.000_244_70)));
        assert_approx_eq!(
            Angle::arcseconds_rem(asec).to_radians(),
            5.132_369_751_108_684,
            atol <= 1e-12
        );
    }

    #[test]
    fn test_angle_seconds_of_time() {
        assert_eq!(Angle::seconds_of_time(43200.0), Angle::PI);
        assert_approx_eq!(
            Angle::degrees(15.0).to_seconds_of_time(),
            3600.0,
            rtol <= 1e-15
        );
    }

    #[test]
    fn test_angle_is_zero() {
        assert!(Angle::ZERO.is_zero());
        assert!(!Angle::PI.is_zero());
    }

    #[test]
    fn test_angle_abs() {
        let a = Angle::radians(-1.5);
        assert_approx_eq!(a.abs().to_radians(), 1.5, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_from_asin() {
        let a = Angle::from_asin(1.0);
        assert_approx_eq!(a.to_degrees(), 90.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_from_acos() {
        let a = Angle::from_acos(1.0);
        assert_approx_eq!(a.to_degrees(), 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_angle_from_atan() {
        let a = Angle::from_atan(1.0);
        assert_approx_eq!(a.to_degrees(), 45.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_from_atan2() {
        let a = Angle::from_atan2(1.0, 1.0); // 45 deg
        assert_approx_eq!(a.to_degrees(), 45.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_from_asinh() {
        let a = Angle::from_asinh(0.0);
        assert_approx_eq!(a.to_radians(), 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_angle_from_acosh() {
        let a = Angle::from_acosh(1.0);
        assert_approx_eq!(a.to_radians(), 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_angle_from_atanh() {
        let a = Angle::from_atanh(0.0);
        assert_approx_eq!(a.to_radians(), 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_angle_trig_functions() {
        let a = Angle::FRAC_PI_2;
        assert_approx_eq!(a.sin(), 1.0, rtol <= 1e-10);
        assert_approx_eq!(a.cos(), 0.0, atol <= 1e-10);
        let (s, c) = a.sin_cos();
        assert_approx_eq!(s, 1.0, rtol <= 1e-10);
        assert_approx_eq!(c, 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_angle_tan() {
        let a = Angle::degrees(45.0);
        assert_approx_eq!(a.tan(), 1.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_hyperbolic_trig() {
        let a = Angle::radians(1.0);
        assert_approx_eq!(a.sinh(), sinh(1.0_f64), rtol <= 1e-10);
        assert_approx_eq!(a.cosh(), cosh(1.0_f64), rtol <= 1e-10);
        assert_approx_eq!(a.tanh(), tanh(1.0_f64), rtol <= 1e-10);
    }

    #[test]
    fn test_angle_wrap_unsigned() {
        let a = Angle::radians(TAU + 1.0).wrap_unsigned();
        assert_approx_eq!(a.to_radians(), 1.0, rtol <= 1e-10);
        let a = Angle::radians(-1.0).wrap_unsigned();
        assert_approx_eq!(a.to_radians(), TAU - 1.0, rtol <= 1e-15);
    }

    #[test]
    fn test_angle_wrap_unsigned_tiny_negative_stays_below_two_pi() {
        // -1e-17 + 2π rounds to 2π itself, which is outside [0, 2π).
        assert_eq!(Angle::radians(-1e-17).wrap_unsigned(), Angle::ZERO);
    }

    #[test]
    fn test_angle_wrap_unsigned_negative_zero_is_positive() {
        let a = Angle::radians(-0.0).wrap_unsigned().to_radians();
        assert!(a == 0.0 && a.is_sign_positive());
        let a = Angle::radians(-TAU).wrap_unsigned().to_radians();
        assert!(a == 0.0 && a.is_sign_positive());
    }

    #[test]
    fn test_angle_wrap_signed() {
        assert_eq!(Angle::PI.wrap_signed(), -Angle::PI);
        assert_approx_eq!(
            Angle::radians(TAU - 1.0).wrap_signed().to_radians(),
            -1.0,
            rtol <= 1e-12
        );
    }

    #[test]
    fn test_angle_wrap_signed_rounding_stays_in_range() {
        // Rounding used to give a value below -π here.
        let a = Angle::radians(-12569.512207012762).wrap_signed();
        assert!(a.to_radians() >= -PI && a.to_radians() < PI);
    }

    #[test]
    fn test_angle_wrap_stays_in_range_for_large_values() {
        assert!(
            Angle::radians(3.611500533622795e19)
                .wrap_signed()
                .to_radians()
                < PI
        );
        // Deterministic pseudo-random magnitudes between 1e3 and 1e20, both signs.
        let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
        for _ in 0..10_000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let mantissa = 1.0 + (state >> 11) as f64 / (1u64 << 53) as f64;
            let exponent = 3 + (state % 18) as i32;
            let sign = if state & 1 == 0 { 1.0 } else { -1.0 };
            let mut x = sign * mantissa;
            for _ in 0..exponent {
                x *= 10.0;
            }
            let a = Angle::radians(x);
            let signed = a.wrap_signed().to_radians();
            assert!((-PI..PI).contains(&signed), "{x}: {signed}");
            let unsigned = a.wrap_unsigned().to_radians();
            assert!((0.0..TAU).contains(&unsigned), "{x}: {unsigned}");
            let around = a.wrap_around(Angle::radians(1.0)).to_radians();
            assert!((1.0 - PI..1.0 + PI).contains(&around), "{x}: {around}");
        }
    }

    #[test]
    fn test_angle_wrap_around_boundaries_with_arbitrary_centers() {
        let center = Angle::radians(-44.88935050308772);
        let a = Angle::radians(-48.030943156677516).wrap_around(center);
        assert!(a >= center - Angle::PI && a < center + Angle::PI);
        // Deterministic pseudo-random centers, with inputs within 1e-13 of the boundaries.
        let mut state: u64 = 0x2545_f491_4f6c_dd1d;
        let mut uniform = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (state >> 11) as f64 / (1u64 << 53) as f64
        };
        for _ in 0..100_000 {
            let c = (uniform() - 0.5) * 200.0;
            let turns = ((uniform() - 0.5) * 20.0).round();
            let x = c + PI + turns * TAU + (uniform() - 0.5) * 1e-13;
            let a = Angle::radians(x)
                .wrap_around(Angle::radians(c))
                .to_radians();
            assert!(a >= c - PI && a < c + PI, "{x} around {c}: {a}");
        }
    }

    #[test]
    fn test_angle_wrap_around_large_value() {
        // Larger than i64::MAX radians; must not saturate.
        let a = Angle::radians(1e20).wrap_around(Angle::ZERO);
        assert!(a.to_radians() >= -PI && a.to_radians() < PI);
    }

    #[test]
    fn test_angle_wrap_non_finite_is_nan() {
        assert!(Angle::radians(f64::NAN).wrap_signed().is_nan());
        assert!(Angle::radians(f64::INFINITY).wrap_signed().is_nan());
        assert!(Angle::radians(f64::INFINITY).wrap_unsigned().is_nan());
    }

    #[test]
    fn test_angle_rem_two_pi() {
        let a = Angle::radians(TAU + 1.0).rem_two_pi();
        assert_approx_eq!(a.to_radians(), 1.0, rtol <= 1e-10);
        let a = Angle::radians(-TAU - 1.0).rem_two_pi();
        assert_approx_eq!(a.to_radians(), -1.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_rotation_matrices() {
        let a = Angle::ZERO;
        // Rotation by zero should give identity
        let rx = a.rotation_x();
        let ry = a.rotation_y();
        let rz = a.rotation_z();
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert_approx_eq!(rx.col(i)[j], expected, atol <= 1e-10);
                assert_approx_eq!(ry.col(i)[j], expected, atol <= 1e-10);
                assert_approx_eq!(rz.col(i)[j], expected, atol <= 1e-10);
            }
        }
    }

    #[test]
    fn test_angle_arithmetic() {
        let a = Angle::degrees(30.0);
        let b = Angle::degrees(60.0);
        assert_approx_eq!((a + b).to_degrees(), 90.0, rtol <= 1e-10);
        assert_approx_eq!((b - a).to_degrees(), 30.0, rtol <= 1e-10);
        let mut c = a;
        c += b;
        assert_approx_eq!(c.to_degrees(), 90.0, rtol <= 1e-10);
        let mut d = b;
        d -= a;
        assert_approx_eq!(d.to_degrees(), 30.0, rtol <= 1e-10);
        let scaled = 2.0 * a;
        assert_approx_eq!(scaled.to_degrees(), 60.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_i64_units() {
        let a = 90_i64.deg();
        assert_approx_eq!(a.to_degrees(), 90.0, rtol <= 1e-10);
        let b = 1_i64.rad();
        assert_approx_eq!(b.to_radians(), 1.0, rtol <= 1e-10);
        let c = 3600_i64.arcsec();
        assert_approx_eq!(c.to_degrees(), 1.0, rtol <= 1e-10);
        let d = 1000_i64.mas();
        assert_approx_eq!(d.to_degrees(), 1.0 / 3600.0, rtol <= 1e-8);
        let e = 1_000_000_i64.uas();
        assert_approx_eq!(e.to_degrees(), 1.0 / 3600.0, rtol <= 1e-8);
    }

    #[test]
    fn test_angle_f64_mas_uas() {
        let a = 1000.0_f64.mas();
        assert_approx_eq!(a.to_degrees(), 1.0 / 3600.0, rtol <= 1e-8);
        let b = 1_000_000.0_f64.uas();
        assert_approx_eq!(b.to_degrees(), 1.0 / 3600.0, rtol <= 1e-8);
    }

    #[test]
    fn test_distance_to_astronomical_units() {
        let d = Distance::astronomical_units(1.0);
        assert_approx_eq!(d.to_astronomical_units(), 1.0, rtol <= 1e-10);
    }

    #[test]
    fn test_distance_meters() {
        let d = Distance::meters(1234.0);
        assert_eq!(d.to_meters(), 1234.0);
    }

    #[test]
    fn test_distance_to_meters() {
        let d = Distance::kilometers(1.0);
        assert_eq!(d.to_meters(), 1000.0);
    }

    #[test]
    fn test_distance_i64_units() {
        let d = 7_i64.km();
        assert_eq!(d.to_meters(), 7000.0);
        let e = 1000_i64.m();
        assert_eq!(e.to_meters(), 1000.0);
        let f = 1_i64.au();
        assert_approx_eq!(f.to_meters(), ASTRONOMICAL_UNIT, rtol <= 1e-10);
    }

    #[test]
    fn test_distance_arithmetic() {
        let a = Distance::meters(100.0);
        let b = Distance::meters(50.0);
        assert_eq!((a + b).to_meters(), 150.0);
        assert_eq!((a - b).to_meters(), 50.0);
        let mut c = a;
        c += b;
        assert_eq!(c.to_meters(), 150.0);
        let mut d = a;
        d -= b;
        assert_eq!(d.to_meters(), 50.0);
        let scaled = 2.0 * a;
        assert_eq!(scaled.to_meters(), 200.0);
    }

    #[test]
    fn test_velocity_astronomical_units_per_day() {
        let v = Velocity::astronomical_units_per_day(1.0);
        let expected = ASTRONOMICAL_UNIT / SECONDS_PER_DAY;
        assert_approx_eq!(v.to_meters_per_second(), expected, rtol <= 1e-10);
        assert_approx_eq!(v.to_astronomical_units_per_day(), 1.0, rtol <= 1e-10);
    }

    #[test]
    fn test_velocity_fraction_of_speed_of_light() {
        let v = Velocity::fraction_of_speed_of_light(1.0);
        assert_approx_eq!(v.to_meters_per_second(), SPEED_OF_LIGHT, rtol <= 1e-10);
        assert_approx_eq!(v.to_fraction_of_speed_of_light(), 1.0, rtol <= 1e-10);
    }

    #[test]
    fn test_velocity_meters_per_second() {
        let v = Velocity::meters_per_second(300.0);
        assert_eq!(v.to_meters_per_second(), 300.0);
    }

    #[test]
    fn test_velocity_to_km_per_second() {
        let v = Velocity::meters_per_second(3000.0);
        assert_approx_eq!(v.to_kilometers_per_second(), 3.0, rtol <= 1e-10);
    }

    #[test]
    fn test_velocity_i64_units() {
        let a = 7_i64.kps();
        assert_eq!(a.to_meters_per_second(), 7000.0);
        let b = 1_i64.mps();
        assert_eq!(b.to_meters_per_second(), 1.0);
        let c = 1_i64.aud();
        assert_approx_eq!(
            c.to_meters_per_second(),
            ASTRONOMICAL_UNIT / SECONDS_PER_DAY,
            rtol <= 1e-10
        );
        let d = 1_i64.c();
        assert_approx_eq!(d.to_meters_per_second(), SPEED_OF_LIGHT, rtol <= 1e-10);
    }

    #[test]
    fn test_velocity_arithmetic() {
        let a = Velocity::meters_per_second(500.0);
        let b = Velocity::meters_per_second(250.0);
        assert_eq!((a + b).to_meters_per_second(), 750.0);
        assert_eq!((a - b).to_meters_per_second(), 250.0);
        let scaled = 3.0 * b;
        assert_eq!(scaled.to_meters_per_second(), 750.0);
    }

    #[test]
    fn test_mass_to_grams() {
        let m = Mass::kilograms(2.0);
        assert_approx_eq!(m.to_grams(), 2000.0, rtol <= 1e-12);
    }

    #[test]
    fn test_mass_to_metric_tons() {
        let m = Mass::kilograms(1000.0);
        assert_approx_eq!(m.to_metric_tons(), 1.0, rtol <= 1e-12);
    }

    #[test]
    fn test_mass_i64_metric_tons() {
        let m = 1_i64.t();
        assert_approx_eq!(m.to_kilograms(), 1000.0, rtol <= 1e-12);
    }

    #[test]
    fn test_mass_arithmetic() {
        let a = Mass::kilograms(100.0);
        let b = Mass::kilograms(50.0);
        assert_eq!((a + b).to_kilograms(), 150.0);
        assert_eq!((a - b).to_kilograms(), 50.0);
        let mut c = a;
        c += b;
        assert_eq!(c.to_kilograms(), 150.0);
        let scaled = 2.0 * b;
        assert_eq!(scaled.to_kilograms(), 100.0);
    }

    #[test]
    fn test_area_to_square_kilometers() {
        let a = Area::square_meters(1_000_000.0);
        assert_approx_eq!(a.to_square_kilometers(), 1.0, rtol <= 1e-12);
    }

    #[test]
    fn test_area_i64_units() {
        let a = 4_i64.m2();
        assert_eq!(a.to_square_meters(), 4.0);
        let b = 2_i64.km2();
        assert_approx_eq!(b.to_square_meters(), 2e6, rtol <= 1e-12);
    }

    #[test]
    fn test_area_arithmetic() {
        let a = Area::square_meters(4.0);
        let b = Area::square_meters(2.0);
        assert_eq!((a + b).to_square_meters(), 6.0);
        assert_eq!((a - b).to_square_meters(), 2.0);
        let scaled = 3.0 * b;
        assert_eq!(scaled.to_square_meters(), 6.0);
    }

    #[test]
    fn test_area_to_mass_i64_units() {
        let r = 1_i64.m2_per_kg();
        assert_eq!(r.to_square_meters_per_kilogram(), 1.0);
    }

    #[test]
    fn test_area_to_mass_arithmetic() {
        let a = AreaToMass::square_meters_per_kilogram(0.1);
        let b = AreaToMass::square_meters_per_kilogram(0.05);
        assert_approx_eq!((a + b).to_square_meters_per_kilogram(), 0.15, rtol <= 1e-10);
        assert_approx_eq!((a - b).to_square_meters_per_kilogram(), 0.05, rtol <= 1e-10);
        let scaled = 2.0 * b;
        assert_approx_eq!(scaled.to_square_meters_per_kilogram(), 0.1, rtol <= 1e-10);
    }

    #[test]
    fn test_frequency_hertz() {
        let f = Frequency::hertz(1e9);
        assert_eq!(f.to_hertz(), 1e9);
    }

    #[test]
    fn test_frequency_to_kilohertz() {
        let f = Frequency::megahertz(1.0);
        assert_approx_eq!(f.to_kilohertz(), 1000.0, rtol <= 1e-10);
    }

    #[test]
    fn test_frequency_to_terahertz() {
        let f = Frequency::terahertz(1.0);
        assert_approx_eq!(f.to_terahertz(), 1.0, rtol <= 1e-10);
    }

    #[test]
    fn test_frequency_arithmetic() {
        let a = Frequency::gigahertz(1.0);
        let b = Frequency::gigahertz(0.5);
        assert_approx_eq!((a + b).to_gigahertz(), 1.5, rtol <= 1e-10);
        assert_approx_eq!((a - b).to_gigahertz(), 0.5, rtol <= 1e-10);
        let scaled = 2.0 * b;
        assert_approx_eq!(scaled.to_gigahertz(), 1.0, rtol <= 1e-10);
    }

    #[test]
    fn test_temperature_neg() {
        let t = Temperature::kelvin(100.0);
        assert_eq!((-t).to_kelvin(), -100.0);
    }

    #[test]
    fn test_temperature_add_assign_sub_assign() {
        let mut a = Temperature::kelvin(200.0);
        a += Temperature::kelvin(50.0);
        assert_eq!(a.to_kelvin(), 250.0);
        a -= Temperature::kelvin(100.0);
        assert_eq!(a.to_kelvin(), 150.0);
    }

    #[test]
    fn test_pressure_pascals() {
        let p = Pressure::pascals(101325.0);
        assert_eq!(p.to_pascals(), 101325.0);
    }

    #[test]
    fn test_pressure_arithmetic() {
        let a = Pressure::pascals(1000.0);
        let b = Pressure::pascals(500.0);
        assert_eq!((a + b).to_pascals(), 1500.0);
        assert_eq!((a - b).to_pascals(), 500.0);
        assert_eq!((-b).to_pascals(), -500.0);
        let scaled = 2.0 * b;
        assert_eq!(scaled.to_pascals(), 1000.0);
    }

    #[test]
    fn test_power_add_assign_sub_assign() {
        let mut p = Power::watts(200.0);
        p += Power::watts(100.0);
        assert_eq!(p.to_watts(), 300.0);
        p -= Power::watts(50.0);
        assert_eq!(p.to_watts(), 250.0);
    }

    #[test]
    fn test_angular_rate_radians_per_second() {
        let ar = AngularRate::radians_per_second(2.0);
        assert_eq!(ar.to_radians_per_second(), 2.0);
    }

    #[test]
    fn test_angular_rate_add_assign_sub_assign() {
        let mut ar = AngularRate::radians_per_second(1.0);
        ar += AngularRate::radians_per_second(0.5);
        assert_approx_eq!(ar.to_radians_per_second(), 1.5, rtol <= 1e-10);
        ar -= AngularRate::radians_per_second(0.5);
        assert_approx_eq!(ar.to_radians_per_second(), 1.0, rtol <= 1e-10);
    }

    #[test]
    fn test_decibel_add_assign_sub_assign() {
        let mut d = Decibel::decibels(10.0);
        d += Decibel::decibels(3.0);
        assert_approx_eq!(d.to_decibels(), 13.0, rtol <= 1e-10);
        d -= Decibel::decibels(3.0);
        assert_approx_eq!(d.to_decibels(), 10.0, rtol <= 1e-10);
    }

    #[test]
    fn test_decibel_i64_units() {
        let d = 10_i64.db();
        assert_eq!(d.to_decibels(), 10.0);
    }

    #[test]
    fn test_decibel_scaling() {
        let d = Decibel::decibels(5.0);
        let scaled = 2.0 * d;
        assert_approx_eq!(scaled.to_decibels(), 10.0, rtol <= 1e-10);
    }

    #[rstest]
    #[case(1.0_f64, Sign::Positive)]
    #[case(-1.0_f64, Sign::Negative)]
    #[case(0.0_f64, Sign::Positive)]
    #[case(-0.0_f64, Sign::Negative)] // IEEE sign bit
    #[case(f64::INFINITY, Sign::Positive)]
    #[case(f64::NEG_INFINITY, Sign::Negative)]
    fn test_sign_from_f64(#[case] input: f64, #[case] expected: Sign) {
        assert_eq!(Sign::from(input), expected);
    }

    #[rstest]
    #[case(1_i32, Sign::Positive)]
    #[case(-1_i32, Sign::Negative)]
    #[case(0_i32, Sign::Positive)]
    fn test_sign_from_i32(#[case] input: i32, #[case] expected: Sign) {
        assert_eq!(Sign::from(input), expected);
    }

    #[test]
    fn test_sign_from_signed_integer_widths() {
        assert_eq!(Sign::from(-1_i8), Sign::Negative);
        assert_eq!(Sign::from(-1_i16), Sign::Negative);
        assert_eq!(Sign::from(-1_i64), Sign::Negative);
        assert_eq!(Sign::from(-1_isize), Sign::Negative);
    }

    #[test]
    fn test_sign_to_f64() {
        assert_eq!(f64::from(Sign::Positive), 1.0);
        assert_eq!(f64::from(Sign::Negative), -1.0);
        assert_eq!(Sign::Positive.as_f64(), 1.0);
        assert_eq!(Sign::Negative.as_f64(), -1.0);
    }

    #[test]
    fn test_sign_display() {
        assert_eq!(format!("{}", Sign::Positive), "+");
        assert_eq!(format!("{}", Sign::Negative), "-");
    }

    #[test]
    fn test_quantity_scaling() {
        let d = Distance::meters(10.0);
        assert_eq!((d * 3.0).to_meters(), 30.0);
        assert_eq!((3.0 * d).to_meters(), 30.0);
        assert_eq!((d / 4.0).to_meters(), 2.5);
        let mut e = d;
        e *= 2.0;
        assert_eq!(e.to_meters(), 20.0);
        e /= 8.0;
        assert_eq!(e.to_meters(), 2.5);
    }

    #[test]
    fn test_quantity_ratio_is_dimensionless() {
        let ratio: f64 = Distance::kilometers(3.0) / Distance::meters(1500.0);
        assert_eq!(ratio, 2.0);
    }

    #[test]
    fn test_quantity_sum() {
        let losses = [Decibel::decibels(1.5), Decibel::decibels(2.0)];
        assert_eq!(losses.iter().sum::<Decibel>().to_decibels(), 3.5);
        assert_eq!(losses.into_iter().sum::<Decibel>().to_decibels(), 3.5);
        assert_eq!(
            core::iter::empty::<Mass>().sum::<Mass>().to_kilograms(),
            0.0
        );
    }

    #[test]
    fn test_quantity_comparisons() {
        let a = Velocity::meters_per_second(-2.0);
        let b = Velocity::meters_per_second(5.0);
        assert_eq!(a.abs().to_meters_per_second(), 2.0);
        assert_eq!(a.min(b), a);
        assert_eq!(a.max(b), b);
        assert_eq!(
            Velocity::meters_per_second(9.0)
                .clamp(a, b)
                .to_meters_per_second(),
            5.0
        );
        assert_eq!(a.total_cmp(&b), core::cmp::Ordering::Less);
        assert_eq!(
            Angle::radians(-0.0).total_cmp(&Angle::radians(0.0)),
            core::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_quantity_zero() {
        assert_eq!(Distance::ZERO.to_meters(), 0.0);
        assert_eq!(Decibel::ZERO.to_decibels(), 0.0);
        assert!(Velocity::meters_per_second(-1.0) < Velocity::ZERO);
    }

    #[test]
    fn test_quantity_non_finite() {
        assert!(Distance::meters(1.0).is_finite());
        assert!(!Distance::meters(f64::INFINITY).is_finite());
        assert!(Distance::meters(f64::NAN).is_nan());
        assert!(!Distance::meters(f64::NAN).is_finite());
    }

    #[test]
    fn test_scale_down_is_correctly_rounded() {
        // Multiplying by the inexact reciprocals 1e-3 and 1e-9 does not round-trip these values.
        assert_eq!(Distance::kilometers(6000.007).to_kilometers(), 6000.007);
        assert_eq!(
            Frequency::gigahertz(0.0009000000000000001).to_gigahertz(),
            0.0009000000000000001
        );
    }

    #[test]
    fn test_angle_small_units() {
        assert_approx_eq!(
            Angle::milliarcseconds(1500.0).to_arcseconds(),
            1.5,
            rtol <= 1e-15
        );
        assert_approx_eq!(
            Angle::microarcseconds(2.0).to_milliarcseconds(),
            2e-3,
            rtol <= 1e-15
        );
        assert_approx_eq!(
            Angle::arcseconds(3.0).to_microarcseconds(),
            3e6,
            rtol <= 1e-15
        );
        assert_eq!(Angle::microradians(5.0).to_radians(), 5e-6);
        assert_approx_eq!(Angle::radians(5e-6).to_microradians(), 5.0, rtol <= 1e-15);
        assert_eq!(1.5.mas(), Angle::milliarcseconds(1.5));
        assert_eq!(2.uas(), Angle::microarcseconds(2.0));
    }

    #[test]
    fn test_pressure_units() {
        let p = Pressure::hectopascals(1013.25);
        assert_eq!(p.to_pascals(), 101325.0);
        assert_eq!(p.to_hectopascals(), 1013.25);
        assert_eq!(Pressure::pascals(100.0).to_hectopascals(), 1.0);
    }

    #[test]
    fn test_data_rate_units() {
        let r = DataRate::megabits_per_second(150.0);
        assert_eq!(r.to_bits_per_second(), 150e6);
        assert_eq!(r.to_kilobits_per_second(), 150e3);
        assert_eq!(r.to_gigabits_per_second(), 0.15);
        assert_eq!(1.5.gbps(), DataRate::gigabits_per_second(1.5));
        assert_eq!(64.kbps(), DataRate::bits_per_second(64e3));
        assert_eq!(9600.bps().to_bits_per_second(), 9600.0);
    }

    #[test]
    fn test_data_volume_units() {
        assert_eq!(DataVolume::bytes(1.0).to_bits(), 8.0);
        assert_eq!(DataVolume::megabytes(1.0).to_bits(), 8e6);
        assert_eq!(DataVolume::mebibytes(1.0).to_bits(), 8.0 * 1_048_576.0);
        assert_eq!(DataVolume::tebibytes(1.0).to_bytes(), 1_099_511_627_776.0);
        assert_eq!(DataVolume::gigabytes(2.5).to_megabytes(), 2500.0);
        assert_eq!(DataVolume::terabits(1.0).to_gigabits(), 1000.0);
        assert_eq!(DataVolume::kibibytes(1.0).to_kilobytes(), 1.024);
        assert_eq!(16.bits(), 2.bytes());
    }

    #[test]
    fn test_decibel_watts_units() {
        let p = DecibelWatts::decibel_milliwatts(30.0);
        assert_eq!(p.to_decibel_watts(), 0.0);
        assert_eq!(p.to_decibel_milliwatts(), 30.0);
        assert_eq!(10.dbw(), DecibelWatts::decibel_watts(10.0));
        assert_eq!(0.0.dbm(), DecibelWatts::decibel_watts(-30.0));
    }

    #[test]
    fn test_decibel_watts_algebra() {
        let eirp = DecibelWatts::decibel_watts(55.0);
        let loss = Decibel::decibels(181.7);
        let received = eirp - loss + Decibel::decibels(30.0);
        assert_approx_eq!(received.to_decibel_watts(), -96.7, atol <= 1e-12);
        assert_approx_eq!(
            (Decibel::decibels(3.0) + eirp).to_decibel_watts(),
            58.0,
            atol <= 1e-12
        );
        let margin: Decibel = received - DecibelWatts::decibel_watts(-100.0);
        assert_approx_eq!(margin.to_decibels(), 3.3, atol <= 1e-12);
        let mut level = eirp;
        level -= Decibel::decibels(5.0);
        level += Decibel::decibels(1.0);
        assert_eq!(level, DecibelWatts::decibel_watts(51.0));
    }

    #[test]
    fn test_decibel_watts_power_conversion() {
        let p = Power::watts(100.0);
        assert_approx_eq!(
            DecibelWatts::from(p).to_decibel_watts(),
            20.0,
            atol <= 1e-12
        );
        assert_approx_eq!(
            DecibelWatts::decibel_watts(20.0).to_power().to_watts(),
            100.0,
            rtol <= 1e-12
        );
    }

    #[test]
    fn test_decibel_units() {
        assert_eq!(Decibel::decibels(3.0).to_decibels(), 3.0);
        assert_eq!(3.db(), Decibel::decibels(3.0));
    }
}
