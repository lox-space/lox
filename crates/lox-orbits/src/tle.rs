// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Two-Line Element sets.

use std::f64::consts::TAU;
use std::fmt;

use lox_core::elements::Eccentricity;
use lox_core::time::constants::SECONDS_PER_DAY;
use lox_core::units::{Angle, AngularRate};
use lox_time::calendar_dates::Date;
use lox_time::subsecond::Subsecond;
use lox_time::time_of_day::TimeOfDay;
use lox_time::utc::Utc;
use lox_time::utc::leap_seconds::DefaultLeapSecondsProvider;
use thiserror::Error;

const LINE_LENGTH: usize = 69;

/// Zero-based columns of line 1 that must be blank.
const LINE_1_SEPARATORS: [usize; 8] = [1, 8, 17, 32, 43, 52, 61, 63];

/// Zero-based columns of line 2 that must be blank.
const LINE_2_SEPARATORS: [usize; 7] = [1, 7, 16, 25, 33, 42, 51];

const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;
const NANOSECONDS_PER_DAY: u64 = SECONDS_PER_DAY as u64 * NANOSECONDS_PER_SECOND;
const ATTOSECONDS_PER_NANOSECOND: i64 = 1_000_000_000;

/// One of the two data lines of a TLE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TleLine {
    /// The first data line.
    One,
    /// The second data line.
    Two,
}

impl fmt::Display for TleLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TleLine::One => f.write_str("line 1"),
            TleLine::Two => f.write_str("line 2"),
        }
    }
}

/// Error returned when a Two-Line Element set cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum TleError {
    /// A line does not have the fixed TLE length.
    #[error("TLE parse error on {line}: expected {LINE_LENGTH} characters, found {found}")]
    BadLength {
        /// The offending line.
        line: TleLine,
        /// The length of the line.
        found: usize,
    },
    /// A line does not start with its line number.
    #[error("TLE parse error on {line}: wrong line number")]
    BadLineNumber {
        /// The offending line.
        line: TleLine,
    },
    /// A separator column is not blank.
    #[error("TLE parse error on {line}: expected a space at column {column}")]
    ExpectedSpace {
        /// The offending line.
        line: TleLine,
        /// The zero-based column.
        column: usize,
    },
    /// The checksum of a line does not match its contents.
    #[error("TLE parse error on {line}: bad checksum")]
    BadChecksum {
        /// The offending line.
        line: TleLine,
    },
    /// The two lines carry different catalog numbers.
    #[error("TLE parse error: the catalog numbers of the two lines differ")]
    NoradIdMismatch,
    /// A field could not be parsed.
    #[error("TLE parse error on {line}: invalid {field}")]
    InvalidField {
        /// The offending line.
        line: TleLine,
        /// The name of the field.
        field: &'static str,
    },
}

/// Security classification of a set of elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    /// Declassified, or without a classification.
    Unclassified,
    /// Classified.
    Classified,
    /// Secret.
    Secret,
}

/// A Two-Line Element set: the SGP4 mean elements of an Earth orbiter.
///
/// With the `serde` feature, this type reads and writes CelesTrak's general
/// perturbations JSON format, with keys such as `OBJECT_NAME` and `MEAN_MOTION`.
#[derive(Debug, Clone, PartialEq)]
pub struct Tle {
    /// The object name, if known.
    pub object_name: Option<String>,
    /// The international designator, e.g. `1998-067A`.
    pub international_designator: Option<String>,
    /// The NORAD catalog number.
    pub norad_id: u64,
    /// The security classification.
    pub classification: Classification,
    /// The element epoch.
    pub epoch: Utc,
    /// Half the first derivative of mean motion, as stored in the TLE (rev/day²).
    pub half_mean_motion_dot_rev_per_day2: f64,
    /// One sixth of the second derivative of mean motion, as stored in the TLE (rev/day³).
    pub sixth_mean_motion_ddot_rev_per_day3: f64,
    /// The BSTAR drag term (Earth radii⁻¹).
    pub drag_term_per_earth_radius: f64,
    /// The ephemeris type.
    pub ephemeris_type: u8,
    /// The element set number.
    pub element_set_number: u64,
    /// The inclination.
    pub inclination: Angle,
    /// The right ascension of the ascending node.
    pub right_ascension: Angle,
    /// The eccentricity.
    pub eccentricity: Eccentricity,
    /// The argument of perigee.
    pub argument_of_perigee: Angle,
    /// The mean anomaly.
    pub mean_anomaly: Angle,
    /// The mean motion (Kozai convention).
    pub mean_motion: AngularRate,
    /// The revolution number at epoch.
    pub revolution_number: u64,
}

impl Tle {
    /// Parses the two data lines of a TLE. `object_name` is the content of line 0, if any.
    pub fn from_lines(
        object_name: Option<String>,
        line_1: impl AsRef<[u8]>,
        line_2: impl AsRef<[u8]>,
    ) -> Result<Self, TleError> {
        let line_1 = line_1.as_ref();
        let line_2 = line_2.as_ref();
        check_line(line_1, TleLine::One, b'1', &LINE_1_SEPARATORS)?;
        check_line(line_2, TleLine::Two, b'2', &LINE_2_SEPARATORS)?;

        let invalid_1 = |field| TleError::InvalidField {
            line: TleLine::One,
            field,
        };
        let invalid_2 = |field| TleError::InvalidField {
            line: TleLine::Two,
            field,
        };

        let norad_id = parse_catalog_number(&line_1[2..7]).ok_or(invalid_1("norad_id"))?;
        let norad_id_2 = parse_catalog_number(&line_2[2..7]).ok_or(invalid_2("norad_id"))?;
        if norad_id != norad_id_2 {
            return Err(TleError::NoradIdMismatch);
        }

        let classification = match line_1[7] {
            b'U' => Classification::Unclassified,
            b'C' => Classification::Classified,
            b'S' => Classification::Secret,
            _ => return Err(invalid_1("classification")),
        };

        let designator = &line_1[9..17];
        let international_designator = if designator.iter().all(|&b| b == b' ') {
            None
        } else {
            Some(
                parse_international_designator(designator)
                    .ok_or(invalid_1("international_designator"))?,
            )
        };

        let epoch = parse_epoch(&line_1[18..20], &line_1[20..32]).ok_or(invalid_1("epoch"))?;
        let half_mean_motion_dot_rev_per_day2 =
            parse_decimal(&line_1[33..43]).ok_or(invalid_1("mean_motion_dot"))?;
        let sixth_mean_motion_ddot_rev_per_day3 =
            parse_exponential(&line_1[44..52]).ok_or(invalid_1("mean_motion_ddot"))?;
        let drag_term_per_earth_radius =
            parse_exponential(&line_1[53..61]).ok_or(invalid_1("drag_term"))?;
        let ephemeris_type = parse_integer(&line_1[62..63])
            .and_then(|value| u8::try_from(value).ok())
            .ok_or(invalid_1("ephemeris_type"))?;
        let element_set_number =
            parse_integer(&line_1[64..68]).ok_or(invalid_1("element_set_number"))?;

        let inclination = parse_decimal(&line_2[8..16]).ok_or(invalid_2("inclination"))?;
        let right_ascension = parse_decimal(&line_2[17..25]).ok_or(invalid_2("right_ascension"))?;
        let eccentricity = parse_eccentricity(&line_2[26..33]).ok_or(invalid_2("eccentricity"))?;
        let argument_of_perigee =
            parse_decimal(&line_2[34..42]).ok_or(invalid_2("argument_of_perigee"))?;
        let mean_anomaly = parse_decimal(&line_2[43..51]).ok_or(invalid_2("mean_anomaly"))?;
        let mean_motion = parse_decimal(&line_2[52..63]).ok_or(invalid_2("mean_motion"))?;
        let revolution_number =
            parse_integer(&line_2[63..68]).ok_or(invalid_2("revolution_number"))?;

        Ok(Self {
            object_name,
            international_designator,
            norad_id,
            classification,
            epoch,
            half_mean_motion_dot_rev_per_day2,
            sixth_mean_motion_ddot_rev_per_day3,
            drag_term_per_earth_radius,
            ephemeris_type,
            element_set_number,
            inclination: Angle::degrees(inclination),
            right_ascension: Angle::degrees(right_ascension),
            eccentricity,
            argument_of_perigee: Angle::degrees(argument_of_perigee),
            mean_anomaly: Angle::degrees(mean_anomaly),
            mean_motion: AngularRate::radians_per_second(
                mean_motion * TAU / SECONDS_PER_DAY as f64,
            ),
            revolution_number,
        })
    }
}

/// Checks the length, line number, separator columns and checksum of a data line.
///
/// All other parsing relies on this check for the line being [`LINE_LENGTH`] bytes long.
fn check_line(
    bytes: &[u8],
    line: TleLine,
    number: u8,
    separators: &[usize],
) -> Result<(), TleError> {
    if bytes.len() != LINE_LENGTH {
        return Err(TleError::BadLength {
            line,
            found: bytes.len(),
        });
    }
    if bytes[0] != number {
        return Err(TleError::BadLineNumber { line });
    }
    if let Some(&column) = separators.iter().find(|&&column| bytes[column] != b' ') {
        return Err(TleError::ExpectedSpace { line, column });
    }
    let checksum = bytes[LINE_LENGTH - 1];
    let sum: u32 = bytes[..LINE_LENGTH - 1]
        .iter()
        .map(|&b| match b {
            b'0'..=b'9' => u32::from(b - b'0'),
            b'-' => 1,
            _ => 0,
        })
        .sum();
    if !checksum.is_ascii_digit() || u32::from(checksum - b'0') != sum % 10 {
        return Err(TleError::BadChecksum { line });
    }
    Ok(())
}

/// Removes the blanks a numeric field may be left-padded with.
fn strip_padding(field: &[u8]) -> &[u8] {
    let start = field.iter().position(|&b| b != b' ').unwrap_or(field.len());
    &field[start..]
}

/// Returns `bytes` as text if it is a non-empty run of ASCII digits.
fn digits(bytes: &[u8]) -> Option<&str> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(bytes).ok()
}

fn parse_integer(field: &[u8]) -> Option<u64> {
    digits(strip_padding(field))?.parse().ok()
}

/// Parses a decimal number with an optional sign and an optional decimal point.
///
/// The syntax is checked here because `f64::from_str` also accepts `NaN`, `inf` and exponents.
fn parse_decimal(field: &[u8]) -> Option<f64> {
    let text = strip_padding(field);
    let unsigned = match text {
        [b'-' | b'+', rest @ ..] => rest,
        _ => text,
    };
    let digit_count = unsigned.iter().filter(|b| b.is_ascii_digit()).count();
    let dot_count = unsigned.iter().filter(|&&b| b == b'.').count();
    if digit_count == 0 || dot_count > 1 || digit_count + dot_count != unsigned.len() {
        return None;
    }
    std::str::from_utf8(text).ok()?.parse().ok()
}

/// Parses a field with a sign, five digits behind an implied leading decimal point and a signed
/// one-digit power of ten, e.g. `-11606-4` for -0.11606e-4.
///
/// Padding blanks stand in for the leading digits, so they do not move the decimal point.
fn parse_exponential(field: &[u8]) -> Option<f64> {
    let (mantissa, exponent) = field.split_at_checked(6)?;
    let padded = strip_padding(mantissa);
    let (sign, mantissa) = match padded {
        [b'-', rest @ ..] => ("-", rest),
        [b'+', rest @ ..] => ("", rest),
        rest if rest.len() < 6 => ("", rest),
        _ => return None,
    };
    let mantissa = digits(mantissa)?;
    let (exponent_sign, exponent) = match exponent {
        [b'-', digit] => ('-', digit),
        [b'+' | b' ', digit] => ('+', digit),
        _ => return None,
    };
    let exponent = digits(std::slice::from_ref(exponent))?;
    format!("{sign}.{mantissa:0>5}e{exponent_sign}{exponent}")
        .parse()
        .ok()
}

/// Parses the seven digits behind the implied leading decimal point of the eccentricity.
///
/// A sign is not part of the field, which rejects negative values.
fn parse_eccentricity(field: &[u8]) -> Option<Eccentricity> {
    let digits = digits(strip_padding(field))?;
    let value = format!(".{digits:0>7}").parse().ok()?;
    Eccentricity::try_new(value).ok()
}

/// Maps the leading letter of an Alpha-5 catalog number to the value it stands for.
///
/// The letters I and O are not used.
fn alpha5_value(letter: u8) -> Option<u64> {
    let value = match letter {
        b'A'..=b'H' => letter - b'A' + 10,
        b'J'..=b'N' => letter - b'J' + 18,
        b'P'..=b'Z' => letter - b'P' + 23,
        _ => return None,
    };
    Some(u64::from(value))
}

fn parse_catalog_number(field: &[u8]) -> Option<u64> {
    let (&first, rest) = field.split_first()?;
    match alpha5_value(first) {
        Some(value) => {
            let rest: u64 = digits(rest)?.parse().ok()?;
            Some(value * 10_000 + rest)
        }
        None => parse_integer(field),
    }
}

/// Expands a two-digit year: 57 to 99 are 1957 to 1999, and 00 to 56 are 2000 to 2056.
fn expand_year(two_digits: u64) -> i64 {
    let two_digits = two_digits as i64;
    if two_digits < 57 {
        2000 + two_digits
    } else {
        1900 + two_digits
    }
}

/// Parses a non-blank international designator into e.g. `1998-067A`.
fn parse_international_designator(field: &[u8]) -> Option<String> {
    let year = expand_year(parse_integer(field.get(..2)?)?);
    let number = parse_integer(field.get(2..5)?)?;
    let piece = field.get(5..)?;
    let end = piece.iter().rposition(|&b| b != b' ')? + 1;
    let piece = &piece[..end];
    if !piece.iter().all(u8::is_ascii_uppercase) {
        return None;
    }
    let piece = std::str::from_utf8(piece).ok()?;
    Some(format!("{year:04}-{number:03}{piece}"))
}

/// Parses the epoch from its two-digit year and its day of year, where day 1.0 is the start of
/// January 1.
///
/// The fraction is converted in integer arithmetic. A day has 864 * 10^11 nanoseconds and the
/// field has room for at most ten fractional digits, so the conversion is exact, which also makes
/// it the nearest whole nanosecond.
fn parse_epoch(year: &[u8], day: &[u8]) -> Option<Utc> {
    let year = expand_year(parse_integer(year)?);
    let day = strip_padding(day);
    let (whole, fraction) = match day.iter().position(|&b| b == b'.') {
        Some(dot) => (&day[..dot], &day[dot + 1..]),
        None => (day, &day[day.len()..]),
    };
    let day_of_year: u16 = digits(whole)?.parse().ok()?;
    // `Date::from_day_of_year` does not reject every out-of-range day by itself.
    if !(1..=366).contains(&day_of_year) {
        return None;
    }
    let nanoseconds = if fraction.is_empty() {
        0
    } else {
        let scale = 10u64.checked_pow(fraction.len() as u32)?;
        let value: u64 = digits(fraction)?.parse().ok()?;
        value.checked_mul(NANOSECONDS_PER_DAY / scale)?
    };

    let seconds = nanoseconds / NANOSECONDS_PER_SECOND;
    let attoseconds = (nanoseconds % NANOSECONDS_PER_SECOND) as i64 * ATTOSECONDS_PER_NANOSECOND;
    let time = TimeOfDay::new(
        (seconds / 3600) as u8,
        (seconds / 60 % 60) as u8,
        (seconds % 60) as u8,
    )
    .ok()?
    .with_subsecond(Subsecond::from_attoseconds(attoseconds));
    let date = Date::from_day_of_year(year, day_of_year).ok()?;
    Utc::new(date, time, &DefaultLeapSecondsProvider).ok()
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use rstest::rstest;

    use super::*;

    const LINE_1: &str = "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996";
    const LINE_2: &str = "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731";

    /// Replaces the bytes at `column` and recomputes the checksum.
    fn patched(line: &str, column: usize, replacement: &str) -> Vec<u8> {
        let mut bytes = line.as_bytes().to_vec();
        bytes[column..column + replacement.len()].copy_from_slice(replacement.as_bytes());
        let sum: u32 = bytes[..68]
            .iter()
            .map(|b| match b {
                b'-' => 1,
                b'0'..=b'9' => (b - b'0') as u32,
                _ => 0,
            })
            .sum();
        bytes[68] = b'0' + (sum % 10) as u8;
        bytes
    }

    #[test]
    fn test_from_lines() {
        let tle = Tle::from_lines(Some("ISS (ZARYA)".to_string()), LINE_1, LINE_2).unwrap();
        assert_eq!(tle.object_name.as_deref(), Some("ISS (ZARYA)"));
        assert_eq!(tle.international_designator.as_deref(), Some("1998-067A"));
        assert_eq!(tle.norad_id, 25544);
        assert_eq!(tle.classification, Classification::Unclassified);
        assert_eq!(tle.epoch.to_string(), "2024-06-18T09:00:24.494 UTC");
        assert_approx_eq!(tle.inclination.to_degrees(), 51.6410, rtol <= 1e-12);
        assert_approx_eq!(tle.right_ascension.to_degrees(), 309.3890, rtol <= 1e-12);
        assert_eq!(tle.eccentricity.as_f64(), 0.0010444);
        assert_approx_eq!(
            tle.argument_of_perigee.to_degrees(),
            339.5369,
            rtol <= 1e-12
        );
        assert_approx_eq!(tle.mean_anomaly.to_degrees(), 107.8830, rtol <= 1e-12);
        assert_approx_eq!(
            tle.mean_motion.to_radians_per_second(),
            15.49495945 * std::f64::consts::TAU / 86400.0,
            rtol <= 1e-12
        );
        assert_eq!(tle.half_mean_motion_dot_rev_per_day2, 0.00016566);
        assert_eq!(tle.sixth_mean_motion_ddot_rev_per_day3, 0.0);
        assert_approx_eq!(tle.drag_term_per_earth_radius, 0.30244e-3, rtol <= 1e-12);
        assert_eq!(tle.element_set_number, 999);
        assert_eq!(tle.revolution_number, 45873);
        assert_eq!(tle.ephemeris_type, 0);
    }

    #[test]
    fn test_from_lines_accepts_bytes() {
        let tle = Tle::from_lines(None, LINE_1.as_bytes(), LINE_2.as_bytes()).unwrap();
        assert_eq!(tle.object_name, None);
    }

    #[test]
    fn test_blank_international_designator() {
        let line_1 = patched(LINE_1, 9, "        ");
        let tle = Tle::from_lines(None, line_1, LINE_2).unwrap();
        assert_eq!(tle.international_designator, None);
    }

    #[rstest]
    #[case("A5544", 105544)]
    #[case("Z9999", 339999)]
    #[case(" 5544", 5544)]
    fn test_alpha5_norad_id(#[case] id: &str, #[case] expected: u64) {
        let line_1 = patched(LINE_1, 2, id);
        let line_2 = patched(LINE_2, 2, id);
        assert_eq!(
            Tle::from_lines(None, line_1, line_2).unwrap().norad_id,
            expected
        );
    }

    #[test]
    fn test_bad_length() {
        let err = Tle::from_lines(None, &LINE_1[..60], LINE_2).unwrap_err();
        assert_eq!(
            err,
            TleError::BadLength {
                line: TleLine::One,
                found: 60
            }
        );
        assert!(err.to_string().contains("TLE parse error"));
    }

    #[test]
    fn test_bad_line_number() {
        let line_2 = patched(LINE_2, 0, "3");
        let err = Tle::from_lines(None, LINE_1, line_2).unwrap_err();
        assert_eq!(err, TleError::BadLineNumber { line: TleLine::Two });
    }

    #[test]
    fn test_expected_space() {
        let line_1 = patched(LINE_1, 17, "X");
        let err = Tle::from_lines(None, line_1, LINE_2).unwrap_err();
        assert_eq!(
            err,
            TleError::ExpectedSpace {
                line: TleLine::One,
                column: 17
            }
        );
    }

    #[rstest]
    #[case("7")]
    #[case("X")]
    fn test_bad_checksum(#[case] checksum: &str) {
        let line_1 = format!("{}{checksum}", &LINE_1[..68]);
        let err = Tle::from_lines(None, line_1, LINE_2).unwrap_err();
        assert_eq!(err, TleError::BadChecksum { line: TleLine::One });
    }

    #[test]
    fn test_norad_id_mismatch() {
        let line_2 = patched(LINE_2, 2, "25545");
        let err = Tle::from_lines(None, LINE_1, line_2).unwrap_err();
        assert_eq!(err, TleError::NoradIdMismatch);
    }

    #[rstest]
    #[case::norad_id(2, "  A55", "norad_id")]
    #[case::classification(7, "X", "classification")]
    #[case::launch_year(9, "XX", "international_designator")]
    #[case::epoch_day_zero(18, "24000", "epoch")]
    #[case::epoch_day_367(18, "24367", "epoch")]
    #[case::epoch_day_366_non_leap(18, "23366", "epoch")]
    #[case::epoch_year(18, "XX", "epoch")]
    #[case::mean_motion_dot(33, "       NaN", "mean_motion_dot")]
    #[case::mean_motion_ddot(44, "abcdef", "mean_motion_ddot")]
    #[case::drag_term(53, "1.2345", "drag_term")]
    #[case::ephemeris_type(62, "X", "ephemeris_type")]
    #[case::element_set_number(64, "XXXX", "element_set_number")]
    fn test_invalid_field_line_1(
        #[case] column: usize,
        #[case] replacement: &str,
        #[case] field: &'static str,
    ) {
        let line_1 = patched(LINE_1, column, replacement);
        let line_2 = patched(LINE_2, 2, &String::from_utf8_lossy(&line_1[2..7]));
        let err = Tle::from_lines(None, line_1, line_2).unwrap_err();
        assert_eq!(
            err,
            TleError::InvalidField {
                line: TleLine::One,
                field
            }
        );
    }

    #[rstest]
    #[case::inclination(8, "     NaN", "inclination")]
    #[case::right_ascension(17, "     inf", "right_ascension")]
    #[case::eccentricity(26, "XXXXXXX", "eccentricity")]
    #[case::negative_eccentricity(26, "-001044", "eccentricity")]
    #[case::argument_of_perigee(34, "XXXXXXXX", "argument_of_perigee")]
    #[case::mean_anomaly(43, "XXXXXXXX", "mean_anomaly")]
    #[case::mean_motion(52, "XXXXXXXXXXX", "mean_motion")]
    #[case::revolution_number(63, "XXXXX", "revolution_number")]
    fn test_invalid_field_line_2(
        #[case] column: usize,
        #[case] replacement: &str,
        #[case] field: &'static str,
    ) {
        let line_2 = patched(LINE_2, column, replacement);
        let err = Tle::from_lines(None, LINE_1, line_2).unwrap_err();
        assert_eq!(
            err,
            TleError::InvalidField {
                line: TleLine::Two,
                field
            }
        );
    }

    #[test]
    fn test_non_utf8_field() {
        let mut line_1 = LINE_1.as_bytes().to_vec();
        line_1[14] = 0xFF;
        let err = Tle::from_lines(None, line_1, LINE_2).unwrap_err();
        assert_eq!(
            err,
            TleError::InvalidField {
                line: TleLine::One,
                field: "international_designator"
            }
        );
    }
}
