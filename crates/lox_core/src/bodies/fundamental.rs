/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::PI;

type Radians = f64;

/// Strictly TDB, TT is sufficient for most applications.
type TDBJulianCenturiesSinceJ2000_0 = f64;

pub fn mean_moon_sun_elongation(t: TDBJulianCenturiesSinceJ2000_0) -> Radians {
    let arcsec: f64 = fast_polynomial::poly(
        t,
        &[260.703692, 1602961601.2090, -6.3706, 0.006593, -0.00003169],
    );
    arcsec_to_rad_wrapping(arcsec)
}

const ARCSECONDS_IN_CIRCLE: f64 = 360.0 * 60.0 * 60.0;
const RADIANS_IN_ARCSECOND: Radians = 2.0 * PI / ARCSECONDS_IN_CIRCLE;

#[inline]
fn arcsec_to_rad_wrapping(arcsec: f64) -> Radians {
    arcsec_to_rad(arcsec % ARCSECONDS_IN_CIRCLE)
}

#[inline]
fn arcsec_to_rad(arcsec: f64) -> Radians {
    arcsec * RADIANS_IN_ARCSECOND
}
