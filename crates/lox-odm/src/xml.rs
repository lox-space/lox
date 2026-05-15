// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! CCSDS XML reader and writer.
//!
//! Implements the XML wire format from CCSDS 502.0-B-3 for OPM, OEM,
//! and OMM. Private mirror structs in the per-kind sub-modules ([`opm`],
//! [`oem`], [`omm`]) hold the serde+quick-xml deserialisation shape;
//! the projection layer converts them to/from the typed
//! [`crate::types`] values.

pub mod error;
pub mod oem;
pub mod omm;
pub mod opm;

pub use error::XmlError;
pub use oem::{read_oem, write_oem};
pub use omm::{read_omm, write_omm};
pub use opm::{read_opm, write_opm};
