use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while1};
use nom::character::complete::one_of;
use nom::combinator::{map, map_res, recognize};
use nom::multi::{fold_many1, many1};
use nom::number::complete::{double, float};
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

#[derive(Debug)]
enum PckArray {
    Str(Vec<String>),
    Float(Vec<f64>),
}

impl PartialEq for PckArray {
    fn eq(&self, other: &Self) -> bool {
        match self {
            PckArray::Str(v1) => match other {
                PckArray::Str(v2) => v1 == v2,
                PckArray::Float(_) => false,
            },
            PckArray::Float(v1) => match other {
                PckArray::Str(_) => false,
                PckArray::Float(v2) => v1 == v2,
            },
        }
    }
}

fn fortran_double(s: &str) -> IResult<&str, f64> {
    let mut parser = map_res(
        recognize(tuple((double, one_of("dD"), float))),
        |s: &str| str::replace(s, ['d', 'D'], "e").parse(),
    );
    parser(s)
}

fn pck_double(s: &str) -> IResult<&str, f64> {
    let mut parser = alt((fortran_double, double));
    parser(s)
}

fn pck_string(s: &str) -> IResult<&str, String> {
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

fn separator(s: &str) -> IResult<&str, &str> {
    take_while1(|x: char| x.is_whitespace() || x == ',')(s)
}

fn float_array(s: &str) -> IResult<&str, PckArray> {
    let mut parser = map(
        delimited(
            terminated(tag("("), separator),
            many1(terminated(pck_double, separator)),
            tag(")"),
        ),
        PckArray::Float,
    );
    parser(s)
}

fn string_array(s: &str) -> IResult<&str, PckArray> {
    let mut parser = map(
        delimited(
            terminated(tag("("), separator),
            many1(terminated(pck_string, separator)),
            tag(")"),
        ),
        PckArray::Str,
    );
    parser(s)
}

fn array(s: &str) -> IResult<&str, PckArray> {
    let mut parser = alt((float_array, string_array));
    parser(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pck_double() {
        assert_eq!(pck_double("+6378.1366"), Ok(("", 6378.1366)));
        assert_eq!(pck_double("6.3781366D3"), Ok(("", 6378.1366)));
        assert_eq!(pck_double("6.3781366d3"), Ok(("", 6378.1366)));
        assert_eq!(pck_double("6.3781366E3"), Ok(("", 6378.1366)));
        assert_eq!(pck_double("6.3781366e3"), Ok(("", 6378.1366)));
        assert_eq!(pck_double("6378"), Ok(("", 6378.0)));

        assert_eq!(pck_double("11e-1"), Ok(("", 1.1)));
        assert_eq!(pck_double("123E-02"), Ok(("", 1.23)));
        assert_eq!(pck_double("123K-01"), Ok(("K-01", 123.0)));
        assert!(pck_double("abc").is_err());
    }

    #[test]
    fn test_pck_string() {
        assert_eq!(
            pck_string("'KILOMETERS'"),
            Ok(("", "KILOMETERS".to_string()))
        );
        assert_eq!(
            pck_string("'You can''t always get what you want.'"),
            Ok(("", "You can't always get what you want.".to_string()))
        );
    }

    #[test]
    fn test_separator() {
        assert_eq!(separator("   "), Ok(("", "   ")));
        assert_eq!(separator(" , "), Ok(("", " , ")));
        assert!(separator("foo").is_err());
    }

    #[test]
    fn test_float_array() {
        assert_eq!(
            float_array("( 6378.1366     6378.1366     6356.7519   )"),
            Ok(("", PckArray::Float(vec!(6378.1366, 6378.1366, 6356.7519))))
        );
        assert_eq!(
            float_array("( 6378.1366, 6378.1366, 6356.7519 )"),
            Ok(("", PckArray::Float(vec!(6378.1366, 6378.1366, 6356.7519))))
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
                PckArray::Str(vec!(
                    "KILOMETERS".to_string(),
                    "SECONDS".to_string(),
                    "KILOMETERS/SECOND".to_string()
                ))
            ))
        );
    }
}
