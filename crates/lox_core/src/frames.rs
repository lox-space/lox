/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Debug, Display, Formatter};

use glam::{DMat3, DVec3};

pub mod iau;

// TODO: Replace with proper `Epoch` type
type Epoch = f64;

pub trait ReferenceFrame {
    fn is_inertial(&self) -> bool;

    fn is_rotating(&self) -> bool {
        !self.is_inertial()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Icrf;

impl Display for Icrf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ICRF")
    }
}

impl ReferenceFrame for Icrf {
    fn is_inertial(&self) -> bool {
        true
    }
}

pub fn rotation_matrix_derivative(m: DMat3, v: DVec3) -> DMat3 {
    let sx = DVec3::new(0.0, v.z, v.y);
    let sy = DVec3::new(-v.z, 0.0, v.x);
    let sz = DVec3::new(v.y, -v.x, 0.0);
    let s = DMat3::from_cols(sx, sy, sz);
    -s * m
}

type State = (DVec3, DVec3);

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

    pub fn with_velocity(mut self, v: DVec3) -> Self {
        self.dm = rotation_matrix_derivative(self.m, v);
        self
    }

    fn transpose(&self) -> Self {
        let m = self.m.transpose();
        let dm = self.dm.transpose();
        Self { m, dm }
    }

    pub fn apply(&self, (pos, vel): State) -> State {
        (self.m * pos, self.dm * pos + self.m * vel)
    }
}

pub trait FromFrame<T: ReferenceFrame> {
    fn rotation_from(&self, frame: T, t: Epoch) -> Rotation;

    fn transform_from(&self, frame: T, t: Epoch, state: State) -> State {
        let rotation = self.rotation_from(frame, t);
        rotation.apply(state)
    }
}

impl<T: ReferenceFrame> FromFrame<T> for T {
    fn rotation_from(&self, _: T, _: Epoch) -> Rotation {
        Rotation::new(DMat3::IDENTITY)
    }
}

pub trait IntoFrame<T: ReferenceFrame> {
    fn rotation_into(&self, frame: T, t: Epoch) -> Rotation;

    fn transform_into(&self, frame: T, t: Epoch, state: State) -> State {
        let rotation = self.rotation_into(frame, t);
        rotation.apply(state)
    }
}

impl<T, U: ReferenceFrame> IntoFrame<U> for T
where
    T: FromFrame<U>,
{
    fn rotation_into(&self, frame: U, t: Epoch) -> Rotation {
        self.rotation_from(frame, t).transpose()
    }
}
