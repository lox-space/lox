/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_math::math::RADIANS_IN_ARCSECOND;

use crate::nutation::Nutation;

const RADIANS_IN_MILLIARCSECOND: f64 = RADIANS_IN_ARCSECOND / 1e3;

/// 2000B uses fixed offsets for ψ and ε in lieu of planetary terms.
pub(crate) static OFFSETS: &Nutation = &Nutation {
    longitude: -0.135 * RADIANS_IN_MILLIARCSECOND,
    obliquity: 0.388 * RADIANS_IN_MILLIARCSECOND,
};
