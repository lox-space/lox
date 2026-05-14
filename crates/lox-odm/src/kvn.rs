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
//! - [`projection`]: AST ↔ typed-message conversions, one sub-module
//!   per message kind ([`projection::opm`], plus OEM/OMM/CI as their
//!   phases land).
//!
//! [`Display`]: std::fmt::Display
//! [`LineClass`]: grammar::LineClass

pub mod ast;
pub mod error;
pub mod grammar;
pub mod parser;
pub mod projection;
pub mod writer;

pub use ast::{KvnDocument, KvnEntry, KvnField, KvnRow, KvnSection};
pub use error::{KvnError, KvnErrorKind, Span};
pub use parser::parse;
pub use projection::oem::{read_oem, write_oem};
pub use projection::omm::{read_omm, write_omm};
pub use projection::opm::{read_opm, write_opm};
