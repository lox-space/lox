// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_frames::transformations::Rotation;
use lox_time::{Time, time_scales::Tt};

use crate::precession::BiasPrecessionIau2000;

pub mod iers1996;
pub mod iers2003;
pub mod iers2010;

pub fn icrf_to_j2000(time: Time<Tt>) -> Rotation {
    let bp = BiasPrecessionIau2000::new(time);
    Rotation::new(bp.rb)
}

pub fn j2000_to_icrf(time: Time<Tt>) -> Rotation {
    icrf_to_j2000(time).transpose()
}
