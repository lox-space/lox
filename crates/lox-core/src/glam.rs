// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Extension traits for `glam` vector types.

use glam::DVec3;

use crate::units::Angle;

/// Computes the azimuth angle of a vector in the XY plane.
pub trait Azimuth {
    /// Returns the azimuth as an [`Angle`], measured from the X axis toward the Y axis.
    fn azimuth(&self) -> Angle;
}

impl Azimuth for DVec3 {
    fn azimuth(&self) -> Angle {
        Angle::from_atan2(self.y, self.x)
    }
}
