/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module tio exposes functions for calculating the Terrestrial Intermediate Origin (TIO) locator,
//! s', which places the TIO on the equator of the Celestial Intermediate Pole (CIP).

use lox_units::{Angle, AngleUnits, types::units::JulianCenturies};

type ArcsecondsPerCentury = f64;

/// The TIO locator is unpredictable, being dependent on the integration of observations of polar
/// motion, but is dominated by secular drift, providing a close approximation.
const SECULAR_DRIFT: ArcsecondsPerCentury = -47e-6;

/// Approximate the TIO locator, s', in radians using the IAU 2000 model.
#[inline]
pub fn tio_locator(centuries_since_j2000_tt: JulianCenturies) -> Angle {
    (SECULAR_DRIFT * centuries_since_j2000_tt).arcsec()
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn test_sp_00() {
        let t = 123.45;
        let expected = -2.812961699849694e-8.rad();
        assert_approx_eq!(expected, tio_locator(t), rtol <= 1e-12);
    }
}
