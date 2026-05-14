// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! CCSDS KVN (Keyword-Value Notation) reader and writer.
//!
//! This module is split into:
//! - [`ast`]: the [`KvnDocument`] lossless AST.
//! - [`writer`]: [`Display`] implementation that emits canonical
//!   KVN-formatted text.
//! - (planned) `grammar`: the nom-based parser. Lands in Phase 2b-parser.
//! - (planned) `error`: position-aware [`KvnError`]. Lands in Phase 2b-parser.
//! - (planned) `project`: AST ↔ typed-message conversions. Lands per
//!   message in Phase 2b-opm/oem/omm/ci.
//!
//! [`Display`]: std::fmt::Display

pub mod ast;
pub mod writer;

pub use ast::{KvnDocument, KvnEntry, KvnField, KvnRow, KvnSection};
