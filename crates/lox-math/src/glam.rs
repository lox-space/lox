use glam::DVec3;

use lox_units::types::units::Radians;

pub trait Azimuth {
    fn azimuth(&self) -> Radians;
}

impl Azimuth for DVec3 {
    fn azimuth(&self) -> Radians {
        self.y.atan2(self.x)
    }
}
