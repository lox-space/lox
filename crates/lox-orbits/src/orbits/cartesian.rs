// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::ops::Sub;

use glam::{DMat3, DVec3};
use lox_bodies::{
    DynOrigin, Origin, PointMass, RotationalElements, Spheroid, TryPointMass, TrySpheroid,
    UndefinedOriginPropertyError,
};
use lox_core::coords::Cartesian;
use lox_core::coords::LonLatAlt;
use lox_ephem::Ephemeris;
use lox_frames::{
    DynFrame, Iau, Icrf, ReferenceFrame, TryBodyFixed, rotations::TryRotation, traits::frame_id,
};
use lox_math::roots::RootFinderError;
use lox_time::offsets::{DefaultOffsetProvider, Offset};
use lox_time::time_scales::{Tdb, TimeScale};
use thiserror::Error;

use crate::ground::{DynGroundLocation, GroundLocation};

use super::{CartesianOrbit, DynCartesianOrbit, KeplerianOrbit, Orbit};

impl<T, O, R> CartesianOrbit<T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    pub const fn new(cartesian: Cartesian, time: lox_time::Time<T>, origin: O, frame: R) -> Self {
        Self::from_state(cartesian, time, origin, frame)
    }

    pub fn position(&self) -> DVec3 {
        self.state().position()
    }

    pub fn velocity(&self) -> DVec3 {
        self.state().velocity()
    }

    pub fn to_keplerian(&self) -> KeplerianOrbit<T, O, R>
    where
        T: Copy,
        O: Copy + PointMass,
        R: Copy,
    {
        Orbit::from_state(
            self.state().to_keplerian(self.gravitational_parameter()),
            self.time(),
            self.origin(),
            self.reference_frame(),
        )
    }

    pub fn try_to_keplerian(&self) -> Result<KeplerianOrbit<T, O, R>, UndefinedOriginPropertyError>
    where
        T: Copy,
        O: Copy + TryPointMass,
        R: Copy,
    {
        Ok(Orbit::from_state(
            self.state()
                .to_keplerian(self.try_gravitational_parameter()?),
            self.time(),
            self.origin(),
            self.reference_frame(),
        ))
    }

    pub fn try_to_frame<R1, P>(
        &self,
        frame: R1,
        provider: &P,
    ) -> Result<CartesianOrbit<T, O, R1>, P::Error>
    where
        R: Copy,
        P: TryRotation<R, R1, T>,
        R1: ReferenceFrame + Copy,
        O: Copy,
        T: Copy,
    {
        if let (Some(id0), Some(id1)) = (frame_id(&self.reference_frame()), frame_id(&frame))
            && id0 == id1
        {
            return Ok(CartesianOrbit::new(
                self.state(),
                self.time(),
                self.origin(),
                frame,
            ));
        }
        let rot = provider.try_rotation(self.reference_frame(), frame, self.time())?;
        let (r1, v1) = rot.rotate_state(self.state().position(), self.state().velocity());
        Ok(CartesianOrbit::new(
            Cartesian::from_vecs(r1, v1),
            self.time(),
            self.origin(),
            frame,
        ))
    }
}

impl<T, O> CartesianOrbit<T, O, Iau<O>>
where
    T: TimeScale,
    O: Origin + RotationalElements + Spheroid + Copy,
{
    pub fn to_ground_location(&self) -> Result<GroundLocation<O>, RootFinderError> {
        let origin = self.origin();
        let coords = LonLatAlt::from_body_fixed(
            self.position(),
            origin.equatorial_radius(),
            origin.flattening(),
        )?;
        Ok(GroundLocation::new(coords, origin))
    }
}

#[derive(Debug, Error)]
pub enum StateToDynGroundError {
    #[error("equatorial radius and flattening factor are not available for origin `{}`", .0.name())]
    UndefinedSpheroid(DynOrigin),
    #[error(transparent)]
    RootFinderError(#[from] RootFinderError),
    #[error("not a body-fixed frame {0}")]
    NonBodyFixedFrame(String),
}

fn rotation_lvlh(position: DVec3, velocity: DVec3) -> DMat3 {
    let r = position.normalize();
    let v = velocity.normalize();
    let z = -r;
    let y = -r.cross(v);
    let x = y.cross(z);
    DMat3::from_cols(x, y, z)
}

impl<T, O> CartesianOrbit<T, O, Icrf>
where
    T: TimeScale,
    O: Origin,
{
    pub fn rotation_lvlh(&self) -> DMat3 {
        rotation_lvlh(self.position(), self.velocity())
    }
}

impl<T, O> CartesianOrbit<T, O, Icrf>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
{
    pub fn to_origin<O1: Origin + Copy, E: Ephemeris>(
        &self,
        target: O1,
        ephemeris: &E,
    ) -> Result<CartesianOrbit<T, O1, Icrf>, E::Error>
    where
        DefaultOffsetProvider: Offset<T, Tdb>,
    {
        if self.origin().id() == target.id() {
            return Ok(CartesianOrbit::new(self.state(), self.time(), target, Icrf));
        }
        let tdb = self.time().to_scale(Tdb);
        let delta = ephemeris.state(tdb, self.origin(), target)?;
        Ok(CartesianOrbit::new(
            self.state() - delta,
            self.time(),
            target,
            Icrf,
        ))
    }
}

impl DynCartesianOrbit {
    pub fn try_to_origin<E: Ephemeris>(
        &self,
        target: DynOrigin,
        ephemeris: &E,
    ) -> Result<DynCartesianOrbit, E::Error> {
        if self.origin() == target {
            return Ok(CartesianOrbit::new(
                self.state(),
                self.time(),
                target,
                self.reference_frame(),
            ));
        }
        let tdb = self
            .time()
            .try_to_scale(Tdb, &DefaultOffsetProvider)
            .unwrap();
        let delta = ephemeris.state(tdb, self.origin(), target)?;
        Ok(CartesianOrbit::new(
            self.state() - delta,
            self.time(),
            target,
            DynFrame::Icrf,
        ))
    }

    pub fn try_to_ground_location(&self) -> Result<DynGroundLocation, StateToDynGroundError> {
        let frame = self.reference_frame();
        let origin = self.origin();
        if frame.try_body_fixed().is_err() {
            return Err(StateToDynGroundError::NonBodyFixedFrame(
                frame.name().to_string(),
            ));
        }
        let (Ok(r_eq), Ok(f)) = (origin.try_equatorial_radius(), origin.try_flattening()) else {
            return Err(StateToDynGroundError::UndefinedSpheroid(origin));
        };

        let coords = LonLatAlt::from_body_fixed(self.position(), r_eq, f)?;
        Ok(DynGroundLocation::try_new(coords, origin).unwrap())
    }

    pub fn try_rotation_lvlh(&self) -> Result<DMat3, &'static str> {
        if self.reference_frame() != DynFrame::Icrf {
            return Err("only valid for ICRF");
        }
        Ok(rotation_lvlh(self.position(), self.velocity()))
    }
}

impl<T, O, R> Sub for CartesianOrbit<T, O, R>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
    R: ReferenceFrame + Copy,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let state = Cartesian::from_vecs(
            self.position() - rhs.position(),
            self.velocity() - rhs.velocity(),
        );
        CartesianOrbit::new(state, self.time(), self.origin(), self.reference_frame())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use glam::DVec3;
    use lox_bodies::{Earth, Jupiter, Venus};
    use lox_core::coords::Cartesian;
    use lox_ephem::Ephemeris;
    use lox_ephem::spk::parser::Spk;
    use lox_frames::providers::DefaultRotationProvider;
    use lox_test_utils::{assert_approx_eq, data_file};
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
        let s = CartesianOrbit::new(Cartesian::from_vecs(r0, v0), tdb, Jupiter, Icrf);
        let s1 = s
            .try_to_frame(iau_jupiter, &DefaultRotationProvider)
            .unwrap();
        let s0 = s1.try_to_frame(Icrf, &DefaultRotationProvider).unwrap();

        assert_approx_eq!(s0.position(), r0, rtol <= 1e-8);
        assert_approx_eq!(s0.velocity(), v0, rtol <= 1e-8);

        assert_approx_eq!(s1.position(), r1, rtol <= 1e-8);
        assert_approx_eq!(s1.velocity(), v1, rtol <= 1e-8);
    }

    #[test]
    fn test_cartesian_to_keplerian_roundtrip() {
        let time = time!(Tdb, 2023, 3, 25, 21, 8, 0.0).expect("time should be valid");
        let pos = DVec3::new(
            -0.107622532467967e7,
            -0.676589636432773e7,
            -0.332308783350379e6,
        );
        let vel = DVec3::new(
            0.935685775154103e4,
            -0.331234775037644e4,
            -0.118801577532701e4,
        );

        let cartesian = CartesianOrbit::new(Cartesian::from_vecs(pos, vel), time, Earth, Icrf);
        let cartesian1 = cartesian.to_keplerian().to_cartesian();

        assert_eq!(cartesian1.time(), time);
        assert_eq!(cartesian1.origin(), Earth);
        assert_eq!(cartesian1.reference_frame(), Icrf);

        assert_approx_eq!(cartesian.position(), cartesian1.position(), rtol <= 1e-8);
        assert_approx_eq!(cartesian.velocity(), cartesian1.velocity(), rtol <= 1e-6);
    }

    #[test]
    fn test_to_ground_location() {
        let lat_exp = 51.484f64.to_radians();
        let lon_exp = -35.516f64.to_radians();
        let alt_exp = 237.434; // km

        let position = DVec3::new(3359927.0, -2398072.0, 5153000.0);
        let velocity = DVec3::new(5065.7, 5485.0, -744.0);
        let time = time!(Tdb, 2012, 7, 1).unwrap();
        let state = CartesianOrbit::new(
            Cartesian::from_vecs(position, velocity),
            time,
            Earth,
            Iau::new(Earth),
        );
        let ground = state.to_ground_location().unwrap();
        assert_approx_eq!(ground.latitude(), lat_exp, rtol <= 1e-4);
        assert_approx_eq!(ground.longitude(), lon_exp, rtol <= 1e-4);
        assert_approx_eq!(ground.altitude(), alt_exp, rtol <= 1e-4);
    }

    #[test]
    fn test_to_origin() {
        let r = DVec3::new(6068279.27, -1692843.94, -2516619.18);
        let v = DVec3::new(-660.415582, 5495.938726, -5303.093233);

        let utc = Utc::from_iso("2016-05-30T12:00:00.000").unwrap();
        let tai = utc.to_time();
        let tdb = tai.to_scale(Tdb);

        // Compute the expected ephemeris delta using the new trait directly
        let delta = ephemeris().state(tdb, Earth, Venus).unwrap();
        let r_exp = r - delta.position();
        let v_exp = v - delta.velocity();

        let s_earth = CartesianOrbit::new(Cartesian::from_vecs(r, v), tai, Earth, Icrf);
        let s_venus = s_earth.to_origin(Venus, ephemeris()).unwrap();

        assert_approx_eq!(s_venus.position(), r_exp);
        assert_approx_eq!(s_venus.velocity(), v_exp);
    }

    #[test]
    fn test_rotation_lvlh() {
        let time = time!(Tdb, 2023, 3, 25, 21, 8, 0.0).unwrap();
        let pos = DVec3::new(6678.0, 0.0, 0.0);
        let vel = DVec3::new(0.0, 7.73, 0.0);

        let state = CartesianOrbit::new(Cartesian::from_vecs(pos, vel), time, Earth, Icrf);
        let rot = state.rotation_lvlh();

        // For a prograde equatorial orbit at x-axis, LVLH z should point to -x (nadir),
        // y should point to -z (cross-track), x should point to +y (velocity direction)
        let z = rot.col(2);
        let x = rot.col(0);
        assert_approx_eq!(z, -DVec3::X, atol <= 1e-10);
        assert_approx_eq!(x, DVec3::Y, atol <= 1e-10);
    }

    #[test]
    fn test_sub_operator() {
        let time = time!(Tdb, 2023, 3, 25, 21, 8, 0.0).unwrap();
        let s1 = CartesianOrbit::new(
            Cartesian::from_vecs(DVec3::new(10.0, 20.0, 30.0), DVec3::new(1.0, 2.0, 3.0)),
            time,
            Earth,
            Icrf,
        );
        let s2 = CartesianOrbit::new(
            Cartesian::from_vecs(DVec3::new(3.0, 5.0, 7.0), DVec3::new(0.5, 1.0, 1.5)),
            time,
            Earth,
            Icrf,
        );
        let diff = s1 - s2;
        assert_approx_eq!(diff.position(), DVec3::new(7.0, 15.0, 23.0));
        assert_approx_eq!(diff.velocity(), DVec3::new(0.5, 1.0, 1.5));
    }

    #[test]
    fn test_into_dyn() {
        use lox_bodies::DynOrigin;
        use lox_frames::DynFrame;
        use lox_time::time_scales::DynTimeScale;

        let time = time!(Tdb, 2023, 3, 25, 21, 8, 0.0).unwrap();
        let pos = DVec3::new(1000.0, 2000.0, 3000.0);
        let vel = DVec3::new(1.0, 2.0, 3.0);
        let state = CartesianOrbit::new(Cartesian::from_vecs(pos, vel), time, Earth, Icrf);
        let dyn_state = state.into_dyn();

        assert_eq!(dyn_state.origin(), DynOrigin::Earth);
        assert_eq!(dyn_state.reference_frame(), DynFrame::Icrf);
        assert_eq!(dyn_state.time().scale(), DynTimeScale::Tdb);
        assert_approx_eq!(dyn_state.position(), pos);
        assert_approx_eq!(dyn_state.velocity(), vel);
    }

    #[test]
    fn test_try_to_frame_identity() {
        use lox_bodies::DynOrigin;
        use lox_frames::DynFrame;

        let time = Utc::from_iso("2024-07-05T09:09:18.173")
            .unwrap()
            .to_dyn_time();
        let pos = DVec3::new(6068.27927, -1692.84394, -2516.61918);
        let vel = DVec3::new(-0.660415582, 5.495938726, -5.303093233);
        let state = CartesianOrbit::new(
            Cartesian::from_vecs(pos, vel),
            time,
            DynOrigin::Earth,
            DynFrame::Icrf,
        );

        // Converting ICRF→ICRF should be a no-op that preserves state exactly
        let same = state
            .try_to_frame(DynFrame::Icrf, &DefaultRotationProvider)
            .unwrap();
        assert_eq!(same.position(), pos);
        assert_eq!(same.velocity(), vel);
    }

    #[test]
    fn test_to_origin_same_origin() {
        let r = DVec3::new(6068279.27, -1692843.94, -2516619.18);
        let v = DVec3::new(-660.415582, 5495.938726, -5303.093233);
        let utc = Utc::from_iso("2016-05-30T12:00:00.000").unwrap();
        let tai = utc.to_time();

        let state = CartesianOrbit::new(Cartesian::from_vecs(r, v), tai, Earth, Icrf);
        // Same origin should return identical state without needing ephemeris
        let same = state.to_origin(Earth, ephemeris()).unwrap();
        assert_eq!(same.position(), r);
        assert_eq!(same.velocity(), v);
    }

    #[test]
    fn test_try_to_origin_same_origin() {
        use lox_bodies::DynOrigin;
        use lox_frames::DynFrame;

        let r = DVec3::new(6068279.27, -1692843.94, -2516619.18);
        let v = DVec3::new(-660.415582, 5495.938726, -5303.093233);
        let utc = Utc::from_iso("2016-05-30T12:00:00.000").unwrap();
        let time = utc.to_dyn_time();

        let state = CartesianOrbit::new(
            Cartesian::from_vecs(r, v),
            time,
            DynOrigin::Earth,
            DynFrame::Icrf,
        );
        let same = state.try_to_origin(DynOrigin::Earth, ephemeris()).unwrap();
        assert_eq!(same.position(), r);
        assert_eq!(same.velocity(), v);
    }

    fn ephemeris() -> &'static Spk {
        static EPHEMERIS: OnceLock<Spk> = OnceLock::new();
        EPHEMERIS.get_or_init(|| Spk::from_file(data_file("spice/de440s.bsp")).unwrap())
    }
}
