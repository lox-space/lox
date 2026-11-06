// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Physical constants.

use crate::units::{Distance, Velocity};

/// The astronomical unit (IAU 2012 Resolution B2).
pub const ASTRONOMICAL_UNIT: Distance = Distance::meters(1.495978707e11);

/// The speed of light in vacuum.
pub const SPEED_OF_LIGHT: Velocity = Velocity::meters_per_second(299792458.0);
