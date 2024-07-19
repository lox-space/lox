/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module tio exposes functions for calculating the Terrestrial Intermediate Origin (TIO) locator,
//! s', which places the TIO on the equator of the Celestial Intermediate Pole (CIP).

use lox_math::math::RADIANS_IN_ARCSECOND;
use lox_math::types::units::{JulianCenturies, Radians};

type ArcsecondsPerCentury = f64;

/// The TIO locator is unpredictable, being dependent on the integration of observations of polar
/// motion, but is dominated by secular drift, providing a close approximation.
const SECULAR_DRIFT: ArcsecondsPerCentury = -47e-6;

/// Approximate the TIO locator, s', in radians using the IAU 2000 model.
#[inline]
pub fn sp_00(centuries_since_j2000_tt: JulianCenturies) -> Radians {
    SECULAR_DRIFT * centuries_since_j2000_tt * RADIANS_IN_ARCSECOND
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_sp_00() {
        let t = 123.45;
        let expected = -2.812961699849694e-8;
        assert_float_eq!(expected, sp_00(t), rel <= 1e-12);
    }
}
