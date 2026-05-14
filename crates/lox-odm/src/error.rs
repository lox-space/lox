// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Umbrella error for top-level format-agnostic functions.

use crate::format::Format;
use crate::json::JsonError;
use crate::kvn::KvnError;
use crate::types::common::MessageKind;
use crate::xml::XmlError;

/// Error returned by the top-level format-agnostic readers/writers.
///
/// The per-format `KvnError`, `XmlError`, and `JsonError` types remain
/// available for callers who know what they're parsing and want the
/// richest diagnostics; this umbrella lifts any of them via `?`.
#[derive(Debug, thiserror::Error)]
pub enum OdmError {
    /// KVN reader/writer error.
    #[error(transparent)]
    Kvn(#[from] KvnError),
    /// XML reader/writer error.
    #[error(transparent)]
    Xml(#[from] XmlError),
    /// JSON reader/writer error.
    #[error(transparent)]
    Json(#[from] JsonError),
    /// Filesystem I/O error from `read_*_file` / `write_*_file`.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Top-level auto-detect couldn't determine the format from the
    /// input's first non-whitespace byte (empty input).
    #[error("could not detect format from input")]
    UndetectableFormat,
    /// The requested (message kind, format) combination is not
    /// supported — currently means JSON for any type other than OMM.
    #[error("format {format:?} is not supported for {kind:?}")]
    UnsupportedFormat {
        /// The message kind that was being read/written.
        kind: MessageKind,
        /// The format that's unsupported for that kind.
        format: Format,
    },
}
