/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

// KVN spec section 7.4 of https://public.ccsds.org/Pubs/502x0b3e1.pdf

use lox_derive::KvnDeserialize;
use nom::bytes::complete as nb;
use nom::character::complete as nc;
use nom::sequence as ns;
use regex::Regex;

type KvnStringValue = KvnValue<String, String>;
type KvnIntegerValue = KvnValue<i32, String>;
type KvnNumericValue = KvnValue<f64, String>;

#[derive(KvnDeserialize, Default, Debug, PartialEq)]
pub struct Opm {
    //@TODO the unit is always fixed and the same
    ccsds_opm_vers: KvnStringValue,
    comment: KvnStringValue,
    creation_date: KvnDateTimeValue,
    originator: KvnStringValue,
    object_name: KvnStringValue,
    object_id: KvnStringValue,
    center_name: KvnStringValue,
    ref_frame: KvnStringValue,
    // ref_frame_epoch: KvnDateTimeValue,
    time_system: KvnStringValue,
    // comment: KvnStringValue,
    epoch: KvnDateTimeValue,
    x: KvnNumericValue,
    y: KvnNumericValue,
    z: KvnNumericValue,
    x_dot: KvnNumericValue,
    y_dot: KvnNumericValue,
    z_dot: KvnNumericValue,
    // comment: KvnStringValue,
    semi_major_axis: KvnNumericValue,
    eccentricity: KvnNumericValue, //@TODO no unit
    inclination: KvnNumericValue,
    ra_of_asc_node: KvnNumericValue,
    arg_of_pericenter: KvnNumericValue,
    true_anomaly: KvnNumericValue,
    gm: KvnNumericValue,
    // comment: KvnStringValue,
    mass: KvnNumericValue,
    solar_rad_area: KvnNumericValue,
    solar_rad_coeff: KvnNumericValue,
    drag_area: KvnNumericValue,
    drag_coeff: KvnNumericValue, //@TODO no unit
    // comment: KvnStringValue,
    man_epoch_ignition: KvnDateTimeValue,
    man_duration: KvnNumericValue,
    man_delta_mass: KvnNumericValue,
    man_ref_frame: KvnStringValue,
    man_dv_1: KvnNumericValue,
    man_dv_2: KvnNumericValue,
    man_dv_3: KvnNumericValue,
}

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
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    fractional_second: f64,
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

fn parse_kvn_string_line<'a>(
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

fn parse_kvn_integer_line<'a>(
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

fn parse_kvn_numeric_line<'a>(
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

fn parse_kvn_datetime_line<'a>(
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

    #[test]
    fn test_parse_json() {
        /*
        This is the original file with multi-line comments:

        CCSDS_OPM_VERS = 3.0
        COMMENT Generated by GSOC, R. Kiehling
        COMMENT Current intermediate orbit IO2 and maneuver planning data
        CREATION_DATE = 2021-06-03T05:33:00.123
        ORIGINATOR = GSOC
        OBJECT_NAME = EUTELSAT W4
        OBJECT_ID = 2021-028A
        CENTER_NAME = EARTH
        REF_FRAME = TOD
        TIME_SYSTEM = UTC
        COMMENT State Vector
        EPOCH = 2021-06-03T00:00:00.000
        X = 6655.9942 [km]
        Y = -40218.5751 [km]
        Z = -82.9177 [km]
        X_DOT = 3.11548208 [km/s]
        Y_DOT = 0.47042605 [km/s]
        Z_DOT = -0.00101495 [km/s]
        COMMENT Keplerian elements
        SEMI_MAJOR_AXIS = 41399.5123 [km]
        ECCENTRICITY = 0.020842611
        INCLINATION = 0.117746 [deg]
        RA_OF_ASC_NODE = 17.604721 [deg]
        ARG_OF_PERICENTER = 218.242943 [deg]
        TRUE_ANOMALY = 41.922339 [deg]
        GM = 398600.4415 [km**3/s**2]
        COMMENT Spacecraft parameters
        MASS = 1913.000 [kg]
        SOLAR_RAD_AREA = 10.000 [m**2]
        SOLAR_RAD_COEFF = 1.300
        DRAG_AREA = 10.000 [m**2]
        DRAG_COEFF = 2.300
        COMMENT 2 planned maneuvers
        COMMENT First maneuver: AMF-3
        COMMENT Non-impulsive, thrust direction fixed in inertial frame
        MAN_EPOCH_IGNITION = 2021-06-03T09:00:34.1
        MAN_DURATION = 132.60 [s]
        MAN_DELTA_MASS = -18.418 [kg]
        MAN_REF_FRAME = EME2000
        MAN_DV_1 = -0.02325700 [km/s]
        MAN_DV_2 = 0.01683160 [km/s]
        MAN_DV_3 = -0.00893444 [km/s]
        COMMENT Second maneuver: first station acquisition maneuver
        COMMENT impulsive, thrust direction fixed in RTN frame
        MAN_EPOCH_IGNITION = 2021-06-05T18:59:21.0
        MAN_DURATION = 0.00 [s]
        MAN_DELTA_MASS = -1.469 [kg]
        MAN_REF_FRAME = RTN
        MAN_DV_1 = 0.00101500 [km/s]
        MAN_DV_2 = -0.00187300 [km/s]
        MAN_DV_3 = 0.00000000 [km/s]
        */

        let kvn = r#"CCSDS_OPM_VERS = 3.0
COMMENT Generated by GSOC, R. Kiehling
CREATION_DATE = 2021-06-03T05:33:00.123
ORIGINATOR = GSOC
OBJECT_NAME = EUTELSAT W4
OBJECT_ID = 2021-028A
CENTER_NAME = EARTH
REF_FRAME = TOD
TIME_SYSTEM = UTC
EPOCH = 2021-06-03T00:00:00.000
X = 6655.9942 [km]
Y = -40218.5751 [km]
Z = -82.9177 [km]
X_DOT = 3.11548208 [km/s]
Y_DOT = 0.47042605 [km/s]
Z_DOT = -0.00101495 [km/s]
SEMI_MAJOR_AXIS = 41399.5123 [km]
ECCENTRICITY = 0.020842611
INCLINATION = 0.117746 [deg]
RA_OF_ASC_NODE = 17.604721 [deg]
ARG_OF_PERICENTER = 218.242943 [deg]
TRUE_ANOMALY = 41.922339 [deg]
GM = 398600.4415 [km**3/s**2]
MASS = 1913.000 [kg]
SOLAR_RAD_AREA = 10.000 [m**2]
SOLAR_RAD_COEFF = 1.300
DRAG_AREA = 10.000 [m**2]
DRAG_COEFF = 2.300
MAN_EPOCH_IGNITION = 2021-06-03T09:00:34.1
MAN_DURATION = 132.60 [s]
MAN_DELTA_MASS = -18.418 [kg]
MAN_REF_FRAME = EME2000
MAN_DV_1 = -0.02325700 [km/s]
MAN_DV_2 = 0.01683160 [km/s]
MAN_DV_3 = -0.00893444 [km/s]
MAN_EPOCH_IGNITION = 2021-06-05T18:59:21.0
MAN_DURATION = 0.00 [s]
MAN_DELTA_MASS = -1.469 [kg]
MAN_REF_FRAME = RTN
MAN_DV_1 = 0.00101500 [km/s]
MAN_DV_2 = -0.00187300 [km/s]
MAN_DV_3 = 0.00000000 [km/s]"#;

        let mut lines = kvn.lines();

        let opm = Opm::deserialize(&mut lines);

        assert_eq!(
            opm,
            Ok(Opm {
                ccsds_opm_vers: KvnValue {
                    value: "3.0".to_string(),
                    unit: None,
                },
                comment: KvnValue {
                    value: "Generated by GSOC, R. Kiehling".to_string(),
                    unit: None,
                },
                creation_date: KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 5,
                    minute: 33,
                    second: 0,
                    fractional_second: 0.123,
                },
                originator: KvnValue {
                    value: "GSOC".to_string(),
                    unit: None,
                },
                object_name: KvnValue {
                    value: "EUTELSAT W4".to_string(),
                    unit: None,
                },
                object_id: KvnValue {
                    value: "2021-028A".to_string(),
                    unit: None,
                },
                center_name: KvnValue {
                    value: "EARTH".to_string(),
                    unit: None,
                },
                ref_frame: KvnValue {
                    value: "TOD".to_string(),
                    unit: None,
                },
                time_system: KvnValue {
                    value: "UTC".to_string(),
                    unit: None,
                },
                epoch: KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    fractional_second: 0.0,
                },
                x: KvnValue {
                    value: 6655.9942,
                    unit: Some("km".to_string(),),
                },
                y: KvnValue {
                    value: -40218.5751,
                    unit: Some("km".to_string(),),
                },
                z: KvnValue {
                    value: -82.9177,
                    unit: Some("km".to_string(),),
                },
                x_dot: KvnValue {
                    value: 3.11548208,
                    unit: Some("km/s".to_string(),),
                },
                y_dot: KvnValue {
                    value: 0.47042605,
                    unit: Some("km/s".to_string(),),
                },
                z_dot: KvnValue {
                    value: -0.00101495,
                    unit: Some("km/s".to_string(),),
                },
                semi_major_axis: KvnValue {
                    value: 41399.5123,
                    unit: Some("km".to_string(),),
                },
                eccentricity: KvnValue {
                    value: 0.020842611,
                    unit: None,
                },
                inclination: KvnValue {
                    value: 0.117746,
                    unit: Some("deg".to_string(),),
                },
                ra_of_asc_node: KvnValue {
                    value: 17.604721,
                    unit: Some("deg".to_string(),),
                },
                arg_of_pericenter: KvnValue {
                    value: 218.242943,
                    unit: Some("deg".to_string(),),
                },
                true_anomaly: KvnValue {
                    value: 41.922339,
                    unit: Some("deg".to_string(),),
                },
                gm: KvnValue {
                    value: 398600.4415,
                    unit: Some("km**3/s**2".to_string(),),
                },
                mass: KvnValue {
                    value: 1913.0,
                    unit: Some("kg".to_string(),),
                },
                solar_rad_area: KvnValue {
                    value: 10.0,
                    unit: Some("m**2".to_string(),),
                },
                solar_rad_coeff: KvnValue {
                    value: 1.3,
                    unit: None,
                },
                drag_area: KvnValue {
                    value: 10.0,
                    unit: Some("m**2".to_string(),),
                },
                drag_coeff: KvnValue {
                    value: 2.3,
                    unit: None,
                },
                man_epoch_ignition: KvnDateTimeValue {
                    year: 2021,
                    month: 6,
                    day: 3,
                    hour: 9,
                    minute: 0,
                    second: 34,
                    fractional_second: 0.10000000000000142,
                },
                man_duration: KvnValue {
                    value: 132.6,
                    unit: Some("s".to_string(),),
                },
                man_delta_mass: KvnValue {
                    value: -18.418,
                    unit: Some("kg".to_string(),),
                },
                man_ref_frame: KvnValue {
                    value: "EME2000".to_string(),
                    unit: None,
                },
                man_dv_1: KvnValue {
                    value: -0.023257,
                    unit: Some("km/s".to_string(),),
                },
                man_dv_2: KvnValue {
                    value: 0.0168316,
                    unit: Some("km/s".to_string(),),
                },
                man_dv_3: KvnValue {
                    value: -0.00893444,
                    unit: Some("km/s".to_string(),),
                },
            },)
        );
    }
}
