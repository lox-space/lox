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
use lox_time::{julian_dates::JulianDate, transformations::ToTdb};
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
        let s0 = State::new(tdb, Jupiter, Icrf, r0, v0);
        let s1 = s0.try_to_frame(iau_jupiter, &NoOpFrameTransformationProvider);
    }
}
