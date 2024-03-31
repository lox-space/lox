// KVN spec section 7.4 of https://public.ccsds.org/Pubs/502x0b3e1.pdf

use nom::bytes::complete as nb;
use nom::character::complete as nc;
use nom::sequence as ns;
use regex::Regex;

#[derive(PartialEq, Debug)]
pub enum KvnLineParserErr<I> {
    ParseError(nom::Err<nom::error::Error<I>>),
    EmptyValue,
    EmptyKey,
}

#[derive(PartialEq, Debug)]
pub struct KvnValue<V, U> {
    pub value: V,
    pub unit: Option<U>,
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

fn parse_kvn_line<'a>(
    key: &'a str,
    input: &'a str,
    with_unit: bool,
) -> Result<(&'a str, KvnValue<&'a str, &'a str>), KvnLineParserErr<&'a str>> {
    if key == "COMMENT" {
        let (_, comment) = comment_line(input).map_err(|e| KvnLineParserErr::ParseError(e))?;

        return Ok((
            "",
            KvnValue {
                value: comment,
                unit: None,
            },
        ));
    }

    let (remaining, result) = kvn_line(key, input).map_err(|e| KvnLineParserErr::ParseError(e))?;

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
        return Err(KvnLineParserErr::EmptyValue);
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

mod test {
    use super::*;

    #[test]
    fn test_parse_kvn_line() {
        // 7.5.1 A non-empty value field must be assigned to each mandatory keyword except for *‘_START’ and *‘_STOP’ keyword values
        // 7.4.6 Any white space immediately preceding or following the ‘equals’ sign shall not be significant.
        assert_eq!(
            parse_kvn_line("ASD", "ASD = ASDFG", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG",
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_line("ASD", "ASD    =   ASDFG", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG",
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_line("ASD", "ASD    = ASDFG", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG",
                    unit: None
                }
            ))
        );
        assert_eq!(
            parse_kvn_line("ASD", "ASD =    ", true),
            Err(KvnLineParserErr::EmptyValue)
        );
        assert_eq!(
            parse_kvn_line("ASD", "ASD = ", true),
            Err(KvnLineParserErr::EmptyValue)
        );
        assert_eq!(
            parse_kvn_line("ASD", "ASD =", true),
            Err(KvnLineParserErr::EmptyValue)
        );

        // 7.4.7 Any white space immediately preceding the end of line shall not be significant.
        assert_eq!(
            parse_kvn_line("ASD", "ASD = ASDFG          ", true),
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
            parse_kvn_line("ASD", "ASD = ASDFG [km]", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG",
                    unit: Some("km")
                }
            ))
        );
        assert_eq!(
            parse_kvn_line("ASD", "ASD = ASDFG             [km]", true),
            Ok((
                "",
                KvnValue {
                    value: "ASDFG",
                    unit: Some("km")
                }
            ))
        );

        assert_eq!(
            parse_kvn_line("ASD", "ASD =  [km]", true),
            Err(KvnLineParserErr::EmptyValue)
        );

        assert_eq!(
            parse_kvn_line("ASD", "ASD   [km]", true),
            Err(KvnLineParserErr::ParseError(nom::Err::Error(
                nom::error::Error {
                    input: "[km]",
                    code: nom::error::ErrorKind::Char
                }
            )))
        );
        assert_eq!(
            parse_kvn_line("ASD", " =  [km]", true),
            Err(KvnLineParserErr::ParseError(nom::Err::Error(
                nom::error::Error {
                    input: "=  [km]",
                    code: nom::error::ErrorKind::Tag
                }
            )))
        );

        // 7.4.5 Any white space immediately preceding or following the keyword shall not be significant.
        assert_eq!(
            parse_kvn_line("ASD", "  ASD  = ASDFG", true),
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
            parse_kvn_line("COMMENT", "  COMMENT asd a    asd a ads as ", true),
            Ok((
                "",
                KvnValue {
                    value: "asd a    asd a ads as ",
                    unit: None
                }
            ))
        );

        assert_eq!(
            parse_kvn_line("COMMENT", "  COMMENT ", true),
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
    fn test_parse_json() {
        let kvn = r#"CCSDS_OPM_VERS = 3.0
COMMENT Generated by GSOC, R. Kiehling
COMMENT Current intermediate orbit IO2 and maneuver planning data
CREATION_DATE = 2021-06-03T05:33:00.000
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
            parse_kvn_line("CCSDS_OPM_VERS", lines.next().unwrap(), false),
            Ok((
                "",
                KvnValue {
                    value: "3.0",
                    unit: None,
                },
            ),)
        );

        assert_eq!(
            parse_kvn_line("COMMENT", lines.next().unwrap(), false),
            Ok((
                "",
                KvnValue {
                    value: "Generated by GSOC, R. Kiehling",
                    unit: None,
                },
            ),)
        );
    }
}
