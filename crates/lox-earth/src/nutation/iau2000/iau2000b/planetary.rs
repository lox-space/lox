// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-License-Identifier: MPL-2.0

use lox_units::Angle;

use crate::nutation::Nutation;

/// 2000B uses fixed offsets for ψ and ε in lieu of planetary terms.
pub(crate) static OFFSETS: &Nutation = &Nutation {
    longitude: Angle::arcseconds(-0.135 * 1e-3),
    obliquity: Angle::arcseconds(0.388 * 1e-3),
};
