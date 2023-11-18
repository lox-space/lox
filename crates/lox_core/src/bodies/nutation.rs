use crate::bodies::nutation::iau1980::nutation_iau1980;
use crate::time::epochs::Epoch;
use crate::time::intervals::{tdb_julian_centuries_since_j2000, TDBJulianCenturiesSinceJ2000};
use crate::types::Radians;

mod iau1980;

pub enum Model {
    IAU1980,
    IAU2000A,
    IAU2000B,
    IAU2006A,
}

/// Nutation components with respect to some ecliptic of date.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Nutation {
    /// δψ
    pub longitude: Radians,
    /// δε
    pub obliquity: Radians,
}

struct Coefficients {
    /// `l`.
    l: f64,
    /// `l'`.
    lp: f64,
    /// `F`.
    f: f64,
    /// `D`.
    d: f64,
    /// `Ω`.
    om: f64,
    /// Longitude sine.
    long_sin_1: f64,
    long_sin_t: f64,
    /// Obliquity cosine.
    obl_cos_1: f64,
    obl_cos_t: f64,
}

pub fn nutation(model: Model, epoch: Epoch) -> Nutation {
    // TODO: This call is placeholder. We need to ensure correct calculation of the Julian interval
    // from the epoch.
    let t = tdb_julian_centuries_since_j2000(epoch);
    match model {
        Model::IAU1980 => nutation_iau1980(t),
        Model::IAU2000A => nutation_iau2000a(t),
        Model::IAU2000B => nutation_iau2000b(t),
        Model::IAU2006A => nutation_iau2006a(t),
    }
}

fn nutation_iau2000a(t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    todo!()
}

fn nutation_iau2000b(t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    todo!()
}

fn nutation_iau2006a(t: TDBJulianCenturiesSinceJ2000) -> Nutation {
    todo!()
}
