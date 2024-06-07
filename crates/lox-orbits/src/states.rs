/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
    frames::{
        BodyFixed, CoordinateSystem, FrameTransformationProvider, Icrf,
        NoOpFrameTransformationProvider, ReferenceFrame, TryToFrame,
    },
    origins::{CoordinateOrigin, Origin},
};
use glam::DVec3;
use lox_bodies::RotationalElements;
use lox_time::{
    julian_dates::JulianDate,
    time_scales::Tdb,
    transformations::{ToTdb, TryToScale},
    ut1::DeltaUt1TaiProvider,
};
use std::convert::Infallible;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<T, O: Origin, R: ReferenceFrame> {
    time: T,
    origin: O,
    frame: R,
    position: DVec3,
    velocity: DVec3,
}

impl<T, O, R> State<T, O, R>
where
    O: Origin,
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

    pub fn time(&self) -> T
    where
        T: Clone,
    {
        self.time.clone()
    }

    pub fn position(&self) -> DVec3 {
        self.position
    }

    pub fn velocity(&self) -> DVec3 {
        self.velocity
    }
}

impl<T, O, R> CoordinateSystem<R> for State<T, O, R>
where
    O: Origin,
    R: ReferenceFrame + Clone,
{
    fn reference_frame(&self) -> R {
        self.frame.clone()
    }
}

impl<T, O, R> CoordinateOrigin<O> for State<T, O, R>
where
    O: Origin + Clone,
    R: ReferenceFrame,
{
    fn origin(&self) -> O {
        self.origin.clone()
    }
}

impl<T, O, R, U> TryToFrame<T, O, BodyFixed<R>, U> for State<T, O, Icrf>
where
    T: TryToScale<Tdb, U> + JulianDate + Clone,
    O: Origin + Clone,
    R: RotationalElements + Clone,
    U: FrameTransformationProvider,
{
    type Output = State<T, O, BodyFixed<R>>;

    fn try_to_frame(&self, frame: BodyFixed<R>, provider: &U) -> Result<Self::Output, U::Error> {
        let seconds = self
            .time()
            .try_to_scale(Tdb, provider)?
            .seconds_since_j2000();
        let rot = frame.rotation(seconds);
        let (pos, vel) = rot.apply(self.position, self.velocity);
        Ok(State::new(self.time(), self.origin(), frame, pos, vel))
    }
}

impl<T, O, R, U> TryToFrame<T, O, Icrf, U> for State<T, O, BodyFixed<R>>
where
    T: TryToScale<Tdb, U> + JulianDate + Clone,
    O: Origin + Clone,
    R: RotationalElements,
    U: DeltaUt1TaiProvider + FrameTransformationProvider,
{
    type Output = State<T, O, Icrf>;

    fn try_to_frame(&self, frame: Icrf, provider: &U) -> Result<Self::Output, U::Error> {
        let seconds = self
            .time()
            .try_to_scale(Tdb, provider)?
            .seconds_since_j2000();
        let rot = self.frame.rotation(seconds).transpose();
        let (pos, vel) = rot.apply(self.position, self.velocity);
        Ok(State::new(self.time(), self.origin(), frame, pos, vel))
    }
}

impl<T, O, R> TryToFrame<T, O, Icrf, NoOpFrameTransformationProvider> for State<T, O, BodyFixed<R>>
where
    T: ToTdb + JulianDate + Clone,
    O: Origin + Clone,
    R: RotationalElements,
{
    type Output = State<T, O, Icrf>;

    fn try_to_frame(
        &self,
        frame: Icrf,
        _provider: &NoOpFrameTransformationProvider,
    ) -> Result<Self::Output, Infallible> {
        let seconds = self.time().to_tdb().seconds_since_j2000();
        let rot = self.frame.rotation(seconds).transpose();
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
