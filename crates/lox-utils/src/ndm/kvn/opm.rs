// KVN spec section 7.4 of https://public.ccsds.org/Pubs/502x0b3e1.pdf

use nom::bytes::complete as nb;
use nom::character::complete as nc;
use nom::sequence as ns;
use regex::Regex;

#[derive(PartialEq, Debug)]
pub enum KvnStringLineParserErr<I> {
    ParseError(nom::Err<nom::error::Error<I>>),
    EmptyValue,
}

#[derive(PartialEq, Debug)]
pub enum KvnIntegerLineParserErr<I> {
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
pub struct KvnValue<V, U> {
    pub value: V,
    pub unit: Option<U>,
}

#[derive(PartialEq, Debug)]
pub struct KvnDateTimeValue {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    fractional_second: f64,
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
) -> Result<(&'a str, KvnValue<&'a str, &'a str>), KvnStringLineParserErr<&'a str>> {
    if key == "COMMENT" {
        let (_, comment) =
            comment_line(input).map_err(|e| KvnStringLineParserErr::ParseError(e))?;

        return Ok((
            "",
            KvnValue {
                value: comment,
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
            value: parsed_value,
            unit: parsed_unit,
        },
    ))
}

fn parse_kvn_integer_line<'a>(
    key: &'a str,
    input: &'a str,
    with_unit: bool,
) -> Result<(&'a str, KvnValue<i32, &'a str>), KvnIntegerLineParserErr<&'a str>> {
    parse_kvn_string_line(key, input, with_unit)
        .map_err(|e| match e {
            KvnStringLineParserErr::EmptyValue => KvnIntegerLineParserErr::EmptyValue,
            KvnStringLineParserErr::ParseError(e) => KvnIntegerLineParserErr::ParseError(e),
        })
        .and_then(|result| {
            let value = result
                .1
                .value
                .parse::<i32>()
                .map_err(|_| KvnIntegerLineParserErr::InvalidFormat)?;

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
                    value: "ASDFG",
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD    =   ASDFG", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG",
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD    = ASDFG", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG",
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
                    value: "ASDFG",
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
                    value: "ASDFG",
                    unit: Some("km")
                }
            ))
        );
        assert_eq!(
            parse_kvn_string_line("ASD", "ASD = ASDFG             [km]", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG",
                    unit: Some("km")
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
                    value: "ASDFG",
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
                    value: "asd a    asd a ads as ",
                    unit: None
                }
            ))
        );

        assert_eq!(
            parse_kvn_string_line("COMMENT", "  COMMENT ", true),
            Ok((
                "",
                KvnValue {
                    value: "",
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
                    unit: Some("s")
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
                    unit: Some("s")
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
                    unit: Some("s")
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
            parse_kvn_integer_line("SCLK_OFFSET_AT_EPOCH", "SCLK_OFFSET_AT_EPOCH = [s]", true),
            Err(KvnIntegerLineParserErr::EmptyValue)
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
        let kvn = r#"CCSDS_OPM_VERS = 3.0
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
MAN_DV_3 = 0.00000000 [km/s]"#;

        let mut lines = kvn.lines();

        assert_eq!(
            parse_kvn_string_line("CCSDS_OPM_VERS", lines.next().unwrap(), false),
            Ok((
                "",
                KvnValue {
                    value: "3.0",
                    unit: None,
                },
            ),)
        );

        assert_eq!(
            parse_kvn_string_line("COMMENT", lines.next().unwrap(), false),
            Ok((
                "",
                KvnValue {
                    value: "Generated by GSOC, R. Kiehling",
                    unit: None,
                },
            ),)
        );

        assert_eq!(
            parse_kvn_string_line("COMMENT", lines.next().unwrap(), false),
            Ok((
                "",
                KvnValue {
                    value: "Current intermediate orbit IO2 and maneuver planning data",
                    unit: None,
                },
            ),)
        );

        assert_eq!(
            parse_kvn_datetime_line("CREATION_DATE", lines.next().unwrap()),
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
    }
}
