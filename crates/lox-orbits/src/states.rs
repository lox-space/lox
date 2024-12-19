/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */
use glam::{DMat3, DVec3};
use itertools::Itertools;
use lox_bodies::{
    DynOrigin, Origin, PointMass, RotationalElements, Spheroid, TryPointMass, TrySpheroid,
    UndefinedOriginPropertyError,
};
use lox_ephem::{path_from_ids, Ephemeris};
use lox_math::glam::Azimuth;
use lox_math::math::{mod_two_pi, normalize_two_pi};
use lox_math::roots::{BracketError, FindRoot, Secant};
use lox_time::{julian_dates::JulianDate, time_scales::Tdb, transformations::TryToScale, TimeLike};
use std::f64::consts::{PI, TAU};
use std::ops::Sub;
use thiserror::Error;

use crate::anomalies::{eccentric_to_true, hyperbolic_to_true};
use crate::elements::{is_circular, is_equatorial, DynKeplerian, Keplerian, KeplerianElements};
use crate::frames::{
    BodyFixed, CoordinateSystem, DynFrame, FrameTransformationProvider, Icrf, ReferenceFrame,
    TryToFrame,
};
use crate::ground::{DynGroundLocation, GroundLocation};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<T: TimeLike, O: Origin, R: ReferenceFrame> {
    time: T,
    origin: O,
    frame: R,
    position: DVec3,
    velocity: DVec3,
}

pub type DynState<T> = State<T, DynOrigin, DynFrame>;

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

    pub fn origin(&self) -> O
    where
        O: Clone,
    {
        self.origin.clone()
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

fn rotation_lvlh(position: DVec3, velocity: DVec3) -> DMat3 {
    let r = position.normalize();
    let v = velocity.normalize();
    let z = -r;
    let y = -r.cross(v);
    let x = y.cross(z);
    DMat3::from_cols(x, y, z)
}

impl<T, O> State<T, O, Icrf>
where
    T: TimeLike,
    O: Origin,
{
    pub fn rotation_lvlh(&self) -> DMat3 {
        rotation_lvlh(self.position(), self.velocity())
    }
}

impl<T> DynState<T>
where
    T: TimeLike,
{
    pub fn try_rotation_lvlh(&self) -> Result<DMat3, &'static str> {
        if self.frame != DynFrame::Icrf {
            // TODO: better error type
            return Err("only valid for ICRF");
        }
        Ok(rotation_lvlh(self.position(), self.velocity()))
    }
}

type LonLatAlt = (f64, f64, f64);

fn rv_to_lla(r: DVec3, r_eq: f64, f: f64) -> Result<LonLatAlt, BracketError> {
    let rm = r.length();
    let r_delta = (r.x.powi(2) + r.y.powi(2)).sqrt();
    let mut lon = r.y.atan2(r.x);

    if lon.abs() >= PI {
        if lon < 0.0 {
            lon += TAU;
        } else {
            lon -= TAU;
        }
    }

    let delta = (r.z / rm).asin();

    let root_finder = Secant::default();

    let lat = root_finder.find(
        |lat| {
            let e = (2.0 * f - f.powi(2)).sqrt();
            let c = r_eq / (1.0 - e.powi(2) * lat.sin().powi(2)).sqrt();
            (r.z + c * e.powi(2) * lat.sin()) / r_delta - lat.tan()
        },
        delta,
    )?;

    let e = (2.0 * f - f.powi(2)).sqrt();
    let c = r_eq / (1.0 - e.powi(2) * lat.sin().powi(2)).sqrt();

    let alt = r_delta / lat.cos() - c;

    Ok((lon, lat, alt))
}

impl<T, O> State<T, O, BodyFixed<O>>
where
    T: TimeLike,
    O: Origin + RotationalElements + Spheroid + Clone,
{
    pub fn to_ground_location(&self) -> Result<GroundLocation<O>, BracketError> {
        let r = self.position();
        let r_eq = self.origin.equatorial_radius();
        let f = self.origin.flattening();
        let (lon, lat, alt) = rv_to_lla(r, r_eq, f)?;

        Ok(GroundLocation::new(lon, lat, alt, self.origin()))
    }
}

#[derive(Debug, Clone, Error, Eq, PartialEq)]
pub enum StateToDynGroundError {
    #[error("equatorial radius and flattening factor are not available for origin `{}`", .0.name())]
    UndefinedSpheroid(DynOrigin),
    #[error(transparent)]
    BracketError(#[from] BracketError),
    #[error("not a body-fixed frame {0}")]
    NonBodyFixedFrame(String),
}

impl<T: TimeLike + Clone> DynState<T> {
    pub fn to_dyn_ground_location(&self) -> Result<DynGroundLocation, StateToDynGroundError> {
        if !self.frame.is_rotating() {
            return Err(StateToDynGroundError::NonBodyFixedFrame(
                self.frame.name().to_string(),
            ));
        }
        let r = self.position();
        // TODO: Check/transform frame
        let (Ok(r_eq), Ok(f)) = (
            self.origin.try_equatorial_radius(),
            self.origin.try_flattening(),
        ) else {
            return Err(StateToDynGroundError::UndefinedSpheroid(self.origin));
        };

        let (lon, lat, alt) = rv_to_lla(r, r_eq, f)?;

        Ok(DynGroundLocation::with_dynamic(lon, lat, alt, self.origin).unwrap())
    }
}

impl<T, O, R> Sub for State<T, O, R>
where
    T: TimeLike,
    O: Origin,
    R: ReferenceFrame,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let position = self.position - rhs.position;
        let velocity = self.velocity - rhs.velocity;
        State::new(self.time, position, velocity, self.origin, self.frame)
    }
}

fn eccentricity_vector(r: DVec3, v: DVec3, mu: f64) -> DVec3 {
    let rm = r.length();
    let v2 = v.dot(v);
    let rv = r.dot(v);

    ((v2 - mu / rm) * r - rv * v) / mu
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

impl<T, O> State<T, O, Icrf>
where
    T: TimeLike + Clone,
    O: Origin + Clone,
{
    pub fn to_origin<O1: Origin + Clone, E: Ephemeris>(
        &self,
        target: O1,
        ephemeris: &E,
    ) -> Result<State<T, O1, Icrf>, E::Error> {
        // TODO: Fix time scale
        let epoch = self.time().seconds_since_j2000();
        let mut pos = self.position();
        let mut vel = self.velocity();
        let mut pos_eph = DVec3::ZERO;
        let mut vel_eph = DVec3::ZERO;
        let origin_id = self.origin.id();
        let target_id = target.id();
        let path = path_from_ids(origin_id.0, target_id.0);
        for (origin, target) in path.into_iter().tuple_windows() {
            let (p, v) = ephemeris.state(epoch, origin, target)?;
            let p: DVec3 = p.into();
            let v: DVec3 = v.into();
            pos_eph += p;
            vel_eph += v;
        }
        pos -= pos_eph;
        vel -= vel_eph;
        Ok(State::new(self.time(), pos, vel, target, Icrf))
    }
}

impl<T> DynState<T>
where
    T: TimeLike + Clone,
{
    pub fn to_origin_dynamic<O1: Origin + Clone, E: Ephemeris>(
        &self,
        target: O1,
        ephemeris: &E,
    ) -> Result<State<T, O1, DynFrame>, E::Error> {
        // TODO: Fix time scale
        let epoch = self.time().seconds_since_j2000();
        let mut pos = self.position();
        let mut vel = self.velocity();
        let mut pos_eph = DVec3::ZERO;
        let mut vel_eph = DVec3::ZERO;
        let origin_id = self.origin.id();
        let target_id = target.id();
        let path = path_from_ids(origin_id.0, target_id.0);
        for (origin, target) in path.into_iter().tuple_windows() {
            let (p, v) = ephemeris.state(epoch, origin, target)?;
            let p: DVec3 = p.into();
            let v: DVec3 = v.into();
            pos_eph += p;
            vel_eph += v;
        }
        pos -= pos_eph;
        vel -= vel_eph;
        Ok(State::new(self.time(), pos, vel, target, DynFrame::Icrf))
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

pub(crate) fn rv_to_keplerian(r: DVec3, v: DVec3, mu: f64) -> KeplerianElements {
    let rm = r.length();
    let vm = v.length();
    let h = r.cross(v);
    let hm = h.length();
    let node = DVec3::Z.cross(h);
    let e = eccentricity_vector(r, v, mu);
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

    KeplerianElements {
        semi_major_axis,
        eccentricity,
        inclination,
        longitude_of_ascending_node: mod_two_pi(longitude_of_ascending_node),
        argument_of_periapsis: mod_two_pi(argument_of_periapsis),
        true_anomaly: normalize_two_pi(true_anomaly, 0.0),
    }
}

impl<T, O> State<T, O, Icrf>
where
    T: TimeLike + Clone,
    O: PointMass + Clone,
{
    pub fn to_keplerian(&self) -> Keplerian<T, O, Icrf> {
        let r = self.position();
        let v = self.velocity();
        let mu = self.origin.gravitational_parameter();
        let elements = rv_to_keplerian(r, v, mu);

        Keplerian::new(
            self.time(),
            self.origin(),
            elements.semi_major_axis,
            elements.eccentricity,
            elements.inclination,
            elements.longitude_of_ascending_node,
            elements.argument_of_periapsis,
            elements.true_anomaly,
        )
    }
}

impl<T> DynState<T>
where
    T: TimeLike + Clone,
{
    pub fn try_to_keplerian(&self) -> Result<DynKeplerian<T>, UndefinedOriginPropertyError> {
        let mu = self.origin.try_gravitational_parameter()?;

        let r = self.position();
        let v = self.velocity();
        let elements = rv_to_keplerian(r, v, mu);

        Keplerian::with_dynamic(
            self.time(),
            self.origin(),
            elements.semi_major_axis,
            elements.eccentricity,
            elements.inclination,
            elements.longitude_of_ascending_node,
            elements.argument_of_periapsis,
            elements.true_anomaly,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::OnceLock};

    use float_eq::assert_float_eq;

    use lox_bodies::{Earth, Jupiter, Venus};
    use lox_ephem::spk::parser::{parse_daf_spk, Spk};
    use lox_math::assert_close;
    use lox_math::is_close::IsClose;
    use lox_time::{time, time_scales::Tdb, transformations::ToTai, utc::Utc, Time};

    use crate::frames::NoOpFrameTransformationProvider;

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

    #[test]
    fn test_state_to_ground_location() {
        let lat_exp = 51.484f64.to_radians();
        let lon_exp = -35.516f64.to_radians();
        let alt_exp = 237.434; // km

        let position = DVec3::new(3359.927, -2398.072, 5153.0);
        let velocity = DVec3::new(5.0657, 5.485, -0.744);
        let time = time!(Tdb, 2012, 7, 1).unwrap();
        let state = State::new(time, position, velocity, Earth, BodyFixed(Earth));
        let ground = state.to_ground_location().unwrap();
        assert_float_eq!(ground.latitude(), lat_exp, rel <= 1e-4);
        assert_float_eq!(ground.longitude(), lon_exp, rel <= 1e-4);
        assert_float_eq!(ground.altitude(), alt_exp, rel <= 1e-4);
    }

    pub fn data_dir() -> PathBuf {
        PathBuf::from(format!("{}/../../data", env!("CARGO_MANIFEST_DIR")))
    }

    #[test]
    fn test_state_to_origin() {
        let r_venus = DVec3::new(
            1.001977553295792e8,
            2.200234656010247e8,
            9.391473630346918e7,
        );
        let v_venus = DVec3::new(-59.08617935009049, 22.682387107225292, 12.05029567478702);
        let r = DVec3::new(6068279.27, -1692843.94, -2516619.18) / 1e3;

        let v = DVec3::new(-660.415582, 5495.938726, -5303.093233) / 1e3;

        let r_exp = r - r_venus;
        let v_exp = v - v_venus;

        let utc = Utc::from_iso("2016-05-30T12:00:00.000").unwrap();
        let tai = utc.to_tai();

        let s_earth = State::new(tai, r, v, Earth, Icrf);
        let s_venus = s_earth.to_origin(Venus, ephemeris()).unwrap();

        let r_act = s_venus.position();
        let v_act = s_venus.velocity();

        assert_close!(r_act, r_exp);
        assert_close!(v_act, v_exp);
    }

    fn ephemeris() -> &'static Spk {
        let contents = std::fs::read(data_dir().join("de440s.bsp")).unwrap();
        static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
        EPHEMERIS.get_or_init(|| parse_daf_spk(&contents).unwrap())
    }
}
