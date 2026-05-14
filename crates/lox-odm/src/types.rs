// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Typed data model for CCSDS Orbit Data Messages.
//!
//! Types in this module are format-agnostic — they represent the
//! semantic content of an ODM independent of whether it was parsed
//! from KVN, XML, or JSON.

pub mod common;
pub mod oem;
pub mod omm;
pub mod opm;

pub use common::{
    Covariance, CustomBodyOrFrameError, MessageKind, OdmCenter, OdmFrame, OdmHeader,
    SpacecraftParameters,
};
pub use oem::{Oem, OemCovariance, OemMetadata, OemSegment, OemTrajectoryError};
pub use omm::{Omm, OmmMetadata, TleParameters};
pub use opm::{Maneuver, Opm, OpmMetadata};
