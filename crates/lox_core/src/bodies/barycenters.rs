use super::NaifId;

pub struct SolarSystemBarycenter;

impl NaifId for SolarSystemBarycenter {
    fn id() -> i32 {
        0
    }
}

pub struct MercuryBarycenter;

impl NaifId for MercuryBarycenter {
    fn id() -> i32 {
        1
    }
}

pub struct VenusBarycenter;

impl NaifId for VenusBarycenter {
    fn id() -> i32 {
        2
    }
}

pub struct EarthBarycenter;

impl NaifId for EarthBarycenter {
    fn id() -> i32 {
        3
    }
}

pub struct MarsBarycenter;

impl NaifId for MarsBarycenter {
    fn id() -> i32 {
        4
    }
}

pub struct JupiterBarycenter;

impl NaifId for JupiterBarycenter {
    fn id() -> i32 {
        5
    }
}

pub struct SaturnBarycenter;

impl NaifId for SaturnBarycenter {
    fn id() -> i32 {
        6
    }
}

pub struct UranusBarycenter;

impl NaifId for UranusBarycenter {
    fn id() -> i32 {
        7
    }
}

pub struct NeptuneBarycenter;

impl NaifId for NeptuneBarycenter {
    fn id() -> i32 {
        8
    }
}

pub struct PlutoBarycenter;

impl NaifId for PlutoBarycenter {
    fn id() -> i32 {
        9
    }
}
