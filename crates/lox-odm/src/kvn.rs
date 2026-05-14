// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! CCSDS KVN (Keyword-Value Notation) reader and writer.
//!
//! This module is split into:
//! - [`ast`]: the [`KvnDocument`] lossless AST.
//! - [`error`]: position-aware [`KvnError`] and [`Span`].
//! - [`grammar`]: per-line nom parsers and [`LineClass`] classification.
//! - [`writer`]: [`Display`] implementation that emits canonical
//!   KVN-formatted text.
//! - (planned) `project`: AST ↔ typed-message conversions. Lands per
//!   message in Phase 2b-opm/oem/omm/ci.
//!
//! [`Display`]: std::fmt::Display
//! [`LineClass`]: grammar::LineClass

pub mod ast;
pub mod error;
pub mod grammar;
pub mod parser;
pub mod writer;

pub use ast::{KvnDocument, KvnEntry, KvnField, KvnRow, KvnSection};
pub use error::{KvnError, KvnErrorKind, Span};
pub use parser::parse;
