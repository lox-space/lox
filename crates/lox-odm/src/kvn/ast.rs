// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Lossless KVN AST.
//!
//! `COMMENT` lines are first-class entries via [`KvnEntry::Comment`] —
//! they preserve their wire-order position alongside fields and rows
//! without any "comment ownership" disambiguation at the AST layer.
//! The projection layer (Phase 2b-{opm,oem,omm,ci}) decides which typed
//! `comments: Vec<String>` field receives the comments of each section.

use crate::types::common::MessageKind;

/// A complete KVN-encoded ODM message in AST form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KvnDocument {
    /// Which ODM message type this document represents.
    pub message_kind: MessageKind,
    /// The wire-format `CCSDS_<KIND>_VERS` literal (e.g. `"3.0"`).
    pub version: String,
    /// `COMMENT` lines that appear before the version-header line.
    /// Empty in most files; populated when an originator prepends a
    /// banner.
    pub preamble: Vec<String>,
    /// Sections in document order. The first section is conventionally
    /// the header (with `keyword == "HEADER"`, `bracketed == false`).
    pub sections: Vec<KvnSection>,
}

/// One section of a KVN document.
///
/// A section may be bracketed (delimited on the wire by `KEYWORD_START`
/// / `KEYWORD_STOP` markers, e.g. `META_START`/`META_STOP`,
/// `COVARIANCE_START`/`COVARIANCE_STOP`) or implicit (no markers; the
/// parser synthesises them, e.g. `HEADER` and `DATA`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KvnSection {
    /// Section identifier — for bracketed sections this is the `KEYWORD`
    /// from `KEYWORD_START` (without the `_START` suffix). For implicit
    /// sections, conventional values are `"HEADER"`, `"DATA"`.
    pub keyword: String,
    /// Whether to emit `KEYWORD_START` / `KEYWORD_STOP` markers on
    /// write. `true` for genuinely bracketed sections; `false` for
    /// implicit ones synthesised from document structure.
    pub bracketed: bool,
    /// The section's content, in document order. `COMMENT` lines appear
    /// as [`KvnEntry::Comment`] entries interleaved with fields, rows,
    /// and subsections.
    pub entries: Vec<KvnEntry>,
}

/// One entry within a [`KvnSection`].
///
/// `KvnEntry` is an enum (rather than separate `Vec`s for each kind)
/// so that document order is preserved across mixed content. Comments
/// are first-class entries — no separate "leading/trailing comments"
/// bookkeeping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KvnEntry {
    /// A standard `KEY = VALUE [unit]` line.
    Field(KvnField),
    /// A positional row of values (OEM/OCM ephemeris bodies).
    Row(KvnRow),
    /// A nested section (e.g. a single maneuver block inside the OPM
    /// `DATA` section, or a covariance epoch inside an OEM segment).
    Subsection(KvnSection),
    /// A `COMMENT` line. The string is the text following the `COMMENT`
    /// keyword (a single leading space is stripped on parse and
    /// re-inserted on write).
    Comment(String),
}

/// A standard `KEY = VALUE [unit]` line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KvnField {
    /// The keyword, as it appears on the wire (case preserved —
    /// canonical form is uppercase but the AST does not normalise).
    pub key: String,
    /// The raw text of the value, as it appears on the wire. Type-aware
    /// parsing (datetime, f64, vector, etc.) happens in the projection
    /// layer (Phase 2b-{opm,oem,omm,ci}), not in the AST.
    pub value: String,
    /// Optional `[unit]` annotation. Stored without the surrounding
    /// brackets (e.g. `"km"`, not `"[km]"`).
    pub unit: Option<String>,
}

/// A positional row of values — used for OEM and OCM ephemeris bodies
/// where state vectors appear as `EPOCH X Y Z VX VY VZ` (whitespace-
/// separated) rather than as keyed fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KvnRow {
    /// The fields of the row, as raw text in document order. The
    /// projection layer assigns column semantics based on the enclosing
    /// section's expected schema.
    pub values: Vec<String>,
}
