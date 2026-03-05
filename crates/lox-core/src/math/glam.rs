// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Extension traits for `glam` vector types.

use glam::DVec3;

use crate::types::units::Radians;

/// Computes the azimuth angle of a vector in the XY plane.
pub trait Azimuth {
    /// Returns the azimuth angle in radians, measured from the X axis toward the Y axis.
    fn azimuth(&self) -> Radians;
}

impl Azimuth for DVec3 {
    fn azimuth(&self) -> Radians {
        self.y.atan2(self.x)
    }
}
