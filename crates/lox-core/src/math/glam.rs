// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use glam::DVec3;

use crate::types::units::Radians;

pub trait Azimuth {
    fn azimuth(&self) -> Radians;
}

impl Azimuth for DVec3 {
    fn azimuth(&self) -> Radians {
        self.y.atan2(self.x)
    }
}
