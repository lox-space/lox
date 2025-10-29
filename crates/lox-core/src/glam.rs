// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use glam::DVec3;

use crate::Angle;

pub trait Azimuth {
    fn azimuth(&self) -> Angle;
}

impl Azimuth for DVec3 {
    fn azimuth(&self) -> Angle {
        Angle::from_atan2(self.y, self.x)
    }
}
