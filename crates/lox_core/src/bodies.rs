use crate::bodies::pck_constants::BODY10_RADII;

pub mod barycenters;
pub mod minor;
#[allow(clippy::approx_constant, clippy::excessive_precision, dead_code)]
pub mod pck_constants;
pub mod planets;
pub mod satellites;

pub trait NaifId {
    fn id() -> i32;
}

pub trait Ellipsoid {
    fn max_equatorial_radius() -> f64;
    fn min_equatorial_radius() -> f64;
    fn polar_radius() -> f64;
    fn mean_radius() -> f64;
}

pub struct Sun;

impl NaifId for Sun {
    fn id() -> i32 {
        10
    }
}

impl Ellipsoid for Sun {
    fn max_equatorial_radius() -> f64 {
        BODY10_RADII[0]
    }
    fn min_equatorial_radius() -> f64 {
        BODY10_RADII[1]
    }
    fn polar_radius() -> f64 {
        BODY10_RADII[2]
    }
    fn mean_radius() -> f64 {
        let sum: f64 = BODY10_RADII.iter().sum();
        sum / 3.0
    }
}
