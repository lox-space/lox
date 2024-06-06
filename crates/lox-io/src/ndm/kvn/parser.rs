/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

// KVN spec section 7.4 of https://public.ccsds.org/Pubs/502x0b3e1.pdf

use nom::bytes::complete as nb;
use nom::character::complete as nc;
use nom::error::{ErrorKind, ParseError};
use nom::sequence as ns;
use regex::Regex;

use super::deserializer::KvnDeserializerErr;

#[derive(Debug, PartialEq)]
pub enum KvnStringParserErr<I> {
    EmptyKeyword { input: I },
    EmptyValue { input: I },
    InvalidFormat { input: I },
    ParserError(I, ErrorKind),
}

#[derive(Debug, PartialEq)]
pub enum KvnKeyMatchErr<I> {
    KeywordNotFound { expected: I },
}

#[derive(PartialEq, Debug)]
pub enum KvnNumberParserErr<I> {
    ParserError(I, ErrorKind),
    EmptyKeyword { input: I },
    EmptyValue { input: I },
    InvalidFormat { input: I },
}

#[derive(PartialEq, Debug)]
pub enum KvnDateTimeParserErr<I> {
    ParserError(I, ErrorKind),
    EmptyKeyword { input: I },
    EmptyValue { input: I },
    InvalidFormat { input: I },
}

impl<I> From<nom::Err<KvnStringParserErr<I>>> for KvnDeserializerErr<I> {
    fn from(value: nom::Err<KvnStringParserErr<I>>) -> Self {
        match value {
            nom::Err::Error(KvnStringParserErr::EmptyValue { input })
            | nom::Err::Failure(KvnStringParserErr::EmptyValue { input }) => {
                KvnDeserializerErr::EmptyValue { input }
            }
            nom::Err::Error(KvnStringParserErr::EmptyKeyword { input })
            | nom::Err::Failure(KvnStringParserErr::EmptyKeyword { input }) => {
                KvnDeserializerErr::EmptyKeyword { input }
            }
            nom::Err::Error(KvnStringParserErr::InvalidFormat { input })
            | nom::Err::Failure(KvnStringParserErr::InvalidFormat { input }) => {
                KvnDeserializerErr::InvalidStringFormat { input }
            }
            nom::Err::Error(KvnStringParserErr::ParserError(i, k))
            | nom::Err::Failure(KvnStringParserErr::ParserError(i, k)) => {
                KvnDeserializerErr::GeneralParserError(i, k)
            }
            // We don't use streaming deserialization
            nom::Err::Incomplete(_) => unimplemented!(),
        }
    }
}

impl<I> From<nom::Err<KvnDateTimeParserErr<I>>> for KvnDeserializerErr<I> {
    fn from(value: nom::Err<KvnDateTimeParserErr<I>>) -> Self {
        match value {
            nom::Err::Error(KvnDateTimeParserErr::EmptyValue { input })
            | nom::Err::Failure(KvnDateTimeParserErr::EmptyValue { input }) => {
                KvnDeserializerErr::EmptyValue { input }
            }
            nom::Err::Error(KvnDateTimeParserErr::EmptyKeyword { input })
            | nom::Err::Failure(KvnDateTimeParserErr::EmptyKeyword { input }) => {
                KvnDeserializerErr::EmptyKeyword { input }
            }
            nom::Err::Error(KvnDateTimeParserErr::InvalidFormat { input })
            | nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat { input }) => {
                KvnDeserializerErr::InvalidDateTimeFormat { input }
            }
            nom::Err::Error(KvnDateTimeParserErr::ParserError(i, k))
            | nom::Err::Failure(KvnDateTimeParserErr::ParserError(i, k)) => {
                KvnDeserializerErr::GeneralParserError(i, k)
            }
            // We don't use streaming deserialization
            nom::Err::Incomplete(_) => unimplemented!(),
        }
    }
}

impl<I> From<nom::Err<KvnNumberParserErr<I>>> for KvnDeserializerErr<I> {
    fn from(value: nom::Err<KvnNumberParserErr<I>>) -> Self {
        match value {
            nom::Err::Error(KvnNumberParserErr::EmptyValue { input })
            | nom::Err::Failure(KvnNumberParserErr::EmptyValue { input }) => {
                KvnDeserializerErr::EmptyValue { input }
            }
            nom::Err::Error(KvnNumberParserErr::EmptyKeyword { input })
            | nom::Err::Failure(KvnNumberParserErr::EmptyKeyword { input }) => {
                KvnDeserializerErr::EmptyKeyword { input }
            }
            nom::Err::Error(KvnNumberParserErr::InvalidFormat { input })
            | nom::Err::Failure(KvnNumberParserErr::InvalidFormat { input }) => {
                KvnDeserializerErr::InvalidDateTimeFormat { input }
            }
            nom::Err::Error(KvnNumberParserErr::ParserError(i, k))
            | nom::Err::Failure(KvnNumberParserErr::ParserError(i, k)) => {
                KvnDeserializerErr::GeneralParserError(i, k)
            }
            // We don't use streaming deserialization
            nom::Err::Incomplete(_) => unimplemented!(),
        }
    }
}

impl<I> From<KvnKeyMatchErr<I>> for KvnDeserializerErr<I> {
    fn from(value: KvnKeyMatchErr<I>) -> Self {
        match value {
            KvnKeyMatchErr::KeywordNotFound { expected } => {
                KvnDeserializerErr::KeywordNotFound { expected }
            }
        }
    }
}

impl<I> ParseError<I> for KvnStringParserErr<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        KvnStringParserErr::ParserError(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

#[derive(PartialEq, Debug, Default)]
pub struct KvnValue<V, U> {
    pub value: V,
    pub unit: Option<U>,
}

#[derive(PartialEq, Debug, Default)]
pub struct KvnDateTimeValue {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub fractional_second: f64,
    pub full_value: String,
}

//@TODO remove _new suffix
pub fn kvn_line_matches_key_new<'a>(
    key: &'a str,
    input: &'a str,
) -> Result<bool, KvnKeyMatchErr<&'a str>> {
    if key == "COMMENT" {
        ns::tuple((nc::space0::<_, nom::error::Error<_>>, nb::tag(key)))(input)
            .map(|_| true)
            .or(Ok(false))
    } else {
        let mut equals = ns::tuple((nc::space0::<_, nom::error::Error<_>>, nc::char('=')));

        if equals(input).is_ok() {
            return Err(KvnKeyMatchErr::KeywordNotFound { expected: key });
        }

        ns::delimited(nc::space0, nb::tag(key), equals)(input)
            .map(|_| true)
            .or(Ok(false))
    }
}

pub fn parse_kvn_string_line_new(
    input: &str,
) -> nom::IResult<&str, KvnValue<String, String>, KvnStringParserErr<&str>> {
    if input.trim_start().starts_with("COMMENT") {
        return Ok((
            "",
            KvnValue {
                value: input
                    .trim_start()
                    .trim_start_matches("COMMENT")
                    .trim_start()
                    .to_owned(),
                unit: None,
            },
        ));
    }

    if is_empty_value(input) {
        Err(nom::Err::Failure(KvnStringParserErr::EmptyValue { input }))?
    };

    // Inspired by figure F-8: CCSDS 502.0-B-3, but accepts a more relaxed input. Orekit seems to suggest that there
    // are quite a few messages being used which are not strictly compliant.
    let re =
        Regex::new(r"^(?:\s*)(?<keyword>[0-9A-Z_]*)(?:\s*)=(?:\s*)(?<value>(?:(?:.*)))(?:\s*)$")
            .unwrap();

    let captures =
        re.captures(input)
            .ok_or(nom::Err::Failure(KvnStringParserErr::InvalidFormat {
                input,
            }))?;

    let keyword = captures
        .name("keyword")
        // This unwrap is okay because the keyword uses * so it will always capture
        .unwrap()
        .as_str()
        .trim_end()
        .to_owned();

    if keyword.is_empty() {
        return Err(nom::Err::Failure(KvnStringParserErr::EmptyKeyword {
            input,
        }));
    }

    let value = captures
        .name("value")
        // This unwrap is okay because the value uses * so it will always capture
        .unwrap()
        .as_str()
        .trim_end()
        .to_owned();

    if value.is_empty() {
        return Err(nom::Err::Failure(KvnStringParserErr::EmptyValue { input }));
    }

    Ok(("", KvnValue { value, unit: None }))
}

pub fn parse_kvn_integer_line_new<T>(
    input: &str,
    with_unit: bool,
) -> nom::IResult<&str, KvnValue<T, String>, KvnNumberParserErr<&str>>
where
    T: std::str::FromStr,
{
    if is_empty_value(input) {
        Err(nom::Err::Failure(KvnNumberParserErr::EmptyValue { input }))?
    };

    let regex_pattern = if with_unit {
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?<value>(?:[-+]?)(?:[0-9]+)(?:\.\d*)?)(?:(?:\s*)(?:\[(?<unit>[0-9A-Za-z/_*]*)\]?))?(?:\s*)?$"
    } else {
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?<value>(?:[-+]?)(?:[0-9]+)(?:\.\d*)?)(?:\s*)$"
    };

    // Modified from Figure F-9: CCSDS 502.0-B-3
    let re = Regex::new(regex_pattern).unwrap();

    let captures =
        re.captures(input)
            .ok_or(nom::Err::Failure(KvnNumberParserErr::InvalidFormat {
                input,
            }))?;

    let keyword = captures
        .name("keyword")
        // This unwrap is okay because the keyword is marked as * so it will always capture
        .unwrap()
        .as_str()
        .trim_end()
        .to_owned();

    if keyword.is_empty() {
        return Err(nom::Err::Failure(KvnNumberParserErr::EmptyKeyword {
            input,
        }));
    }

    // This unwrap is okay because the value uses * so it will always capture
    let value = captures.name("value").unwrap().as_str();
    let unit = captures.name("unit").map(|x| x.as_str().to_owned());

    let value = value
        .parse::<T>()
        .map_err(|_| nom::Err::Failure(KvnNumberParserErr::InvalidFormat { input }))?;

    Ok(("", KvnValue { value, unit }))
}

fn is_empty_value(input: &str) -> bool {
    let re = Regex::new(
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?:\[(?<unit>[0-9A-Za-z/_*]*)\]?)?$",
    )
    .unwrap();

    re.is_match(input)
}

pub fn parse_kvn_numeric_line_new(
    input: &str,
    with_unit: bool,
) -> nom::IResult<&str, KvnValue<f64, String>, KvnNumberParserErr<&str>> {
    if is_empty_value(input) {
        Err(nom::Err::Failure(KvnNumberParserErr::EmptyValue { input }))?
    };

    let regex_pattern = if with_unit {
        // Figure F-9: CCSDS 502.0-B-3
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?<value>(?:[-+]?)(?:[0-9]+)(?:\.\d*)?(?:[eE][+-]?(?:\d+))?)(?:(?:\s*)(?:\[(?<unit>[0-9A-Za-z/_*]*)\]?))?(?:\s*)?$"
    } else {
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?<value>(?:[-+]?)(?:[0-9]+)(?:\.\d*)?(?:[eE][+-]?(?:\d+))?)(?:\s*)?$"
    };

    let re = Regex::new(regex_pattern).unwrap();

    let captures =
        re.captures(input)
            .ok_or(nom::Err::Failure(KvnNumberParserErr::InvalidFormat {
                input,
            }))?;

    let keyword = captures
        .name("keyword")
        // This unwrap is okay because the keyword is marked as * so it will always capture
        .unwrap()
        .as_str()
        .trim_end()
        .to_owned();

    if keyword.is_empty() {
        return Err(nom::Err::Failure(KvnNumberParserErr::EmptyKeyword {
            input,
        }));
    }

    // This unwrap is okay because the value uses * so it will always capture
    let value = captures.name("value").unwrap().as_str();
    let unit = captures.name("unit").map(|x| x.as_str().to_owned());

    let value = fast_float::parse(value)
        .map_err(|_| nom::Err::Failure(KvnNumberParserErr::InvalidFormat { input }))?;

    Ok(("", KvnValue { value, unit }))
}

pub fn parse_kvn_datetime_line_new(
    input: &str,
) -> nom::IResult<&str, KvnDateTimeValue, KvnDateTimeParserErr<&str>> {
    if is_empty_value(input) {
        Err(nom::Err::Failure(KvnDateTimeParserErr::EmptyValue {
            input,
        }))?
    };

    // Modified from Figure F-5: CCSDS 502.0-B-3
    let re = Regex::new(r"^(?:\s*)?(?<keyword>[0-9A-Z_]*)(?:\s*)?=(?:\s*)?(?<value>(?<yr>(?:\d{4}))-(?<mo>(?:\d{1,2}))-(?<dy>(?:\d{1,2}))T(?<hr>(?:\d{1,2})):(?<mn>(?:\d{1,2})):(?<sc>(?:\d{0,2}(?:\.\d*)?)))(?:\s*)?$").unwrap();

    let captures =
        re.captures(input)
            .ok_or(nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat {
                input,
            }))?;

    let keyword = captures
        // This unwrap is okay because the keyword is marked as * so it will always capture
        .name("keyword")
        .unwrap()
        .as_str()
        .trim_end()
        .to_owned();

    if keyword.is_empty() {
        return Err(nom::Err::Failure(KvnDateTimeParserErr::EmptyKeyword {
            input,
        }));
    }

    // yr is a mandatory decimal in the regex so we expect the capture to be
    // always there and unwrap is fine
    let year = captures
        .name("yr")
        .unwrap()
        .as_str()
        .parse::<u16>()
        .unwrap();

    // We don't do full validation of the date values. We only care if they
    // have the expected number of digits

    // mo is a mandatory decimal in the regex so we expect the capture to be
    // always there and unwrap is fine
    let month = captures.name("mo").unwrap().as_str().parse::<u8>().unwrap();

    // day is a mandatory decimal in the regex so we expect the capture to be
    // always there and unwrap is fine
    let day = captures.name("dy").unwrap().as_str().parse::<u8>().unwrap();

    // hr is a mandatory decimal in the regex so we expect the capture to be
    // always there and unwrap is fine
    let hour = captures.name("hr").unwrap().as_str().parse::<u8>().unwrap();

    // mn is a mandatory decimal in the regex so we expect the capture to be
    // always there and unwrap is fine
    let minute = captures.name("mn").unwrap().as_str().parse::<u8>().unwrap();

    // sc is a mandatory decimal in the regex so we expect the capture to be
    // always there and unwrap is fine
    let full_second = captures
        .name("sc")
        .unwrap()
        .as_str()
        .parse::<f64>()
        .unwrap();

    let second = full_second.floor() as u8;

    let fractional_second = full_second.fract();

    let full_value = captures.name("value").unwrap().as_str().to_owned();

    Ok((
        "",
        KvnDateTimeValue {
            year,
            month,
            day,
            hour,
            minute,
            second,
            fractional_second,
            full_value,
        },
    ))
}

#[cfg(test)]
mod test {
    use lox_derive::KvnDeserialize;

    use super::*;

    #[test]
    fn test_kvn_line_matches_key_new() {
        assert_eq!(
            kvn_line_matches_key_new("ASD", "AAAAAD123 = ASDFG"),
            Ok(false)
        );
        assert_eq!(kvn_line_matches_key_new("ASD", "ASD = ASDFG"), Ok(true));

        assert_eq!(
            kvn_line_matches_key_new("ASD", "ASD ASD = ASDFG"),
            Ok(false)
        );
        assert_eq!(
            kvn_line_matches_key_new("ASD", "= ASDFG"),
            Err(KvnKeyMatchErr::KeywordNotFound { expected: "ASD" })
        );
    }

    #[test]
    fn test_parse_kvn_string_line_new() {
        // 7.5.1 A non-empty value field must be assigned to each mandatory keyword except for *‘_START’ and *‘_STOP’ keyword values
        // 7.4.6 Any white space immediately preceding or following the ‘equals’ sign shall not be significant.
        assert_eq!(
            parse_kvn_string_line_new("ASD = ASDFG"),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_string_line_new("ASD    =   ASDFG"),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_string_line_new("ASD    = ASDFG"),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_string_line_new("ASD =    "),
            Err(nom::Err::Failure(KvnStringParserErr::EmptyValue {
                input: "ASD =    "
            }))
        );
        assert_eq!(
            parse_kvn_string_line_new("ASD = "),
            Err(nom::Err::Failure(KvnStringParserErr::EmptyValue {
                input: "ASD = "
            }))
        );
        assert_eq!(
            parse_kvn_string_line_new("ASD ="),
            Err(nom::Err::Failure(KvnStringParserErr::EmptyValue {
                input: "ASD ="
            }))
        );

        assert_eq!(
            parse_kvn_string_line_new("ASD   [km]"),
            Err(nom::Err::Failure(KvnStringParserErr::InvalidFormat {
                input: "ASD   [km]"
            }))
        );
        assert_eq!(
            parse_kvn_string_line_new(" = asd [km]"),
            Err(nom::Err::Failure(KvnStringParserErr::EmptyKeyword {
                input: " = asd [km]"
            }))
        );

        // 7.4.7 Any white space immediately preceding the end of line shall not be significant.
        assert_eq!(
            parse_kvn_string_line_new("ASD = ASDFG          "),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: None
                }
            ))
        );

        // 7.4.5 Any white space immediately preceding or following the keyword shall not be significant.
        assert_eq!(
            parse_kvn_string_line_new("  ASD  = ASDFG"),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: None
                }
            ))
        );

        // 7.8.5 All comment lines shall begin with the ‘COMMENT’ keyword followed by at least one space.
        // [...] White space shall be retained (shall be significant) in comment values.

        assert_eq!(
            parse_kvn_string_line_new("  COMMENT asd a    asd a ads as "),
            Ok((
                "",
                KvnValue {
                    value: "asd a    asd a ads as ".to_string(),
                    unit: None
                }
            ))
        );

        assert_eq!(
            parse_kvn_string_line_new("  COMMENT "),
            Ok((
                "",
                KvnValue {
                    value: "".to_string(),
                    unit: None
                }
            ))
        );
    }

    #[test]
    fn test_parse_kvn_integer_line_new() {
        // a) there must be at least one blank character between the value and the units text;
        // b) the units must be enclosed within square brackets (e.g., ‘[m]’);
        assert_eq!(
            parse_kvn_integer_line_new("SCLK_OFFSET_AT_EPOCH = 28800 [s]", true),
            Ok((
                "",
                KvnValue {
                    value: 28800,
                    unit: Some("s".to_string())
                },
            ))
        );

        // 7.4.7 Any white space immediately preceding the end of line shall not be significant.

        assert_eq!(
            parse_kvn_integer_line_new("SCLK_OFFSET_AT_EPOCH = 28800             [s]", true),
            Ok((
                "",
                KvnValue {
                    value: 28800,
                    unit: Some("s".to_string())
                }
            ))
        );

        assert_eq!(
            parse_kvn_integer_line_new("SCLK_OFFSET_AT_EPOCH = 28800             ", false),
            Ok((
                "",
                KvnValue {
                    value: 28800,
                    unit: None
                }
            ))
        );

        // 7.4.5 Any white space immediately preceding or following the keyword shall not be significant.

        assert_eq!(
            parse_kvn_integer_line_new("          SCLK_OFFSET_AT_EPOCH = 28800", false),
            Ok((
                "",
                KvnValue {
                    value: 28800,
                    unit: None
                }
            ))
        );

        assert_eq!(
            parse_kvn_integer_line_new("SCLK_OFFSET_AT_EPOCH = 00028800 [s]", true),
            Ok((
                "",
                KvnValue {
                    value: 28800,
                    unit: Some("s".to_string())
                },
            ))
        );

        assert_eq!(
            parse_kvn_integer_line_new("SCLK_OFFSET_AT_EPOCH = -28800 [s]", true),
            Ok((
                "",
                KvnValue {
                    value: -28800,
                    unit: Some("s".to_string())
                },
            ))
        );

        assert_eq!(
            parse_kvn_integer_line_new("SCLK_OFFSET_AT_EPOCH = -28800", true),
            Ok((
                "",
                KvnValue {
                    value: -28800,
                    unit: None
                },
            ))
        );

        assert_eq!(
            parse_kvn_integer_line_new("SCLK_OFFSET_AT_EPOCH = 28800 [s]", true),
            Ok((
                "",
                KvnValue {
                    value: 28800,
                    unit: Some("s".to_string())
                },
            ))
        );

        assert_eq!(
            parse_kvn_integer_line_new::<u32>("SCLK_OFFSET_AT_EPOCH = 28800 [s]", false),
            Err(nom::Err::Failure(KvnNumberParserErr::InvalidFormat {
                input: "SCLK_OFFSET_AT_EPOCH = 28800 [s]"
            }))
        );

        assert_eq!(
            parse_kvn_integer_line_new::<u32>("SCLK_OFFSET_AT_EPOCH = -asd", true),
            Err(nom::Err::Failure(KvnNumberParserErr::InvalidFormat {
                input: "SCLK_OFFSET_AT_EPOCH = -asd"
            }))
        );

        assert_eq!(
            parse_kvn_integer_line_new::<u32>("SCLK_OFFSET_AT_EPOCH = [s]", true),
            Err(nom::Err::Failure(KvnNumberParserErr::EmptyValue {
                input: "SCLK_OFFSET_AT_EPOCH = [s]"
            }))
        );

        assert_eq!(
            parse_kvn_integer_line_new::<u32>("SCLK_OFFSET_AT_EPOCH =    ", false),
            Err(nom::Err::Failure(KvnNumberParserErr::EmptyValue {
                input: "SCLK_OFFSET_AT_EPOCH =    "
            }))
        );
        assert_eq!(
            parse_kvn_integer_line_new::<u32>("SCLK_OFFSET_AT_EPOCH = ", false),
            Err(nom::Err::Failure(KvnNumberParserErr::EmptyValue {
                input: "SCLK_OFFSET_AT_EPOCH = "
            }))
        );
        assert_eq!(
            parse_kvn_integer_line_new::<u32>("SCLK_OFFSET_AT_EPOCH =", false),
            Err(nom::Err::Failure(KvnNumberParserErr::EmptyValue {
                input: "SCLK_OFFSET_AT_EPOCH ="
            }))
        );

        assert_eq!(
            parse_kvn_integer_line_new::<u32>("SCLK_OFFSET_AT_EPOCH   [km]", true),
            Err(nom::Err::Failure(KvnNumberParserErr::InvalidFormat {
                input: "SCLK_OFFSET_AT_EPOCH   [km]"
            }))
        );
        assert_eq!(
            parse_kvn_integer_line_new::<u32>(" = 123 [km]", true),
            Err(nom::Err::Failure(KvnNumberParserErr::EmptyKeyword {
                input: " = 123 [km]"
            }))
        );
    }

    #[test]
    fn test_parse_kvn_numeric_line_new() {
        // a) there must be at least one blank character between the value and the units text;
        // b) the units must be enclosed within square brackets (e.g., ‘[m]’);
        assert_eq!(
            parse_kvn_numeric_line_new("X = 66559942 [km]", true),
            Ok((
                "",
                KvnValue {
                    value: 66559942f64,
                    unit: Some("km".to_string())
                },
            ))
        );

        // 7.4.7 Any white space immediately preceding the end of line shall not be significant.

        assert_eq!(
            parse_kvn_numeric_line_new("X = 66559942             [km]", true),
            Ok((
                "",
                KvnValue {
                    value: 66559942f64,
                    unit: Some("km".to_string())
                }
            ))
        );

        assert_eq!(
            parse_kvn_numeric_line_new("X = 66559942             ", false),
            Ok((
                "",
                KvnValue {
                    value: 66559942f64,
                    unit: None
                }
            ))
        );

        // 7.4.5 Any white space immediately preceding or following the keyword shall not be significant.

        assert_eq!(
            parse_kvn_numeric_line_new("          X = 66559942", false),
            Ok((
                "",
                KvnValue {
                    value: 66559942f64,
                    unit: None
                }
            ))
        );

        assert_eq!(
            parse_kvn_numeric_line_new("X = 6655.9942 [km]", true),
            Ok((
                "",
                KvnValue {
                    value: 6655.9942,
                    unit: Some("km".to_string())
                },
            ))
        );

        assert_eq!(
            parse_kvn_numeric_line_new("CX_X =  5.801003223606e-05", true),
            Ok((
                "",
                KvnValue {
                    value: 5.801003223606e-05,
                    unit: None
                },
            ))
        );

        assert_eq!(
            parse_kvn_numeric_line_new("X = -asd", true),
            Err(nom::Err::Failure(KvnNumberParserErr::InvalidFormat {
                input: "X = -asd"
            }))
        );

        assert_eq!(
            parse_kvn_numeric_line_new("X = [s]", true),
            Err(nom::Err::Failure(KvnNumberParserErr::EmptyValue {
                input: "X = [s]"
            }))
        );

        assert_eq!(
            parse_kvn_numeric_line_new("X =    ", false),
            Err(nom::Err::Failure(KvnNumberParserErr::EmptyValue {
                input: "X =    "
            }))
        );
        assert_eq!(
            parse_kvn_numeric_line_new("X = ", false),
            Err(nom::Err::Failure(KvnNumberParserErr::EmptyValue {
                input: "X = "
            }))
        );
        assert_eq!(
            parse_kvn_numeric_line_new("X =", false),
            Err(nom::Err::Failure(KvnNumberParserErr::EmptyValue {
                input: "X ="
            }))
        );

        assert_eq!(
            parse_kvn_numeric_line_new("X   [km]", true),
            Err(nom::Err::Failure(KvnNumberParserErr::InvalidFormat {
                input: "X   [km]"
            }))
        );
        assert_eq!(
            parse_kvn_numeric_line_new(" = 123 [km]", true),
            Err(nom::Err::Failure(KvnNumberParserErr::EmptyKeyword {
                input: " = 123 [km]"
            }))
        );
    }

    #[test]
    fn test_parse_kvn_datetime_line_new() {
        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE = 2021-06-03T05:33:00.123"),
            Ok((
                "",
                KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 5,
                    minute: 33,
                    second: 0,
                    fractional_second: 0.123,
                    full_value: "2021-06-03T05:33:00.123".to_string(),
                },
            ))
        );

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE = 2021-06-03T05:33:01"),
            Ok((
                "",
                KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 5,
                    minute: 33,
                    second: 1,
                    fractional_second: 0.0,
                    full_value: "2021-06-03T05:33:01".to_string(),
                },
            ))
        );

        // 7.4.7 Any white space immediately preceding the end of line shall not be significant.

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE = 2021-06-03T05:33:01           "),
            Ok((
                "",
                KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 5,
                    minute: 33,
                    second: 1,
                    fractional_second: 0.0,
                    full_value: "2021-06-03T05:33:01".to_string(),
                },
            ))
        );

        // 7.4.5 Any white space immediately preceding or following the keyword shall not be significant.

        assert_eq!(
            parse_kvn_datetime_line_new("          CREATION_DATE = 2021-06-03T05:33:01"),
            Ok((
                "",
                KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 5,
                    minute: 33,
                    second: 1,
                    fractional_second: 0.0,
                    full_value: "2021-06-03T05:33:01".to_string(),
                },
            ))
        );

        // @TODO add support for ddd format

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE = 2021,06,03Q05!33!00-123"),
            Err(nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat {
                input: "CREATION_DATE = 2021,06,03Q05!33!00-123"
            }))
        );

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE = asdffggg"),
            Err(nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat {
                input: "CREATION_DATE = asdffggg"
            }))
        );

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE = "),
            Err(nom::Err::Failure(KvnDateTimeParserErr::EmptyValue {
                input: "CREATION_DATE = "
            }))
        );

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE =    "),
            Err(nom::Err::Failure(KvnDateTimeParserErr::EmptyValue {
                input: "CREATION_DATE =    "
            }))
        );

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE ="),
            Err(nom::Err::Failure(KvnDateTimeParserErr::EmptyValue {
                input: "CREATION_DATE ="
            }))
        );

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE     "),
            Err(nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat {
                input: "CREATION_DATE     "
            }))
        );
        assert_eq!(
            parse_kvn_datetime_line_new(" = 2021-06-03T05:33:01"),
            Err(nom::Err::Failure(KvnDateTimeParserErr::EmptyKeyword {
                input: " = 2021-06-03T05:33:01"
            }))
        );
    }

    #[derive(Default, Debug, PartialEq)]
    pub struct PositionUnits(pub std::string::String);

    #[derive(KvnDeserialize, Default, Debug, PartialEq)]
    #[kvn(value_unit_struct)]
    pub struct DistanceType {
        pub base: f64,
        pub units: Option<PositionUnits>,
    }

    #[derive(KvnDeserialize, Default, Debug, PartialEq)]
    struct AsdType {
        pub semi_major_axis: DistanceType,
        pub asdfg: f64,
        pub version: String,
    }

    #[test]
    fn test_parse_with_unit_struct() {
        let kvn = r#"CCSDS_ASD_VERS = 3.0
        SEMI_MAJOR_AXIS = 41399.5123 [km]
        ASDFG = 12333.5123"#;

        assert_eq!(
            AsdType::deserialize(&mut kvn.lines().peekable()),
            Ok(AsdType {
                semi_major_axis: DistanceType {
                    base: 41399.5123,
                    units: Some(PositionUnits("km".to_string(),)),
                },
                asdfg: 12333.5123f64,
                version: "3.0".to_string(),
            },)
        )
    }
}
