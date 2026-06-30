// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2013-2021 NumFOCUS Foundation
//
// SPDX-License-Identifier: MPL-2.0 AND LicenseRef-ERFA

//! Newtype wrappers for unitful [`f64`] double precision values

use core::{
    f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU},
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

use glam::DMat3;
use lox_approx::ApproxEq;

use crate::comms::FrequencyBand;
use crate::f64::consts::SECONDS_PER_DAY;
use crate::math::float::{
    abs, acos, acosh, asin, asinh, atan, atan2, atanh, cos, cosh, log10, powf, round, sin, sin_cos,
    sinh, tan, tanh, to_degrees, to_radians,
};

/// Degrees in full circle
pub const DEGREES_IN_CIRCLE: f64 = 360.0;

/// Arcseconds in full circle
pub const ARCSECONDS_IN_CIRCLE: f64 = DEGREES_IN_CIRCLE * 60.0 * 60.0;

/// Radians per arcsecond
pub const RADIANS_IN_ARCSECOND: f64 = TAU / ARCSECONDS_IN_CIRCLE;

/// Rounding granularity (arcseconds) used by the HMS/DMS decomposition
/// methods to suppress floating-point undershoot at integer boundaries.
/// See [`Angle::to_dms`] / [`Angle::to_hms`].
const HMS_DMS_ROUNDING_ARCSEC: f64 = 1e-9;

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
    pub const fn as_f64(&self) -> f64 {
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

type Radians = f64;

/// Decomposes a signed arcsecond total into `(Sign, hours-or-degrees,
/// minutes, seconds)`. Used by [`Angle::to_dms`] and [`Angle::to_hms`].
///
/// `unit_arcsec` is the number of arcseconds per major unit
/// (3600.0 for degrees, 3600.0 × 15.0 = 54000.0 for hours).
fn decompose_signed_arcseconds(total_arcsec: f64, unit_arcsec: f64) -> (Sign, u32, u8, f64) {
    let sign = Sign::from(total_arcsec);
    let abs_arcsec = abs(total_arcsec);
    // Round to the nearest HMS_DMS_ROUNDING_ARCSEC to suppress floating-point
    // undershoot at integer boundaries (e.g. 1799.999…98 → 1800.0).
    let abs_arcsec = round(abs_arcsec / HMS_DMS_ROUNDING_ARCSEC) * HMS_DMS_ROUNDING_ARCSEC;
    let major = (abs_arcsec / unit_arcsec) as u32;
    let rem = abs_arcsec - (major as f64) * unit_arcsec;
    // 60 arcseconds in an arcminute, 60 seconds in a minute.
    let minutes = (rem / 60.0) as u8;
    let seconds = rem - (minutes as f64) * 60.0;
    (sign, major, minutes, seconds)
}

/// Angle in radians.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        Self(to_radians(deg))
    }

    /// Creates a new angle from sign, hours, minutes, and seconds (HMS notation).
    ///
    /// All magnitude components are non-negative; the sign is carried by the
    /// `Sign` argument. This shape lets the function represent angles in
    /// `(−1h, 0h)` (e.g. `−0h 30m 0s`) that an `i64 hours` argument cannot.
    ///
    /// Inverse of [`Angle::to_hms`]; round-trip is perfect.
    ///
    /// # References
    ///
    /// - ERFA [`tf2a`](https://github.com/liberfa/erfa/blob/master/src/tf2a.c)
    pub const fn from_hms(sign: Sign, hours: u32, minutes: u8, seconds: f64) -> Self {
        let mag = 15.0 * (hours as f64 + minutes as f64 / 60.0 + seconds / 3600.0);
        Self::degrees(sign.as_f64() * mag)
    }

    /// Creates a new angle from sign, degrees, arcminutes, and arcseconds.
    ///
    /// All magnitude components are non-negative; the sign is carried by the
    /// `Sign` argument. Inverse of [`Angle::to_dms`].
    ///
    /// # References
    ///
    /// - ERFA [`af2a`](https://github.com/liberfa/erfa/blob/master/src/af2a.c)
    pub const fn from_dms(sign: Sign, degrees: u32, minutes: u8, seconds: f64) -> Self {
        let mag = degrees as f64 + minutes as f64 / 60.0 + seconds / 3600.0;
        Self::degrees(sign.as_f64() * mag)
    }

    /// Creates a new angle from an `f64` value in degrees and normalize the angle
    /// to the interval [0.0, 2π).
    pub const fn degrees_normalized(deg: f64) -> Self {
        Self(to_radians(deg % DEGREES_IN_CIRCLE)).mod_two_pi()
    }

    /// Creates a new angle from an `f64` value in degrees and normalize the angle
    /// to the interval (-2π, 2π).
    pub const fn degrees_normalized_signed(deg: f64) -> Self {
        Self(to_radians(deg % DEGREES_IN_CIRCLE))
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

    /// Returns `true` if the angle is exactly zero.
    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }

    /// Returns the absolute value of the angle.
    pub fn abs(&self) -> Self {
        Self(abs(self.0))
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
    pub fn cos(&self) -> f64 {
        cos(self.0)
    }

    /// Returns the hyperbolic cosine of the angle.
    pub fn cosh(&self) -> f64 {
        cosh(self.0)
    }

    /// Returns the sine of the angle.
    pub fn sin(&self) -> f64 {
        sin(self.0)
    }

    /// Returns the hyperbolic sine of the angle.
    pub fn sinh(&self) -> f64 {
        sinh(self.0)
    }

    /// Returns sine and cosine of the angle.
    pub fn sin_cos(&self) -> (f64, f64) {
        sin_cos(self.0)
    }

    /// Returns the tangent of the angle.
    pub fn tan(&self) -> f64 {
        tan(self.0)
    }

    /// Returns the hyperbolic tangent of the angle.
    pub fn tanh(&self) -> f64 {
        tanh(self.0)
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
        // Inline const-compatible floor (f64::floor isn't const in no_std).
        // The quotient is bounded by the angle range so the `as i64` cast is safe.
        let q = (self.0 + PI - center.0) / TAU;
        let i = q as i64 as f64;
        let floor_q = if q < i { i - 1.0 } else { i };
        Self(self.0 - TAU * floor_q)
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
        to_degrees(self.0)
    }

    /// Returns the value of the angle in arcseconds.
    pub const fn to_arcseconds(&self) -> f64 {
        self.0 / RADIANS_IN_ARCSECOND
    }

    /// Returns the 3×3 rotation matrix for a rotation about the X axis.
    pub fn rotation_x(&self) -> DMat3 {
        DMat3::from_rotation_x(-self.to_radians())
    }

    /// Returns the 3×3 rotation matrix for a rotation about the Y axis.
    pub fn rotation_y(&self) -> DMat3 {
        DMat3::from_rotation_y(-self.to_radians())
    }

    /// Returns the 3×3 rotation matrix for a rotation about the Z axis.
    pub fn rotation_z(&self) -> DMat3 {
        DMat3::from_rotation_z(-self.to_radians())
    }

    /// Decomposes the angle into (sign, hours, minutes, seconds).
    ///
    /// Treats the angle as a clock-time hour angle (15° per hour). The
    /// returned `hours` and `minutes` are non-negative; `seconds` is in
    /// `[0.0, 60.0)`.
    ///
    /// Inverse of [`Angle::from_hms`]; round-trip is perfect.
    ///
    /// # References
    ///
    /// - ERFA [`a2tf`](https://github.com/liberfa/erfa/blob/master/src/a2tf.c)
    pub fn to_hms(&self) -> (Sign, u32, u8, f64) {
        // 1 hour-of-angle = 15° = 15 × 3600 arcseconds.
        // Divide arcseconds by 15 so `decompose_signed_arcseconds` treats
        // 3600 units-per-major as 3600 time-seconds-per-hour.
        decompose_signed_arcseconds(self.to_arcseconds() / 15.0, 3600.0)
    }

    /// Decomposes the angle into (sign, degrees, arcminutes, arcseconds).
    ///
    /// The returned `degrees` and `arcminutes` are non-negative; `arcseconds`
    /// is in `[0.0, 60.0)`. Arcseconds are rounded to the nearest 1e-9
    /// arcsecond to suppress floating-point undershoot at integer boundaries.
    ///
    /// Inverse of [`Angle::from_dms`]; round-trip is perfect.
    ///
    /// # References
    ///
    /// - ERFA [`a2af`](https://github.com/liberfa/erfa/blob/master/src/a2af.c)
    pub fn to_dms(&self) -> (Sign, u32, u8, f64) {
        decompose_signed_arcseconds(self.to_arcseconds(), 3600.0)
    }
}

impl Display for Angle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        to_degrees(self.0).fmt(f)?;
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub const fn astronomical_units(au: f64) -> Self {
        Self(au * ASTRONOMICAL_UNIT)
    }

    /// Returns the value of the distance in meters as an `f64`.
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Creates a new velocity from an `f64` value in au/d.
    pub const fn astronomical_units_per_day(aud: f64) -> Self {
        Self(aud * ASTRONOMICAL_UNIT / SECONDS_PER_DAY)
    }

    /// Creates a new velocity from an `f64` value in 1/c.
    pub const fn fraction_of_speed_of_light(c: f64) -> Self {
        Self(c * SPEED_OF_LIGHT)
    }

    /// Returns the value of the velocity in m/s as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the value of the velocity in m/s.
    pub const fn to_meters_per_second(&self) -> f64 {
        self.0
    }

    /// Returns the value of the velocity in km/s.
    pub const fn to_kilometers_per_second(&self) -> f64 {
        self.0 * 1e-3
    }

    /// Returns the value of the velocity in au/d.
    pub const fn to_astronomical_units_per_day(&self) -> f64 {
        self.0 * SECONDS_PER_DAY / ASTRONOMICAL_UNIT
    }

    /// Returns the value of the velocity in 1/c.
    pub const fn to_fraction_of_speed_of_light(&self) -> f64 {
        self.0 / SPEED_OF_LIGHT
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

type Hertz = f64;

/// Frequency in Hertz
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        FrequencyBand::ALL.into_iter().find(|band| {
            let (min, max) = band.bounds();
            min.to_hertz() <= self.0 && self.0 < max.to_hertz()
        })
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

type Kilograms = f64;

/// Mass in kilograms.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Mass(Kilograms);

impl Mass {
    /// Creates a new mass from an `f64` value in kilograms.
    pub const fn new(kg: f64) -> Self {
        Self(kg)
    }

    /// Creates a new mass from an `f64` value in kilograms.
    pub const fn kilograms(kg: f64) -> Self {
        Self(kg)
    }

    /// Creates a new mass from an `f64` value in grams.
    pub const fn grams(g: f64) -> Self {
        Self(g * 1e-3)
    }

    /// Creates a new mass from an `f64` value in metric tons (1000 kg).
    pub const fn metric_tons(t: f64) -> Self {
        Self(t * 1e3)
    }

    /// Returns the value of the mass in kilograms as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the value of the mass in kilograms.
    pub const fn to_kilograms(&self) -> f64 {
        self.0
    }

    /// Returns the value of the mass in grams.
    pub const fn to_grams(&self) -> f64 {
        self.0 * 1e3
    }

    /// Returns the value of the mass in metric tons.
    pub const fn to_metric_tons(&self) -> f64 {
        self.0 * 1e-3
    }
}

impl Display for Mass {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)?;
        write!(f, " kg")
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

type SquareMeters = f64;

/// Area in square meters.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Area(SquareMeters);

impl Area {
    /// Creates a new area from an `f64` value in square meters.
    pub const fn new(m2: f64) -> Self {
        Self(m2)
    }

    /// Creates a new area from an `f64` value in square meters.
    pub const fn square_meters(m2: f64) -> Self {
        Self(m2)
    }

    /// Creates a new area from an `f64` value in square kilometers.
    pub const fn square_kilometers(km2: f64) -> Self {
        Self(km2 * 1e6)
    }

    /// Returns the value of the area in square meters as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the value of the area in square meters.
    pub const fn to_square_meters(&self) -> f64 {
        self.0
    }

    /// Returns the value of the area in square kilometers.
    pub const fn to_square_kilometers(&self) -> f64 {
        self.0 * 1e-6
    }
}

impl Display for Area {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)?;
        write!(f, " m²")
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

type SquareMetersPerKilogram = f64;

/// Area-to-mass ratio in m²/kg.
///
/// Used for ballistic and radiation-pressure coefficients (e.g. the OMM
/// `BTERM` and `AGOM` fields in CCSDS 502.0-B-3).
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct AreaToMass(SquareMetersPerKilogram);

impl AreaToMass {
    /// Creates a new area-to-mass ratio from an `f64` value in m²/kg.
    pub const fn new(m2_per_kg: f64) -> Self {
        Self(m2_per_kg)
    }

    /// Creates a new area-to-mass ratio from an `f64` value in m²/kg.
    pub const fn square_meters_per_kilogram(m2_per_kg: f64) -> Self {
        Self(m2_per_kg)
    }

    /// Returns the value in m²/kg as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the value in m²/kg.
    pub const fn to_square_meters_per_kilogram(&self) -> f64 {
        self.0
    }
}

impl Display for AreaToMass {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)?;
        write!(f, " m²/kg")
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

/// Temperature in Kelvin (deprecated type alias, use [`Temperature`] instead).
pub type Kelvin = f64;

type KelvinValue = f64;

/// Temperature in Kelvin.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Temperature(KelvinValue);

impl Temperature {
    /// Creates a new temperature from an `f64` value in Kelvin.
    pub const fn new(k: f64) -> Self {
        Self(k)
    }

    /// Creates a new temperature from an `f64` value in Kelvin.
    pub const fn kelvin(k: f64) -> Self {
        Self(k)
    }

    /// Returns the value in Kelvin as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the value in Kelvin.
    pub const fn to_kelvin(&self) -> f64 {
        self.0
    }
}

impl Display for Temperature {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)?;
        write!(f, " K")
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

type Pascals = f64;

/// Pressure in pascals.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Pressure(Pascals);

impl Pressure {
    /// Creates a new pressure from an `f64` value in pascals.
    pub const fn new(pa: f64) -> Self {
        Self(pa)
    }

    /// Creates a new pressure from an `f64` value in pascals.
    pub const fn pa(pa: f64) -> Self {
        Self(pa)
    }

    /// Creates a new pressure from an `f64` value in hectopascals.
    pub const fn hpa(hpa: f64) -> Self {
        Self(hpa * 100.0)
    }

    /// Returns the value in pascals as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the value in pascals.
    pub const fn to_pa(&self) -> f64 {
        self.0
    }

    /// Returns the value in hectopascals.
    pub const fn to_hpa(&self) -> f64 {
        self.0 * 0.01
    }
}

impl Display for Pressure {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)?;
        write!(f, " Pa")
    }
}

type Watts = f64;

/// Power in Watts.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Power(Watts);

impl Power {
    /// Creates a new power from an `f64` value in Watts.
    pub const fn new(w: f64) -> Self {
        Self(w)
    }

    /// Creates a new power from an `f64` value in Watts.
    pub const fn watts(w: f64) -> Self {
        Self(w)
    }

    /// Creates a new power from an `f64` value in kilowatts.
    pub const fn kilowatts(kw: f64) -> Self {
        Self(kw * 1e3)
    }

    /// Returns the value in Watts as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the value in Watts.
    pub const fn to_watts(&self) -> f64 {
        self.0
    }

    /// Returns the value in kilowatts.
    pub const fn to_kilowatts(&self) -> f64 {
        self.0 * 1e-3
    }

    /// Returns the value in dBW.
    pub fn to_dbw(&self) -> f64 {
        10.0 * log10(self.0)
    }
}

impl Display for Power {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)?;
        write!(f, " W")
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

type RadiansPerSecond = f64;

/// Angular rate in radians per second.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct AngularRate(RadiansPerSecond);

impl AngularRate {
    /// Creates a new angular rate from an `f64` value in rad/s.
    pub const fn new(rps: f64) -> Self {
        Self(rps)
    }

    /// Creates a new angular rate from an `f64` value in rad/s.
    pub const fn radians_per_second(rps: f64) -> Self {
        Self(rps)
    }

    /// Creates a new angular rate from an `f64` value in deg/s.
    pub const fn degrees_per_second(dps: f64) -> Self {
        Self(to_radians(dps))
    }

    /// Returns the value in rad/s as an `f64`.
    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the value in rad/s.
    pub const fn to_radians_per_second(&self) -> f64 {
        self.0
    }

    /// Returns the value in deg/s.
    pub const fn to_degrees_per_second(&self) -> f64 {
        to_degrees(self.0)
    }
}

impl Display for AngularRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        to_degrees(self.0).fmt(f)?;
        write!(f, " deg/s")
    }
}

type DecibelValue = f64;

/// A value in decibels.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct Decibel(DecibelValue);

impl Decibel {
    /// Creates a new `Decibel` from a value already in dB.
    pub const fn new(db: f64) -> Self {
        Self(db)
    }

    /// Converts a linear power-ratio value to decibels.
    pub fn from_linear(val: f64) -> Self {
        Self(10.0 * log10(val))
    }

    /// Converts this decibel value to a linear power-ratio.
    pub fn to_linear(self) -> f64 {
        powf(10.0_f64, self.0 / 10.0)
    }

    /// Returns the raw `f64` value in dB.
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

impl Display for Decibel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)?;
        write!(f, " dB")
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
/// assert_eq!(d.as_f64(), 3.0);
/// ```
pub trait DecibelUnits {
    /// Creates a decibel value.
    fn db(&self) -> Decibel;
}

impl DecibelUnits for f64 {
    fn db(&self) -> Decibel {
        Decibel::new(*self)
    }
}

impl DecibelUnits for i64 {
    fn db(&self) -> Decibel {
        Decibel::new(*self as f64)
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

trait_impls!(
    Angle,
    AngularRate,
    Area,
    AreaToMass,
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
    fn test_temperature_units_trait() {
        assert_eq!(290.0.k().to_kelvin(), 290.0);
        assert_eq!(290.k().to_kelvin(), 290.0);
        assert_eq!(290.0.k().as_f64(), 290.0);
    }

    #[test]
    fn test_power_units_trait() {
        assert_eq!(2.0.w().to_watts(), 2.0);
        assert_eq!(2.w().to_watts(), 2.0);
        assert_eq!(1.5.kw().to_watts(), 1500.0);
        assert_eq!(2.kw().to_kilowatts(), 2.0);
        assert_eq!(10.0.w().to_dbw(), 10.0);
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

    #[test]
    fn test_decibel_db() {
        let d = 3.0.db();
        assert_eq!(d.as_f64(), 3.0);
    }

    #[test]
    fn test_decibel_from_linear() {
        let d = Decibel::from_linear(100.0);
        assert_approx_eq!(d.0, 20.0, rtol <= 1e-10);
    }

    #[test]
    fn test_decibel_to_linear() {
        let d = Decibel::new(20.0);
        assert_approx_eq!(d.to_linear(), 100.0, rtol <= 1e-10);
    }

    #[test]
    fn test_decibel_roundtrip() {
        let val = 42.5;
        let d = Decibel::new(val);
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
        assert_eq!(-3.0.db(), Decibel::new(-3.0));
    }

    #[test]
    fn test_decibel_display() {
        let d = 3.0.db();
        assert_eq!(format!("{:.1}", d), "3.0 dB");
    }

    // --- Temperature ---

    #[test]
    fn test_temperature_new() {
        let t = Temperature::new(290.0);
        assert_eq!(t.as_f64(), 290.0);
    }

    #[test]
    fn test_temperature_kelvin() {
        let t = Temperature::kelvin(300.0);
        assert_eq!(t.to_kelvin(), 300.0);
    }

    #[test]
    fn test_temperature_display() {
        let t = Temperature::new(290.0);
        assert_eq!(format!("{}", t), "290 K");
    }

    #[test]
    fn test_temperature_arithmetic() {
        let a = Temperature::new(100.0);
        let b = Temperature::new(200.0);
        assert_eq!((a + b).as_f64(), 300.0);
        assert_eq!((b - a).as_f64(), 100.0);
        assert_eq!((-a).as_f64(), -100.0);
        assert_eq!((2.0 * a).as_f64(), 200.0);
    }

    // --- Power ---

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
        assert_approx_eq!(p.to_dbw(), 20.0, rtol <= 1e-10);
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
        assert_eq!((a + b).as_f64(), 200.0);
        assert_eq!((b - a).as_f64(), 100.0);
        assert_eq!((-a).as_f64(), -50.0);
    }

    // --- AngularRate ---

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
        let s = format!("{}", ar);
        assert!(s.contains("deg/s"));
    }

    #[test]
    fn test_angular_rate_arithmetic() {
        let a = AngularRate::new(1.0);
        let b = AngularRate::new(2.0);
        assert_eq!((a + b).as_f64(), 3.0);
        assert_eq!((b - a).as_f64(), 1.0);
        assert_eq!((-a).as_f64(), -1.0);
        assert_eq!((3.0 * a).as_f64(), 3.0);
    }

    // --- Pressure ---

    #[test]
    fn test_pressure_hpa() {
        let p = Pressure::hpa(1013.25);
        assert_eq!(p.to_hpa(), 1013.25);
        assert_approx_eq!(p.to_pa(), 101325.0, rtol <= 1e-10);
    }

    #[test]
    fn test_pressure_pa() {
        let p = Pressure::pa(101325.0);
        assert_approx_eq!(p.to_hpa(), 1013.25, rtol <= 1e-10);
    }

    #[test]
    fn test_pressure_display() {
        let p = Pressure::pa(101325.0);
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

    // -----------------------------------------------------------------------
    // Angle — additional constructors, accessors, and trig functions
    // -----------------------------------------------------------------------

    #[test]
    fn test_angle_from_hms_erfa_tf2a() {
        // ERFA t_erfa_c.c::t_tf2a: tf2a('+', 4, 58, 20.2) = 1.301739278189537429 rad
        let a = Angle::from_hms(Sign::Positive, 4, 58, 20.2);
        assert_approx_eq!(a.to_radians(), 1.301_739_278_189_537_4, atol <= 1e-12);
    }

    #[test]
    fn test_angle_from_hms_negative_within_one_hour() {
        // -0h 30m 0s must be representable as a negative angle.
        let a = Angle::from_hms(Sign::Negative, 0, 30, 0.0);
        assert!(a.to_radians() < 0.0);
        assert_approx_eq!(a.to_degrees(), -7.5, atol <= 1e-12);
    }

    #[test]
    fn test_angle_arcseconds_roundtrip() {
        let a = Angle::arcseconds(3600.0); // = 1 degree
        assert_approx_eq!(a.to_degrees(), 1.0, rtol <= 1e-10);
        assert_approx_eq!(a.to_arcseconds(), 3600.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_arcseconds_normalized() {
        // ARCSECONDS_IN_CIRCLE + 3600 should normalize to 3600 arcsec = 1 deg
        let a = Angle::arcseconds_normalized(ARCSECONDS_IN_CIRCLE + 3600.0);
        assert_approx_eq!(a.to_arcseconds(), 3600.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_arcseconds_normalized_signed() {
        let a = Angle::arcseconds_normalized_signed(-ARCSECONDS_IN_CIRCLE - 3600.0);
        // Signed normalisation: just removes full circles, sign preserved
        let deg = a.to_degrees();
        assert!(deg < 0.0);
    }

    #[test]
    fn test_angle_degrees_normalized() {
        let a = Angle::degrees_normalized(370.0); // 370 mod 360 = 10, then to [0, 2π)
        assert_approx_eq!(a.to_degrees(), 10.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_degrees_normalized_signed() {
        let a = Angle::degrees_normalized_signed(-10.0);
        assert_approx_eq!(a.to_degrees(), -10.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_radians_normalized() {
        use core::f64::consts::TAU;
        let a = Angle::radians_normalized(TAU + 0.5);
        assert_approx_eq!(a.as_f64(), 0.5, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_radians_normalized_signed() {
        use core::f64::consts::TAU;
        let a = Angle::radians_normalized_signed(-TAU - 0.5);
        assert!(a.as_f64() < 0.0);
    }

    #[test]
    fn test_angle_is_zero() {
        assert!(Angle::ZERO.is_zero());
        assert!(!Angle::PI.is_zero());
    }

    #[test]
    fn test_angle_abs() {
        let a = Angle::radians(-1.5);
        assert_approx_eq!(a.abs().as_f64(), 1.5, rtol <= 1e-10);
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
        assert_approx_eq!(a.as_f64(), 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_angle_from_acosh() {
        let a = Angle::from_acosh(1.0);
        assert_approx_eq!(a.as_f64(), 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_angle_from_atanh() {
        let a = Angle::from_atanh(0.0);
        assert_approx_eq!(a.as_f64(), 0.0, atol <= 1e-10);
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
    fn test_angle_mod_two_pi() {
        use core::f64::consts::TAU;
        let a = Angle::radians(TAU + 1.0).mod_two_pi();
        assert_approx_eq!(a.as_f64(), 1.0, rtol <= 1e-10);
    }

    #[test]
    fn test_angle_mod_two_pi_signed() {
        use core::f64::consts::TAU;
        let a = Angle::radians(TAU + 1.0).mod_two_pi_signed();
        assert_approx_eq!(a.as_f64(), 1.0, rtol <= 1e-10);
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
        let f: f64 = a.into();
        assert_approx_eq!(f, a.as_f64(), rtol <= 1e-10);
    }

    #[test]
    fn test_angle_i64_units() {
        let a = 90_i64.deg();
        assert_approx_eq!(a.to_degrees(), 90.0, rtol <= 1e-10);
        let b = 1_i64.rad();
        assert_approx_eq!(b.as_f64(), 1.0, rtol <= 1e-10);
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

    // -----------------------------------------------------------------------
    // Distance — additional accessors and units
    // -----------------------------------------------------------------------

    #[test]
    fn test_distance_to_astronomical_units() {
        let d = Distance::astronomical_units(1.0);
        assert_approx_eq!(d.to_astronomical_units(), 1.0, rtol <= 1e-10);
    }

    #[test]
    fn test_distance_as_f64() {
        let d = Distance::meters(5000.0);
        assert_eq!(d.as_f64(), 5000.0);
    }

    #[test]
    fn test_distance_new() {
        let d = Distance::new(1234.0);
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
        let f: f64 = a.into();
        assert_eq!(f, 100.0);
    }

    // -----------------------------------------------------------------------
    // Velocity — additional constructors/accessors
    // -----------------------------------------------------------------------

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
    fn test_velocity_new() {
        let v = Velocity::new(300.0);
        assert_eq!(v.as_f64(), 300.0);
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
        let f: f64 = a.into();
        assert_eq!(f, 500.0);
    }

    // -----------------------------------------------------------------------
    // Mass — additional constructors and i64 metric_tons
    // -----------------------------------------------------------------------

    #[test]
    fn test_mass_new() {
        let m = Mass::new(1.5);
        assert_eq!(m.as_f64(), 1.5);
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
        let f: f64 = a.into();
        assert_eq!(f, 100.0);
    }

    // -----------------------------------------------------------------------
    // Area — new/as_f64
    // -----------------------------------------------------------------------

    #[test]
    fn test_area_new() {
        let a = Area::new(5.0);
        assert_eq!(a.as_f64(), 5.0);
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
        let f: f64 = a.into();
        assert_eq!(f, 4.0);
    }

    // -----------------------------------------------------------------------
    // AreaToMass — new/as_f64/i64 units/arithmetic
    // -----------------------------------------------------------------------

    #[test]
    fn test_area_to_mass_new() {
        let r = AreaToMass::new(0.1);
        assert_eq!(r.as_f64(), 0.1);
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
        assert_approx_eq!((a + b).as_f64(), 0.15, rtol <= 1e-10);
        assert_approx_eq!((a - b).as_f64(), 0.05, rtol <= 1e-10);
        let scaled = 2.0 * b;
        assert_approx_eq!(scaled.as_f64(), 0.1, rtol <= 1e-10);
        let f: f64 = a.into();
        assert_approx_eq!(f, 0.1, rtol <= 1e-10);
    }

    // -----------------------------------------------------------------------
    // Frequency — additional
    // -----------------------------------------------------------------------

    #[test]
    fn test_frequency_new() {
        let f = Frequency::new(1e9);
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
        let f: f64 = a.into();
        assert_approx_eq!(f, 1e9, rtol <= 1e-10);
    }

    // -----------------------------------------------------------------------
    // Temperature — arithmetic
    // -----------------------------------------------------------------------

    #[test]
    fn test_temperature_neg() {
        let t = Temperature::kelvin(100.0);
        assert_eq!((-t).as_f64(), -100.0);
    }

    #[test]
    fn test_temperature_add_assign_sub_assign() {
        let mut a = Temperature::kelvin(200.0);
        a += Temperature::kelvin(50.0);
        assert_eq!(a.as_f64(), 250.0);
        a -= Temperature::kelvin(100.0);
        assert_eq!(a.as_f64(), 150.0);
    }

    // -----------------------------------------------------------------------
    // Pressure — new/arithmetic
    // -----------------------------------------------------------------------

    #[test]
    fn test_pressure_new() {
        let p = Pressure::new(101325.0);
        assert_eq!(p.as_f64(), 101325.0);
    }

    #[test]
    fn test_pressure_arithmetic() {
        let a = Pressure::pa(1000.0);
        let b = Pressure::pa(500.0);
        assert_eq!((a + b).to_pa(), 1500.0);
        assert_eq!((a - b).to_pa(), 500.0);
        assert_eq!((-b).to_pa(), -500.0);
        let scaled = 2.0 * b;
        assert_eq!(scaled.to_pa(), 1000.0);
        let f: f64 = a.into();
        assert_eq!(f, 1000.0);
    }

    // -----------------------------------------------------------------------
    // Power — new/add_assign/sub_assign
    // -----------------------------------------------------------------------

    #[test]
    fn test_power_new() {
        let p = Power::new(500.0);
        assert_eq!(p.as_f64(), 500.0);
    }

    #[test]
    fn test_power_add_assign_sub_assign() {
        let mut p = Power::watts(200.0);
        p += Power::watts(100.0);
        assert_eq!(p.to_watts(), 300.0);
        p -= Power::watts(50.0);
        assert_eq!(p.to_watts(), 250.0);
    }

    // -----------------------------------------------------------------------
    // AngularRate — new/as_f64/arithmetic
    // -----------------------------------------------------------------------

    #[test]
    fn test_angular_rate_new() {
        let ar = AngularRate::new(2.0);
        assert_eq!(ar.as_f64(), 2.0);
    }

    #[test]
    fn test_angular_rate_add_assign_sub_assign() {
        let mut ar = AngularRate::radians_per_second(1.0);
        ar += AngularRate::radians_per_second(0.5);
        assert_approx_eq!(ar.as_f64(), 1.5, rtol <= 1e-10);
        ar -= AngularRate::radians_per_second(0.5);
        assert_approx_eq!(ar.as_f64(), 1.0, rtol <= 1e-10);
        let f: f64 = ar.into();
        assert_approx_eq!(f, 1.0, rtol <= 1e-10);
    }

    // -----------------------------------------------------------------------
    // Decibel — add_assign/sub_assign/neg/i64
    // -----------------------------------------------------------------------

    #[test]
    fn test_decibel_add_assign_sub_assign() {
        let mut d = Decibel::new(10.0);
        d += Decibel::new(3.0);
        assert_approx_eq!(d.as_f64(), 13.0, rtol <= 1e-10);
        d -= Decibel::new(3.0);
        assert_approx_eq!(d.as_f64(), 10.0, rtol <= 1e-10);
    }

    #[test]
    fn test_decibel_i64_units() {
        let d = 10_i64.db();
        assert_eq!(d.as_f64(), 10.0);
    }

    #[test]
    fn test_decibel_mul_and_into() {
        let d = Decibel::new(5.0);
        let scaled = 2.0 * d;
        assert_approx_eq!(scaled.as_f64(), 10.0, rtol <= 1e-10);
        let f: f64 = d.into();
        assert_eq!(f, 5.0);
    }

    // -----------------------------------------------------------------------
    // Sign
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // Angle::from_dms (ERFA af2a)
    // -----------------------------------------------------------------------

    #[test]
    fn test_angle_from_dms_erfa_af2a() {
        // ERFA t_erfa_c.c::t_af2a: af2a('-', 45, 13, 27.2) = -0.7893115794313644842 rad
        let a = Angle::from_dms(Sign::Negative, 45, 13, 27.2);
        assert_approx_eq!(a.to_radians(), -0.789_311_579_431_364_4, atol <= 1e-12);
    }

    #[test]
    fn test_angle_from_dms_negative_within_one_degree() {
        // -0° 30' 0" must be representable.
        let a = Angle::from_dms(Sign::Negative, 0, 30, 0.0);
        assert!(a.to_radians() < 0.0);
        assert_approx_eq!(a.to_degrees(), -0.5, atol <= 1e-12);
    }

    // -----------------------------------------------------------------------
    // Angle::to_hms (ERFA a2tf)
    // -----------------------------------------------------------------------

    #[test]
    fn test_angle_to_hms_erfa_a2tf() {
        // ERFA t_erfa_c.c::t_a2tf: a2tf(4, -3.01234) -> -11h 30m 22.6484s
        let (sign, hours, min, sec) = Angle::radians(-3.01234).to_hms();
        assert_eq!(sign, Sign::Negative);
        assert_eq!(hours, 11);
        assert_eq!(min, 30);
        assert_approx_eq!(sec, 22.6484, atol <= 1e-4);
    }

    #[test]
    fn test_angle_to_hms_negative_within_one_hour() {
        // -7.5° = -0h 30m 0s. Tests that the `-0h` case is representable.
        let (sign, hours, min, sec) = Angle::degrees(-7.5).to_hms();
        assert_eq!(sign, Sign::Negative);
        assert_eq!(hours, 0);
        assert_eq!(min, 30);
        assert_approx_eq!(sec, 0.0, atol <= 1e-10);
    }

    // -----------------------------------------------------------------------
    // Angle::to_dms (ERFA a2af)
    // -----------------------------------------------------------------------

    #[test]
    fn test_angle_to_dms_erfa_a2af() {
        // ERFA t_erfa_c.c::t_a2af: a2af(4, 2.345) -> +134° 21' 30.9706"
        let (sign, deg, min, sec) = Angle::radians(2.345).to_dms();
        assert_eq!(sign, Sign::Positive);
        assert_eq!(deg, 134);
        assert_eq!(min, 21);
        assert_approx_eq!(sec, 30.9706, atol <= 1e-4);
    }

    #[test]
    fn test_angle_to_dms_negative() {
        // -0.5° should round-trip via to_dms exactly.
        let (sign, deg, min, sec) = Angle::degrees(-0.5).to_dms();
        assert_eq!(sign, Sign::Negative);
        assert_eq!(deg, 0);
        assert_eq!(min, 30);
        assert_approx_eq!(sec, 0.0, atol <= 1e-10);
    }

    #[test]
    fn test_angle_to_dms_zero() {
        let (sign, deg, min, sec) = Angle::ZERO.to_dms();
        assert_eq!(sign, Sign::Positive);
        assert_eq!(deg, 0);
        assert_eq!(min, 0);
        assert_eq!(sec, 0.0);
    }

    #[rstest]
    #[case(0.0)]
    #[case(0.5)]
    #[case(-0.5)]
    #[case(2.345)]
    #[case(-3.01234)]
    #[case(core::f64::consts::PI)]
    #[case(-core::f64::consts::PI)]
    fn test_angle_dms_roundtrip(#[case] radians: f64) {
        let a = Angle::radians(radians);
        let (sign, deg, min, sec) = a.to_dms();
        let b = Angle::from_dms(sign, deg, min, sec);
        assert_approx_eq!(a.to_radians(), b.to_radians(), atol <= 1e-12);
    }

    #[rstest]
    #[case(0.0)]
    #[case(0.5)]
    #[case(-0.5)]
    #[case(2.345)]
    #[case(-3.01234)]
    #[case(core::f64::consts::PI)]
    #[case(-core::f64::consts::PI)]
    fn test_angle_hms_roundtrip(#[case] radians: f64) {
        let a = Angle::radians(radians);
        let (sign, hours, min, sec) = a.to_hms();
        let b = Angle::from_hms(sign, hours, min, sec);
        assert_approx_eq!(a.to_radians(), b.to_radians(), atol <= 1e-12);
    }
}
