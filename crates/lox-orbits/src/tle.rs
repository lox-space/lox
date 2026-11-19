// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Two-Line Element sets.

use std::f64::consts::TAU;
use std::fmt;
use std::str::FromStr;

use lox_core::time::constants::SECONDS_PER_DAY;
use lox_core::units::{Angle, AngularRate};
use lox_time::calendar_dates::Date;
use lox_time::subsecond::Subsecond;
use lox_time::time_of_day::TimeOfDay;
use lox_time::utc::Utc;
use lox_time::utc::leap_seconds::DefaultLeapSecondsProvider;
use thiserror::Error;

const LINE_LENGTH: usize = 69;
const LINE_1_SPACES: [usize; 8] = [1, 8, 17, 32, 43, 52, 61, 63];
const LINE_2_SPACES: [usize; 7] = [1, 7, 16, 25, 33, 42, 51];
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
    pub eccentricity: f64,
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
        let l1 = line_1.as_ref();
        let l2 = line_2.as_ref();
        check_layout(l1, TleLine::One, b'1', &LINE_1_SPACES)?;
        check_layout(l2, TleLine::Two, b'2', &LINE_2_SPACES)?;

        let norad_id = parse_norad_id(&l1[2..7], TleLine::One)?;
        if norad_id != parse_norad_id(&l2[2..7], TleLine::Two)? {
            return Err(TleError::NoradIdMismatch);
        }
        check_checksum(l1, TleLine::One)?;
        check_checksum(l2, TleLine::Two)?;

        use TleLine::{One, Two};
        Ok(Self {
            object_name,
            international_designator: parse_designator(&l1[9..17])?,
            norad_id,
            classification: parse_classification(l1[7])?,
            epoch: parse_epoch(&l1[18..20], &l1[20..32])?,
            half_mean_motion_dot_rev_per_day2: float(&l1[33..43], One, "mean_motion_dot")?,
            sixth_mean_motion_ddot_rev_per_day3: exponential(
                &l1[44..50],
                &l1[50..52],
                "mean_motion_ddot",
            )?,
            drag_term_per_earth_radius: exponential(&l1[53..59], &l1[59..61], "drag_term")?,
            ephemeris_type: integer(&l1[62..63], One, "ephemeris_type")?,
            element_set_number: integer(&l1[64..68], One, "element_set_number")?,
            inclination: Angle::degrees(float(&l2[8..16], Two, "inclination")?),
            right_ascension: Angle::degrees(float(&l2[17..25], Two, "right_ascension")?),
            eccentricity: assumed_decimal(&l2[26..33], Two, "eccentricity")?,
            argument_of_perigee: Angle::degrees(float(&l2[34..42], Two, "argument_of_perigee")?),
            mean_anomaly: Angle::degrees(float(&l2[43..51], Two, "mean_anomaly")?),
            mean_motion: AngularRate::radians_per_second(
                float(&l2[52..63], Two, "mean_motion")? * TAU / SECONDS_PER_DAY as f64,
            ),
            revolution_number: integer(&l2[63..68], Two, "revolution_number")?,
        })
    }
}

fn check_layout(
    raw: &[u8],
    line: TleLine,
    line_number: u8,
    spaces: &[usize],
) -> Result<(), TleError> {
    if raw.len() != LINE_LENGTH {
        return Err(TleError::BadLength {
            line,
            found: raw.len(),
        });
    }
    if raw[0] != line_number {
        return Err(TleError::BadLineNumber { line });
    }
    match spaces.iter().find(|&&column| raw[column] != b' ') {
        Some(&column) => Err(TleError::ExpectedSpace { line, column }),
        None => Ok(()),
    }
}

fn check_checksum(raw: &[u8], line: TleLine) -> Result<(), TleError> {
    let sum: u32 = raw[..LINE_LENGTH - 1]
        .iter()
        .map(|byte| match byte {
            b'-' => 1,
            b'0'..=b'9' => (byte - b'0') as u32,
            _ => 0,
        })
        .sum();
    let checksum = raw[LINE_LENGTH - 1];
    if checksum.is_ascii_digit() && sum % 10 == (checksum - b'0') as u32 {
        Ok(())
    } else {
        Err(TleError::BadChecksum { line })
    }
}

fn text<'a>(raw: &'a [u8], line: TleLine, field: &'static str) -> Result<&'a str, TleError> {
    std::str::from_utf8(raw)
        .map(str::trim_ascii_start)
        .map_err(|_| TleError::InvalidField { line, field })
}

fn integer<T: FromStr>(raw: &[u8], line: TleLine, field: &'static str) -> Result<T, TleError> {
    text(raw, line, field)?
        .parse()
        .map_err(|_| TleError::InvalidField { line, field })
}

fn finite(value: Option<f64>, line: TleLine, field: &'static str) -> Result<f64, TleError> {
    value
        .filter(|value| value.is_finite())
        .ok_or(TleError::InvalidField { line, field })
}

fn float(raw: &[u8], line: TleLine, field: &'static str) -> Result<f64, TleError> {
    finite(text(raw, line, field)?.parse().ok(), line, field)
}

/// Parses a fraction whose leading decimal point is implied, e.g. `0010444`.
fn assumed_decimal(raw: &[u8], line: TleLine, field: &'static str) -> Result<f64, TleError> {
    let trimmed = text(raw, line, field)?;
    let (sign, digits) = match trimmed.strip_prefix('-') {
        Some(digits) => ("-", digits),
        None => ("", trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(TleError::InvalidField { line, field });
    }
    finite(format!("{sign}.{digits}").parse().ok(), line, field)
}

/// Parses a line 1 field in the TLE exponential notation, e.g. `30244-3`.
fn exponential(mantissa: &[u8], exponent: &[u8], field: &'static str) -> Result<f64, TleError> {
    let line = TleLine::One;
    let exponent: i8 = integer(exponent, line, field)?;
    Ok(assumed_decimal(mantissa, line, field)? * 10.0_f64.powi(exponent as i32))
}

/// Parses a catalog number in the legacy or the Alpha-5 format.
fn parse_norad_id(raw: &[u8], line: TleLine) -> Result<u64, TleError> {
    let field = "norad_id";
    let invalid = TleError::InvalidField { line, field };
    let trimmed = text(raw, line, field)?;
    if trimmed.len() != 5 {
        return integer(trimmed.as_bytes(), line, field);
    }
    let leading = match trimmed.as_bytes()[0] {
        digit @ b'0'..=b'9' => (digit - b'0') as u64,
        letter @ b'A'..=b'H' => (letter - b'A' + 10) as u64,
        letter @ b'J'..=b'N' => (letter - b'J' + 18) as u64,
        letter @ b'P'..=b'Z' => (letter - b'P' + 23) as u64,
        _ => return Err(invalid),
    };
    let rest: u64 = trimmed[1..].parse().map_err(|_| invalid)?;
    Ok(leading * 10_000 + rest)
}

fn parse_classification(byte: u8) -> Result<Classification, TleError> {
    match byte {
        b'U' => Ok(Classification::Unclassified),
        b'C' => Ok(Classification::Classified),
        b'S' => Ok(Classification::Secret),
        _ => Err(TleError::InvalidField {
            line: TleLine::One,
            field: "classification",
        }),
    }
}

/// Expands a two-digit year, using the NORAD convention that 57–99 are 19xx.
fn full_year(two_digits: u8) -> i64 {
    match two_digits {
        year if year < 57 => 2000 + year as i64,
        year => 1900 + year as i64,
    }
}

fn parse_designator(raw: &[u8]) -> Result<Option<String>, TleError> {
    let line = TleLine::One;
    let field = "international_designator";
    if raw.iter().all(u8::is_ascii_whitespace) {
        return Ok(None);
    }
    let year = full_year(integer(&raw[..2], line, field)?);
    let piece = text(&raw[2..], line, field)?.trim_ascii_end();
    Ok(Some(format!("{year}-{piece}")))
}

fn parse_epoch(year: &[u8], day: &[u8]) -> Result<Utc, TleError> {
    let line = TleLine::One;
    let field = "epoch";
    let invalid = TleError::InvalidField { line, field };

    let year = full_year(integer(year, line, field)?);
    let day = float(day, line, field)?;
    if !(1.0..367.0).contains(&day) {
        return Err(invalid);
    }
    // Rounds the day fraction to whole nanoseconds, the resolution SGP4 works in.
    let seconds = day.fract() * SECONDS_PER_DAY as f64;
    let mut nanos = (seconds.fract() * 1e9).round() as i64;
    let mut seconds = seconds as u32;
    if nanos >= 1_000_000_000 {
        nanos -= 1_000_000_000;
        seconds += 1;
    }

    let date = Date::from_day_of_year(year, day as u16).map_err(|_| invalid.clone())?;
    let time = TimeOfDay::new(
        (seconds / 3600) as u8,
        (seconds / 60 % 60) as u8,
        (seconds % 60) as u8,
    )
    .map_err(|_| invalid.clone())?
    .with_subsecond(Subsecond::from_attoseconds(
        nanos * ATTOSECONDS_PER_NANOSECOND,
    ));
    Utc::new(date, time, &DefaultLeapSecondsProvider).map_err(|_| invalid)
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
        assert_eq!(tle.eccentricity, 0.0010444);
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
