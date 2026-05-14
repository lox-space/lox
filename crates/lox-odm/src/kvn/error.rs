// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Position-aware error type for the KVN parser.

/// A position in the input — 1-based line number plus 0-based column
/// range. `end_col` is exclusive (matches Rust slice semantics).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Span {
    /// 1-based line number.
    pub line: usize,
    /// 0-based start column (byte offset within the line).
    pub col_start: usize,
    /// 0-based end column (exclusive).
    pub col_end: usize,
}

impl Span {
    /// Span covering an entire line.
    pub fn whole_line(line: usize, line_len: usize) -> Self {
        Span {
            line,
            col_start: 0,
            col_end: line_len,
        }
    }
}

/// Error returned by the KVN parser.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("KVN parse error at line {span_line}: {kind}", span_line = span.line)]
pub struct KvnError {
    /// Where in the input the error was detected.
    pub span: Span,
    /// What went wrong.
    pub kind: KvnErrorKind,
}

/// Categorisation of KVN parser failures.
///
/// Covers both grammar-level concerns of the AST parser and the
/// projection layer (Phase 2b-{opm,oem,omm,ci}) that maps a parsed
/// [`KvnDocument`](crate::kvn::ast::KvnDocument) into the typed
/// message structures.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum KvnErrorKind {
    /// The input is empty.
    #[error("input is empty")]
    EmptyInput,
    /// The version-header line is missing or malformed.
    #[error("expected `CCSDS_<KIND>_VERS = <version>` on the first non-blank line")]
    MissingVersionHeader,
    /// The version-header line referred to an unknown message-kind token.
    #[error("unknown message kind in version header: {0:?}")]
    UnknownMessageKind(String),
    /// A line matched no grammar production.
    #[error("malformed line: {0:?}")]
    MalformedLine(String),
    /// A `*_STOP` marker did not match the currently open `*_START`.
    #[error("unexpected `{found}_STOP` (expected `{expected}_STOP`)")]
    UnexpectedStop {
        /// The keyword on the stray `_STOP`.
        found: String,
        /// The keyword of the currently open section.
        expected: String,
    },
    /// EOF reached while a bracketed section was still open.
    #[error("unterminated section `{0}_START` — missing `{0}_STOP`")]
    UnterminatedSection(String),
    /// A keyword required by the message type's schema was not present
    /// in the parsed AST.
    #[error("required field `{0}` is missing")]
    MissingRequiredField(String),
    /// A field's raw value text couldn't be parsed into the expected type.
    #[error("invalid value for `{keyword}`: {reason}")]
    InvalidValue {
        /// The wire keyword whose value failed to parse.
        keyword: String,
        /// Underlying parse error message.
        reason: String,
    },
    /// A keyword appeared that is not valid in this section / message kind.
    #[error("unexpected keyword `{0}`")]
    UnexpectedKeyword(String),
    /// A scalar keyword that's only allowed once appeared more than once.
    #[error("field `{0}` appeared more than once")]
    DuplicateField(String),
    /// An epoch field's value couldn't be parsed against the message's
    /// `TIME_SYSTEM`.
    #[error("invalid epoch `{value}` under TIME_SYSTEM `{time_system}`: {reason}")]
    InvalidEpoch {
        /// The raw value text on the wire.
        value: String,
        /// The TIME_SYSTEM keyword the value was parsed under.
        time_system: String,
        /// Underlying parse error message.
        reason: String,
    },
}
