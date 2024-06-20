use glam::{DMat3, DVec3};

pub fn rotation_matrix_derivative(m: DMat3, v: DVec3) -> DMat3 {
    let sx = DVec3::new(0.0, v.z, v.y);
    let sy = DVec3::new(-v.z, 0.0, v.x);
    let sz = DVec3::new(v.y, -v.x, 0.0);
    let s = DMat3::from_cols(sx, sy, sz);
    -s * m
}

pub struct Rotation {
    m: DMat3,
    dm: DMat3,
}

impl Rotation {
    pub fn new(m: DMat3) -> Self {
        Self { m, dm: DMat3::ZERO }
    }

    pub fn with_derivative(mut self, dm: DMat3) -> Self {
        self.dm = dm;
        self
    }

    pub fn with_angular_velocity(mut self, v: DVec3) -> Self {
        self.dm = rotation_matrix_derivative(self.m, v);
        self
    }

    pub fn transpose(&self) -> Self {
        let m = self.m.transpose();
        let dm = self.dm.transpose();
        Self { m, dm }
    }

    pub fn rotate_position(&self, pos: DVec3) -> DVec3 {
        self.m * pos
    }

    pub fn rotate_velocity(&self, pos: DVec3, vel: DVec3) -> DVec3 {
        self.dm * pos + self.m * vel
    }

    pub fn rotate_state(&self, pos: DVec3, vel: DVec3) -> (DVec3, DVec3) {
        (self.rotate_position(pos), self.rotate_velocity(pos, vel))
    }
}
