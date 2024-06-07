/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::anomalies::{eccentric_to_true, hyperbolic_to_true};
use crate::elements::{
    azimuth, eccentricity_vector, is_circular, is_equatorial, Keplerian, ToKeplerian,
};
use crate::{
    frames::{
        BodyFixed, CoordinateSystem, FrameTransformationProvider, Icrf,
        NoOpFrameTransformationProvider, ReferenceFrame, TryToFrame,
    },
    origins::{CoordinateOrigin, Origin},
};
use glam::DVec3;
use lox_bodies::{PointMass, RotationalElements};
use lox_time::{
    julian_dates::JulianDate,
    time_scales::Tdb,
    transformations::{ToTdb, TryToScale},
    ut1::DeltaUt1TaiProvider,
    Datetime,
};
use std::convert::Infallible;

pub trait ToCartesian<T: Datetime, O: Origin, R: ReferenceFrame> {
    fn to_cartesian(&self) -> State<T, O, R>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<T: Datetime, O: Origin, R: ReferenceFrame> {
    time: T,
    origin: O,
    frame: R,
    position: DVec3,
    velocity: DVec3,
}

impl<T, O, R> State<T, O, R>
where
    T: Datetime,
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
    T: Datetime,
    O: Origin,
    R: ReferenceFrame + Clone,
{
    fn reference_frame(&self) -> R {
        self.frame.clone()
    }
}

impl<T, O, R> CoordinateOrigin<O> for State<T, O, R>
where
    T: Datetime,
    O: Origin + Clone,
    R: ReferenceFrame,
{
    fn origin(&self) -> O {
        self.origin.clone()
    }
}

impl<T, O, R, U> TryToFrame<T, O, BodyFixed<R>, U> for State<T, O, Icrf>
where
    T: TryToScale<Tdb, U> + Datetime + Clone,
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
    T: TryToScale<Tdb, U> + Datetime + Clone,
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
    T: ToTdb + Datetime + Clone,
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

impl<T, O, R> ToKeplerian<T, O> for State<T, O, R>
where
    T: Datetime + Clone,
    O: PointMass + Clone,
    R: ReferenceFrame,
{
    fn to_keplerian(&self) -> Keplerian<T, O> {
        let grav_param = self.origin.gravitational_parameter();
        let r = self.position.length();
        let v = self.velocity.length();
        let h = self.position.cross(self.velocity);
        let hm = h.length();
        let node = DVec3::Z.cross(h);
        let e = eccentricity_vector(grav_param, self.position, self.velocity);
        let eccentricity = e.length();
        let inclination = h.angle_between(DVec3::Z);

        let equatorial = is_equatorial(inclination);
        let circular = is_circular(eccentricity);

        let semi_major = if circular {
            hm.powi(2) / grav_param
        } else {
            -grav_param / (2.0 * (v.powi(2) / 2.0 - grav_param / r))
        };

        let ascending_node;
        let periapsis_arg;
        let true_anomaly;
        if equatorial && !circular {
            ascending_node = 0.0;
            periapsis_arg = azimuth(e);
            true_anomaly = (h.dot(e.cross(self.position)) / hm).atan2(self.position.dot(e));
        } else if !equatorial && circular {
            ascending_node = azimuth(node);
            periapsis_arg = 0.0;
            true_anomaly = (self.position.dot(h.cross(node)) / hm).atan2(self.position.dot(node));
        } else if equatorial && circular {
            ascending_node = 0.0;
            periapsis_arg = 0.0;
            true_anomaly = azimuth(self.position);
        } else {
            if semi_major > 0.0 {
                let e_se = self.position.dot(self.velocity) / (grav_param * semi_major).sqrt();
                let e_ce = (r * v.powi(2)) / grav_param - 1.0;
                true_anomaly = eccentric_to_true(e_se.atan2(e_ce), eccentricity);
            } else {
                let e_sh = self.position.dot(self.velocity) / (-grav_param * semi_major).sqrt();
                let e_ch = (r * v.powi(2)) / grav_param - 1.0;
                true_anomaly =
                    hyperbolic_to_true(((e_ch + e_sh) / (e_ch - e_sh)).ln() / 2.0, eccentricity);
            }
            ascending_node = azimuth(node);
            let px = self.position.dot(node);
            let py = self.position.dot(h.cross(node)) / hm;
            periapsis_arg = py.atan2(px) - true_anomaly;
        }

        Keplerian::new(
            self.time(),
            self.origin(),
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        )
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
