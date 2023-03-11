use crate::bodies::NaifId;

pub struct Mercury;

impl NaifId for Mercury {
    fn id() -> i32 {
        199
    }
}

pub struct Venus;

impl NaifId for Venus {
    fn id() -> i32 {
        299
    }
}

pub struct Earth;

impl NaifId for Earth {
    fn id() -> i32 {
        399
    }
}

pub struct Mars;

impl NaifId for Mars {
    fn id() -> i32 {
        499
    }
}

pub struct Jupiter;

impl NaifId for Jupiter {
    fn id() -> i32 {
        599
    }
}

pub struct Saturn;

impl NaifId for Saturn {
    fn id() -> i32 {
        699
    }
}

pub struct Uranus;

impl NaifId for Uranus {
    fn id() -> i32 {
        799
    }
}

pub struct Neptune;

impl NaifId for Neptune {
    fn id() -> i32 {
        899
    }
}

pub struct Pluto;

impl NaifId for Pluto {
    fn id() -> i32 {
        999
    }
}
