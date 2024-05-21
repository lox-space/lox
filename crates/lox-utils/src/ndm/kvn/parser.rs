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

pub type KvnStringValue = KvnValue<String, String>;
pub type KvnIntegerValue = KvnValue<i32, String>;
pub type KvnNumericValue = KvnValue<f64, String>;

#[derive(Debug, PartialEq)]
pub enum KvnParserErr<I> {
    KeywordNotFound { expected: I },
    EmptyValue { keyword: I },
    ParserError(I, ErrorKind),
}

#[derive(Debug, PartialEq)]
pub enum KvnKeyMatchErr<I> {
    KeywordNotFound { expected: I },
}

#[derive(PartialEq, Debug)]
pub enum KvnNumberLineParserErr<I> {
    ParserError(I, ErrorKind),
    KeywordNotFound { expected: I },
    EmptyValue { keyword: I },
    InvalidFormat { keyword: I },
}

#[derive(PartialEq, Debug)]
pub enum KvnDateTimeParserErr<I> {
    ParserError(I, ErrorKind),
    KeywordNotFound { expected: I },
    EmptyValue { keyword: I },
    InvalidFormat { keyword: I },
}

#[derive(PartialEq, Debug)]
pub enum KvnDeserializerErr<I> {
    InvalidDateTimeFormat { keyword: I },
    InvalidNumberFormat { keyword: I },
    KeywordNotFound { expected: I },
    UnexpectedKeyword { found: I, expected: I },
    EmptyValue { keyword: I },
    UnexpectedEndOfInput { keyword: I },
    GeneralParserError(I, ErrorKind),
}

impl<I> From<nom::Err<KvnParserErr<I>>> for KvnDeserializerErr<I> {
    fn from(value: nom::Err<KvnParserErr<I>>) -> Self {
        match value {
            nom::Err::Error(KvnParserErr::EmptyValue { keyword })
            | nom::Err::Failure(KvnParserErr::EmptyValue { keyword }) => {
                KvnDeserializerErr::EmptyValue { keyword }
            }
            nom::Err::Error(KvnParserErr::KeywordNotFound { expected })
            | nom::Err::Failure(KvnParserErr::KeywordNotFound { expected }) => {
                KvnDeserializerErr::KeywordNotFound { expected }
            }
            nom::Err::Error(KvnParserErr::ParserError(i, k))
            | nom::Err::Failure(KvnParserErr::ParserError(i, k)) => {
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
            nom::Err::Error(KvnDateTimeParserErr::EmptyValue { keyword })
            | nom::Err::Failure(KvnDateTimeParserErr::EmptyValue { keyword }) => {
                KvnDeserializerErr::EmptyValue { keyword }
            }
            nom::Err::Error(KvnDateTimeParserErr::KeywordNotFound { expected })
            | nom::Err::Failure(KvnDateTimeParserErr::KeywordNotFound { expected }) => {
                KvnDeserializerErr::KeywordNotFound { expected }
            }
            nom::Err::Error(KvnDateTimeParserErr::InvalidFormat { keyword })
            | nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat { keyword }) => {
                KvnDeserializerErr::InvalidDateTimeFormat { keyword }
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

impl<I> From<nom::Err<KvnNumberLineParserErr<I>>> for KvnDeserializerErr<I> {
    fn from(value: nom::Err<KvnNumberLineParserErr<I>>) -> Self {
        match value {
            nom::Err::Error(KvnNumberLineParserErr::EmptyValue { keyword })
            | nom::Err::Failure(KvnNumberLineParserErr::EmptyValue { keyword }) => {
                KvnDeserializerErr::EmptyValue { keyword }
            }
            nom::Err::Error(KvnNumberLineParserErr::KeywordNotFound { expected })
            | nom::Err::Failure(KvnNumberLineParserErr::KeywordNotFound { expected }) => {
                KvnDeserializerErr::KeywordNotFound { expected }
            }
            nom::Err::Error(KvnNumberLineParserErr::InvalidFormat { keyword })
            | nom::Err::Failure(KvnNumberLineParserErr::InvalidFormat { keyword }) => {
                KvnDeserializerErr::InvalidDateTimeFormat { keyword }
            }
            nom::Err::Error(KvnNumberLineParserErr::ParserError(i, k))
            | nom::Err::Failure(KvnNumberLineParserErr::ParserError(i, k)) => {
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

impl<I> ParseError<I> for KvnParserErr<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        KvnParserErr::ParserError(input, kind)
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
}

pub trait KvnDeserializer {
    fn deserialize<'a>(
        lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    ) -> Result<Self, KvnDeserializerErr<&'a str>>
    where
        Self: Sized;
}

fn comment_line<'a>(input: &'a str) -> nom::IResult<&'a str, &'a str, KvnParserErr<&'a str>> {
    let (input, _) = nc::space0(input)?;

    let (remaining, _) = nb::tag("COMMENT ")(input)?;

    Ok(("", remaining))
}

pub fn kvn_line_matches_key<'a>(key: &'a str, input: &'a str) -> bool {
    if key == "COMMENT" {
        ns::tuple((nc::space0::<_, nom::error::Error<_>>, nb::tag(key)))(input).is_ok()
    } else {
        let mut equals = ns::tuple((nc::space0::<_, nom::error::Error<_>>, nc::char('=')));

        // This function should return true in the case of malformed lines which
        // are missing the keyword
        if equals(input).is_ok() {
            return true;
        }

        ns::delimited(nc::space0, nb::tag(key), equals)(input).is_ok()
    }
}

pub fn kvn_line_matches_key_new<'a>(
    key: &'a str,
    input: &'a str,
) -> Result<bool, KvnKeyMatchErr<&'a str>> {
    if key == "COMMENT" {
        ns::tuple((nc::space0::<_, nom::error::Error<_>>, nb::tag(key)))(input)
            .and_then(|_| Ok(true))
            .or_else(|_| Ok(false))
    } else {
        let mut equals = ns::tuple((nc::space0::<_, nom::error::Error<_>>, nc::char('=')));

        if equals(input).is_ok() {
            return Err(KvnKeyMatchErr::KeywordNotFound { expected: key });
        }

        ns::delimited(nc::space0, nb::tag(key), equals)(input)
            .and_then(|_| Ok(true))
            .or_else(|_| Ok(false))
    }
}

fn kvn_line<'a>(
    key: &'a str,
    input: &'a str,
) -> nom::IResult<&'a str, (&'a str, &'a str), KvnParserErr<&'a str>> {
    let mut equals = ns::tuple((nc::space0::<_, KvnParserErr<_>>, nc::char('='), nc::space0));

    match equals(input) {
        Ok(_) => {
            return Err(nom::Err::Failure(KvnParserErr::KeywordNotFound {
                expected: key,
            }))
        }
        Err(_) => (),
    }

    let value = nc::not_line_ending;

    let kvn = ns::separated_pair(nb::tag(key), equals, value);

    ns::delimited(nc::space0, kvn, nc::space0)(input)
}

// fn kvn_line_new<'a>(
//     input: &'a str,
// ) -> nom::IResult<&'a str, (&'a str, &'a str), KvnParserErr<&'a str>> {
//     let mut equals = ns::tuple((nc::space0::<_, KvnParserErr<_>>, nc::char('='), nc::space0));

//     match equals(input) {
//         Ok(_) => {
//             return Err(nom::Err::Failure(KvnParserErr::KeywordNotFound {
//                 //@TODO
//                 expected: "blalalal",
//             }))
//         }
//         Err(_) => (),
//     }

//     let value = nc::not_line_ending;

//     let kvn = ns::separated_pair(nb::tag(key), equals, value);

//     ns::delimited(nc::space0, kvn, nc::space0)(input)
// }

pub fn parse_kvn_string_line<'a>(
    key: &'a str,
    input: &'a str,
    with_unit: bool,
) -> nom::IResult<&'a str, KvnStringValue, KvnParserErr<&'a str>> {
    if key == "COMMENT" {
        let (_, comment) = comment_line(input)?;

        return Ok((
            "",
            KvnValue {
                value: comment.to_owned(),
                unit: None,
            },
        ));
    }

    let (remaining, result) = kvn_line(key, input)?;

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
        return Err(nom::Err::Failure(KvnParserErr::EmptyValue { keyword: key }));
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

pub fn parse_kvn_string_line_new<'a>(
    input: &'a str,
) -> nom::IResult<&'a str, KvnStringValue, KvnParserErr<&'a str>> {
    // if key == "COMMENT" {
    //     let (_, comment) = comment_line(input)?;

    //     return Ok((
    //         "",
    //         KvnValue {
    //             value: comment.to_owned(),
    //             unit: None,
    //         },
    //     ));
    // }

    // Figure F-8: CCSDS 502.0-B-3
    let re = Regex::new(r"^(?:\s*)(?<keyword>[0-9A-Z_]*)(?:\s*)=(?:\s*)(?<value>(?:(?:[0-9A-Z_\.\- ]*)|(?:[0-9a-z_\.\- ]*)))(?:\s*)$").unwrap();

    // @TODO unwrap
    let captures = re.captures(input).unwrap();

    // @TODO unwrap
    let value = captures
        .name("value")
        .unwrap()
        .as_str()
        .trim_end()
        .to_owned();

    if value.len() == 0 {
        //@TODO
        return Err(nom::Err::Failure(KvnParserErr::EmptyValue {
            keyword: "ASD",
        }));
    }

    Ok(("", KvnValue { value, unit: None }))
}

pub fn parse_kvn_integer_line<'a>(
    key: &'a str,
    input: &'a str,
    with_unit: bool,
) -> nom::IResult<&'a str, KvnIntegerValue, KvnNumberLineParserErr<&'a str>> {
    parse_kvn_string_line(key, input, with_unit)
        .map_err(|e| match e {
            nom::Err::Failure(KvnParserErr::EmptyValue { keyword }) => {
                nom::Err::Failure(KvnNumberLineParserErr::EmptyValue { keyword })
            }
            nom::Err::Error(KvnParserErr::EmptyValue { keyword }) => {
                nom::Err::Error(KvnNumberLineParserErr::EmptyValue { keyword })
            }
            nom::Err::Failure(KvnParserErr::ParserError(input, code)) => {
                nom::Err::Failure(KvnNumberLineParserErr::ParserError(input, code))
            }
            nom::Err::Error(KvnParserErr::ParserError(input, code)) => {
                nom::Err::Error(KvnNumberLineParserErr::ParserError(input, code))
            }
            nom::Err::Failure(KvnParserErr::KeywordNotFound { expected }) => {
                nom::Err::Failure(KvnNumberLineParserErr::KeywordNotFound { expected })
            }
            nom::Err::Error(KvnParserErr::KeywordNotFound { expected }) => {
                nom::Err::Error(KvnNumberLineParserErr::KeywordNotFound { expected })
            }
            nom::Err::Incomplete(needed) => nom::Err::Incomplete(needed),
        })
        .and_then(|result| {
            let value = result.1.value.parse::<i32>().map_err(|_| {
                nom::Err::Failure(KvnNumberLineParserErr::InvalidFormat { keyword: key })
            })?;

            Ok((
                "",
                KvnValue {
                    value,
                    unit: result.1.unit,
                },
            ))
        })
}

pub fn parse_kvn_integer_line_new<'a>(
    input: &'a str,
    with_unit: bool,
) -> nom::IResult<&'a str, KvnIntegerValue, KvnNumberLineParserErr<&'a str>> {
    if is_empty_value(input) {
        Err(nom::Err::Failure(KvnNumberLineParserErr::EmptyValue {
            //@TODO
            keyword: "SCLK_OFFSET_AT_EPOCH",
        }))?
    };

    // Modified from Figure F-9: CCSDS 502.0-B-3
    let re = Regex::new(r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?<value>(?:[-+]?)(?:[0-9]+)(?:\.\d*)?)(?:(?:\s*)(?:\[(?<unit>[0-9A-Za-z/_*]*)\]?))?(?:\s*)?$")
    .unwrap();

    // @TODO unwrap
    let captures =
        re.captures(input)
            .ok_or(nom::Err::Failure(KvnNumberLineParserErr::InvalidFormat {
                //@TODO
                keyword: "SCLK_OFFSET_AT_EPOCH",
            }))?;

    // @TODO unwrap
    let value = captures.name("value").unwrap().as_str();
    let unit = captures.name("unit").map(|x| x.as_str().to_owned());

    let value = value.parse::<i32>().map_err(|_| {
        nom::Err::Failure(KvnNumberLineParserErr::InvalidFormat {
            //@TODO
            keyword: "SCLK_OFFSET_AT_EPOCH",
        })
    })?;

    Ok(("", KvnValue { value, unit }))
}

fn is_empty_value(input: &str) -> bool {
    let re = Regex::new(
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?:\[(?<unit>[0-9A-Za-z/_*]*)\]?)?$",
    )
    .unwrap();

    re.is_match(input)
}

pub fn parse_kvn_numeric_line<'a>(
    key: &'a str,
    input: &'a str,
    with_unit: bool,
) -> nom::IResult<&'a str, KvnNumericValue, KvnNumberLineParserErr<&'a str>> {
    parse_kvn_string_line(key, input, with_unit)
        .map_err(|e| match e {
            nom::Err::Failure(KvnParserErr::EmptyValue { keyword }) => {
                nom::Err::Failure(KvnNumberLineParserErr::EmptyValue { keyword })
            }
            nom::Err::Error(KvnParserErr::EmptyValue { keyword }) => {
                nom::Err::Error(KvnNumberLineParserErr::EmptyValue { keyword })
            }
            nom::Err::Failure(KvnParserErr::ParserError(input, code)) => {
                nom::Err::Failure(KvnNumberLineParserErr::ParserError(input, code))
            }
            nom::Err::Error(KvnParserErr::ParserError(input, code)) => {
                nom::Err::Error(KvnNumberLineParserErr::ParserError(input, code))
            }
            nom::Err::Failure(KvnParserErr::KeywordNotFound { expected }) => {
                nom::Err::Failure(KvnNumberLineParserErr::KeywordNotFound { expected })
            }
            nom::Err::Error(KvnParserErr::KeywordNotFound { expected }) => {
                nom::Err::Error(KvnNumberLineParserErr::KeywordNotFound { expected })
            }
            nom::Err::Incomplete(needed) => nom::Err::Incomplete(needed),
        })
        .and_then(|result| {
            let value = fast_float::parse(result.1.value).map_err(|_| {
                nom::Err::Failure(KvnNumberLineParserErr::InvalidFormat { keyword: key })
            })?;

            Ok((
                "",
                KvnValue {
                    value,
                    unit: result.1.unit,
                },
            ))
        })
}

pub fn parse_kvn_numeric_line_new<'a>(
    input: &'a str,
    with_unit: bool,
) -> nom::IResult<&'a str, KvnNumericValue, KvnNumberLineParserErr<&'a str>> {
    if is_empty_value(input) {
        Err(nom::Err::Failure(KvnNumberLineParserErr::EmptyValue {
            //@TODO
            keyword: "SCLK_OFFSET_AT_EPOCH",
        }))?
    };

    // Figure F-9: CCSDS 502.0-B-3
    let re = Regex::new(r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?<value>(?:[-+]?)(?:[0-9]+)(?:\.\d*)?(?:[eE][+-]?(?:\d+))?)(?:(?:\s*)(?:\[(?<unit>[0-9A-Za-z/_*]*)\]?))?(?:\s*)?$").unwrap();

    // @TODO unwrap
    let captures = re.captures(input).unwrap();

    // @TODO unwrap
    let value = captures.name("value").unwrap().as_str();
    let unit = captures.name("unit").map(|x| x.as_str().to_owned());

    let value = fast_float::parse(value).map_err(|_| {
        //@TODO
        nom::Err::Failure(KvnNumberLineParserErr::InvalidFormat { keyword: "blalala" })
    })?;

    Ok(("", KvnValue { value, unit }))
}

pub fn parse_kvn_datetime_line<'a>(
    key: &'a str,
    input: &'a str,
) -> nom::IResult<&'a str, KvnDateTimeValue, KvnDateTimeParserErr<&'a str>> {
    let (_, result) = kvn_line(key, input).map_err(|e| match e {
        nom::Err::Failure(KvnParserErr::EmptyValue { keyword }) => {
            nom::Err::Failure(KvnDateTimeParserErr::EmptyValue { keyword })
        }
        nom::Err::Error(KvnParserErr::EmptyValue { keyword }) => {
            nom::Err::Error(KvnDateTimeParserErr::EmptyValue { keyword })
        }
        nom::Err::Failure(KvnParserErr::ParserError(input, code)) => {
            nom::Err::Failure(KvnDateTimeParserErr::ParserError(input, code))
        }
        nom::Err::Error(KvnParserErr::ParserError(input, code)) => {
            nom::Err::Error(KvnDateTimeParserErr::ParserError(input, code))
        }
        nom::Err::Failure(KvnParserErr::KeywordNotFound { expected }) => {
            nom::Err::Failure(KvnDateTimeParserErr::KeywordNotFound { expected })
        }
        nom::Err::Error(KvnParserErr::KeywordNotFound { expected }) => {
            nom::Err::Error(KvnDateTimeParserErr::KeywordNotFound { expected })
        }
        nom::Err::Incomplete(needed) => nom::Err::Incomplete(needed),
    })?;

    let parsed_value = result.1;

    if parsed_value.len() == 0 {
        return Err(nom::Err::Failure(KvnDateTimeParserErr::EmptyValue {
            keyword: key,
        }));
    }

    let parsed_value = parsed_value.trim_end();

    // Taken from CCSDS 502.0-B-3 Figure F-5: Regex Pattern for CCSDS Timecode
    // This unwrap is okay here because this regex never changes after testing
    let re = Regex::new(
        r"^(?<yr>(?:\d{4}))-(?<mo>(?:\d{1,2}))-(?<dy>(?:\d{1,2}))T(?<hr>(?:\d{1,2})):(?<mn>(?:\d{1,2})):(?<sc>(?:\d{0,2}(?:\.\d*)?))$",
    )
    .unwrap();

    let captures = re.captures(parsed_value).ok_or(nom::Err::Failure(
        KvnDateTimeParserErr::InvalidFormat { keyword: key },
    ))?;

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

pub fn parse_kvn_datetime_line_new<'a>(
    input: &'a str,
) -> nom::IResult<&'a str, KvnDateTimeValue, KvnDateTimeParserErr<&'a str>> {
    if is_empty_value(input) {
        Err(nom::Err::Failure(KvnDateTimeParserErr::EmptyValue {
            //@TODO
            keyword: "CREATION_DATE",
        }))?
    };

    // Figure F-5: CCSDS 502.0-B-3
    let re = Regex::new(r"^(?:\s*)?(?<keyword>[0-9A-Z_]*)(?:\s*)?=(?:\s*)?(?<yr>(?:\d{4}))-(?<mo>(?:\d{1,2}))-(?<dy>(?:\d{1,2}))T(?<hr>(?:\d{1,2})):(?<mn>(?:\d{1,2})):(?<sc>(?:\d{0,2}(?:\.\d*)?))(?:\s*)?$").unwrap();

    let captures =
        re.captures(input)
            .ok_or(nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat {
                //@TODO
                keyword: "CREATION_DATE",
            }))?;

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
    use lox_derive::KvnDeserialize;

    use super::*;

    #[test]
    fn test_kvn_line_matches_key() {
        assert!(!kvn_line_matches_key("ASD", "AAAAAD123 = ASDFG"));
        assert!(kvn_line_matches_key("ASD", "ASD = ASDFG"));
        assert!(!kvn_line_matches_key("ASD", "ASD ASD = ASDFG"));
        assert!(kvn_line_matches_key("ASD", "= ASDFG"));
    }

    #[test]
    fn test_kvn_line_matches_key_new() {
        assert_eq!(
            kvn_line_matches_key_new("ASD", "AAAAAD123 = ASDFG"),
            Ok(false)
        );
        assert_eq!(kvn_line_matches_key_new("ASD", "ASD = ASDFG"), Ok(true));

        // 7.4.4 Keywords must be uppercase and must not contain blanks
        // @TODO
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
            Err(nom::Err::Failure(KvnParserErr::EmptyValue {
                keyword: "ASD"
            }))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD = ", true),
            Err(nom::Err::Failure(KvnParserErr::EmptyValue {
                keyword: "ASD"
            }))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD =", true),
            Err(nom::Err::Failure(KvnParserErr::EmptyValue {
                keyword: "ASD"
            }))
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
            Err(nom::Err::Failure(KvnParserErr::EmptyValue {
                keyword: "ASD"
            }))
        );

        assert_eq!(
            parse_kvn_string_line("ASD", "ASD   [km]", true),
            Err(nom::Err::Error(KvnParserErr::ParserError(
                "[km]",
                nom::error::ErrorKind::Char
            )))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", " =  [km]", true),
            Err(nom::Err::Failure(KvnParserErr::KeywordNotFound {
                expected: "ASD"
            }))
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
            Err(nom::Err::Failure(KvnParserErr::EmptyValue {
                keyword: "ASD"
            }))
        );
        assert_eq!(
            parse_kvn_string_line_new("ASD = "),
            Err(nom::Err::Failure(KvnParserErr::EmptyValue {
                keyword: "ASD"
            }))
        );
        assert_eq!(
            parse_kvn_string_line_new("ASD ="),
            Err(nom::Err::Failure(KvnParserErr::EmptyValue {
                keyword: "ASD"
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

        //@TODO move to numeric test
        // a) there must be at least one blank character between the value and the units text;
        // b) the units must be enclosed within square brackets (e.g., ‘[m]’);
        // assert_eq!(
        //     parse_kvn_string_line_new("ASD = ASDFG [km]"),
        //     Ok((
        //         "",
        //         KvnValue {
        //             value: "ASDFG".to_string(),
        //             unit: Some("km".to_string())
        //         }
        //     ))
        // );
        // assert_eq!(
        //     parse_kvn_string_line_new("ASD = ASDFG             [km]"),
        //     Ok((
        //         "",
        //         KvnValue {
        //             value: "ASDFG".to_string(),
        //             unit: Some("km".to_string())
        //         }
        //     ))
        // );

        // assert_eq!(
        //     parse_kvn_string_line_new("ASD =  [km]"),
        //     Err(nom::Err::Failure(KvnParserErr::EmptyValue {
        //         keyword: "ASD"
        //     }))
        // );

        // assert_eq!(
        //     parse_kvn_string_line_new("ASD   [km]"),
        //     Err(nom::Err::Error(KvnParserErr::ParserError(
        //         "[km]",
        //         nom::error::ErrorKind::Char
        //     )))
        // );
        // assert_eq!(
        //     parse_kvn_string_line_new(" =  [km]"),
        //     Err(nom::Err::Failure(KvnParserErr::KeywordNotFound {
        //         expected: "ASD"
        //     }))
        // );

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

        //@TODO implement COMMENT
        // 7.8.5 All comment lines shall begin with the ‘COMMENT’ keyword followed by at least one space.
        // [...] White space shall be retained (shall be significant) in comment values.

        // assert_eq!(
        //     parse_kvn_string_line_new("  COMMENT asd a    asd a ads as "),
        //     Ok((
        //         "",
        //         KvnValue {
        //             value: "asd a    asd a ads as ".to_string(),
        //             unit: None
        //         }
        //     ))
        // );

        // assert_eq!(
        //     parse_kvn_string_line_new("  COMMENT "),
        //     Ok((
        //         "",
        //         KvnValue {
        //             value: "".to_string(),
        //             unit: None
        //         }
        //     ))
        // );
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
            Err(nom::Err::Failure(KvnNumberLineParserErr::InvalidFormat {
                keyword: "SCLK_OFFSET_AT_EPOCH"
            }))
        );

        assert_eq!(
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH", "SCLK_OFFSET_AT_EPOCH = [s]", true),
            Err(nom::Err::Failure(KvnNumberLineParserErr::EmptyValue {
                keyword: "SCLK_OFFSET_AT_EPOCH"
            }))
        );
    }

    #[test]
    fn test_parse_kvn_integer_line_new() {
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
            parse_kvn_integer_line_new("SCLK_OFFSET_AT_EPOCH = -asd", true),
            Err(nom::Err::Failure(KvnNumberLineParserErr::InvalidFormat {
                keyword: "SCLK_OFFSET_AT_EPOCH"
            }))
        );

        assert_eq!(
            parse_kvn_integer_line_new("SCLK_OFFSET_AT_EPOCH = [s]", true),
            Err(nom::Err::Failure(KvnNumberLineParserErr::EmptyValue {
                keyword: "SCLK_OFFSET_AT_EPOCH"
            }))
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
            Err(nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat {
                keyword: "CREATION_DATE"
            }))
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE", "CREATION_DATE = asdffggg"),
            Err(nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat {
                keyword: "CREATION_DATE"
            }))
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE", "CREATION_DATE = "),
            Err(nom::Err::Failure(KvnDateTimeParserErr::EmptyValue {
                keyword: "CREATION_DATE"
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
                },
            ))
        );

        // @TODO add support for ddd format

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE = 2021,06,03Q05!33!00-123"),
            Err(nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat {
                keyword: "CREATION_DATE"
            }))
        );

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE = asdffggg"),
            Err(nom::Err::Failure(KvnDateTimeParserErr::InvalidFormat {
                keyword: "CREATION_DATE"
            }))
        );

        assert_eq!(
            parse_kvn_datetime_line_new("CREATION_DATE = "),
            Err(nom::Err::Failure(KvnDateTimeParserErr::EmptyValue {
                keyword: "CREATION_DATE"
            }))
        );
    }

    #[derive(KvnDeserialize, Default, Debug, PartialEq)]
    pub struct KvnChildStruct {
        child_date_value: KvnDateTimeValue,
        child_numeric_value: KvnNumericValue,
    }

    #[derive(KvnDeserialize, Default, Debug, PartialEq)]
    pub struct KvnOptionChildStruct {
        option_child_date_value: KvnDateTimeValue,
        option_child_numeric_value: KvnNumericValue,
    }

    #[derive(KvnDeserialize, Default, Debug, PartialEq)]
    pub struct KvnStruct {
        comment_list: Vec<KvnStringValue>,
        string_value: KvnStringValue,
        date_value: KvnDateTimeValue,
        option_child_struct: Option<KvnOptionChildStruct>,
        numeric_value: KvnNumericValue,
        option_numeric_value: Option<KvnNumericValue>,
        integer_value: KvnIntegerValue,
        option_date_value: Option<KvnDateTimeValue>,
        vec_child_struct_list: Vec<KvnChildStruct>,
    }

    #[test]
    fn test_parse_whole_struct() {
        let kvn = r#"COMMENT Generated by GSOC, R. Kiehling
COMMENT asdsda a1 adsd
STRING_VALUE = EUTELSAT W4
DATE_VALUE = 2021-06-03T05:33:00.123
OPTION_CHILD_DATE_VALUE = 2021-06-03T05:33:00.123
OPTION_CHILD_NUMERIC_VALUE = 6655.9942 [km]
NUMERIC_VALUE = 6655.9942 [km]
INTEGER_VALUE = 123 [km]
OPTION_DATE_VALUE = 2021-06-03T05:33:00.123
CHILD_DATE_VALUE = 2021-06-03T05:33:00.123
CHILD_NUMERIC_VALUE = 6655.9942 [km]
CHILD_DATE_VALUE = 2021-02-03T05:33:00.123
CHILD_NUMERIC_VALUE = 1122.9942 [km]"#;

        assert_eq!(
            KvnStruct::deserialize(&mut kvn.lines().peekable()),
            Ok(KvnStruct {
                comment_list: vec![
                    KvnValue {
                        value: "Generated by GSOC, R. Kiehling".to_string(),
                        unit: None,
                    },
                    KvnValue {
                        value: "asdsda a1 adsd".to_string(),
                        unit: None,
                    },
                ],
                string_value: KvnValue {
                    value: "EUTELSAT W4".to_string(),
                    unit: None,
                },
                date_value: KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 5,
                    minute: 33,
                    second: 0,
                    fractional_second: 0.123,
                },
                option_child_struct: Some(KvnOptionChildStruct {
                    option_child_date_value: KvnDateTimeValue {
                        year: 2021,
                        month: 6,
                        day: 3,
                        hour: 5,
                        minute: 33,
                        second: 0,
                        fractional_second: 0.123,
                    },
                    option_child_numeric_value: KvnValue {
                        value: 6655.9942,
                        unit: Some("km".to_string(),),
                    },
                }),
                numeric_value: KvnValue {
                    value: 6655.9942,
                    unit: Some("km".to_string(),),
                },
                option_numeric_value: None,
                integer_value: KvnValue {
                    value: 123,
                    unit: Some("km".to_string(),),
                },
                option_date_value: Some(KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 5,
                    minute: 33,
                    second: 0,
                    fractional_second: 0.123,
                },),
                vec_child_struct_list: vec![
                    KvnChildStruct {
                        child_date_value: KvnDateTimeValue {
                            year: 2021,
                            month: 6,
                            day: 3,
                            hour: 5,
                            minute: 33,
                            second: 0,
                            fractional_second: 0.123,
                        },
                        child_numeric_value: KvnValue {
                            value: 6655.9942,
                            unit: Some("km".to_string(),),
                        },
                    },
                    KvnChildStruct {
                        child_date_value: KvnDateTimeValue {
                            year: 2021,
                            month: 2,
                            day: 3,
                            hour: 5,
                            minute: 33,
                            second: 0,
                            fractional_second: 0.123,
                        },
                        child_numeric_value: KvnValue {
                            value: 1122.9942,
                            unit: Some("km".to_string(),),
                        },
                    },
                ],
            },)
        );

        let kvn = r#"COMMENT Generated by GSOC, R. Kiehling
COMMENT asdsda a1 adsd
STRING_VALUE = EUTELSAT W4
DATE_VALUE = 2021-06-03T05:33:00.123
NUMERIC_VALUE = 6655.9942 [km]
INTEGER_VALUE = 123 [km]
OPTION_DATE_VALUE = 2021-06-03T05:33:00.123"#;

        assert_eq!(
            KvnStruct::deserialize(&mut kvn.lines().peekable()),
            Ok(KvnStruct {
                comment_list: vec![
                    KvnValue {
                        value: "Generated by GSOC, R. Kiehling".to_string(),
                        unit: None,
                    },
                    KvnValue {
                        value: "asdsda a1 adsd".to_string(),
                        unit: None,
                    },
                ],
                string_value: KvnValue {
                    value: "EUTELSAT W4".to_string(),
                    unit: None,
                },
                date_value: KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 5,
                    minute: 33,
                    second: 0,
                    fractional_second: 0.123,
                },
                option_child_struct: None,
                numeric_value: KvnValue {
                    value: 6655.9942,
                    unit: Some("km".to_string(),),
                },
                option_numeric_value: None,
                integer_value: KvnValue {
                    value: 123,
                    unit: Some("km".to_string(),),
                },
                option_date_value: Some(KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 5,
                    minute: 33,
                    second: 0,
                    fractional_second: 0.123,
                },),
                vec_child_struct_list: vec![],
            },)
        );
    }

    #[test]
    fn test_parse_whole_struct_with_empty_vec_and_option() {
        let kvn = r#"STRING_VALUE = EUTELSAT W4
        DATE_VALUE = 2021-06-03T05:33:00.123
        NUMERIC_VALUE = 6655.9942 [km]
        INTEGER_VALUE = 123 [km]"#;

        assert_eq!(
            KvnStruct::deserialize(&mut kvn.lines().peekable()),
            Ok(KvnStruct {
                comment_list: vec![],
                string_value: KvnValue {
                    value: "EUTELSAT W4".to_string(),
                    unit: None,
                },
                date_value: KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 5,
                    minute: 33,
                    second: 0,
                    fractional_second: 0.123,
                },
                option_child_struct: None,
                numeric_value: KvnValue {
                    value: 6655.9942,
                    unit: Some("km".to_string(),),
                },
                option_numeric_value: None,
                integer_value: KvnValue {
                    value: 123,
                    unit: Some("km".to_string(),),
                },
                option_date_value: None,
                vec_child_struct_list: vec![],
            },)
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
    struct KvnWithUnitStruct {
        semi_major_axis: DistanceType,
    }

    #[test]
    fn test_parse_with_unit_struct() {
        let kvn = r#"SEMI_MAJOR_AXIS = 41399.5123 [km]"#;

        assert_eq!(
            KvnWithUnitStruct::deserialize(&mut kvn.lines().peekable()),
            Ok(KvnWithUnitStruct {
                semi_major_axis: DistanceType {
                    base: 41399.5123,
                    units: Some(PositionUnits("km".to_string(),)),
                },
            },)
        )
    }
}
