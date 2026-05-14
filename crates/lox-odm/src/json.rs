// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! CCSDS OMM JSON reader and writer.
//!
//! Only the OMM message kind has a JSON wire format in CCSDS
//! 502.0-B-3. The wire shape matches Space-Track / Celestrak's
//! widely-used flat-object form: every CCSDS keyword is a top-level
//! key, numbers may be sent as strings, and Space-Track adds extra
//! non-CCSDS fields (TLE_LINE0/1/2, OBJECT_TYPE, COUNTRY_CODE, etc.)
//! that are silently dropped on read.

pub mod error;
pub mod omm;

pub use error::JsonError;
pub use omm::{read_omm, read_omm_list, write_omm, write_omm_list};
