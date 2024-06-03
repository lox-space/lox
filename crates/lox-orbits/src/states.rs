/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::frames::{
    BodyFixed, FrameTransformationProvider, Icrf, NoOpFrameTransformationProvider, ReferenceFrame,
};
use glam::DVec3;
use lox_bodies::RotationalElements;
use lox_time::{
    julian_dates::JulianDate,
    time_scales::Tdb,
    transformations::{ToTdb, TryToScale},
};
use std::convert::Infallible;

pub trait TryToFrame<
    T,
    O,
    R: ReferenceFrame,
    P: FrameTransformationProvider = NoOpFrameTransformationProvider,
    E = Infallible,
>
{
    fn try_to_frame(&self, frame: R, provider: &P) -> Result<State<T, O, R>, E>;
}

pub struct State<T, O, R: ReferenceFrame> {
    time: T,
    origin: O,
    frame: R,
    position: DVec3,
    velocity: DVec3,
}

impl<T, O, R> State<T, O, R>
where
    R: ReferenceFrame,
{
    pub fn new(time: T, origin: O, frame: R, position: DVec3, velocity: DVec3) -> Self {
        Self {
            time,
            origin,
            frame,
            position,
            velocity,
        }
    }

    pub fn with_frame<U: ReferenceFrame>(&self, frame: U) -> State<T, O, U>
    where
        T: Clone,
        O: Clone,
    {
        State::new(
            self.time(),
            self.origin(),
            frame,
            self.position,
            self.velocity,
        )
    }

    pub fn with_time<U>(&self, time: U) -> State<U, O, R>
    where
        O: Clone,
        R: Clone,
    {
        State::new(
            time,
            self.origin(),
            self.reference_frame(),
            self.position,
            self.velocity,
        )
    }

    pub fn time(&self) -> T
    where
        T: Clone,
    {
        self.time.clone()
    }

    pub fn origin(&self) -> O
    where
        O: Clone,
    {
        self.origin.clone()
    }

    pub fn reference_frame(&self) -> R
    where
        R: Clone,
    {
        self.frame.clone()
    }

    pub fn position(&self) -> DVec3 {
        self.position
    }

    pub fn velocity(&self) -> DVec3 {
        self.velocity
    }
}

impl<T, O, R> TryToFrame<T, O, BodyFixed<R>> for State<T, O, Icrf>
where
    T: ToTdb + JulianDate + Clone,
    O: Clone,
    R: RotationalElements,
{
    fn try_to_frame(
        &self,
        frame: BodyFixed<R>,
        _: &NoOpFrameTransformationProvider,
    ) -> Result<State<T, O, BodyFixed<R>>, Infallible> {
        let rot = frame.rotation(self.time());
        let (pos, vel) = rot.apply(self.position, self.velocity);
        Ok(State::new(self.time(), self.origin(), frame, pos, vel))
    }
}

impl<T, O, R> TryToFrame<T, O, Icrf> for State<T, O, BodyFixed<R>>
where
    T: ToTdb + JulianDate + Clone,
    O: Clone,
    R: RotationalElements,
{
    fn try_to_frame(
        &self,
        frame: Icrf,
        _: &NoOpFrameTransformationProvider,
    ) -> Result<State<T, O, Icrf>, Infallible> {
        let rot = self.frame.rotation(self.time()).transpose();
        let (pos, vel) = rot.apply(self.position, self.velocity);
        Ok(State::new(self.time(), self.origin(), frame, pos, vel))
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use lox_bodies::Jupiter;
    use lox_time::{time, time_scales::Tdb, Time};

    use super::*;

    #[test]
    fn test_bodyfixed() {
        let iau_jupiter = BodyFixed(Jupiter);

        let r0 = DVec3::new(6068.27927, -1692.84394, -2516.61918);
        let v0 = DVec3::new(-0.660415582, 5.495938726, -5.303093233);
        let r1 = DVec3::new(3922.220687351738, 5289.381014412637, -1631.4837924820245);
        let v1 = DVec3::new(-1.852284168309543, -0.8227941105651749, -7.14175174489828);

        let tdb = time!(Tdb, 2000, 1, 1, 12).unwrap();
        let s = State::new(tdb, Jupiter, Icrf, r0, v0);
        let s1 = s
            .try_to_frame(iau_jupiter, &NoOpFrameTransformationProvider)
            .unwrap();
        let s0 = s1
            .try_to_frame(Icrf, &NoOpFrameTransformationProvider)
            .unwrap();

        assert_float_eq!(s0.position().x, r0.x, rel <= 1e-8);
        assert_float_eq!(s0.position().y, r0.y, rel <= 1e-8);
        assert_float_eq!(s0.position().z, r0.z, rel <= 1e-8);
        assert_float_eq!(s0.velocity().x, v0.x, rel <= 1e-8);
        assert_float_eq!(s0.velocity().y, v0.y, rel <= 1e-8);
        assert_float_eq!(s0.velocity().z, v0.z, rel <= 1e-8);

        assert_float_eq!(s1.position().x, r1.x, rel <= 1e-8);
        assert_float_eq!(s1.position().y, r1.y, rel <= 1e-8);
        assert_float_eq!(s1.position().z, r1.z, rel <= 1e-8);
        assert_float_eq!(s1.velocity().x, v1.x, rel <= 1e-8);
        assert_float_eq!(s1.velocity().y, v1.y, rel <= 1e-8);
        assert_float_eq!(s1.velocity().z, v1.z, rel <= 1e-8);
    }
}
