// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.1511-2: Topography for Earth-to-space propagation modelling.
//!
//! Provides topographic altitude above mean sea level for any location on Earth.

use lox_core::units::{Angle, Distance};

use crate::data::LazyGrid;

static TOPO: LazyGrid = LazyGrid::new("1511/v2_topo.bin.zst");

/// Returns the topographic altitude above mean sea level at the given location.
pub fn topographic_altitude(lat: Angle, lon: Angle) -> Distance {
    // P.1511 v2 data is in metres
    Distance::meters(TOPO.get().bilinear(lat.to_degrees(), lon.to_degrees()))
}
