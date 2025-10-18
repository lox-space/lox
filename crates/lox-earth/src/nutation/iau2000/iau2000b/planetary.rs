/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_units::Angle;

use crate::nutation::Nutation;

/// 2000B uses fixed offsets for ψ and ε in lieu of planetary terms.
pub(crate) static OFFSETS: &Nutation = &Nutation {
    longitude: Angle::asec(-0.135 * 1e-3),
    obliquity: Angle::asec(0.388 * 1e-3),
};
