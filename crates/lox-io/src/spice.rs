/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while, take_while1};
use nom::character::complete::{alpha1, digit1, line_ending, multispace0, one_of};
use nom::combinator::{map, map_res, recognize, rest};
use nom::error::Error;
use nom::multi::{fold_many1, many0, many1};
use nom::number::complete::{double, float};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::{Finish, IResult};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
#[error(transparent)]
pub struct KernelError(#[from] Error<String>);

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Double(f64),
    String(String),
    Timestamp(String),
    DoubleArray(Vec<f64>),
    StringArray(Vec<String>),
    TimestampArray(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Kernel {
    type_id: String,
    items: HashMap<String, Value>,
}

type Entries = Vec<(String, Value)>;

impl Kernel {
    pub fn from_string(input: &str) -> Result<Self, KernelError> {
        let result = kernel(input).map_err(|e| e.to_owned()).finish();
        match result {
            Ok((_, (type_id, entries, _))) => Ok(Self {
                type_id: type_id.to_string(),
                items: entries.into_iter().collect(),
            }),
            Err(err) => Err(KernelError(err)),
        }
    }

    pub fn type_id(&self) -> &str {
        &self.type_id
    }

    pub fn get_double(&self, key: &str) -> Option<f64> {
        let value = self.items.get(key)?;
        if let Value::Double(v) = value {
            Some(*v)
        } else {
            None
        }
    }

    pub fn get_double_array(&self, key: &str) -> Option<&Vec<f64>> {
        let value = self.items.get(key)?;
        if let Value::DoubleArray(v) = value {
            Some(v)
        } else {
            None
        }
    }

    pub fn get_timestamp_array(&self, key: &str) -> Option<&Vec<String>> {
        let value = self.items.get(key)?;
        if let Value::TimestampArray(v) = value {
            Some(v)
        } else {
            None
        }
    }

    pub fn keys(&self) -> Vec<&String> {
        self.items.keys().collect()
    }
}

fn kernel(s: &str) -> IResult<&str, (&str, Entries, &str)> {
    let header = preceded(tag("KPL/"), alpha1);
    let mut parser = tuple((
        header,
        fold_many1(
            preceded(
                alt((take_until("\\begindata\n"), take_until("\\begindata\r"))),
                data_block,
            ),
            Vec::new,
            |mut out: Entries, item: Entries| {
                out.extend(item);
                out
            },
        ),
        rest,
    ));
    parser(s)
}

fn fortran_double(s: &str) -> IResult<&str, f64> {
    let mut parser = map_res(
        recognize(tuple((double, one_of("dD"), float))),
        |s: &str| str::replace(s, ['d', 'D'], "e").parse(),
    );
    parser(s)
}

fn spice_double(s: &str) -> IResult<&str, f64> {
    let mut parser = alt((fortran_double, double));
    parser(s)
}

fn spice_string(s: &str) -> IResult<&str, String> {
    let mut parser = fold_many1(
        delimited(tag("'"), take_until("'"), tag("'")),
        String::new,
        |mut out: String, item: &str| {
            if !out.is_empty() {
                out.push('\'');
            }
            out.push_str(item);
            out
        },
    );
    parser(s)
}

fn timestamp(s: &str) -> IResult<&str, String> {
    let mut parser = map(
        // NASA NAIF's LSK kernels break their own rules and mix timestamps with integers within a
        // single array.
        // See: https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/kernel.html#Variable%20Value%20Rules
        alt((
            preceded(tag("@"), take_while1(|c| !is_separator(c))),
            digit1,
        )),
        String::from,
    );
    parser(s)
}

fn is_separator(c: char) -> bool {
    c.is_whitespace() || c == ','
}

fn separator(s: &str) -> IResult<&str, &str> {
    take_while1(is_separator)(s)
}

fn double_array(s: &str) -> IResult<&str, Value> {
    let mut parser = map(
        delimited(
            terminated(tag("("), separator),
            many1(terminated(spice_double, separator)),
            tag(")"),
        ),
        Value::DoubleArray,
    );
    parser(s)
}

fn string_array(s: &str) -> IResult<&str, Value> {
    let mut parser = map(
        delimited(
            terminated(tag("("), separator),
            many1(terminated(spice_string, separator)),
            tag(")"),
        ),
        Value::StringArray,
    );
    parser(s)
}

fn timestamp_array(s: &str) -> IResult<&str, Value> {
    let mut parser = map(
        delimited(
            terminated(tag("("), separator),
            many1(terminated(timestamp, separator)),
            tag(")"),
        ),
        Value::TimestampArray,
    );
    parser(s)
}

fn double_value(s: &str) -> IResult<&str, Value> {
    let mut parser = map(spice_double, Value::Double);
    parser(s)
}

fn string_value(s: &str) -> IResult<&str, Value> {
    let mut parser = map(spice_string, Value::String);
    parser(s)
}

fn timestamp_value(s: &str) -> IResult<&str, Value> {
    let mut parser = map(timestamp, Value::Timestamp);
    parser(s)
}

fn array_value(s: &str) -> IResult<&str, Value> {
    let mut parser = alt((double_array, string_array, timestamp_array));
    parser(s)
}

fn key_value(s: &str) -> IResult<&str, (String, Value)> {
    let mut parser = map(
        separated_pair(
            terminated(
                take_while1(|x: char| !x.is_whitespace() && x != '='),
                take_while(char::is_whitespace),
            ),
            terminated(tag("="), take_while1(char::is_whitespace)),
            alt((double_value, string_value, timestamp_value, array_value)),
        ),
        |kv: (&str, Value)| (kv.0.to_string(), kv.1),
    );
    parser(s)
}

fn start_tag(s: &str) -> IResult<&str, &str> {
    let mut parser = terminated(tag("\\begindata"), line_ending);
    parser(s)
}

fn end_tag(s: &str) -> IResult<&str, &str> {
    let parser = tag("\\begintext");
    parser(s)
}

fn data_block(s: &str) -> IResult<&str, Entries> {
    let mut parser = delimited(
        start_tag,
        many0(preceded(multispace0, key_value)),
        preceded(multispace0, end_tag),
    );
    parser(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_err() {
        let kernel = Kernel::from_string("foo");
        assert!(kernel.is_err());
    }

    #[test]
    fn test_double() {
        assert_eq!(spice_double("6.3781366e3"), Ok(("", 6378.1366)));
        assert_eq!(spice_double("+6378.1366"), Ok(("", 6378.1366)));
        assert_eq!(spice_double("6.3781366D3"), Ok(("", 6378.1366)));
        assert_eq!(spice_double("6.3781366d3"), Ok(("", 6378.1366)));
        assert_eq!(spice_double("6.3781366E3"), Ok(("", 6378.1366)));
        assert_eq!(spice_double("6378"), Ok(("", 6378.0)));

        assert_eq!(
            double_value("6.3781366e3"),
            Ok(("", Value::Double(6378.1366)))
        );

        assert_eq!(spice_double("11e-1"), Ok(("", 1.1)));
        assert_eq!(spice_double("123E-02"), Ok(("", 1.23)));
        assert_eq!(spice_double("123K-01"), Ok(("K-01", 123.0)));
        assert!(spice_double("abc").is_err());
    }

    #[test]
    fn test_string() {
        assert_eq!(
            spice_string("'KILOMETERS'"),
            Ok(("", "KILOMETERS".to_string()))
        );
        assert_eq!(
            string_value("'KILOMETERS'"),
            Ok(("", Value::String("KILOMETERS".to_string())))
        );
        assert_eq!(
            spice_string("'You can''t always get what you want.'"),
            Ok(("", "You can't always get what you want.".to_string()))
        );
    }

    #[test]
    fn test_timestamp() {
        assert_eq!(timestamp("@1972-JAN-1"), Ok(("", "1972-JAN-1".to_string())));
    }

    #[test]
    fn test_separator() {
        assert_eq!(separator("   "), Ok(("", "   ")));
        assert_eq!(separator(" , "), Ok(("", " , ")));
        assert!(separator("foo").is_err());
    }

    #[test]
    fn test_double_array() {
        assert_eq!(
            double_array("( 6378.1366     6378.1366     6356.7519   )"),
            Ok((
                "",
                Value::DoubleArray(vec!(6378.1366, 6378.1366, 6356.7519))
            ))
        );
        assert_eq!(
            double_array("( 6378.1366, 6378.1366, 6356.7519 )"),
            Ok((
                "",
                Value::DoubleArray(vec!(6378.1366, 6378.1366, 6356.7519))
            ))
        );
        assert_eq!(
            double_array("( 2.2031868551400003E+04 )"),
            Ok(("", Value::DoubleArray(vec!(2.2031868551400003e4))))
        )
    }

    #[test]
    fn test_string_array() {
        let input = "( 'KILOMETERS','SECONDS' \
            'KILOMETERS/SECOND' )";
        assert_eq!(
            string_array(input),
            Ok((
                "",
                Value::StringArray(vec!(
                    "KILOMETERS".to_string(),
                    "SECONDS".to_string(),
                    "KILOMETERS/SECOND".to_string()
                ))
            ))
        );
    }

    #[test]
    fn test_timestamp_array() {
        let input = "( @1972-JAN-1,@1972-JAN-1 \
            @1972-JAN-1 )";
        assert_eq!(
            timestamp_array(input),
            Ok((
                "",
                Value::TimestampArray(vec!(
                    "1972-JAN-1".to_string(),
                    "1972-JAN-1".to_string(),
                    "1972-JAN-1".to_string()
                ))
            ))
        );
    }

    #[test]
    fn test_array() {
        let exp_float = Value::DoubleArray(vec![6378.1366, 6378.1366, 6356.7519]);
        let exp_string = Value::StringArray(vec![
            "KILOMETERS".to_string(),
            "SECONDS".to_string(),
            "KILOMETERS/SECOND".to_string(),
        ]);
        assert_ne!(Value::Double(3.0), Value::Double(3.1));
        assert_ne!(exp_float, exp_string);
        assert_ne!(exp_string, exp_float);
        assert_eq!(
            array_value("( 6378.1366, 6378.1366, 6356.7519 )"),
            Ok(("", exp_float))
        );
        let input = "( 'KILOMETERS','SECONDS' \
            'KILOMETERS/SECOND' )";
        assert_eq!(array_value(input), Ok(("", exp_string)));
    }

    #[test]
    fn test_key_value() {
        let input = "BODY399_RADII     = ( 6378.1366     6378.1366     6356.7519   )";
        let exp_value = Value::DoubleArray(vec![6378.1366, 6378.1366, 6356.7519]);
        let exp_key = "BODY399_RADII".to_string();
        assert_eq!(key_value(input), Ok(("", (exp_key, exp_value))));
        let input = "BODY1_GM       = ( 2.2031868551400003E+04 )";
        let exp_value = Value::DoubleArray(vec![2.2031868551400003e4]);
        let exp_key = "BODY1_GM".to_string();
        assert_eq!(key_value(input), Ok(("", (exp_key, exp_value))));
    }

    #[test]
    fn test_data_block() {
        assert_eq!(start_tag("\\begindata\n"), Ok(("", "\\begindata")));
        assert!(start_tag("foo \\begindata bar\n").is_err());

        let block = "\\begindata

        BODY499_POLE_RA          = (  317.269202  -0.10927547        0.  )
        BODY499_POLE_DEC         = (   54.432516  -0.05827105        0.  )
        BODY499_PM               = (  176.049863  +350.891982443297  0.  )

        BODY499_NUT_PREC_RA      = (  0     0     0     0     0
                                      0     0     0     0     0
                                      0.000068
                                      0.000238
                                      0.000052
                                      0.000009
                                      0.419057                  )


        BODY499_NUT_PREC_DEC     = (  0     0     0     0     0
                                      0     0     0     0     0
                                      0     0     0     0     0
                                      0.000051
                                      0.000141
                                      0.000031
                                      0.000005
                                      1.591274                  )


        BODY499_NUT_PREC_PM      = (  0     0     0     0     0
                                      0     0     0     0     0
                                      0     0     0     0     0
                                      0     0     0     0     0
                                      0.000145
                                      0.000157
                                      0.000040
                                      0.000001
                                      0.000001
                                      0.584542                  )

        \\begintext";
        let k1 = "BODY499_POLE_RA".to_string();
        let v1 = Value::DoubleArray(vec![317.269202, -0.10927547, 0.]);
        let k2 = "BODY499_POLE_DEC".to_string();
        let v2 = Value::DoubleArray(vec![54.432516, -0.05827105, 0.]);
        let k3 = "BODY499_PM".to_string();
        let v3 = Value::DoubleArray(vec![176.049863, 350.891982443297, 0.]);
        let k4 = "BODY499_NUT_PREC_RA".to_string();
        let v4 = Value::DoubleArray(vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.000068, 0.000238, 0.000052,
            0.000009, 0.419057,
        ]);
        let k5 = "BODY499_NUT_PREC_DEC".to_string();
        let v5 = Value::DoubleArray(vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.000051,
            0.000141, 0.000031, 0.000005, 1.591274,
        ]);
        let k6 = "BODY499_NUT_PREC_PM".to_string();
        let v6 = Value::DoubleArray(vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.000145, 0.000157, 0.000040, 0.000001, 0.000001, 0.584542,
        ]);
        let exp = vec![(k1, v1), (k2, v2), (k3, v3), (k4, v4), (k5, v5), (k6, v6)];
        assert_eq!(data_block(block), Ok(("", exp)));
    }
}
