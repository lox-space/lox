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
use nom::sequence as ns;
use regex::Regex;

pub type KvnStringValue = KvnValue<String, String>;
pub type KvnIntegerValue = KvnValue<i32, String>;
pub type KvnNumericValue = KvnValue<f64, String>;

#[derive(PartialEq, Debug)]
pub enum KvnStringLineParserErr<I> {
    ParseError(nom::Err<nom::error::Error<I>>),
    EmptyValue,
}

#[derive(PartialEq, Debug)]
pub enum KvnNumberLineParserErr<I> {
    ParseError(nom::Err<nom::error::Error<I>>),
    EmptyValue,
    InvalidFormat,
}

#[derive(PartialEq, Debug)]
pub enum KvnDateTimeParserErr<I> {
    ParseError(nom::Err<nom::error::Error<I>>),
    InvalidDateFormat,
    EmptyValue,
}

#[derive(PartialEq, Debug)]
pub enum KvnDeserializerErr<I> {
    String(KvnStringLineParserErr<I>),
    DateTime(KvnDateTimeParserErr<I>),
    Number(KvnNumberLineParserErr<I>),
    UnexpectedEndOfInput,
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
}

pub trait KvnDeserializer<T> {
    fn deserialize<'a>(
        lines: &mut dyn Iterator<Item = &'a str>,
    ) -> Result<T, KvnDeserializerErr<&'a str>>;
}

fn comment_line<'a>(input: &'a str) -> nom::IResult<&'a str, &'a str> {
    let (input, _) = nc::space0(input)?;

    let (remaining, _) = nb::tag("COMMENT ")(input)?;

    Ok(("", remaining))
}

fn kvn_line<'a>(key: &'a str, input: &'a str) -> nom::IResult<&'a str, (&'a str, &'a str)> {
    let equals = ns::tuple((
        nc::space0::<_, nom::error::Error<_>>,
        nc::char('='),
        nc::space0,
    ));

    let value = nc::not_line_ending;

    let kvn = ns::separated_pair(nb::tag(key), equals, value);

    ns::delimited(nc::space0, kvn, nc::space0)(input)
}

pub fn parse_kvn_string_line<'a>(
    key: &'a str,
    input: &'a str,
    with_unit: bool,
) -> Result<(&'a str, KvnStringValue), KvnStringLineParserErr<&'a str>> {
    if key == "COMMENT" {
        let (_, comment) =
            comment_line(input).map_err(|e| KvnStringLineParserErr::ParseError(e))?;

        return Ok((
            "",
            KvnValue {
                value: comment.to_owned(),
                unit: None,
            },
        ));
    }

    let (remaining, result) =
        kvn_line(key, input).map_err(|e| KvnStringLineParserErr::ParseError(e))?;

    let parsed_right_hand_side = result.1;

    let (parsed_value, parsed_unit) = if with_unit {
        // This unwrap is okay here because this regex never changes after testing
        let re = Regex::new(r"(.*?)(\[(.*)\])?$").unwrap();

        // This unwrap is okay because .* will always match
        let captures = re.captures(parsed_right_hand_side).unwrap();

        (
            captures.get(1).unwrap().as_str(),
            captures.get(3).map(|f| f.as_str()),
        )
    } else {
        (parsed_right_hand_side, None)
    };

    if parsed_value.len() == 0 {
        return Err(KvnStringLineParserErr::EmptyValue);
    }

    let parsed_value = parsed_value.trim_end();

    Ok((
        remaining,
        KvnValue {
            value: parsed_value.to_owned(),
            unit: parsed_unit.map(|x| x.to_owned()),
        },
    ))
}

pub fn parse_kvn_integer_line<'a>(
    key: &'a str,
    input: &'a str,
    with_unit: bool,
) -> Result<(&'a str, KvnIntegerValue), KvnNumberLineParserErr<&'a str>> {
    parse_kvn_string_line(key, input, with_unit)
        .map_err(|e| match e {
            KvnStringLineParserErr::EmptyValue => KvnNumberLineParserErr::EmptyValue,
            KvnStringLineParserErr::ParseError(e) => KvnNumberLineParserErr::ParseError(e),
        })
        .and_then(|result| {
            let value = result
                .1
                .value
                .parse::<i32>()
                .map_err(|_| KvnNumberLineParserErr::InvalidFormat)?;

            Ok((
                "",
                KvnValue {
                    value,
                    unit: result.1.unit,
                },
            ))
        })
}

pub fn parse_kvn_numeric_line<'a>(
    key: &'a str,
    input: &'a str,
    with_unit: bool,
) -> Result<(&'a str, KvnNumericValue), KvnNumberLineParserErr<&'a str>> {
    parse_kvn_string_line(key, input, with_unit)
        .map_err(|e| match e {
            KvnStringLineParserErr::EmptyValue => KvnNumberLineParserErr::EmptyValue,
            KvnStringLineParserErr::ParseError(e) => KvnNumberLineParserErr::ParseError(e),
        })
        .and_then(|result| {
            let value = fast_float::parse(result.1.value)
                .map_err(|_| KvnNumberLineParserErr::InvalidFormat)?;

            Ok((
                "",
                KvnValue {
                    value,
                    unit: result.1.unit,
                },
            ))
        })
}

pub fn parse_kvn_datetime_line<'a>(
    key: &'a str,
    input: &'a str,
) -> Result<(&'a str, KvnDateTimeValue), KvnDateTimeParserErr<&'a str>> {
    let (_, result) = kvn_line(key, input).map_err(|e| KvnDateTimeParserErr::ParseError(e))?;

    let parsed_value = result.1;

    if parsed_value.len() == 0 {
        return Err(KvnDateTimeParserErr::EmptyValue);
    }

    let parsed_value = parsed_value.trim_end();

    // Taken from CCSDS 502.0-B-3 Figure F-5: Regex Pattern for CCSDS Timecode
    // This unwrap is okay here because this regex never changes after testing
    let re = Regex::new(
        r"^(?<yr>(?:\d{4}))-(?<mo>(?:\d{1,2}))-(?<dy>(?:\d{1,2}))T(?<hr>(?:\d{1,2})):(?<mn>(?:\d{1,2})):(?<sc>(?:\d{0,2}(?:\.\d*)?))$",
    )
    .unwrap();

    let captures = re
        .captures(parsed_value)
        .ok_or(KvnDateTimeParserErr::InvalidDateFormat)?;

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
        },
    ))
}

mod test {
    use super::*;

    #[test]
    fn test_parse_kvn_string_line() {
        // 7.5.1 A non-empty value field must be assigned to each mandatory keyword except for *‘_START’ and *‘_STOP’ keyword values
        // 7.4.6 Any white space immediately preceding or following the ‘equals’ sign shall not be significant.
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD = ASDFG", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD    =   ASDFG", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD    = ASDFG", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD =    ", true),
            Err(KvnStringLineParserErr::EmptyValue)
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD = ", true),
            Err(KvnStringLineParserErr::EmptyValue)
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD =", true),
            Err(KvnStringLineParserErr::EmptyValue)
        );

        // 7.4.7 Any white space immediately preceding the end of line shall not be significant.
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD = ASDFG          ", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: None
                }
            ))
        );

        // a) there must be at least one blank character between the value and the units text;
        // b) the units must be enclosed within square brackets (e.g., ‘[m]’);
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD = ASDFG [km]", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: Some("km".to_string())
                }
            ))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD = ASDFG             [km]", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG".to_string(),
                    unit: Some("km".to_string())
                }
            ))
        );

        assert_eq!(
            parse_kvn_string_line("ASD", "ASD =  [km]", true),
            Err(KvnStringLineParserErr::EmptyValue)
        );

        assert_eq!(
            parse_kvn_string_line("ASD", "ASD   [km]", true),
            Err(KvnStringLineParserErr::ParseError(nom::Err::Error(
                nom::error::Error {
                    input: "[km]",
                    code: nom::error::ErrorKind::Char
                }
            )))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", " =  [km]", true),
            Err(KvnStringLineParserErr::ParseError(nom::Err::Error(
                nom::error::Error {
                    input: "=  [km]",
                    code: nom::error::ErrorKind::Tag
                }
            )))
        );

        // 7.4.5 Any white space immediately preceding or following the keyword shall not be significant.
        assert_eq!(
            parse_kvn_string_line("ASD", "  ASD  = ASDFG", true),
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
            parse_kvn_string_line("COMMENT", "  COMMENT asd a    asd a ads as ", true),
            Ok((
                "",
                KvnValue {
                    value: "asd a    asd a ads as ".to_string(),
                    unit: None
                }
            ))
        );

        assert_eq!(
            parse_kvn_string_line("COMMENT", "  COMMENT ", true),
            Ok((
                "",
                KvnValue {
                    value: "".to_string(),
                    unit: None
                }
            ))
        );

        // 7.4.4 Keywords must be uppercase and must not contain blanks
        //@TODO parse dates and floats and integers
        //@TODO return error code when the key doesn't exist
    }

    #[test]
    fn test_parse_kvn_integer_line() {
        assert_eq!(
            parse_kvn_integer_line(
                "SCLK_OFFSET_AT_EPOCH",
                "SCLK_OFFSET_AT_EPOCH = 28800 [s]",
                true
            ),
            Ok((
                "",
                KvnValue {
                    value: 28800,
                    unit: Some("s".to_string())
                },
            ))
        );

        assert_eq!(
            parse_kvn_integer_line(
                "SCLK_OFFSET_AT_EPOCH",
                "SCLK_OFFSET_AT_EPOCH = 00028800 [s]",
                true
            ),
            Ok((
                "",
                KvnValue {
                    value: 28800,
                    unit: Some("s".to_string())
                },
            ))
        );

        assert_eq!(
            parse_kvn_integer_line(
                "SCLK_OFFSET_AT_EPOCH",
                "SCLK_OFFSET_AT_EPOCH = -28800 [s]",
                true
            ),
            Ok((
                "",
                KvnValue {
                    value: -28800,
                    unit: Some("s".to_string())
                },
            ))
        );

        assert_eq!(
            parse_kvn_integer_line(
                "SCLK_OFFSET_AT_EPOCH",
                "SCLK_OFFSET_AT_EPOCH = -28800",
                true
            ),
            Ok((
                "",
                KvnValue {
                    value: -28800,
                    unit: None
                },
            ))
        );

        assert_eq!(
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH", "SCLK_OFFSET_AT_EPOCH = -asd", true),
            Err(KvnNumberLineParserErr::InvalidFormat)
        );

        assert_eq!(
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH", "SCLK_OFFSET_AT_EPOCH = [s]", true),
            Err(KvnNumberLineParserErr::EmptyValue)
        );
    }

    #[test]
    fn test_parse_kvn_numeric_line() {
        assert_eq!(
            parse_kvn_numeric_line("X", "X = 66559942 [km]", true),
            Ok((
                "",
                KvnValue {
                    value: 66559942f64,
                    unit: Some("km".to_string())
                },
            ))
        );

        assert_eq!(
            parse_kvn_numeric_line("X", "X = 6655.9942 [km]", true),
            Ok((
                "",
                KvnValue {
                    value: 6655.9942,
                    unit: Some("km".to_string())
                },
            ))
        );

        assert_eq!(
            parse_kvn_numeric_line("CX_X", "CX_X =  5.801003223606e-05", true),
            Ok((
                "",
                KvnValue {
                    value: 5.801003223606e-05,
                    unit: None
                },
            ))
        );
    }

    #[test]
    fn test_parse_kvn_datetime_line() {
        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE", "CREATION_DATE = 2021-06-03T05:33:00.123"),
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
                },
            ))
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE", "CREATION_DATE = 2021-06-03T05:33:01"),
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
                },
            ))
        );

        // @TODO add support for ddd format

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE", "CREATION_DATE = 2021,06,03Q05!33!00-123"),
            Err(KvnDateTimeParserErr::InvalidDateFormat)
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE", "CREATION_DATE = asdffggg"),
            Err(KvnDateTimeParserErr::InvalidDateFormat)
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE", "CREATION_DATE = "),
            Err(KvnDateTimeParserErr::EmptyValue)
        );
    }
}
