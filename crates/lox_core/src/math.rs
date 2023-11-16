use crate::types::{Arcsec, MilliArcsec, Radians};
use std::f64::consts::{PI, TAU};

/// Module math provides common mathematical functions shared by many parts of the library.

/// Normalizes an angle `a` to the range [center-π, center+π].
pub(crate) fn normalize_two_pi(a: Radians, center: Radians) -> Radians {
    a - 2.0 * PI * ((a + PI - center) / (2.0 * PI)).floor()
}

pub(crate) const ARCSECONDS_IN_CIRCLE: f64 = 360.0 * 60.0 * 60.0;

pub(crate) const RADIANS_IN_ARCSECOND: Radians = TAU / ARCSECONDS_IN_CIRCLE;

pub(crate) const RADIANS_IN_MILLIARCSECOND: Radians = RADIANS_IN_ARCSECOND / 1000.0;

/// Converts arcseconds to radians, modulo 2π.
#[inline]
pub(crate) fn arcsec_to_rad_two_pi(arcsec: Arcsec) -> Radians {
    arcsec_to_rad(arcsec % ARCSECONDS_IN_CIRCLE)
}

#[inline]
pub(crate) fn arcsec_to_rad(arcsec: Arcsec) -> Radians {
    arcsec * RADIANS_IN_ARCSECOND
}

#[inline]
pub(crate) fn milliarcsec_to_rad(mas: MilliArcsec) -> Radians {
    mas * RADIANS_IN_MILLIARCSECOND
}
