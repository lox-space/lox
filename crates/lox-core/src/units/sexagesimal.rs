// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Sexagesimal representations of angles: degrees-arcminutes-arcseconds and
//! hours-minutes-seconds.

use core::fmt::{Display, Formatter, Result, Write};

use crate::math::float::{abs, round};

use super::{Angle, Sign};

/// Steps per second (or arcsecond) to which decomposed seconds are rounded, which suppresses
/// floating-point noise such as 29.999999999999996 for 30.
const SECONDS_RESOLUTION: f64 = 1e9;

/// Error returned when sexagesimal components are out of range.
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum SexagesimalError {
    /// The minutes component is 60 or larger.
    #[error("minutes must be less than 60 but were {0}")]
    InvalidMinutes(u8),
    /// The seconds component is non-finite, negative, or 60 or larger.
    #[error("seconds must be finite and in [0, 60) but were {0}")]
    InvalidSeconds(f64),
}

/// Records the first two bytes written to it and whether a non-zero digit occurs, without
/// allocating.
#[derive(Default)]
struct Probe {
    prefix: [u8; 2],
    bytes: usize,
    nonzero_digit: bool,
}

impl Write for Probe {
    fn write_str(&mut self, s: &str) -> Result {
        for &b in s.as_bytes() {
            if self.bytes < 2 {
                self.prefix[self.bytes] = b;
            }
            self.bytes += 1;
            self.nonzero_digit |= matches!(b, b'1'..=b'9');
        }
        Ok(())
    }
}

/// The sign and components shared by [`Dms`] and [`Hms`]: a major unit (degrees or hours),
/// minutes, and seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Parts {
    sign: Sign,
    major: u32,
    minutes: u8,
    seconds: f64,
}

impl Parts {
    /// Validates the components. A zero value is always positive, and negative zero seconds
    /// become positive zero.
    const fn new(
        sign: Sign,
        major: u32,
        minutes: u8,
        seconds: f64,
    ) -> core::result::Result<Self, SexagesimalError> {
        if minutes >= 60 {
            return Err(SexagesimalError::InvalidMinutes(minutes));
        }
        if !(seconds >= 0.0 && seconds < 60.0) {
            return Err(SexagesimalError::InvalidSeconds(seconds));
        }
        Ok(Self::normalized(sign, major, minutes, seconds))
    }

    const fn normalized(sign: Sign, major: u32, minutes: u8, seconds: f64) -> Self {
        // Adding zero turns -0.0 into 0.0.
        let seconds = seconds + 0.0;
        let sign = if major == 0 && minutes == 0 && seconds == 0.0 {
            Sign::Positive
        } else {
            sign
        };
        Self {
            sign,
            major,
            minutes,
            seconds,
        }
    }

    /// Splits a signed total, in units of the smallest component, into its parts. Returns
    /// `None` if the total is non-finite or the major unit does not fit into a `u32`.
    fn decompose(total: f64) -> Option<Self> {
        if !total.is_finite() {
            return None;
        }
        let magnitude = abs(total);
        let major = (magnitude / 3600.0) as u64;
        if major > u32::MAX as u64 {
            return None;
        }
        let mut major = major as u32;
        let rem = magnitude - major as f64 * 3600.0;
        let mut minutes = (rem / 60.0) as i64;
        let mut seconds =
            round((rem - minutes as f64 * 60.0) * SECONDS_RESOLUTION) / SECONDS_RESOLUTION;
        // Correct the truncations if rounding pushed a component out of its range.
        if seconds < 0.0 {
            minutes -= 1;
            seconds += 60.0;
        } else if seconds >= 60.0 {
            minutes += 1;
            seconds -= 60.0;
        }
        if minutes < 0 {
            major = major.checked_sub(1)?;
            minutes += 60;
        } else if minutes >= 60 {
            major = major.checked_add(1)?;
            minutes -= 60;
        }
        Some(Self::normalized(
            Sign::from(total),
            major,
            minutes as u8,
            seconds,
        ))
    }

    /// Returns the signed total in units of the smallest component.
    const fn total(self) -> f64 {
        let magnitude = 60.0 * (60.0 * self.major as f64 + self.minutes as f64) + self.seconds;
        self.sign.as_f64() * magnitude
    }

    /// Formats the parts with the given unit symbols. The formatter's precision applies to the
    /// seconds, and rounding carries into the larger components when the rounded seconds reach
    /// 60. The `+` flag prints a sign for positive values.
    fn fmt(&self, f: &mut Formatter<'_>, symbols: [&'static str; 3]) -> Result {
        let [major_symbol, minutes_symbol, seconds_symbol] = symbols;
        let precision = f.precision();
        let (mut major, mut minutes, mut seconds) = (self.major as u64, self.minutes, self.seconds);
        // Format the seconds the same way as below to see whether they round up to 60 or
        // down to zero.
        let mut probe = Probe::default();
        match precision {
            Some(precision) => write!(probe, "{seconds:.precision$}")?,
            None => write!(probe, "{seconds}")?,
        }
        if &probe.prefix == b"60" {
            seconds = 0.0;
            minutes += 1;
            if minutes == 60 {
                minutes = 0;
                major += 1;
            }
        }
        let shows_zero = major == 0 && minutes == 0 && !probe.nonzero_digit;
        let sign = match self.sign {
            Sign::Negative if !shows_zero => "-",
            _ if f.sign_plus() => "+",
            _ => "",
        };
        write!(f, "{sign}{major}{major_symbol}{minutes:02}{minutes_symbol}")?;
        // The seconds are zero-padded to two integer digits.
        match precision {
            Some(precision) => {
                let width = if precision == 0 { 2 } else { precision + 3 };
                write!(f, "{seconds:0width$.precision$}")?;
            }
            None => {
                if seconds < 10.0 {
                    f.write_char('0')?;
                }
                write!(f, "{seconds}")?;
            }
        }
        f.write_str(seconds_symbol)
    }
}

/// An angle in degrees, arcminutes, and arcseconds.
///
/// The magnitude components are non-negative and the sign is stored separately, so that angles
/// in `(−1°, 0°)` are representable. A zero angle is always positive.
///
/// `Display` prints e.g. `-12°03′07.5″`. The formatter's precision applies to the arcseconds,
/// and rounding carries into the arcminutes and degrees. The `+` flag prints a sign for positive
/// angles.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "DmsRepr", into = "DmsRepr")
)]
pub struct Dms(Parts);

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct DmsRepr {
    sign: Sign,
    degrees: u32,
    arcminutes: u8,
    arcseconds: f64,
}

#[cfg(feature = "serde")]
impl From<Dms> for DmsRepr {
    fn from(dms: Dms) -> Self {
        Self {
            sign: dms.sign(),
            degrees: dms.degrees(),
            arcminutes: dms.arcminutes(),
            arcseconds: dms.arcseconds(),
        }
    }
}

#[cfg(feature = "serde")]
impl TryFrom<DmsRepr> for Dms {
    type Error = SexagesimalError;

    fn try_from(repr: DmsRepr) -> core::result::Result<Self, Self::Error> {
        Self::new(repr.sign, repr.degrees, repr.arcminutes, repr.arcseconds)
    }
}

impl Dms {
    /// Creates a new [`Dms`].
    ///
    /// # Errors
    ///
    /// Returns a [`SexagesimalError`] if `arcminutes` is 60 or larger, or if `arcseconds` is
    /// non-finite or outside `[0, 60)`.
    pub const fn new(
        sign: Sign,
        degrees: u32,
        arcminutes: u8,
        arcseconds: f64,
    ) -> core::result::Result<Self, SexagesimalError> {
        match Parts::new(sign, degrees, arcminutes, arcseconds) {
            Ok(parts) => Ok(Self(parts)),
            Err(err) => Err(err),
        }
    }

    /// Returns the sign.
    pub const fn sign(self) -> Sign {
        self.0.sign
    }

    /// Returns the whole degrees.
    pub const fn degrees(self) -> u32 {
        self.0.major
    }

    /// Returns the arcminutes in `[0, 60)`.
    pub const fn arcminutes(self) -> u8 {
        self.0.minutes
    }

    /// Returns the arcseconds in `[0, 60)`.
    pub const fn arcseconds(self) -> f64 {
        self.0.seconds
    }

    /// Converts to an [`Angle`].
    ///
    /// # References
    ///
    /// - ERFA [`af2a`](https://github.com/liberfa/erfa/blob/master/src/af2a.c)
    pub const fn to_angle(self) -> Angle {
        Angle::arcseconds(self.0.total())
    }
}

impl From<Dms> for Angle {
    fn from(dms: Dms) -> Self {
        dms.to_angle()
    }
}

impl Display for Dms {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f, ["°", "′", "″"])
    }
}

/// An angle in hours, minutes, and seconds of time, with 24 hours to the full circle.
///
/// The magnitude components are non-negative and the sign is stored separately, so that angles
/// in `(−1h, 0h)` are representable. A zero angle is always positive.
///
/// `Display` prints e.g. `12h03m07.5s`. The formatter's precision applies to the seconds, and
/// rounding carries into the minutes and hours. The `+` flag prints a sign for positive angles.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "HmsRepr", into = "HmsRepr")
)]
pub struct Hms(Parts);

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct HmsRepr {
    sign: Sign,
    hours: u32,
    minutes: u8,
    seconds: f64,
}

#[cfg(feature = "serde")]
impl From<Hms> for HmsRepr {
    fn from(hms: Hms) -> Self {
        Self {
            sign: hms.sign(),
            hours: hms.hours(),
            minutes: hms.minutes(),
            seconds: hms.seconds(),
        }
    }
}

#[cfg(feature = "serde")]
impl TryFrom<HmsRepr> for Hms {
    type Error = SexagesimalError;

    fn try_from(repr: HmsRepr) -> core::result::Result<Self, Self::Error> {
        Self::new(repr.sign, repr.hours, repr.minutes, repr.seconds)
    }
}

impl Hms {
    /// Creates a new [`Hms`].
    ///
    /// # Errors
    ///
    /// Returns a [`SexagesimalError`] if `minutes` is 60 or larger, or if `seconds` is
    /// non-finite or outside `[0, 60)`.
    pub const fn new(
        sign: Sign,
        hours: u32,
        minutes: u8,
        seconds: f64,
    ) -> core::result::Result<Self, SexagesimalError> {
        match Parts::new(sign, hours, minutes, seconds) {
            Ok(parts) => Ok(Self(parts)),
            Err(err) => Err(err),
        }
    }

    /// Returns the sign.
    pub const fn sign(self) -> Sign {
        self.0.sign
    }

    /// Returns the whole hours.
    pub const fn hours(self) -> u32 {
        self.0.major
    }

    /// Returns the minutes in `[0, 60)`.
    pub const fn minutes(self) -> u8 {
        self.0.minutes
    }

    /// Returns the seconds in `[0, 60)`.
    pub const fn seconds(self) -> f64 {
        self.0.seconds
    }

    /// Converts to an [`Angle`].
    ///
    /// # References
    ///
    /// - ERFA [`tf2a`](https://github.com/liberfa/erfa/blob/master/src/tf2a.c)
    pub const fn to_angle(self) -> Angle {
        Angle::seconds_of_time(self.0.total())
    }
}

impl From<Hms> for Angle {
    fn from(hms: Hms) -> Self {
        hms.to_angle()
    }
}

impl Display for Hms {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f, ["h", "m", "s"])
    }
}

impl Angle {
    /// Creates a new angle from sign, degrees, arcminutes, and arcseconds.
    ///
    /// # Errors
    ///
    /// Returns a [`SexagesimalError`] if the components are out of range, see [`Dms::new`].
    pub const fn from_dms(
        sign: Sign,
        degrees: u32,
        arcminutes: u8,
        arcseconds: f64,
    ) -> core::result::Result<Self, SexagesimalError> {
        match Dms::new(sign, degrees, arcminutes, arcseconds) {
            Ok(dms) => Ok(dms.to_angle()),
            Err(err) => Err(err),
        }
    }

    /// Creates a new angle from sign, hours, minutes, and seconds of time.
    ///
    /// # Errors
    ///
    /// Returns a [`SexagesimalError`] if the components are out of range, see [`Hms::new`].
    pub const fn from_hms(
        sign: Sign,
        hours: u32,
        minutes: u8,
        seconds: f64,
    ) -> core::result::Result<Self, SexagesimalError> {
        match Hms::new(sign, hours, minutes, seconds) {
            Ok(hms) => Ok(hms.to_angle()),
            Err(err) => Err(err),
        }
    }

    /// Decomposes the angle into degrees, arcminutes, and arcseconds.
    ///
    /// The arcseconds are rounded to the nearest 1e-9 arcsecond, which removes floating-point
    /// noise as long as the angle itself is that precise, i.e. for angles below about 1000°.
    /// Returns `None` if the angle is non-finite or its whole degrees do not fit into a `u32`.
    ///
    /// # References
    ///
    /// - ERFA [`a2af`](https://github.com/liberfa/erfa/blob/master/src/a2af.c)
    pub fn to_dms(self) -> Option<Dms> {
        Parts::decompose(self.to_arcseconds()).map(Dms)
    }

    /// Decomposes the angle into hours, minutes, and seconds of time.
    ///
    /// The seconds are rounded to the nearest 1e-9 second, which removes floating-point noise as
    /// long as the angle itself is that precise, i.e. for angles below about 1000 hours. Returns
    /// `None` if the angle is non-finite or its whole hours do not fit into a `u32`.
    ///
    /// # References
    ///
    /// - ERFA [`a2tf`](https://github.com/liberfa/erfa/blob/master/src/a2tf.c)
    pub fn to_hms(self) -> Option<Hms> {
        Parts::decompose(self.to_seconds_of_time()).map(Hms)
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString;

    use lox_approx::assert_approx_eq;
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_dms_rejects_out_of_range_components() {
        assert_eq!(
            Dms::new(Sign::Positive, 1, 60, 0.0),
            Err(SexagesimalError::InvalidMinutes(60))
        );
        assert_eq!(
            Dms::new(Sign::Positive, 1, 0, 60.0),
            Err(SexagesimalError::InvalidSeconds(60.0))
        );
        assert_eq!(
            Dms::new(Sign::Positive, 1, 0, -1.0),
            Err(SexagesimalError::InvalidSeconds(-1.0))
        );
        assert!(Dms::new(Sign::Positive, 1, 0, f64::NAN).is_err());
        assert!(Hms::new(Sign::Positive, 1, 61, 0.0).is_err());
        assert!(Angle::from_hms(Sign::Positive, 0, 0, 86400.0).is_err());
    }

    #[test]
    fn test_angle_from_dms_erfa_af2a() {
        // ERFA t_af2a: -45° 13′ 27.2″
        let a = Angle::from_dms(Sign::Negative, 45, 13, 27.2).unwrap();
        assert_approx_eq!(a.to_radians(), -0.789_311_579_431_364_4, atol <= 1e-12);
    }

    #[test]
    fn test_angle_from_hms_erfa_tf2a() {
        // ERFA t_tf2a: 4h 58m 20.2s
        let a = Angle::from_hms(Sign::Positive, 4, 58, 20.2).unwrap();
        assert_approx_eq!(a.to_radians(), 1.301_739_278_189_537_4, atol <= 1e-12);
    }

    #[test]
    fn test_angle_from_dms_negative_within_one_degree() {
        let a = Angle::from_dms(Sign::Negative, 0, 30, 0.0).unwrap();
        assert_approx_eq!(a.to_degrees(), -0.5, atol <= 1e-15);
    }

    #[test]
    fn test_angle_to_dms_erfa_a2af() {
        // ERFA t_a2af: 2.345 rad → +134° 21′ 30.9706″
        let dms = Angle::radians(2.345).to_dms().unwrap();
        assert_eq!(dms.sign(), Sign::Positive);
        assert_eq!(dms.degrees(), 134);
        assert_eq!(dms.arcminutes(), 21);
        assert_approx_eq!(dms.arcseconds(), 30.9706, atol <= 1e-4);
    }

    #[test]
    fn test_angle_to_hms_erfa_a2tf() {
        // ERFA t_a2tf: -3.01234 rad → -11h 30m 22.6484s
        let hms = Angle::radians(-3.01234).to_hms().unwrap();
        assert_eq!(hms.sign(), Sign::Negative);
        assert_eq!(hms.hours(), 11);
        assert_eq!(hms.minutes(), 30);
        assert_approx_eq!(hms.seconds(), 22.6484, atol <= 1e-4);
    }

    #[test]
    fn test_angle_to_hms_negative_within_one_hour() {
        let hms = Angle::from_hms(Sign::Negative, 0, 30, 0.0)
            .unwrap()
            .to_hms()
            .unwrap();
        assert_eq!(hms.sign(), Sign::Negative);
        assert_eq!((hms.hours(), hms.minutes()), (0, 30));
        assert_approx_eq!(hms.seconds(), 0.0, atol <= 1e-9);
    }

    #[test]
    fn test_angle_to_dms_integer_boundary() {
        let dms = Angle::degrees(30.0).to_dms().unwrap();
        assert_eq!((dms.degrees(), dms.arcminutes()), (30, 0));
        assert_approx_eq!(dms.arcseconds(), 0.0, atol <= 1e-9);
    }

    #[test]
    fn test_angle_to_dms_zero() {
        let dms = Angle::ZERO.to_dms().unwrap();
        assert_eq!(dms, Dms::new(Sign::Positive, 0, 0, 0.0).unwrap());
    }

    #[rstest]
    #[case(f64::NAN)]
    #[case(f64::INFINITY)]
    #[case(1e20)]
    fn test_angle_decomposition_out_of_range(#[case] radians: f64) {
        assert_eq!(Angle::radians(radians).to_dms(), None);
        assert_eq!(Angle::radians(radians).to_hms(), None);
    }

    #[rstest]
    #[case(0.0)]
    #[case(0.5)]
    #[case(-0.5)]
    #[case(2.345)]
    #[case(-3.01234)]
    #[case(core::f64::consts::PI)]
    #[case(-core::f64::consts::PI)]
    fn test_angle_sexagesimal_round_trip(#[case] radians: f64) {
        let a = Angle::radians(radians);
        let dms = a.to_dms().unwrap();
        assert_approx_eq!(Angle::from(dms).to_radians(), radians, atol <= 1e-12);
        let hms = a.to_hms().unwrap();
        assert_approx_eq!(Angle::from(hms).to_radians(), radians, atol <= 1e-12);
    }

    #[test]
    fn test_dms_display() {
        let dms = Dms::new(Sign::Negative, 12, 3, 7.5).unwrap();
        assert_eq!(dms.to_string(), "-12°03′07.5″");
        assert_eq!(format!("{dms:.3}"), "-12°03′07.500″");
        assert_eq!(format!("{dms:.0}"), "-12°03′08″");
    }

    #[test]
    fn test_hms_display() {
        let hms = Hms::new(Sign::Positive, 4, 58, 20.2).unwrap();
        assert_eq!(hms.to_string(), "4h58m20.2s");
        assert_eq!(format!("{hms:.2}"), "4h58m20.20s");
    }

    #[test]
    fn test_sexagesimal_display_flags() {
        let dms = Dms::new(Sign::Positive, 1, 2, 3.0).unwrap();
        assert_eq!(dms.to_string(), "1°02′03″");
        assert_eq!(format!("{dms:+}"), "+1°02′03″");
        let hms = Hms::new(Sign::Negative, 1, 2, 3.5).unwrap();
        assert_eq!(format!("{hms:+.2}"), "-1h02m03.50s");
    }

    #[test]
    fn test_angle_to_dms_has_no_float_noise() {
        let dms = Angle::arcseconds(30.0).to_dms().unwrap();
        assert_eq!(dms.arcseconds(), 30.0);
        assert_eq!(dms.to_string(), "0°00′30″");
        let dms = Angle::degrees(359.999).to_dms().unwrap();
        assert_eq!(dms.to_string(), "359°59′56.4″");
    }

    #[test]
    fn test_negative_zero_seconds_are_positive_zero() {
        let dms = Dms::new(Sign::Positive, 1, 2, -0.0).unwrap();
        assert!(dms.arcseconds().is_sign_positive());
        assert_eq!(dms.to_string(), "1°02′00″");
        let hms = Hms::new(Sign::Negative, 0, 0, -0.0).unwrap();
        assert_eq!(hms.sign(), Sign::Positive);
    }

    #[test]
    fn test_display_of_values_rounding_to_zero_is_positive() {
        let dms = Dms::new(Sign::Negative, 0, 0, 0.0004).unwrap();
        assert_eq!(format!("{dms:.3}"), "0°00′00.000″");
        assert_eq!(format!("{dms:+.3}"), "+0°00′00.000″");
        assert_eq!(format!("{dms:.4}"), "-0°00′00.0004″");
        let hms = Hms::new(Sign::Negative, 0, 0, 0.04).unwrap();
        assert_eq!(format!("{hms:.1}"), "0h00m00.0s");
        // A carry into the minutes keeps the sign.
        let dms = Dms::new(Sign::Negative, 0, 0, 59.9996).unwrap();
        assert_eq!(format!("{dms:.3}"), "-0°01′00.000″");
    }

    #[test]
    fn test_negative_zero_components_are_positive() {
        assert_eq!(
            Dms::new(Sign::Negative, 0, 0, 0.0).unwrap(),
            Angle::ZERO.to_dms().unwrap()
        );
        assert_eq!(
            Hms::new(Sign::Negative, 0, 0, 0.0).unwrap().to_string(),
            "0h00m00s"
        );
        assert_eq!(
            Dms::new(Sign::Negative, 0, 0, 1.0).unwrap().sign(),
            Sign::Negative
        );
    }

    #[test]
    fn test_angle_decomposition_of_negative_zero_is_positive() {
        let zero = Angle::ZERO.to_dms().unwrap();
        assert_eq!(Angle::radians(-0.0).to_dms().unwrap(), zero);
        assert_eq!(Angle::radians(-1e-16).to_dms().unwrap(), zero);
        assert_eq!(
            Angle::radians(-1e-16).to_hms().unwrap().to_string(),
            "0h00m00s"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_sexagesimal_serde_round_trip_and_validation() {
        let dms = Dms::new(Sign::Negative, 12, 3, 7.5).unwrap();
        let json = serde_json::to_string(&dms).unwrap();
        assert_eq!(serde_json::from_str::<Dms>(&json).unwrap(), dms);
        let hms = Hms::new(Sign::Positive, 4, 58, 20.2).unwrap();
        let json = serde_json::to_string(&hms).unwrap();
        assert_eq!(serde_json::from_str::<Hms>(&json).unwrap(), hms);
        let bad = r#"{"sign":"Positive","degrees":1,"arcminutes":60,"arcseconds":0.0}"#;
        assert!(serde_json::from_str::<Dms>(bad).is_err());
    }

    #[test]
    fn test_sexagesimal_display_carries_rounded_seconds() {
        let dms = Dms::new(Sign::Positive, 12, 59, 59.9996).unwrap();
        assert_eq!(format!("{dms:.3}"), "13°00′00.000″");
        let hms = Hms::new(Sign::Positive, 23, 58, 59.96).unwrap();
        assert_eq!(format!("{hms:.1}"), "23h59m00.0s");
    }
}
