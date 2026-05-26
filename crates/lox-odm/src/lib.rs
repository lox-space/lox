// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Rust types and (de-)serialization for CCSDS Orbit Data Messages.

pub mod error;
pub mod format;
pub mod json;
pub mod kvn;
pub mod types;
pub mod xml;

pub use error::OdmError;
pub use format::{Format, detect_format};

use std::path::Path;

use crate::types::common::MessageKind;
use crate::types::oem::Oem;
use crate::types::omm::Omm;
use crate::types::opm::Opm;

// ----------------------------------------------------------------------------
// Top-level read/write (format-agnostic)
// ----------------------------------------------------------------------------

/// Parse an OPM from `input`, auto-detecting the wire format.
///
/// Supported formats: KVN, XML. JSON is OMM-only in CCSDS 502.0-B-3,
/// so a JSON input returns [`OdmError::UnsupportedFormat`].
pub fn read_opm(input: &str) -> Result<Opm, OdmError> {
    match detect_format(input)? {
        Format::Kvn => Ok(kvn::read_opm(input)?),
        Format::Xml => Ok(xml::read_opm(input)?),
        Format::Json => Err(OdmError::UnsupportedFormat {
            kind: MessageKind::Opm,
            format: Format::Json,
        }),
    }
}

/// Parse an OEM from `input`, auto-detecting the wire format.
pub fn read_oem(input: &str) -> Result<Oem, OdmError> {
    match detect_format(input)? {
        Format::Kvn => Ok(kvn::read_oem(input)?),
        Format::Xml => Ok(xml::read_oem(input)?),
        Format::Json => Err(OdmError::UnsupportedFormat {
            kind: MessageKind::Oem,
            format: Format::Json,
        }),
    }
}

/// Parse an OMM from `input`, auto-detecting the wire format
/// (KVN, XML, or JSON).
pub fn read_omm(input: &str) -> Result<Omm, OdmError> {
    match detect_format(input)? {
        Format::Kvn => Ok(kvn::read_omm(input)?),
        Format::Xml => Ok(xml::read_omm(input)?),
        Format::Json => Ok(json::read_omm(input)?),
    }
}

/// Serialise an OPM in the requested wire format.
pub fn write_opm(opm: &Opm, format: Format) -> Result<String, OdmError> {
    match format {
        Format::Kvn => Ok(kvn::write_opm(opm)),
        Format::Xml => Ok(xml::write_opm(opm)?),
        Format::Json => Err(OdmError::UnsupportedFormat {
            kind: MessageKind::Opm,
            format: Format::Json,
        }),
    }
}

/// Serialise an OEM in the requested wire format.
pub fn write_oem(oem: &Oem, format: Format) -> Result<String, OdmError> {
    match format {
        Format::Kvn => Ok(kvn::write_oem(oem)),
        Format::Xml => Ok(xml::write_oem(oem)?),
        Format::Json => Err(OdmError::UnsupportedFormat {
            kind: MessageKind::Oem,
            format: Format::Json,
        }),
    }
}

/// Serialise an OMM in the requested wire format.
pub fn write_omm(omm: &Omm, format: Format) -> Result<String, OdmError> {
    match format {
        Format::Kvn => Ok(kvn::write_omm(omm)),
        Format::Xml => Ok(xml::write_omm(omm)?),
        Format::Json => Ok(json::write_omm(omm)?),
    }
}

// ----------------------------------------------------------------------------
// File helpers
// ----------------------------------------------------------------------------

/// Read an OPM from a file (format auto-detected from contents).
pub fn read_opm_file(path: impl AsRef<Path>) -> Result<Opm, OdmError> {
    let content = std::fs::read_to_string(path)?;
    read_opm(&content)
}

/// Read an OEM from a file (format auto-detected from contents).
pub fn read_oem_file(path: impl AsRef<Path>) -> Result<Oem, OdmError> {
    let content = std::fs::read_to_string(path)?;
    read_oem(&content)
}

/// Read an OMM from a file (format auto-detected from contents).
pub fn read_omm_file(path: impl AsRef<Path>) -> Result<Omm, OdmError> {
    let content = std::fs::read_to_string(path)?;
    read_omm(&content)
}

/// Write an OPM to a file in the requested format.
pub fn write_opm_file(opm: &Opm, path: impl AsRef<Path>, format: Format) -> Result<(), OdmError> {
    let s = write_opm(opm, format)?;
    std::fs::write(path, s).map_err(OdmError::from)
}

/// Write an OEM to a file in the requested format.
pub fn write_oem_file(oem: &Oem, path: impl AsRef<Path>, format: Format) -> Result<(), OdmError> {
    let s = write_oem(oem, format)?;
    std::fs::write(path, s).map_err(OdmError::from)
}

/// Write an OMM to a file in the requested format.
pub fn write_omm_file(omm: &Omm, path: impl AsRef<Path>, format: Format) -> Result<(), OdmError> {
    let s = write_omm(omm, format)?;
    std::fs::write(path, s).map_err(OdmError::from)
}
