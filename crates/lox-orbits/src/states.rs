/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use glam::DVec3;

use lox_bodies::{PointMass, RotationalElements};
use lox_time::{julian_dates::JulianDate, time_scales::Tdb, transformations::TryToScale, TimeLike};
use lox_utils::glam::Azimuth;
use lox_utils::math::{mod_two_pi, normalize_two_pi};

use crate::anomalies::{eccentric_to_true, hyperbolic_to_true};
use crate::elements::{is_circular, is_equatorial, Keplerian, ToKeplerian};
use crate::{
    frames::{
        BodyFixed, CoordinateSystem, FrameTransformationProvider, Icrf, ReferenceFrame, TryToFrame,
    },
    origins::{CoordinateOrigin, Origin},
};

pub trait ToCartesian<T: TimeLike, O: Origin, R: ReferenceFrame> {
    fn to_cartesian(&self) -> State<T, O, R>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<T: TimeLike, O: Origin, R: ReferenceFrame> {
    time: T,
    origin: O,
    frame: R,
    position: DVec3,
    velocity: DVec3,
}

impl<T, O, R> State<T, O, R>
where
    T: TimeLike,
    O: Origin,
    R: ReferenceFrame,
{
    pub fn new(time: T, position: DVec3, velocity: DVec3, origin: O, frame: R) -> Self {
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
            self.position,
            self.velocity,
            self.origin(),
            frame,
        )
    }

    pub fn with_origin<U: Origin>(&self, origin: U) -> State<T, U, R>
    where
        T: Clone,
        R: Clone,
    {
        State::new(
            self.time(),
            self.position,
            self.velocity,
            origin,
            self.reference_frame(),
        )
    }

    pub fn with_origin_and_frame<U: Origin, V: ReferenceFrame>(
        &self,
        origin: U,
        frame: V,
    ) -> State<T, U, V>
    where
        T: Clone,
    {
        State::new(self.time(), self.position, self.velocity, origin, frame)
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

impl<T, O, R> State<T, O, R>
where
    T: TimeLike,
    O: PointMass,
    R: ReferenceFrame,
{
    fn eccentricity_vector(&self) -> DVec3 {
        let r = self.position();
        let v = self.velocity();
        let mu = self.origin.gravitational_parameter();

        let rm = r.length();
        let v2 = v.dot(v);
        let rv = r.dot(v);

        ((v2 - mu / rm) * r - rv * v) / mu
    }
}

impl<T, O, R> CoordinateSystem<R> for State<T, O, R>
where
    T: TimeLike,
    O: Origin,
    R: ReferenceFrame + Clone,
{
    fn reference_frame(&self) -> R {
        self.frame.clone()
    }
}

impl<T, O, R> CoordinateOrigin<O> for State<T, O, R>
where
    T: TimeLike,
    O: Origin + Clone,
    R: ReferenceFrame,
{
    fn origin(&self) -> O {
        self.origin.clone()
    }
}

impl<T, O, R, U> TryToFrame<BodyFixed<R>, U> for State<T, O, Icrf>
where
    T: TryToScale<Tdb, U> + TimeLike + Clone,
    O: Origin + Clone,
    R: RotationalElements + Clone,
    U: FrameTransformationProvider,
{
    type Output = State<T, O, BodyFixed<R>>;
    type Error = U::Error;

    fn try_to_frame(&self, frame: BodyFixed<R>, provider: &U) -> Result<Self::Output, U::Error> {
        let seconds = self
            .time()
            .try_to_scale(Tdb, provider)?
            .seconds_since_j2000();
        let rot = frame.rotation(seconds);
        let (pos, vel) = rot.rotate_state(self.position, self.velocity);
        Ok(State::new(self.time(), pos, vel, self.origin(), frame))
    }
}

impl<T, O, R, U> TryToFrame<Icrf, U> for State<T, O, BodyFixed<R>>
where
    T: TryToScale<Tdb, U> + TimeLike + Clone,
    O: Origin + Clone,
    R: RotationalElements,
    U: FrameTransformationProvider,
{
    type Output = State<T, O, Icrf>;
    type Error = U::Error;

    fn try_to_frame(&self, frame: Icrf, provider: &U) -> Result<Self::Output, U::Error> {
        let seconds = self
            .time()
            .try_to_scale(Tdb, provider)?
            .seconds_since_j2000();
        let rot = self.frame.rotation(seconds).transpose();
        let (pos, vel) = rot.rotate_state(self.position, self.velocity);
        Ok(State::new(self.time(), pos, vel, self.origin(), frame))
    }
}

impl<T, O, R> ToKeplerian<T, O> for State<T, O, R>
where
    T: TimeLike + Clone,
    O: PointMass + Clone,
    R: ReferenceFrame,
{
    fn to_keplerian(&self) -> Keplerian<T, O> {
        let mu = self.origin.gravitational_parameter();
        let r = self.position();
        let v = self.velocity();
        let rm = r.length();
        let vm = v.length();
        let h = r.cross(v);
        let hm = h.length();
        let node = DVec3::Z.cross(h);
        let e = self.eccentricity_vector();
        let eccentricity = e.length();
        let inclination = h.angle_between(DVec3::Z);

        let equatorial = is_equatorial(inclination);
        let circular = is_circular(eccentricity);

        let semi_major_axis = if circular {
            hm.powi(2) / mu
        } else {
            -mu / (2.0 * (vm.powi(2) / 2.0 - mu / rm))
        };

        let longitude_of_ascending_node;
        let argument_of_periapsis;
        let true_anomaly;
        if equatorial && !circular {
            longitude_of_ascending_node = 0.0;
            argument_of_periapsis = e.azimuth();
            true_anomaly = (h.dot(e.cross(r)) / hm).atan2(r.dot(e));
        } else if !equatorial && circular {
            longitude_of_ascending_node = node.azimuth();
            argument_of_periapsis = 0.0;
            true_anomaly = (r.dot(h.cross(node)) / hm).atan2(r.dot(node));
        } else if equatorial && circular {
            longitude_of_ascending_node = 0.0;
            argument_of_periapsis = 0.0;
            true_anomaly = r.azimuth();
        } else {
            if semi_major_axis > 0.0 {
                let e_se = r.dot(v) / (mu * semi_major_axis).sqrt();
                let e_ce = (rm * vm.powi(2)) / mu - 1.0;
                true_anomaly = eccentric_to_true(e_se.atan2(e_ce), eccentricity);
            } else {
                let e_sh = r.dot(v) / (-mu * semi_major_axis).sqrt();
                let e_ch = (rm * vm.powi(2)) / mu - 1.0;
                true_anomaly =
                    hyperbolic_to_true(((e_ch + e_sh) / (e_ch - e_sh)).ln() / 2.0, eccentricity);
            }
            longitude_of_ascending_node = node.azimuth();
            let px = r.dot(node);
            let py = r.dot(h.cross(node)) / hm;
            argument_of_periapsis = py.atan2(px) - true_anomaly;
        }

        Keplerian::new(
            self.time(),
            self.origin(),
            semi_major_axis,
            eccentricity,
            inclination,
            mod_two_pi(longitude_of_ascending_node),
            mod_two_pi(argument_of_periapsis),
            normalize_two_pi(true_anomaly, 0.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use crate::frames::NoOpFrameTransformationProvider;
    use lox_bodies::{Earth, Jupiter};
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
        let s = State::new(tdb, r0, v0, Jupiter, Icrf);
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

    #[test]
    fn test_state_to_keplerian_roundtrip() {
        let time = time!(Tdb, 2023, 3, 25, 21, 8, 0.0).expect("time should be valid");
        let pos = DVec3::new(
            -0.107622532467967e7,
            -0.676589636432773e7,
            -0.332308783350379e6,
        ) * 1e-3;
        let vel = DVec3::new(
            0.935685775154103e4,
            -0.331234775037644e4,
            -0.118801577532701e4,
        ) * 1e-3;

        let cartesian = State::new(time, pos, vel, Earth, Icrf);
        let cartesian1 = cartesian.to_keplerian().to_cartesian();

        assert_eq!(cartesian1.time(), time);
        assert_eq!(cartesian1.origin(), Earth);
        assert_eq!(cartesian1.reference_frame(), Icrf);

        assert_float_eq!(cartesian.position().x, cartesian1.position().x, rel <= 1e-8);
        assert_float_eq!(cartesian.position().y, cartesian1.position().y, rel <= 1e-8);
        assert_float_eq!(cartesian.position().z, cartesian1.position().z, rel <= 1e-8);
        assert_float_eq!(cartesian.velocity().x, cartesian1.velocity().x, rel <= 1e-6);
        assert_float_eq!(cartesian.velocity().y, cartesian1.velocity().y, rel <= 1e-6);
        assert_float_eq!(cartesian.velocity().z, cartesian1.velocity().z, rel <= 1e-6);
    }
}