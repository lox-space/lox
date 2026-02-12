// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{fmt::Display, ops::Mul};

use glam::{DMat3, DVec3};
use lox_bodies::{TryRotationalElements, UndefinedOriginPropertyError};
use lox_core::{coords::Cartesian, f64::consts::ROTATION_RATE_EARTH};
use lox_test_utils::ApproxEq;
use lox_time::{
    Time,
    julian_dates::JulianDate,
    offsets::{OffsetProvider, TryOffset},
    time_scales::{Tdb, TimeScale, Tt, Ut1},
};
use thiserror::Error;

use crate::{
    Iau, ReferenceFrame,
    iau::icrf_to_iau,
    iers::{
        Corrections, ReferenceSystem,
        cio::CioLocator,
        cip::CipCoords,
        earth_rotation::{EarthRotationAngle, EquationOfTheEquinoxes},
        polar_motion::PoleCoords,
    },
};

mod impls;

pub trait TryRotation<Origin, Target, T>
where
    Origin: ReferenceFrame,
    Target: ReferenceFrame,
    T: TimeScale,
{
    type Error: std::error::Error + Send + Sync + 'static;

    fn try_rotation(
        &self,
        origin: Origin,
        target: Target,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error>;
}

pub trait TryComposedRotation<T, P>
where
    T: TimeScale + Copy,
{
    fn try_composed_rotation(&self, provider: &P, time: Time<T>)
    -> Result<Rotation, RotationError>;
}

impl<T, P, R1, R2, R3> TryComposedRotation<T, P> for (R1, R2, R3)
where
    T: TimeScale + Copy,
    R1: ReferenceFrame + Copy,
    R2: ReferenceFrame + Copy,
    R3: ReferenceFrame + Copy,
    P: RotationProvider<T>
        + TryRotation<R1, R2, T, Error = RotationError>
        + TryRotation<R2, R3, T, Error = RotationError>,
{
    fn try_composed_rotation(
        &self,
        provider: &P,
        time: Time<T>,
    ) -> Result<Rotation, RotationError> {
        Ok(provider
            .try_rotation(self.0, self.1, time)?
            .compose(provider.try_rotation(self.1, self.2, time)?))
    }
}

impl<T, P, R1, R2, R3, R4> TryComposedRotation<T, P> for (R1, R2, R3, R4)
where
    T: TimeScale + Copy,
    R1: ReferenceFrame + Copy,
    R2: ReferenceFrame + Copy,
    R3: ReferenceFrame + Copy,
    R4: ReferenceFrame + Copy,
    P: RotationProvider<T>
        + TryRotation<R1, R2, T, Error = RotationError>
        + TryRotation<R2, R3, T, Error = RotationError>
        + TryRotation<R3, R4, T, Error = RotationError>,
{
    fn try_composed_rotation(
        &self,
        provider: &P,
        time: Time<T>,
    ) -> Result<Rotation, RotationError> {
        Ok(provider
            .try_rotation(self.0, self.1, time)?
            .compose(provider.try_rotation(self.1, self.2, time)?)
            .compose(provider.try_rotation(self.2, self.3, time)?))
    }
}

impl<T, P, R1, R2, R3, R4, R5> TryComposedRotation<T, P> for (R1, R2, R3, R4, R5)
where
    T: TimeScale + Copy,
    R1: ReferenceFrame + Copy,
    R2: ReferenceFrame + Copy,
    R3: ReferenceFrame + Copy,
    R4: ReferenceFrame + Copy,
    R5: ReferenceFrame + Copy,
    P: RotationProvider<T>
        + TryRotation<R1, R2, T, Error = RotationError>
        + TryRotation<R2, R3, T, Error = RotationError>
        + TryRotation<R3, R4, T, Error = RotationError>
        + TryRotation<R4, R5, T, Error = RotationError>,
{
    fn try_composed_rotation(
        &self,
        provider: &P,
        time: Time<T>,
    ) -> Result<Rotation, RotationError> {
        Ok(provider
            .try_rotation(self.0, self.1, time)?
            .compose(provider.try_rotation(self.1, self.2, time)?)
            .compose(provider.try_rotation(self.2, self.3, time)?)
            .compose(provider.try_rotation(self.3, self.4, time)?))
    }
}

impl<T, P, R1, R2, R3, R4, R5, R6> TryComposedRotation<T, P> for (R1, R2, R3, R4, R5, R6)
where
    T: TimeScale + Copy,
    R1: ReferenceFrame + Copy,
    R2: ReferenceFrame + Copy,
    R3: ReferenceFrame + Copy,
    R4: ReferenceFrame + Copy,
    R5: ReferenceFrame + Copy,
    R6: ReferenceFrame + Copy,
    P: RotationProvider<T>
        + TryRotation<R1, R2, T, Error = RotationError>
        + TryRotation<R2, R3, T, Error = RotationError>
        + TryRotation<R3, R4, T, Error = RotationError>
        + TryRotation<R4, R5, T, Error = RotationError>
        + TryRotation<R5, R6, T, Error = RotationError>,
{
    fn try_composed_rotation(
        &self,
        provider: &P,
        time: Time<T>,
    ) -> Result<Rotation, RotationError> {
        Ok(provider
            .try_rotation(self.0, self.1, time)?
            .compose(provider.try_rotation(self.1, self.2, time)?)
            .compose(provider.try_rotation(self.2, self.3, time)?)
            .compose(provider.try_rotation(self.3, self.4, time)?)
            .compose(provider.try_rotation(self.4, self.5, time)?))
    }
}

#[derive(Debug)]
pub enum RotationErrorKind {
    Offset,
    Eop,
}

impl Display for RotationErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RotationErrorKind::Offset => "offset error".fmt(f),
            RotationErrorKind::Eop => "EOP error".fmt(f),
        }
    }
}

#[derive(Debug, Error)]
#[error("{kind}: {error}")]
pub struct RotationError {
    kind: RotationErrorKind,
    error: Box<dyn std::error::Error + Send + Sync + 'static>,
}

impl RotationError {
    pub fn offset(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        RotationError {
            kind: RotationErrorKind::Offset,
            error: Box::new(err),
        }
    }

    pub fn eop(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        RotationError {
            kind: RotationErrorKind::Eop,
            error: Box::new(err),
        }
    }
}

#[derive(Debug, Error)]
pub enum DynRotationError {
    #[error(transparent)]
    Offset(#[from] RotationError),
    #[error("incompatible reference systems")]
    IncompatibleReferenceSystems,
    #[error(transparent)]
    UndefinedProperty(#[from] UndefinedOriginPropertyError),
}

pub trait RotationProvider<T: TimeScale>: OffsetProvider {
    type EopError: std::error::Error + Send + Sync + 'static;

    fn corrections(
        &self,
        time: Time<T>,
        sys: ReferenceSystem,
    ) -> Result<Corrections, Self::EopError>;
    fn pole_coords(&self, time: Time<T>) -> Result<PoleCoords, Self::EopError>;

    fn icrf_to_iau<R>(&self, time: Time<T>, frame: Iau<R>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        R: TryRotationalElements,
        Self: TryOffset<T, Tdb>,
    {
        let seconds = time
            .try_to_scale(Tdb, self)
            .map_err(RotationError::offset)?
            .seconds_since_j2000();
        let angles = frame.rotational_elements(seconds);
        let rates = frame.rotational_element_rates(seconds);

        Ok(icrf_to_iau(angles, rates))
    }
    fn iau_to_icrf<R>(&self, time: Time<T>, frame: Iau<R>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        R: TryRotationalElements,
        Self: TryOffset<T, Tdb>,
    {
        Ok(self.icrf_to_iau(time, frame)?.transpose())
    }

    // TODO: Support other IERS conventions
    fn icrf_to_itrf(&self, time: Time<T>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tdb> + TryOffset<T, Tt> + TryOffset<T, Ut1>,
    {
        Ok(self
            .icrf_to_cirf(time)?
            .compose(self.cirf_to_tirf(time)?)
            .compose(self.tirf_to_itrf(time)?))
    }
    fn itrf_to_icrf(&self, time: Time<T>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tdb> + TryOffset<T, Tt> + TryOffset<T, Ut1>,
    {
        Ok(self.icrf_to_itrf(time)?.transpose())
    }

    fn icrf_to_mod(&self, time: Time<T>, sys: ReferenceSystem) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tt>,
    {
        let time = time.try_to_scale(Tt, self).map_err(RotationError::offset)?;
        Ok(sys.bias_precession_matrix(time).into())
    }
    fn mod_to_icrf(&self, time: Time<T>, sys: ReferenceSystem) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tt>,
    {
        Ok(self.icrf_to_mod(time, sys)?.transpose())
    }

    fn mod_to_tod(&self, time: Time<T>, sys: ReferenceSystem) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tdb>,
    {
        let tdb = time
            .try_to_scale(Tdb, self)
            .map_err(RotationError::offset)?;
        let corr = self.corrections(time, sys).map_err(RotationError::eop)?;
        Ok(sys.nutation_matrix(tdb, corr).into())
    }
    fn tod_to_mod(&self, time: Time<T>, sys: ReferenceSystem) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tdb>,
    {
        Ok(self.mod_to_tod(time, sys)?.transpose())
    }

    fn tod_to_pef(&self, time: Time<T>, sys: ReferenceSystem) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tt> + TryOffset<T, Ut1>,
    {
        let tt = time.try_to_scale(Tt, self).map_err(RotationError::offset)?;
        let ut1 = time
            .try_to_scale(Ut1, self)
            .map_err(RotationError::offset)?;
        let corr = self.corrections(time, sys).map_err(RotationError::eop)?;
        Ok(
            Rotation::new(sys.earth_rotation(tt, ut1, corr)).with_angular_velocity(DVec3::new(
                0.0,
                0.0,
                ROTATION_RATE_EARTH,
            )),
        )
    }
    fn pef_to_tod(&self, time: Time<T>, sys: ReferenceSystem) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tt> + TryOffset<T, Ut1>,
    {
        Ok(self.tod_to_pef(time, sys)?.transpose())
    }

    fn pef_to_itrf(&self, time: Time<T>, sys: ReferenceSystem) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tt>,
    {
        let tt = time.try_to_scale(Tt, self).map_err(RotationError::offset)?;
        let pole_coords = self.pole_coords(time).map_err(RotationError::eop)?;
        Ok(sys.polar_motion_matrix(tt, pole_coords).into())
    }

    fn itrf_to_pef(&self, time: Time<T>, sys: ReferenceSystem) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tt>,
    {
        Ok(self.pef_to_itrf(time, sys)?.transpose())
    }

    fn pef_to_teme(&self, time: Time<T>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tdb>,
    {
        let tdb = time
            .try_to_scale(Tdb, self)
            .map_err(RotationError::offset)?;

        // TEME uses IERS 1996 conventions (IAU 1994 EoE) regardless of PEF variant
        let eoe = EquationOfTheEquinoxes::iau1994(tdb);

        // PEF to TEME rotates by negative EoE (removing the nutation effect)
        Ok(Rotation::new((-eoe.0).rotation_z()))
    }

    fn teme_to_pef(&self, time: Time<T>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tdb>,
    {
        Ok(self.pef_to_teme(time)?.transpose())
    }

    fn icrf_to_cirf(&self, time: Time<T>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tdb>,
    {
        let tdb = time
            .try_to_scale(Tdb, self)
            .map_err(RotationError::offset)?;
        let mut xy = CipCoords::iau2006(tdb);
        let s = CioLocator::iau2006(tdb, xy);

        xy += self
            .corrections(time, ReferenceSystem::Iers2010)
            .unwrap_or_default();

        Ok(Rotation::new(xy.celestial_to_intermediate_matrix(s)))
    }
    fn cirf_to_icrf(&self, time: Time<T>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tdb>,
    {
        Ok(self.icrf_to_cirf(time)?.transpose())
    }

    fn cirf_to_tirf(&self, time: Time<T>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Ut1>,
    {
        let time = time
            .try_to_scale(Ut1, self)
            .map_err(RotationError::offset)?;
        let era = EarthRotationAngle::iau2000(time);
        Ok(
            Rotation::new(era.0.rotation_z()).with_angular_velocity(DVec3::new(
                0.0,
                0.0,
                ROTATION_RATE_EARTH,
            )),
        )
    }
    fn tirf_to_cirf(&self, time: Time<T>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Ut1>,
    {
        Ok(self.cirf_to_tirf(time)?.transpose())
    }

    fn tirf_to_itrf(&self, time: Time<T>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tt>,
    {
        let tt = time.try_to_scale(Tt, self).map_err(RotationError::offset)?;
        let pole_coords = self.pole_coords(time).map_err(RotationError::eop)?;
        Ok(ReferenceSystem::Iers2010
            .polar_motion_matrix(tt, pole_coords)
            .into())
    }

    fn itrf_to_tirf(&self, time: Time<T>) -> Result<Rotation, RotationError>
    where
        T: TimeScale + Copy,
        Self: TryOffset<T, Tt>,
    {
        Ok(self.tirf_to_itrf(time)?.transpose())
    }
}

fn rotation_matrix_derivative(m: DMat3, v: DVec3) -> DMat3 {
    let sx = DVec3::new(0.0, v.z, v.y);
    let sy = DVec3::new(-v.z, 0.0, v.x);
    let sz = DVec3::new(v.y, -v.x, 0.0);
    let s = DMat3::from_cols(sx, sy, sz);
    -s * m
}

#[derive(Debug, Clone, Copy, PartialEq, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rotation {
    /// Rotation matrix
    pub m: DMat3,
    /// Time derivative of the rotation matrix
    pub dm: DMat3,
}

impl Rotation {
    pub const IDENTITY: Self = Self {
        m: DMat3::IDENTITY,
        dm: DMat3::ZERO,
    };

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

    pub fn position_matrix(&self) -> DMat3 {
        self.m
    }

    pub fn velocity_matrix(&self) -> DMat3 {
        self.dm
    }

    pub fn compose(self, other: Self) -> Self {
        Self {
            m: other.m * self.m,
            dm: other.dm * self.m + other.m * self.dm,
        }
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

impl Default for Rotation {
    fn default() -> Self {
        Self {
            m: DMat3::IDENTITY,
            dm: DMat3::ZERO,
        }
    }
}

impl Mul<DVec3> for Rotation {
    type Output = DVec3;

    fn mul(self, rhs: DVec3) -> Self::Output {
        self.m * rhs
    }
}

impl Mul<Cartesian> for Rotation {
    type Output = Cartesian;

    fn mul(self, rhs: Cartesian) -> Self::Output {
        let pos = self.m * rhs.position();
        let vel = self.dm * rhs.position() + self.m * rhs.velocity();
        Cartesian::from_vecs(pos, vel)
    }
}

impl From<DMat3> for Rotation {
    fn from(matrix: DMat3) -> Self {
        Rotation::new(matrix)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use lox_test_utils::assert_approx_eq;
    use lox_time::{deltas::TimeDelta, offsets::OffsetProvider};
    use lox_units::AngleUnits;

    use crate::iers::Iau2000Model;

    use super::*;

    #[derive(Debug)]
    struct TestRotationProvider;

    impl OffsetProvider for TestRotationProvider {
        type Error = Infallible;

        fn tai_to_ut1(&self, _delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
            Ok(TimeDelta::from_seconds_f64(-33.072073684954375))
        }

        fn ut1_to_tai(&self, _delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
            unreachable!()
        }
    }

    impl<T> RotationProvider<T> for TestRotationProvider
    where
        T: TimeScale,
    {
        type EopError = Infallible;

        fn corrections(
            &self,
            _time: Time<T>,
            sys: ReferenceSystem,
        ) -> Result<Corrections, Infallible> {
            match sys {
                ReferenceSystem::Iers1996 => {
                    Ok(Corrections(-55.0655e-3.arcsec(), -6.3580e-3.arcsec()))
                }
                ReferenceSystem::Iers2003(_) => {
                    Ok(Corrections(0.1725e-3.arcsec(), -0.2650e-3.arcsec()))
                }
                ReferenceSystem::Iers2010 => {
                    Ok(Corrections(0.1750e-3.arcsec(), -0.2259e-3.arcsec()))
                }
            }
        }

        fn pole_coords(&self, _time: Time<T>) -> Result<PoleCoords, Infallible> {
            Ok(PoleCoords {
                xp: 0.0349282.arcsec(),
                yp: 0.4833163.arcsec(),
            })
        }
    }

    #[test]
    fn test_celestial_to_terrestrial_iers1996() {
        let tt = Time::from_two_part_julian_date(Tt, 2454195.5, 0.500754444444444);
        let sys = ReferenceSystem::Iers1996;

        // let npb_exp = DMat3::from_cols_array(&[
        //     0.999998403176203,
        //     -0.001639032970562,
        //     -0.000712190961847,
        //     0.001639000942243,
        //     0.999998655799521,
        //     -0.000045552846624,
        //     0.000712264667137,
        //     0.000044385492226,
        //     0.999999745354454,
        // ])
        // .transpose();
        let npb_exp = DMat3::from_cols_array(&[
            9.999_984_026_404_259e-1,
            -1.639_348_666_725_915e-3,
            -7.122_166_424_041_306e-4,
            1.639_316_638_909_414_8e-3,
            9.999_986_552_821_435e-1,
            -4.555_065_309_035_662_5e-5,
            7.122_903_580_761_061e-4,
            4.438_303_173_715_299e-5,
            9.999_997_453_362_638e-1,
        ])
        .transpose();
        let c2t_exp = DMat3::from_cols_array(&[
            0.973104317592265,
            0.230363826166883,
            -0.000703332813776,
            -0.230363798723533,
            0.973104570754697,
            0.000120888299841,
            0.000712264667137,
            0.000044385492226,
            0.999999745354454,
        ])
        .transpose();
        let c2t_pm_exp = DMat3::from_cols_array(&[
            0.973104317712772,
            0.230363826174782,
            -0.000703163477127,
            -0.230363800391868,
            0.973104570648022,
            0.000118545116892,
            0.000711560100206,
            0.000046626645796,
            0.999999745754058,
        ])
        .transpose();

        let npb_act = TestRotationProvider
            .icrf_to_mod(tt.with_scale(Tdb), sys)
            .unwrap()
            .compose(
                TestRotationProvider
                    .mod_to_tod(tt.with_scale(Tdb), sys)
                    .unwrap(),
            );
        assert_approx_eq!(npb_act.m, npb_exp, atol <= 1e-12);

        // TODO: Generate reference data including frame bias
        let c2t_act = npb_act.compose(TestRotationProvider.tod_to_pef(tt, sys).unwrap());
        assert_approx_eq!(c2t_act.m, c2t_exp, atol <= 1e-4);

        // TODO: Generate reference data including frame bias
        let c2t_pm_act = c2t_act.compose(TestRotationProvider.pef_to_itrf(tt, sys).unwrap());
        assert_approx_eq!(c2t_pm_act.m, c2t_pm_exp, atol <= 1e-4);
    }

    #[test]
    fn test_celestial_to_terrestrial_iers2003() {
        let tt = Time::from_two_part_julian_date(Tt, 2454195.5, 0.500754444444444);
        let sys = ReferenceSystem::Iers2003(Iau2000Model::A);

        let npb_exp = DMat3::from_cols_array(&[
            0.999998402755640,
            -0.001639289519579,
            -0.000712191013215,
            0.001639257491365,
            0.999998655379006,
            -0.000045552787478,
            0.000712264729795,
            0.000044385250265,
            0.999999745354420,
        ])
        .transpose();
        let c2t_exp = DMat3::from_cols_array(&[
            0.973104317573209,
            0.230363826247361,
            -0.000703332818999,
            -0.230363798803834,
            0.973104570735656,
            0.000120888549787,
            0.000712264729795,
            0.000044385250265,
            0.999999745354420,
        ])
        .transpose();
        let c2t_pm_exp = DMat3::from_cols_array(&[
            0.973104317697618,
            0.230363826238780,
            -0.000703163482352,
            -0.230363800455689,
            0.973104570632883,
            0.000118545366826,
            0.000711560162864,
            0.000046626403835,
            0.999999745754024,
        ])
        .transpose();

        let npb_act = TestRotationProvider
            .icrf_to_mod(tt, sys)
            .unwrap()
            .compose(TestRotationProvider.mod_to_tod(tt, sys).unwrap());
        assert_approx_eq!(npb_act.m, npb_exp, atol <= 1e-12);

        let c2t_act = npb_act.compose(TestRotationProvider.tod_to_pef(tt, sys).unwrap());
        assert_approx_eq!(c2t_act.m, c2t_exp, atol <= 1e-12);

        let c2t_pm_act = c2t_act.compose(TestRotationProvider.pef_to_itrf(tt, sys).unwrap());
        assert_approx_eq!(c2t_pm_act.m, c2t_pm_exp, atol <= 1e-12);
    }

    #[test]
    fn test_celestial_to_terrestrial_iau2006() {
        let tt = Time::from_two_part_julian_date(Tt, 2454195.5, 0.500754444444444);

        let npb_exp = DMat3::from_cols_array(&[
            0.999999746339445,
            -0.000000005138822,
            -0.000712264730072,
            -0.000000026475227,
            0.999999999014975,
            -0.000044385242827,
            0.000712264729599,
            0.000044385250426,
            0.999999745354420,
        ])
        .transpose();
        let c2t_exp = DMat3::from_cols_array(&[
            0.973104317573127,
            0.230363826247709,
            -0.000703332818845,
            -0.230363798804182,
            0.973104570735574,
            0.000120888549586,
            0.000712264729599,
            0.000044385250426,
            0.999999745354420,
        ])
        .transpose();
        let c2t_pm_exp = DMat3::from_cols_array(&[
            0.973104317697535,
            0.230363826239128,
            -0.000703163482198,
            -0.230363800456037,
            0.973104570632801,
            0.000118545366625,
            0.000711560162668,
            0.000046626403995,
            0.999999745754024,
        ])
        .transpose();

        let npb_act = TestRotationProvider.icrf_to_cirf(tt).unwrap();
        assert_approx_eq!(npb_act.m, npb_exp, atol <= 1e-11);

        let c2t_act = npb_act.compose(TestRotationProvider.cirf_to_tirf(tt).unwrap());
        assert_approx_eq!(c2t_act.m, c2t_exp, atol <= 1e-11);

        let c2t_pm_act = c2t_act.compose(TestRotationProvider.tirf_to_itrf(tt).unwrap());
        assert_approx_eq!(c2t_pm_act.m, c2t_pm_exp, atol <= 1e-11);
    }

    #[test]
    fn test_pef_to_teme() {
        // Use the same time as the EoE test to verify against known reference value
        // EoE at this time = 5.357_758_254_609_257e-5 radians (from test_equation_of_the_equinoxes_iau1994)
        let tdb = Time::from_two_part_julian_date(Tdb, 2400000.5, 41234.0);
        let eoe: f64 = 5.357_758_254_609_257e-5; // radians

        let rotation = TestRotationProvider.pef_to_teme(tdb).unwrap();

        // PEF to TEME is R_z(-EoE), so the rotation matrix should be:
        // [cos(EoE)   sin(EoE)  0]
        // [-sin(EoE)  cos(EoE)  0]
        // [0          0         1]
        let (sin_eoe, cos_eoe) = eoe.sin_cos();
        let expected =
            DMat3::from_cols_array(&[cos_eoe, -sin_eoe, 0.0, sin_eoe, cos_eoe, 0.0, 0.0, 0.0, 1.0])
                .transpose();

        assert_approx_eq!(rotation.m, expected, atol <= 1e-15);

        // Verify round-trip
        let roundtrip = rotation.compose(TestRotationProvider.teme_to_pef(tdb).unwrap());
        assert_approx_eq!(roundtrip.m, DMat3::IDENTITY, atol <= 1e-15);
    }

    #[test]
    fn test_teme_icrf_roundtrip() {
        // Test the full TEME <-> ICRF transformation chain
        // Path: ICRF -> MOD -> TOD -> PEF -> TEME and back
        let tt = Time::from_two_part_julian_date(Tt, 2454195.5, 0.500754444444444);
        let sys = ReferenceSystem::Iers1996;

        // Build the full ICRF to TEME transformation
        let icrf_to_mod = TestRotationProvider.icrf_to_mod(tt, sys).unwrap();
        let mod_to_tod = TestRotationProvider.mod_to_tod(tt, sys).unwrap();
        let tod_to_pef = TestRotationProvider.tod_to_pef(tt, sys).unwrap();
        let pef_to_teme = TestRotationProvider.pef_to_teme(tt).unwrap();

        let icrf_to_teme = icrf_to_mod
            .compose(mod_to_tod)
            .compose(tod_to_pef)
            .compose(pef_to_teme);

        // Build the reverse transformation
        let teme_to_pef = TestRotationProvider.teme_to_pef(tt).unwrap();
        let pef_to_tod = TestRotationProvider.pef_to_tod(tt, sys).unwrap();
        let tod_to_mod = TestRotationProvider.tod_to_mod(tt, sys).unwrap();
        let mod_to_icrf = TestRotationProvider.mod_to_icrf(tt, sys).unwrap();

        let teme_to_icrf = teme_to_pef
            .compose(pef_to_tod)
            .compose(tod_to_mod)
            .compose(mod_to_icrf);

        // Round-trip should give identity
        let roundtrip = icrf_to_teme.compose(teme_to_icrf);
        assert_approx_eq!(roundtrip.m, DMat3::IDENTITY, atol <= 1e-14);
    }
}
