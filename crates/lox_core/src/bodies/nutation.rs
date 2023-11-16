use crate::bodies::nutation::iau1980::nutation_iau1980;
use crate::time::epochs::Epoch;
use crate::types::Radians;

mod iau1980;

pub enum Model {
    IAU1980,
    IAU2000A,
    IAU2000B,
    IAU2006A,
}

pub struct Nutation {
    /// δψ
    pub longitude: Radians,
    /// δε
    pub obliquity: Radians,
}

impl Default for Nutation {
    fn default() -> Self {
        Self {
            longitude: 0.0,
            obliquity: 0.0,
        }
    }
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

/// The interval between J2000 and a given Julian date.
pub type JulianInterval = f64;

pub fn nutation(model: Model, epoch: Epoch) -> Nutation {
    // TODO: This call is placeholder. We need to ensure correct calculation of the Julian interval
    // from the epoch.
    let t: JulianInterval = epoch.j2000();
    match model {
        Model::IAU1980 => nutation_iau1980(t),
        Model::IAU2000A => nutation_iau2000a(t),
        Model::IAU2000B => nutation_iau2000b(t),
        Model::IAU2006A => nutation_iau2006a(t),
    }
}

fn nutation_iau2000a(t: JulianInterval) -> Nutation {
    todo!()
}

fn nutation_iau2000b(t: JulianInterval) -> Nutation {
    todo!()
}

fn nutation_iau2006a(t: JulianInterval) -> Nutation {
    todo!()
}
