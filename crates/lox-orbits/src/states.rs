// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-License-Identifier: MPL-2.0

use std::f64::consts::{PI, TAU};
use std::ops::Sub;

use glam::{DMat3, DVec3};
use itertools::Itertools;
use lox_bodies::{
    DynOrigin, Origin, PointMass, RotationalElements, Spheroid, TryPointMass, TrySpheroid,
    UndefinedOriginPropertyError,
};
use lox_ephem::{Ephemeris, path_from_ids};
use lox_frames::transformations::TryTransform;
use lox_math::{
    glam::Azimuth,
    roots::{BracketError, FindRoot, Secant},
};
use lox_time::{Time, julian_dates::JulianDate, time_scales::DynTimeScale, time_scales::TimeScale};
use lox_units::{Angle, AngleUnits};
use thiserror::Error;

use crate::anomalies::{eccentric_to_true, hyperbolic_to_true};
use crate::elements::{DynKeplerian, Keplerian, KeplerianElements, is_circular, is_equatorial};
use crate::ground::{DynGroundLocation, GroundLocation};
use lox_frames::{DynFrame, Iau, Icrf, ReferenceFrame};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<T: TimeScale, O: Origin, R: ReferenceFrame> {
    time: Time<T>,
    origin: O,
    frame: R,
    position: DVec3,
    velocity: DVec3,
}

pub type DynState = State<DynTimeScale, DynOrigin, DynFrame>;

impl<T, O, R> State<T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    pub fn new(time: Time<T>, position: DVec3, velocity: DVec3, origin: O, frame: R) -> Self {
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

    pub fn reference_frame(&self) -> R
    where
        R: Clone,
    {
        self.frame.clone()
    }

    pub fn time(&self) -> Time<T>
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

    pub fn try_to_frame<R1, P>(&self, frame: R1, provider: &P) -> Result<State<T, O, R1>, P::Error>
    where
        R: Copy,
        P: TryTransform<R, R1, T>,
        R1: ReferenceFrame + Copy,
        O: Clone,
        T: Copy,
    {
        let rot = provider.try_transform(self.frame, frame, self.time)?;
        let (r1, v1) = rot.rotate_state(self.position, self.velocity);
        Ok(State::new(self.time(), r1, v1, self.origin(), frame))
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
    T: TimeScale,
    O: Origin,
{
    pub fn rotation_lvlh(&self) -> DMat3 {
        rotation_lvlh(self.position(), self.velocity())
    }
}

impl DynState {
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

impl<T, O> State<T, O, Iau<O>>
where
    T: TimeScale,
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

impl DynState {
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
    T: TimeScale,
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

impl<T, O> State<T, O, Icrf>
where
    T: TimeScale + Clone,
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

impl DynState {
    pub fn to_origin_dynamic<E: Ephemeris>(
        &self,
        target: DynOrigin,
        ephemeris: &E,
    ) -> Result<DynState, E::Error> {
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
        longitude_of_ascending_node: longitude_of_ascending_node.rad().mod_two_pi().to_radians(),
        argument_of_periapsis: argument_of_periapsis.rad().mod_two_pi().to_radians(),
        true_anomaly: true_anomaly
            .rad()
            .normalize_two_pi(Angle::ZERO)
            .to_radians(),
    }
}

impl<T, O> State<T, O, Icrf>
where
    T: TimeScale + Clone,
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

impl DynState {
    pub fn try_to_keplerian(&self) -> Result<DynKeplerian, UndefinedOriginPropertyError> {
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
    use std::sync::OnceLock;

    use lox_bodies::{Earth, Jupiter, Venus};
    use lox_ephem::spk::parser::{Spk, parse_daf_spk};
    use lox_frames::providers::DefaultTransformProvider;
    use lox_test_utils::{assert_approx_eq, data_dir};
    use lox_time::{Time, time, time_scales::Tdb, utc::Utc};

    use super::*;

    #[test]
    fn test_bodyfixed() {
        let iau_jupiter = Iau::new(Jupiter);

        let r0 = DVec3::new(6068.27927, -1692.84394, -2516.61918);
        let v0 = DVec3::new(-0.660415582, 5.495938726, -5.303093233);
        let r1 = DVec3::new(3922.220687351738, 5289.381014412637, -1631.4837924820245);
        let v1 = DVec3::new(-1.852284168309543, -0.8227941105651749, -7.14175174489828);

        let tdb = time!(Tdb, 2000, 1, 1, 12).unwrap();
        let s = State::new(tdb, r0, v0, Jupiter, Icrf);
        let s1 = s
            .try_to_frame(iau_jupiter, &DefaultTransformProvider)
            .unwrap();
        let s0 = s1.try_to_frame(Icrf, &DefaultTransformProvider).unwrap();

        assert_approx_eq!(s0.position(), r0, rtol <= 1e-8);
        assert_approx_eq!(s0.velocity(), v0, rtol <= 1e-8);

        assert_approx_eq!(s1.position(), r1, rtol <= 1e-8);
        assert_approx_eq!(s1.velocity(), v1, rtol <= 1e-8);
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

        assert_approx_eq!(cartesian.position(), cartesian1.position(), rtol <= 1e-8);
        assert_approx_eq!(cartesian.velocity(), cartesian1.velocity(), rtol <= 1e-6);
    }

    #[test]
    fn test_state_to_ground_location() {
        let lat_exp = 51.484f64.to_radians();
        let lon_exp = -35.516f64.to_radians();
        let alt_exp = 237.434; // km

        let position = DVec3::new(3359.927, -2398.072, 5153.0);
        let velocity = DVec3::new(5.0657, 5.485, -0.744);
        let time = time!(Tdb, 2012, 7, 1).unwrap();
        let state = State::new(time, position, velocity, Earth, Iau::new(Earth));
        let ground = state.to_ground_location().unwrap();
        assert_approx_eq!(ground.latitude(), lat_exp, rtol <= 1e-4);
        assert_approx_eq!(ground.longitude(), lon_exp, rtol <= 1e-4);
        assert_approx_eq!(ground.altitude(), alt_exp, rtol <= 1e-4);
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
        let tai = utc.to_time();

        let s_earth = State::new(tai, r, v, Earth, Icrf);
        let s_venus = s_earth.to_origin(Venus, ephemeris()).unwrap();

        let r_act = s_venus.position();
        let v_act = s_venus.velocity();

        assert_approx_eq!(r_act, r_exp);
        assert_approx_eq!(v_act, v_exp);
    }

    fn ephemeris() -> &'static Spk {
        let contents = std::fs::read(data_dir().join("de440s.bsp")).unwrap();
        static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
        EPHEMERIS.get_or_init(|| parse_daf_spk(&contents).unwrap())
    }
}
