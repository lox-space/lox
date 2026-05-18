// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Nom parsers for ISO 8601 extended-format date and time strings.
//!
//! Currently supports:
//! - `[-]YYYY-MM-DD` for dates. The year is at least four digits and may
//!   carry a leading `-`; it is parsed as `i64` to match [`Date`]'s range.
//! - `HH:MM:SS[.fff…]` for times of day. The subsecond fraction has
//!   unbounded digit width and is returned as a raw `&str` so callers can
//!   feed it directly to [`Subsecond::from_str`].
//!
//! Both parsers are partial: they consume their grammar and stop. Callers
//! wrap them in [`nom::combinator::all_consuming`] to forbid trailing input.
//!
//! [`Date`]: crate::time::calendar_dates::Date
//! [`Subsecond`]: crate::time::subsecond::Subsecond

use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_while_m_n},
    character::complete::digit1,
    combinator::{map_res, opt, recognize, verify},
    sequence::preceded,
};

/// Parse the date portion of an ISO 8601 extended-format string.
///
/// Returns `(year, month, day)` without semantic validation.
pub(super) fn date(input: &str) -> IResult<&str, (i64, u8, u8)> {
    let (input, year) = year(input)?;
    let (input, _) = tag("-").parse(input)?;
    let (input, month) = two_digits(input)?;
    let (input, _) = tag("-").parse(input)?;
    let (input, day) = two_digits(input)?;
    Ok((input, (year, month, day)))
}

/// Parse the time portion of an ISO 8601 extended-format string.
///
/// Returns `(hour, minute, second, fraction)` where `fraction` is the digit
/// slice *after* the optional decimal point, without semantic validation.
pub(super) fn time(input: &str) -> IResult<&str, (u8, u8, u8, Option<&str>)> {
    let (input, hour) = two_digits(input)?;
    let (input, _) = tag(":").parse(input)?;
    let (input, minute) = two_digits(input)?;
    let (input, _) = tag(":").parse(input)?;
    let (input, second) = two_digits(input)?;
    let (input, fraction) = opt(preceded(tag("."), digit1)).parse(input)?;
    Ok((input, (hour, minute, second, fraction)))
}

fn year(input: &str) -> IResult<&str, i64> {
    map_res(
        recognize((opt(tag("-")), verify(digit1, |s: &str| s.len() >= 4))),
        str::parse::<i64>,
    )
    .parse(input)
}

fn two_digits(input: &str) -> IResult<&str, u8> {
    map_res(
        take_while_m_n(2, 2, |c: char| c.is_ascii_digit()),
        str::parse::<u8>,
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::combinator::all_consuming;
    use rstest::rstest;

    #[rstest]
    #[case("2000-01-01", (2000, 1, 1))]
    #[case("0000-01-01", (0, 1, 1))]
    #[case("-0001-12-31", (-1, 12, 31))]
    #[case("12345-06-07", (12345, 6, 7))]
    #[case("-12345-06-07", (-12345, 6, 7))]
    fn date_roundtrip(#[case] input: &str, #[case] expected: (i64, u8, u8)) {
        let (rest, got) = all_consuming(date).parse(input).unwrap();
        assert!(rest.is_empty());
        assert_eq!(got, expected);
    }

    #[rstest]
    #[case("200-01-01")] // year too short
    #[case("2000-1-01")] // single-digit month
    #[case("2000-01-1")] // single-digit day
    #[case("2000/01/01")] // wrong separator
    #[case(" 2000-01-01")] // leading whitespace
    #[case("2000-01-01extra")] // trailing junk
    #[case("prefix 2000-01-01")] // leading junk
    fn date_rejects(#[case] input: &str) {
        assert!(all_consuming(date).parse(input).is_err());
    }

    #[rstest]
    #[case("12:13:14", (12, 13, 14, None))]
    #[case("00:00:00", (0, 0, 0, None))]
    #[case("23:59:60", (23, 59, 60, None))] // leap second, validated downstream
    #[case("12:13:14.123", (12, 13, 14, Some("123")))]
    #[case("12:13:14.123456789", (12, 13, 14, Some("123456789")))]
    fn time_roundtrip(#[case] input: &str, #[case] expected: (u8, u8, u8, Option<&str>)) {
        let (rest, got) = all_consuming(time).parse(input).unwrap();
        assert!(rest.is_empty());
        assert_eq!(got, expected);
    }

    #[rstest]
    #[case("2:13:14")] // single-digit hour
    #[case("12:3:14")] // single-digit minute
    #[case("12:13:4")] // single-digit second
    #[case("12:13:14.")] // trailing dot without digits
    #[case("12-13-14")] // wrong separator
    #[case("12:13:14 suffix")] // trailing junk
    fn time_rejects(#[case] input: &str) {
        assert!(all_consuming(time).parse(input).is_err());
    }
}
