/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

// This parser handles the Keyword Value Notation (KVN) defined in section
// 7.4 of CCSDS 502.0-B-3 (https://public.ccsds.org/Pubs/502x0b3e1.pdf).

use regex::Regex;

use super::deserializer::KvnDeserializerErr;

#[derive(Debug, PartialEq)]
pub enum KvnStringParserErr<I> {
    EmptyKeyword { input: I },
    EmptyValue { input: I },
    InvalidFormat { input: I },
}

#[derive(Debug, PartialEq)]
pub enum KvnStateVectorParserErr<I> {
    InvalidFormat { input: I },
}

#[derive(Debug, PartialEq)]
pub enum KvnCovarianceMatrixParserErr<I> {
    InvalidItemCount { input: I },
    InvalidFormat { input: I },
    UnexpectedEndOfInput { keyword: I },
}

#[derive(Debug, PartialEq)]
pub struct KvnKeywordNotFoundErr<I> {
    expected: I,
}

#[derive(PartialEq, Debug)]
pub enum KvnNumberParserErr<I> {
    EmptyKeyword { input: I },
    EmptyValue { input: I },
    InvalidFormat { input: I },
}

#[derive(PartialEq, Debug)]
pub enum KvnDateTimeParserErr<I> {
    EmptyKeyword { input: I },
    EmptyValue { input: I },
    InvalidFormat { input: I },
}

impl From<KvnStateVectorParserErr<&str>> for KvnDeserializerErr<String> {
    fn from(value: KvnStateVectorParserErr<&str>) -> Self {
        match value {
            KvnStateVectorParserErr::InvalidFormat { input } => {
                KvnDeserializerErr::InvalidStateVectorFormat {
                    input: input.to_string(),
                }
            }
        }
    }
}

impl From<KvnCovarianceMatrixParserErr<&str>> for KvnDeserializerErr<String> {
    fn from(value: KvnCovarianceMatrixParserErr<&str>) -> Self {
        match value {
            KvnCovarianceMatrixParserErr::InvalidItemCount { input } => {
                KvnDeserializerErr::InvalidCovarianceMatrixFormat {
                    input: input.to_string(),
                }
            }
            KvnCovarianceMatrixParserErr::InvalidFormat { input } => {
                KvnDeserializerErr::InvalidCovarianceMatrixFormat {
                    input: input.to_string(),
                }
            }
            KvnCovarianceMatrixParserErr::UnexpectedEndOfInput { keyword } => {
                KvnDeserializerErr::UnexpectedEndOfInput {
                    keyword: keyword.to_string(),
                }
            }
        }
    }
}

impl From<KvnStringParserErr<&str>> for KvnDeserializerErr<String> {
    fn from(value: KvnStringParserErr<&str>) -> Self {
        match value {
            KvnStringParserErr::EmptyValue { input } => KvnDeserializerErr::EmptyValue {
                input: input.to_string(),
            },
            KvnStringParserErr::EmptyKeyword { input } => KvnDeserializerErr::EmptyKeyword {
                input: input.to_string(),
            },
            KvnStringParserErr::InvalidFormat { input } => {
                KvnDeserializerErr::InvalidStringFormat {
                    input: input.to_string(),
                }
            }
        }
    }
}

impl From<KvnDateTimeParserErr<&str>> for KvnDeserializerErr<String> {
    fn from(value: KvnDateTimeParserErr<&str>) -> Self {
        match value {
            KvnDateTimeParserErr::EmptyValue { input } => KvnDeserializerErr::EmptyValue {
                input: input.to_string(),
            },
            KvnDateTimeParserErr::EmptyKeyword { input } => KvnDeserializerErr::EmptyKeyword {
                input: input.to_string(),
            },
            KvnDateTimeParserErr::InvalidFormat { input } => {
                KvnDeserializerErr::InvalidDateTimeFormat {
                    input: input.to_string(),
                }
            }
        }
    }
}

impl From<KvnNumberParserErr<&str>> for KvnDeserializerErr<String> {
    fn from(value: KvnNumberParserErr<&str>) -> Self {
        match value {
            KvnNumberParserErr::EmptyValue { input } => KvnDeserializerErr::EmptyValue {
                input: input.to_string(),
            },
            KvnNumberParserErr::EmptyKeyword { input } => KvnDeserializerErr::EmptyKeyword {
                input: input.to_string(),
            },
            KvnNumberParserErr::InvalidFormat { input } => {
                KvnDeserializerErr::InvalidDateTimeFormat {
                    input: input.to_string(),
                }
            }
        }
    }
}

impl From<KvnKeywordNotFoundErr<&str>> for KvnDeserializerErr<String> {
    fn from(value: KvnKeywordNotFoundErr<&str>) -> Self {
        KvnDeserializerErr::KeywordNotFound {
            expected: value.expected.to_string(),
        }
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

#[derive(PartialEq, Debug, Default)]
pub struct KvnStateVectorValue {
    pub epoch: KvnDateTimeValue,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub x_dot: f64,
    pub y_dot: f64,
    pub z_dot: f64,
    pub x_ddot: Option<f64>,
    pub y_ddot: Option<f64>,
    pub z_ddot: Option<f64>,
}

#[derive(PartialEq, Debug, Default)]
pub struct KvnCovarianceMatrixValue {
    pub cx_x: f64,
    pub cy_x: f64,
    pub cy_y: f64,
    pub cz_x: f64,
    pub cz_y: f64,
    pub cz_z: f64,
    pub cx_dot_x: f64,
    pub cx_dot_y: f64,
    pub cx_dot_z: f64,
    pub cx_dot_x_dot: f64,
    pub cy_dot_x: f64,
    pub cy_dot_y: f64,
    pub cy_dot_z: f64,
    pub cy_dot_x_dot: f64,
    pub cy_dot_y_dot: f64,
    pub cz_dot_x: f64,
    pub cz_dot_y: f64,
    pub cz_dot_z: f64,
    pub cz_dot_x_dot: f64,
    pub cz_dot_y_dot: f64,
    pub cz_dot_z_dot: f64,
}

pub fn get_next_nonempty_line<'a>(
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Option<&'a str> {
    loop {
        match lines.peek() {
            None => return None,
            Some(next_line) => {
                if next_line.trim().is_empty() {
                    lines.next();
                    continue;
                } else {
                    return Some(next_line);
                }
            }
        }
    }
}

pub fn kvn_line_matches_key<'a>(
    key: &'a str,
    input: &'a str,
) -> Result<bool, KvnKeywordNotFoundErr<&'a str>> {
    let re = Regex::new(r"^(?:\s*)(?<keyword>[0-9A-Z_]*)(?:\s*)").unwrap();

    let captures = re
        .captures(input)
        .ok_or(KvnKeywordNotFoundErr { expected: key })?;

    let captured_keyword = captures
        .name("keyword")
        // This unwrap is okay because the keyword uses * so it will always capture
        .unwrap()
        .as_str()
        .trim_end()
        .to_string();

    Ok(captured_keyword == key)
}

pub fn parse_kvn_state_vector(
    input: &str,
) -> Result<KvnStateVectorValue, KvnStateVectorParserErr<&str>> {
    // This line is written in regex hell
    let re = Regex::new(
        r"^(?:\s*)?(?<full_date_value>(?<yr>(?:\d{4}))-(?<mo>(?:\d{1,2}))-(?<dy>(?:\d{1,2}))T(?<hr>(?:\d{1,2})):(?<mn>(?:\d{1,2})):(?<sc>(?:\d{0,2}(?:\.\d*)?)))(?:\s+)(?<x>(?:(?:[^ ]*)?))(?:\s+)(?<y>(?:(?:[^ ]*)?))(?:\s+)(?<z>(?:(?:[^ ]*)?))(?:\s+)(?<x_dot>(?:(?:[^ ]*)?))(?:\s+)(?<y_dot>(?:(?:[^ ]*)?))(?:\s+)(?<z_dot>(?:(?:[^ ]*)?))((?:\s+)(?<x_ddot>(?:(?:[^ ]*)?))(?:\s+)(?<y_ddot>(?:(?:[^ ]*)?))(?:\s+)(?<z_ddot>(?:(?:[^ ]*)?)))?(?:\s*)$",)
    .unwrap();

    let captures = re
        .captures(input)
        .ok_or(KvnStateVectorParserErr::InvalidFormat { input })?;

    let datetime = handle_datetime_capture(&captures);

    let x = captures.name("x").unwrap().as_str().parse::<f64>().unwrap();
    let y = captures.name("y").unwrap().as_str().parse::<f64>().unwrap();
    let z = captures.name("z").unwrap().as_str().parse::<f64>().unwrap();

    let x_dot = captures
        .name("x_dot")
        .unwrap()
        .as_str()
        .parse::<f64>()
        .unwrap();
    let y_dot = captures
        .name("y_dot")
        .unwrap()
        .as_str()
        .parse::<f64>()
        .unwrap();
    let z_dot = captures
        .name("z_dot")
        .unwrap()
        .as_str()
        .parse::<f64>()
        .unwrap();

    let x_ddot = captures
        .name("x_ddot")
        .map(|x| x.as_str().parse::<f64>().unwrap());
    let y_ddot = captures
        .name("y_ddot")
        .map(|x| x.as_str().parse::<f64>().unwrap());
    let z_ddot = captures
        .name("z_ddot")
        .map(|x| x.as_str().parse::<f64>().unwrap());

    Ok(KvnStateVectorValue {
        epoch: datetime,
        x,
        y,
        z,
        x_dot,
        y_dot,
        z_dot,
        x_ddot,
        y_ddot,
        z_ddot,
    })
}

fn parse_kvn_covariance_matrix_line<'a, T: Iterator<Item = &'a str> + ?Sized>(
    input: &mut T,
    expected_count: usize,
) -> Result<Vec<f64>, KvnCovarianceMatrixParserErr<&'a str>> {
    let next_line = input
        .next()
        .ok_or(KvnCovarianceMatrixParserErr::UnexpectedEndOfInput {
            keyword: "COVARIANCE_MATRIX ",
        })?;

    let result: Result<Vec<f64>, _> = next_line
        .split_whitespace()
        .map(|matrix_element| {
            matrix_element
                .trim()
                .parse::<f64>()
                .map_err(|_| KvnCovarianceMatrixParserErr::InvalidFormat { input: next_line })
        })
        .collect();

    let result = result?;

    if result.len() != expected_count {
        return Err(KvnCovarianceMatrixParserErr::InvalidItemCount { input: next_line });
    }

    Ok(result)
}

pub fn parse_kvn_covariance_matrix<'a, T: Iterator<Item = &'a str> + ?Sized>(
    input: &mut T,
) -> Result<KvnCovarianceMatrixValue, KvnCovarianceMatrixParserErr<&'a str>> {
    let tokenized_line = parse_kvn_covariance_matrix_line(input, 1)?;

    // Unwrap is okay because we check the number of elements before
    let mut iter = tokenized_line.iter();
    let cx_x = *iter.next().unwrap();

    let tokenized_line = parse_kvn_covariance_matrix_line(input, 2)?;

    // Unwrap is okay because we check the number of elements before
    let mut iter = tokenized_line.iter();
    let cy_x = *iter.next().unwrap();
    let cy_y = *iter.next().unwrap();

    let tokenized_line = parse_kvn_covariance_matrix_line(input, 3)?;

    // Unwrap is okay because we check the number of elements before
    let mut iter = tokenized_line.iter();
    let cz_x = *iter.next().unwrap();
    let cz_y = *iter.next().unwrap();
    let cz_z = *iter.next().unwrap();

    let tokenized_line = parse_kvn_covariance_matrix_line(input, 4)?;

    // Unwrap is okay because we check the number of elements before
    let mut iter = tokenized_line.iter();
    let cx_dot_x = *iter.next().unwrap();
    let cx_dot_y = *iter.next().unwrap();
    let cx_dot_z = *iter.next().unwrap();
    let cx_dot_x_dot = *iter.next().unwrap();

    let tokenized_line = parse_kvn_covariance_matrix_line(input, 5)?;

    // Unwrap is okay because we check the number of elements before
    let mut iter = tokenized_line.iter();
    let cy_dot_x = *iter.next().unwrap();
    let cy_dot_y = *iter.next().unwrap();
    let cy_dot_z = *iter.next().unwrap();
    let cy_dot_x_dot = *iter.next().unwrap();
    let cy_dot_y_dot = *iter.next().unwrap();

    let tokenized_line = parse_kvn_covariance_matrix_line(input, 6)?;

    // Unwrap is okay because we check the number of elements before
    let mut iter = tokenized_line.iter();
    let cz_dot_x = *iter.next().unwrap();
    let cz_dot_y = *iter.next().unwrap();
    let cz_dot_z = *iter.next().unwrap();
    let cz_dot_x_dot = *iter.next().unwrap();
    let cz_dot_y_dot = *iter.next().unwrap();
    let cz_dot_z_dot = *iter.next().unwrap();

    Ok(KvnCovarianceMatrixValue {
        cx_x,
        cy_x,
        cy_y,
        cz_x,
        cz_y,
        cz_z,
        cx_dot_x,
        cx_dot_y,
        cx_dot_z,
        cx_dot_x_dot,
        cy_dot_x,
        cy_dot_y,
        cy_dot_z,
        cy_dot_x_dot,
        cy_dot_y_dot,
        cz_dot_x,
        cz_dot_y,
        cz_dot_z,
        cz_dot_x_dot,
        cz_dot_y_dot,
        cz_dot_z_dot,
    })
}

pub fn parse_kvn_string_line(
    input: &str,
) -> Result<KvnValue<String, String>, KvnStringParserErr<&str>> {
    if input.trim_start().starts_with("COMMENT ") {
        return Ok(KvnValue {
            value: input
                .trim_start()
                .trim_start_matches("COMMENT")
                .trim_start()
                .to_string(),
            unit: None,
        });
    }

    if is_empty_value(input) {
        Err(KvnStringParserErr::EmptyValue { input })?
    };

    // Inspired by figure F-8: CCSDS 502.0-B-3, but accepts a more relaxed input. Orekit seems to suggest that there
    // are quite a few messages being used which are not strictly compliant.
    let re =
        Regex::new(r"^(?:\s*)(?<keyword>[0-9A-Z_]*)(?:\s*)=(?:\s*)(?<value>(?:(?:.*)))(?:\s*)$")
            .unwrap();

    let captures = re
        .captures(input)
        .ok_or(KvnStringParserErr::InvalidFormat { input })?;

    let keyword = captures
        .name("keyword")
        // This unwrap is okay because the keyword uses * so it will always capture
        .unwrap()
        .as_str()
        .trim_end()
        .to_string();

    if keyword.is_empty() {
        return Err(KvnStringParserErr::EmptyKeyword { input });
    }

    let value = captures
        .name("value")
        // This unwrap is okay because the value uses * so it will always capture
        .unwrap()
        .as_str()
        .trim_end()
        .to_string();

    if value.is_empty() {
        return Err(KvnStringParserErr::EmptyValue { input });
    }

    Ok(KvnValue { value, unit: None })
}

pub fn parse_kvn_integer_line<T>(
    input: &str,
    with_unit: bool,
) -> Result<KvnValue<T, String>, KvnNumberParserErr<&str>>
where
    T: std::str::FromStr,
{
    if is_empty_value(input) {
        Err(KvnNumberParserErr::EmptyValue { input })?
    };

    let regex_pattern = if with_unit {
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?<value>(?:[-+]?)(?:[0-9]+)(?:\.\d*)?)(?:(?:\s*)(?:\[(?<unit>[0-9A-Za-z/_*]*)\]?))?(?:\s*)?$"
    } else {
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?<value>(?:[-+]?)(?:[0-9]+)(?:\.\d*)?)(?:\s*)$"
    };

    // Modified from Figure F-9: CCSDS 502.0-B-3
    let re = Regex::new(regex_pattern).unwrap();

    let captures = re
        .captures(input)
        .ok_or(KvnNumberParserErr::InvalidFormat { input })?;

    let keyword = captures
        .name("keyword")
        // This unwrap is okay because the keyword is marked as * so it will always capture
        .unwrap()
        .as_str()
        .trim_end()
        .to_string();

    if keyword.is_empty() {
        return Err(KvnNumberParserErr::EmptyKeyword { input });
    }

    // This unwrap is okay because the value uses * so it will always capture
    let value = captures.name("value").unwrap().as_str();
    let unit = captures.name("unit").map(|x| x.as_str().to_string());

    let value = value
        .parse::<T>()
        .map_err(|_| KvnNumberParserErr::InvalidFormat { input })?;

    Ok(KvnValue { value, unit })
}

fn is_empty_value(input: &str) -> bool {
    let re = Regex::new(
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?:\[(?<unit>[0-9A-Za-z/_*]*)\]?)?$",
    )
    .unwrap();

    re.is_match(input)
}

pub fn parse_kvn_numeric_line(
    input: &str,
    with_unit: bool,
) -> Result<KvnValue<f64, String>, KvnNumberParserErr<&str>> {
    if is_empty_value(input) {
        Err(KvnNumberParserErr::EmptyValue { input })?
    };

    let regex_pattern = if with_unit {
        // Figure F-9: CCSDS 502.0-B-3
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?<value>(?:[-+]?)(?:[0-9]+)(?:\.\d*)?(?:[eE][+-]?(?:\d+))?)(?:(?:\s*)(?:\[(?<unit>[0-9A-Za-z/_*]*)\]?))?(?:\s*)?$"
    } else {
        r"^(?:\s*)(?<keyword>[0-9A-Za-z_]*)(?:\s*)=(?:\s*)(?<value>(?:[-+]?)(?:[0-9]+)(?:\.\d*)?(?:[eE][+-]?(?:\d+))?)(?:\s*)?$"
    };

    let re = Regex::new(regex_pattern).unwrap();

    let captures = re
        .captures(input)
        .ok_or(KvnNumberParserErr::InvalidFormat { input })?;

    let keyword = captures
        .name("keyword")
        // This unwrap is okay because the keyword is marked as * so it will always capture
        .unwrap()
        .as_str()
        .trim_end()
        .to_string();

    if keyword.is_empty() {
        return Err(KvnNumberParserErr::EmptyKeyword { input });
    }

    // This unwrap is okay because the value uses * so it will always capture
    let value = captures.name("value").unwrap().as_str();
    let unit = captures.name("unit").map(|x| x.as_str().to_string());

    let value = value
        .parse::<f64>()
        .map_err(|_| KvnNumberParserErr::InvalidFormat { input })?;

    Ok(KvnValue { value, unit })
}

pub fn handle_datetime_capture(captures: &regex::Captures) -> KvnDateTimeValue {
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

    let full_value = captures
        .name("full_date_value")
        .unwrap()
        .as_str()
        .to_string();

    KvnDateTimeValue {
        year,
        month,
        day,
        hour,
        minute,
        second,
        fractional_second,
        full_value,
    }
}

pub fn parse_kvn_datetime_line(
    input: &str,
) -> Result<KvnDateTimeValue, KvnDateTimeParserErr<&str>> {
    if is_empty_value(input) {
        Err(KvnDateTimeParserErr::EmptyValue { input })?
    };

    // Modified from Figure F-5: CCSDS 502.0-B-3
    let re = Regex::new(r"^(?:\s*)?(?<keyword>[0-9A-Z_]*)(?:\s*)?=(?:\s*)?(?<full_date_value>(?<yr>(?:\d{4}))-(?<mo>(?:\d{1,2}))-(?<dy>(?:\d{1,2}))T(?<hr>(?:\d{1,2})):(?<mn>(?:\d{1,2})):(?<sc>(?:\d{0,2}(?:\.\d*)?)))(?:\s*)?$").unwrap();

    let captures = re
        .captures(input)
        .ok_or(KvnDateTimeParserErr::InvalidFormat { input })?;

    let keyword = captures
        // This unwrap is okay because the keyword is marked as * so it will always capture
        .name("keyword")
        .unwrap()
        .as_str()
        .trim_end()
        .to_string();

    if keyword.is_empty() {
        return Err(KvnDateTimeParserErr::EmptyKeyword { input });
    }

    Ok(handle_datetime_capture(&captures))
}

#[cfg(test)]
mod test {
    use lox_derive::KvnDeserialize;

    use super::*;

    #[test]
    fn test_parse_kvn_string_line() {
        // 7.5.1 A non-empty value field must be assigned to each mandatory keyword except for *‘_START’ and *‘_STOP’ keyword values
        // 7.4.6 Any white space immediately preceding or following the ‘equals’ sign shall not be significant.
        assert_eq!(
            parse_kvn_string_line("ASD = ASDFG"),
            Ok(KvnValue {
                value: "ASDFG".to_string(),
                unit: None
            })
        );
        assert_eq!(
            parse_kvn_string_line("ASD    =   ASDFG"),
            Ok(KvnValue {
                value: "ASDFG".to_string(),
                unit: None
            })
        );
        assert_eq!(
            parse_kvn_string_line("ASD    = ASDFG"),
            Ok(KvnValue {
                value: "ASDFG".to_string(),
                unit: None
            })
        );
        assert_eq!(
            parse_kvn_string_line("ASD =    "),
            Err(KvnStringParserErr::EmptyValue { input: "ASD =    " })
        );
        assert_eq!(
            parse_kvn_string_line("ASD = "),
            Err(KvnStringParserErr::EmptyValue { input: "ASD = " })
        );
        assert_eq!(
            parse_kvn_string_line("ASD ="),
            Err(KvnStringParserErr::EmptyValue { input: "ASD =" })
        );

        assert_eq!(
            parse_kvn_string_line("ASD   [km]"),
            Err(KvnStringParserErr::InvalidFormat {
                input: "ASD   [km]"
            })
        );
        assert_eq!(
            parse_kvn_string_line(" = asd [km]"),
            Err(KvnStringParserErr::EmptyKeyword {
                input: " = asd [km]"
            })
        );

        // 7.4.7 Any white space immediately preceding the end of line shall not be significant.
        assert_eq!(
            parse_kvn_string_line("ASD = ASDFG          "),
            Ok(KvnValue {
                value: "ASDFG".to_string(),
                unit: None
            })
        );

        // 7.4.5 Any white space immediately preceding or following the keyword shall not be significant.
        assert_eq!(
            parse_kvn_string_line("  ASD  = ASDFG"),
            Ok(KvnValue {
                value: "ASDFG".to_string(),
                unit: None
            })
        );

        // 7.8.5 All comment lines shall begin with the ‘COMMENT’ keyword followed by at least one space.
        // [...] White space shall be retained (shall be significant) in comment values.

        assert_eq!(
            parse_kvn_string_line("  COMMENT asd a    asd a ads as "),
            Ok(KvnValue {
                value: "asd a    asd a ads as ".to_string(),
                unit: None
            })
        );

        assert_eq!(
            parse_kvn_string_line("  COMMENT "),
            Ok(KvnValue {
                value: "".to_string(),
                unit: None
            })
        );
    }

    #[test]
    fn test_parse_kvn_integer_line() {
        // a) there must be at least one blank character between the value and the units text;
        // b) the units must be enclosed within square brackets (e.g., ‘[m]’);
        assert_eq!(
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH = 28800 [s]", true),
            Ok(KvnValue {
                value: 28800,
                unit: Some("s".to_string())
            },)
        );

        // 7.4.7 Any white space immediately preceding the end of line shall not be significant.

        assert_eq!(
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH = 28800             [s]", true),
            Ok(KvnValue {
                value: 28800,
                unit: Some("s".to_string())
            })
        );

        assert_eq!(
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH = 28800             ", false),
            Ok(KvnValue {
                value: 28800,
                unit: None
            })
        );

        // 7.4.5 Any white space immediately preceding or following the keyword shall not be significant.

        assert_eq!(
            parse_kvn_integer_line("          SCLK_OFFSET_AT_EPOCH = 28800", false),
            Ok(KvnValue {
                value: 28800,
                unit: None
            })
        );

        assert_eq!(
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH = 00028800 [s]", true),
            Ok(KvnValue {
                value: 28800,
                unit: Some("s".to_string())
            },)
        );

        assert_eq!(
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH = -28800 [s]", true),
            Ok(KvnValue {
                value: -28800,
                unit: Some("s".to_string())
            },)
        );

        assert_eq!(
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH = -28800", true),
            Ok(KvnValue {
                value: -28800,
                unit: None
            },)
        );

        assert_eq!(
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH = 28800 [s]", true),
            Ok(KvnValue {
                value: 28800,
                unit: Some("s".to_string())
            },)
        );

        assert_eq!(
            parse_kvn_integer_line::<u32>("SCLK_OFFSET_AT_EPOCH = 28800 [s]", false),
            Err(KvnNumberParserErr::InvalidFormat {
                input: "SCLK_OFFSET_AT_EPOCH = 28800 [s]"
            })
        );

        assert_eq!(
            parse_kvn_integer_line::<u32>("SCLK_OFFSET_AT_EPOCH = -asd", true),
            Err(KvnNumberParserErr::InvalidFormat {
                input: "SCLK_OFFSET_AT_EPOCH = -asd"
            })
        );

        assert_eq!(
            parse_kvn_integer_line::<u32>("SCLK_OFFSET_AT_EPOCH = [s]", true),
            Err(KvnNumberParserErr::EmptyValue {
                input: "SCLK_OFFSET_AT_EPOCH = [s]"
            })
        );

        assert_eq!(
            parse_kvn_integer_line::<u32>("SCLK_OFFSET_AT_EPOCH =    ", false),
            Err(KvnNumberParserErr::EmptyValue {
                input: "SCLK_OFFSET_AT_EPOCH =    "
            })
        );
        assert_eq!(
            parse_kvn_integer_line::<u32>("SCLK_OFFSET_AT_EPOCH = ", false),
            Err(KvnNumberParserErr::EmptyValue {
                input: "SCLK_OFFSET_AT_EPOCH = "
            })
        );
        assert_eq!(
            parse_kvn_integer_line::<u32>("SCLK_OFFSET_AT_EPOCH =", false),
            Err(KvnNumberParserErr::EmptyValue {
                input: "SCLK_OFFSET_AT_EPOCH ="
            })
        );

        assert_eq!(
            parse_kvn_integer_line::<u32>("SCLK_OFFSET_AT_EPOCH   [km]", true),
            Err(KvnNumberParserErr::InvalidFormat {
                input: "SCLK_OFFSET_AT_EPOCH   [km]"
            })
        );
        assert_eq!(
            parse_kvn_integer_line::<u32>(" = 123 [km]", true),
            Err(KvnNumberParserErr::EmptyKeyword {
                input: " = 123 [km]"
            })
        );
    }

    #[test]
    fn test_parse_kvn_numeric_line() {
        // a) there must be at least one blank character between the value and the units text;
        // b) the units must be enclosed within square brackets (e.g., ‘[m]’);
        assert_eq!(
            parse_kvn_numeric_line("X = 66559942 [km]", true),
            Ok(KvnValue {
                value: 66559942f64,
                unit: Some("km".to_string())
            },)
        );

        // 7.4.7 Any white space immediately preceding the end of line shall not be significant.

        assert_eq!(
            parse_kvn_numeric_line("X = 66559942             [km]", true),
            Ok(KvnValue {
                value: 66559942f64,
                unit: Some("km".to_string())
            })
        );

        assert_eq!(
            parse_kvn_numeric_line("X = 66559942             ", false),
            Ok(KvnValue {
                value: 66559942f64,
                unit: None
            })
        );

        // 7.4.5 Any white space immediately preceding or following the keyword shall not be significant.

        assert_eq!(
            parse_kvn_numeric_line("          X = 66559942", false),
            Ok(KvnValue {
                value: 66559942f64,
                unit: None
            })
        );

        assert_eq!(
            parse_kvn_numeric_line("X = 6655.9942 [km]", true),
            Ok(KvnValue {
                value: 6655.9942,
                unit: Some("km".to_string())
            },)
        );

        assert_eq!(
            parse_kvn_numeric_line("CX_X =  5.801003223606e-05", true),
            Ok(KvnValue {
                value: 5.801003223606e-05,
                unit: None
            },)
        );

        assert_eq!(
            parse_kvn_numeric_line("X = -asd", true),
            Err(KvnNumberParserErr::InvalidFormat { input: "X = -asd" })
        );

        assert_eq!(
            parse_kvn_numeric_line("X = [s]", true),
            Err(KvnNumberParserErr::EmptyValue { input: "X = [s]" })
        );

        assert_eq!(
            parse_kvn_numeric_line("X =    ", false),
            Err(KvnNumberParserErr::EmptyValue { input: "X =    " })
        );
        assert_eq!(
            parse_kvn_numeric_line("X = ", false),
            Err(KvnNumberParserErr::EmptyValue { input: "X = " })
        );
        assert_eq!(
            parse_kvn_numeric_line("X =", false),
            Err(KvnNumberParserErr::EmptyValue { input: "X =" })
        );

        assert_eq!(
            parse_kvn_numeric_line("X   [km]", true),
            Err(KvnNumberParserErr::InvalidFormat { input: "X   [km]" })
        );
        assert_eq!(
            parse_kvn_numeric_line(" = 123 [km]", true),
            Err(KvnNumberParserErr::EmptyKeyword {
                input: " = 123 [km]"
            })
        );
    }

    #[test]
    fn test_parse_kvn_datetime_line() {
        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE = 2021-06-03T05:33:00.123"),
            Ok(KvnDateTimeValue {
                year: 2021,
                month: 6,
                day: 3,
                hour: 5,
                minute: 33,
                second: 0,
                fractional_second: 0.123,
                full_value: "2021-06-03T05:33:00.123".to_string(),
            })
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE = 2021-06-03T05:33:01"),
            Ok(KvnDateTimeValue {
                year: 2021,
                month: 6,
                day: 3,
                hour: 5,
                minute: 33,
                second: 1,
                fractional_second: 0.0,
                full_value: "2021-06-03T05:33:01".to_string(),
            })
        );

        // 7.4.7 Any white space immediately preceding the end of line shall not be significant.

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE = 2021-06-03T05:33:01           "),
            Ok(KvnDateTimeValue {
                year: 2021,
                month: 6,
                day: 3,
                hour: 5,
                minute: 33,
                second: 1,
                fractional_second: 0.0,
                full_value: "2021-06-03T05:33:01".to_string(),
            })
        );

        // 7.4.5 Any white space immediately preceding or following the keyword shall not be significant.

        assert_eq!(
            parse_kvn_datetime_line("          CREATION_DATE = 2021-06-03T05:33:01"),
            Ok(KvnDateTimeValue {
                year: 2021,
                month: 6,
                day: 3,
                hour: 5,
                minute: 33,
                second: 1,
                fractional_second: 0.0,
                full_value: "2021-06-03T05:33:01".to_string(),
            })
        );

        // @TODO add support for ddd format

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE = 2021,06,03Q05!33!00-123"),
            Err(KvnDateTimeParserErr::InvalidFormat {
                input: "CREATION_DATE = 2021,06,03Q05!33!00-123"
            })
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE = asdffggg"),
            Err(KvnDateTimeParserErr::InvalidFormat {
                input: "CREATION_DATE = asdffggg"
            })
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE = "),
            Err(KvnDateTimeParserErr::EmptyValue {
                input: "CREATION_DATE = "
            })
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE =    "),
            Err(KvnDateTimeParserErr::EmptyValue {
                input: "CREATION_DATE =    "
            })
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE ="),
            Err(KvnDateTimeParserErr::EmptyValue {
                input: "CREATION_DATE ="
            })
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE     "),
            Err(KvnDateTimeParserErr::InvalidFormat {
                input: "CREATION_DATE     "
            })
        );
        assert_eq!(
            parse_kvn_datetime_line(" = 2021-06-03T05:33:01"),
            Err(KvnDateTimeParserErr::EmptyKeyword {
                input: " = 2021-06-03T05:33:01"
            })
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
        pub version: String,
        pub semi_major_axis: DistanceType,
        pub asdfg: f64,
    }

    #[test]
    fn test_parse_with_unit_struct() {
        let kvn = r#"CCSDS_ASD_VERS = 3.0
        SEMI_MAJOR_AXIS = 41399.5123 [km]
        ASDFG = 12333.5123"#;

        assert_eq!(
            crate::ndm::kvn::KvnDeserializer::deserialize(&mut kvn.lines().peekable()),
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

    #[test]
    fn test_state_vector_parser() {
        // 5.2.4.1 Each set of ephemeris data, including the time tag, must be
        // provided on a single line. The order in which data items are given
        // shall be fixed: Epoch, X, Y, Z, X_DOT, Y_DOT, Z_DOT, X_DDOT, Y_DDOT, Z_DDOT.

        assert_eq!(
            parse_kvn_state_vector(
                "1996-12-28T21:29:07.0 -2432.166 -063.042 1742.754 7.33702 -3.495867 -1.041945"
            ),
            Ok(KvnStateVectorValue {
                epoch: KvnDateTimeValue {
                    year: 1996,
                    month: 12,
                    day: 28,
                    hour: 21,
                    minute: 29,
                    second: 7,
                    fractional_second: 0.0,
                    full_value: "1996-12-28T21:29:07.0".to_string(),
                },
                x: -2432.166,
                y: -63.042,
                z: 1742.754,
                x_dot: 7.33702,
                y_dot: -3.495867,
                z_dot: -1.041945,
                x_ddot: None,
                y_ddot: None,
                z_ddot: None
            })
        );

        // 5.2.4.2 The position and velocity terms shall be mandatory;
        // acceleration terms may be provided

        assert_eq!(
            parse_kvn_state_vector(
                "1996-12-28T21:29:07.0 -2432.166 -063.042 1742.754 7.33702 -3.495867 -1.041945 1.234 -2.345 3.455"
            ),
            Ok(KvnStateVectorValue {
                epoch: KvnDateTimeValue {
                    year: 1996,
                    month: 12,
                    day: 28,
                    hour: 21,
                    minute: 29,
                    second: 7,
                    fractional_second: 0.0,
                    full_value: "1996-12-28T21:29:07.0".to_string(),
                },
                x: -2432.166,
                y: -63.042,
                z: 1742.754,
                x_dot: 7.33702,
                y_dot: -3.495867,
                z_dot: -1.041945,
                x_ddot: Some(1.234),
                y_ddot: Some(-2.345),
                z_ddot: Some(3.455),
            })
        );

        // 5.2.4.3 At least one space character must be used to separate the
        // items in each ephemeris data line.

        assert_eq!(
            parse_kvn_state_vector(
                "          1996-12-28T21:29:07.0             -2432.166         -063.042       1742.754        7.33702        -3.495867       -1.041945       1.234       -2.345        3.455      "
            ),
            Ok(KvnStateVectorValue {
                epoch: KvnDateTimeValue {
                    year: 1996,
                    month: 12,
                    day: 28,
                    hour: 21,
                    minute: 29,
                    second: 7,
                    fractional_second: 0.0,
                    full_value: "1996-12-28T21:29:07.0".to_string(),
                },
                x: -2432.166,
                y: -63.042,
                z: 1742.754,
                x_dot: 7.33702,
                y_dot: -3.495867,
                z_dot: -1.041945,
                x_ddot: Some(1.234),
                y_ddot: Some(-2.345),
                z_ddot: Some(3.455),
            })
        );
    }

    #[test]
    fn test_covariance_matrix_parser() {
        // 5.2.5.4 Values in the covariance matrix shall be expressed in the
        // applicable reference frame (COV_REF_FRAME keyword if used, or
        // REF_FRAME keyword if not), and shall be presented sequentially from
        // upper left [1,1] to lower right [6,6], lower triangular form, row by
        // row left to right. Variance and covariance values shall be expressed
        // in standard double precision as related in 7.5.

        let kvn = "3.3313494e-04
4.6189273e-04 6.7824216e-04
-3.0700078e-04 -4.2212341e-04 3.2319319e-04
-3.3493650e-07 -4.6860842e-07 2.4849495e-07 4.2960228e-10
-2.2118325e-07 -2.8641868e-07 1.7980986e-07 2.6088992e-10 1.7675147e-10
-3.0413460e-07 -4.9894969e-07 3.5403109e-07 1.8692631e-10 1.0088625e-10 6.2244443e-10";

        assert_eq!(
            parse_kvn_covariance_matrix(&mut kvn.lines()),
            Ok(KvnCovarianceMatrixValue {
                cx_x: 3.3313494e-04,
                cy_x: 4.6189273e-04,
                cy_y: 6.7824216e-04,
                cz_x: -3.0700078e-04,
                cz_y: -4.2212341e-04,
                cz_z: 3.2319319e-04,
                cx_dot_x: -3.3493650e-07,
                cx_dot_y: -4.6860842e-07,
                cx_dot_z: 2.4849495e-07,
                cx_dot_x_dot: 4.2960228e-10,
                cy_dot_x: -2.2118325e-07,
                cy_dot_y: -2.8641868e-07,
                cy_dot_z: 1.7980986e-07,
                cy_dot_x_dot: 2.6088992e-10,
                cy_dot_y_dot: 1.7675147e-10,
                cz_dot_x: -3.0413460e-07,
                cz_dot_y: -4.9894969e-07,
                cz_dot_z: 3.5403109e-07,
                cz_dot_x_dot: 1.8692631e-10,
                cz_dot_y_dot: 1.0088625e-10,
                cz_dot_z_dot: 6.2244443e-10,
            })
        );

        // 5.2.5.5 At least one space character must be used to separate the
        // items in each covariance matrix data line

        let kvn = "  3.3313494e-04
  4.6189273e-04     6.7824216e-04   
-3.0700078e-04     -4.2212341e-04    3.2319319e-04      
  -3.3493650e-07     -4.6860842e-07   2.4849495e-07  4.2960228e-10
-2.2118325e-07  -2.8641868e-07     1.7980986e-07    2.6088992e-10   1.7675147e-10           
  -3.0413460e-07  -4.9894969e-07  3.5403109e-07 1.8692631e-10  1.0088625e-10 6.2244443e-10 ";

        assert_eq!(
            parse_kvn_covariance_matrix(&mut kvn.lines()),
            Ok(KvnCovarianceMatrixValue {
                cx_x: 3.3313494e-04,
                cy_x: 4.6189273e-04,
                cy_y: 6.7824216e-04,
                cz_x: -3.0700078e-04,
                cz_y: -4.2212341e-04,
                cz_z: 3.2319319e-04,
                cx_dot_x: -3.3493650e-07,
                cx_dot_y: -4.6860842e-07,
                cx_dot_z: 2.4849495e-07,
                cx_dot_x_dot: 4.2960228e-10,
                cy_dot_x: -2.2118325e-07,
                cy_dot_y: -2.8641868e-07,
                cy_dot_z: 1.7980986e-07,
                cy_dot_x_dot: 2.6088992e-10,
                cy_dot_y_dot: 1.7675147e-10,
                cz_dot_x: -3.0413460e-07,
                cz_dot_y: -4.9894969e-07,
                cz_dot_z: 3.5403109e-07,
                cz_dot_x_dot: 1.8692631e-10,
                cz_dot_y_dot: 1.0088625e-10,
                cz_dot_z_dot: 6.2244443e-10,
            })
        );
    }
}
