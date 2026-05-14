// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Per-line nom parsers for KVN.
//!
//! Each parser in this module recognises one kind of meaningful line:
//! comments, section start/stop markers, fields, version headers, and
//! ephemeris rows. The line-by-line composition into a [`KvnDocument`]
//! lives in [`super::parser`].

use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, multispace0, space0, space1},
    combinator::{eof, recognize, verify},
    multi::many0,
    sequence::preceded,
};

/// Classification of a single non-blank line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LineClass<'a> {
    /// `CCSDS_<KIND>_VERS = <version>` line — the file header.
    /// `kind_token` is the `<KIND>` portion (e.g. `"OPM"`, `"OEM"`).
    VersionHeader {
        kind_token: &'a str,
        version: &'a str,
    },
    /// `COMMENT <free text>` line.
    Comment(&'a str),
    /// `<KEYWORD>_START` line.
    SectionStart { keyword: &'a str },
    /// `<KEYWORD>_STOP` line.
    SectionStop { keyword: &'a str },
    /// `<KEY> = <VALUE>` or `<KEY> = <VALUE> [<unit>]` line.
    Field {
        key: &'a str,
        value: &'a str,
        unit: Option<&'a str>,
    },
    /// A positional row — whitespace-separated tokens whose first token
    /// looks like a CCSDS epoch (`YYYY-MM-DDThh:mm:ss[.fff]` or
    /// `YYYY-DDDThh:mm:ss[.fff]`). Used for OEM/OCM ephemeris bodies.
    EphemerisRow(Vec<&'a str>),
}

/// Classify a single line (already stripped of trailing newline).
///
/// Blank or whitespace-only lines yield `None`.
pub fn classify_line(line: &str) -> Result<Option<LineClass<'_>>, String> {
    let trimmed = line.trim_end();
    if trimmed.trim_start().is_empty() {
        return Ok(None);
    }

    // Order matters: version header before generic field; comment before
    // section markers; section markers before generic ephemeris row.
    if let Ok((_, vh)) = version_header(trimmed) {
        return Ok(Some(vh));
    }
    if let Ok((_, c)) = comment_line(trimmed) {
        return Ok(Some(LineClass::Comment(c)));
    }
    if let Ok((_, kw)) = section_stop(trimmed) {
        return Ok(Some(LineClass::SectionStop { keyword: kw }));
    }
    if let Ok((_, kw)) = section_start(trimmed) {
        return Ok(Some(LineClass::SectionStart { keyword: kw }));
    }
    if let Ok((_, f)) = field_line(trimmed) {
        return Ok(Some(f));
    }
    // Fallback: treat as positional row (OEM/OCM ephemeris).
    if let Ok((_, r)) = ephemeris_row(trimmed) {
        return Ok(Some(r));
    }
    Err(format!("could not classify line: {trimmed:?}"))
}

fn ident(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_alphanumeric() || c == '_').parse(input)
}

fn version_header(input: &str) -> IResult<&str, LineClass<'_>> {
    // CCSDS_<KIND>_VERS = <version>
    let (input, _) = (multispace0, tag("CCSDS_")).parse(input)?;
    let (input, kind_token) = take_while1(|c: char| c.is_ascii_alphabetic()).parse(input)?;
    let (input, _) = (tag("_VERS"), space0, char('='), space0).parse(input)?;
    let (input, version) = take_while1(|c: char| !c.is_whitespace()).parse(input)?;
    let (input, _) = (space0, eof).parse(input)?;
    Ok((
        input,
        LineClass::VersionHeader {
            kind_token,
            version,
        },
    ))
}

fn comment_line(input: &str) -> IResult<&str, &str> {
    // COMMENT <text> -- text is the remainder of the line after a single
    // space (or empty if the line is just "COMMENT")
    let (input, _) = (multispace0, tag("COMMENT")).parse(input)?;
    // Accept either end-of-line (no text) or " <text>" (single space then text).
    if input.is_empty() {
        return Ok((input, ""));
    }
    let (input, _) = char(' ').parse(input)?;
    Ok(("", input))
}

fn section_start(input: &str) -> IResult<&str, &str> {
    let (input, _) = multispace0(input)?;
    let (input, keyword) = recognize(verify(ident, |s: &str| {
        s.ends_with("_START") && s.len() > "_START".len()
    }))
    .parse(input)?;
    let (input, _) = (space0, eof).parse(input)?;
    Ok((input, &keyword[..keyword.len() - "_START".len()]))
}

fn section_stop(input: &str) -> IResult<&str, &str> {
    let (input, _) = multispace0(input)?;
    let (input, keyword) = recognize(verify(ident, |s: &str| {
        s.ends_with("_STOP") && s.len() > "_STOP".len()
    }))
    .parse(input)?;
    let (input, _) = (space0, eof).parse(input)?;
    Ok((input, &keyword[..keyword.len() - "_STOP".len()]))
}

fn field_line(input: &str) -> IResult<&str, LineClass<'_>> {
    // KEY = VALUE  or  KEY = VALUE [unit]
    let (input, _) = multispace0(input)?;
    let (input, key) = ident(input)?;
    let (input, _) = (space0, char('='), space0).parse(input)?;
    // Value: everything up to optional `[unit]` suffix (or end of line),
    // trimmed of trailing whitespace.
    let (input, value_with_unit) = take_while(|c: char| c != '\n')(input)?;
    let value_with_unit = value_with_unit.trim_end();

    let (value, unit) = match value_with_unit.rfind('[') {
        Some(idx) if value_with_unit.ends_with(']') => {
            let unit = &value_with_unit[idx + 1..value_with_unit.len() - 1];
            let value = value_with_unit[..idx].trim_end();
            (value, Some(unit))
        }
        _ => (value_with_unit, None),
    };

    Ok((input, LineClass::Field { key, value, unit }))
}

fn ephemeris_row(input: &str) -> IResult<&str, LineClass<'_>> {
    let (input, _) = multispace0(input)?;
    // The first token must look like a CCSDS epoch — 4 digits then `-`.
    // This catches both the calendar form `YYYY-MM-DDThh:mm:ss[.fff]` and
    // the day-of-year form `YYYY-DDDThh:mm:ss[.fff]` while rejecting bare
    // operators, words, or other non-timestamp text.
    let (input, first) = verify(take_while1(|c: char| !c.is_whitespace()), |s: &str| {
        looks_like_epoch(s)
    })
    .parse(input)?;
    let (input, mut rest) =
        many0(preceded(space1, take_while1(|c: char| !c.is_whitespace()))).parse(input)?;
    let (input, _) = (space0, eof).parse(input)?;
    let mut values = Vec::with_capacity(1 + rest.len());
    values.push(first);
    values.append(&mut rest);
    Ok((input, LineClass::EphemerisRow(values)))
}

/// Cheap heuristic for "is the leading token plausibly a CCSDS epoch?"
/// Both the calendar form `YYYY-MM-DDThh:mm:ss[.fff]` and the
/// day-of-year form `YYYY-DDDThh:mm:ss[.fff]` start with four ASCII
/// digits followed by `-`. We don't validate further at the grammar
/// layer — full epoch parsing happens in the projection layer where the
/// resulting [`String`] is converted to a `DynTime`.
fn looks_like_epoch(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() >= 5 && bytes[0..4].iter().all(|b| b.is_ascii_digit()) && bytes[4] == b'-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_blank_line_as_none() {
        assert_eq!(classify_line("").unwrap(), None);
        assert_eq!(classify_line("   ").unwrap(), None);
        assert_eq!(classify_line("\t  ").unwrap(), None);
    }

    #[test]
    fn classifies_version_header() {
        let line = "CCSDS_OPM_VERS = 3.0";
        let result = classify_line(line).unwrap().unwrap();
        assert_eq!(
            result,
            LineClass::VersionHeader {
                kind_token: "OPM",
                version: "3.0",
            }
        );
    }

    #[test]
    fn classifies_comment_with_text() {
        let line = "COMMENT this is a comment";
        let result = classify_line(line).unwrap().unwrap();
        assert_eq!(result, LineClass::Comment("this is a comment"));
    }

    #[test]
    fn classifies_bare_comment() {
        let line = "COMMENT";
        let result = classify_line(line).unwrap().unwrap();
        assert_eq!(result, LineClass::Comment(""));
    }

    #[test]
    fn classifies_section_start() {
        let line = "META_START";
        let result = classify_line(line).unwrap().unwrap();
        assert_eq!(result, LineClass::SectionStart { keyword: "META" });
    }

    #[test]
    fn classifies_section_stop() {
        let line = "META_STOP";
        let result = classify_line(line).unwrap().unwrap();
        assert_eq!(result, LineClass::SectionStop { keyword: "META" });
    }

    #[test]
    fn classifies_field_without_unit() {
        let line = "OBJECT_NAME = ISS";
        let result = classify_line(line).unwrap().unwrap();
        assert_eq!(
            result,
            LineClass::Field {
                key: "OBJECT_NAME",
                value: "ISS",
                unit: None,
            }
        );
    }

    #[test]
    fn classifies_field_with_unit() {
        let line = "X = 7000.0 [km]";
        let result = classify_line(line).unwrap().unwrap();
        assert_eq!(
            result,
            LineClass::Field {
                key: "X",
                value: "7000.0",
                unit: Some("km"),
            }
        );
    }

    #[test]
    fn classifies_ephemeris_row() {
        let line = "2024-01-01T00:00:00 7000.0 0.0 0.0 0.0 7.5 0.0";
        let result = classify_line(line).unwrap().unwrap();
        let LineClass::EphemerisRow(values) = result else {
            panic!("expected EphemerisRow, got {result:?}");
        };
        assert_eq!(values.len(), 7);
        assert_eq!(values[0], "2024-01-01T00:00:00");
        assert_eq!(values[6], "0.0");
    }

    #[test]
    fn rejects_non_epoch_leading_row() {
        // A bare `= 5.0` is not a valid KVN line — neither field-shaped
        // nor epoch-leading — so the classifier must reject it.
        assert!(classify_line("= 5.0").is_err());
        // Random word-leading text is also rejected.
        assert!(classify_line("foo 1.0 2.0").is_err());
    }

    #[test]
    fn classifies_doy_format_ephemeris_row() {
        // CCSDS allows day-of-year format epochs: YYYY-DDDThh:mm:ss.
        let line = "2024-001T00:00:00 7000.0 0.0 0.0 0.0 7.5 0.0";
        let result = classify_line(line).unwrap().unwrap();
        let LineClass::EphemerisRow(values) = result else {
            panic!("expected EphemerisRow, got {result:?}");
        };
        assert_eq!(values.len(), 7);
        assert_eq!(values[0], "2024-001T00:00:00");
        assert_eq!(values[6], "0.0");
    }
}
