// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::{DynOrigin, TryRotationalElements};
use lox_time::{
    Time,
    offsets::TryOffset,
    time_scales::{Tdb, TimeScale, Tt, Ut1},
};

use crate::{
    Cirf, DynFrame, Iau, Icrf, Itrf, Tirf,
    frames::{Mod, Pef, Teme, Tod},
    iers::{Iers1996, Iers2003, Iers2010, ReferenceSystem},
    rotations::{
        DynRotationError, Rotation, RotationError, RotationProvider, TryComposedRotation,
        TryRotation,
    },
};

// ICRF <-> IAU

impl<T, R, U> TryRotation<Icrf, Iau<R>, T> for U
where
    T: TimeScale + Copy,
    R: TryRotationalElements + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Icrf,
        target: Iau<R>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.icrf_to_iau(time, target)
    }
}

impl<T, R, U> TryRotation<Iau<R>, Icrf, T> for U
where
    T: TimeScale + Copy,
    R: TryRotationalElements + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        origin: Iau<R>,
        _target: Icrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.iau_to_icrf(time, origin)
    }
}

// ICRF <-> ITRF

impl<T, U> TryRotation<Icrf, Itrf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb> + TryOffset<T, Tt> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Icrf,
        _target: Itrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.icrf_to_itrf(time)
    }
}

impl<T, U> TryRotation<Itrf, Icrf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb> + TryOffset<T, Tt> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Itrf,
        _target: Icrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.itrf_to_icrf(time)
    }
}

// ICRF <-> CIRF

impl<T, U> TryRotation<Icrf, Cirf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Icrf,
        _target: Cirf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.icrf_to_cirf(time)
    }
}

impl<T, U> TryRotation<Cirf, Icrf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Cirf,
        _target: Icrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.cirf_to_icrf(time)
    }
}

// CIRF <-> TIRF

impl<T, U> TryRotation<Cirf, Tirf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Cirf,
        _target: Tirf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.cirf_to_tirf(time)
    }
}

impl<T, U> TryRotation<Tirf, Cirf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Tirf,
        _target: Cirf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.tirf_to_cirf(time)
    }
}

// TIRF <-> ITRF

impl<T, U> TryRotation<Tirf, Itrf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Tirf,
        _target: Itrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.tirf_to_itrf(time)
    }
}

impl<T, U> TryRotation<Itrf, Tirf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Itrf,
        _target: Tirf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.itrf_to_tirf(time)
    }
}

// ICRF <-> MOD

impl<T, U> TryRotation<Icrf, Mod<Iers1996>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Icrf,
        _target: Mod<Iers1996>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.icrf_to_mod(time, ReferenceSystem::Iers1996)
    }
}

impl<T, U> TryRotation<Mod<Iers1996>, Icrf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Mod<Iers1996>,
        _target: Icrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.mod_to_icrf(time, ReferenceSystem::Iers1996)
    }
}

impl<T, U> TryRotation<Icrf, Mod<Iers2003>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Icrf,
        target: Mod<Iers2003>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.icrf_to_mod(time, ReferenceSystem::Iers2003(target.0.0))
    }
}

impl<T, U> TryRotation<Mod<Iers2003>, Icrf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        origin: Mod<Iers2003>,
        _target: Icrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.mod_to_icrf(time, ReferenceSystem::Iers2003(origin.0.0))
    }
}

impl<T, U> TryRotation<Icrf, Mod<Iers2010>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Icrf,
        _target: Mod<Iers2010>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.icrf_to_mod(time, ReferenceSystem::Iers2010)
    }
}

impl<T, U> TryRotation<Mod<Iers2010>, Icrf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Mod<Iers2010>,
        _target: Icrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.mod_to_icrf(time, ReferenceSystem::Iers2010)
    }
}

// MOD <-> TOD

impl<T, U> TryRotation<Mod<Iers1996>, Tod<Iers1996>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Mod<Iers1996>,
        _target: Tod<Iers1996>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.mod_to_tod(time, ReferenceSystem::Iers1996)
    }
}

impl<T, U> TryRotation<Tod<Iers1996>, Mod<Iers1996>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Tod<Iers1996>,
        _target: Mod<Iers1996>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.tod_to_mod(time, ReferenceSystem::Iers1996)
    }
}

impl<T, U> TryRotation<Mod<Iers2003>, Tod<Iers2003>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        origin: Mod<Iers2003>,
        _target: Tod<Iers2003>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.mod_to_tod(time, ReferenceSystem::Iers2003(origin.0.0))
    }
}

impl<T, U> TryRotation<Tod<Iers2003>, Mod<Iers2003>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        origin: Tod<Iers2003>,
        _target: Mod<Iers2003>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.tod_to_mod(time, ReferenceSystem::Iers2003(origin.0.0))
    }
}

impl<T, U> TryRotation<Mod<Iers2010>, Tod<Iers2010>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Mod<Iers2010>,
        _target: Tod<Iers2010>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.mod_to_tod(time, ReferenceSystem::Iers2010)
    }
}

impl<T, U> TryRotation<Tod<Iers2010>, Mod<Iers2010>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Tod<Iers2010>,
        _target: Mod<Iers2010>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.tod_to_mod(time, ReferenceSystem::Iers2010)
    }
}

// TOD <-> PEF

impl<T, U> TryRotation<Tod<Iers1996>, Pef<Iers1996>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Tod<Iers1996>,
        _target: Pef<Iers1996>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.tod_to_pef(time, ReferenceSystem::Iers1996)
    }
}

impl<T, U> TryRotation<Pef<Iers1996>, Tod<Iers1996>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Pef<Iers1996>,
        _target: Tod<Iers1996>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.pef_to_tod(time, ReferenceSystem::Iers1996)
    }
}

impl<T, U> TryRotation<Tod<Iers2003>, Pef<Iers2003>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        origin: Tod<Iers2003>,
        _target: Pef<Iers2003>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.tod_to_pef(time, ReferenceSystem::Iers2003(origin.0.0))
    }
}

impl<T, U> TryRotation<Pef<Iers2003>, Tod<Iers2003>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        origin: Pef<Iers2003>,
        _target: Tod<Iers2003>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.pef_to_tod(time, ReferenceSystem::Iers2003(origin.0.0))
    }
}

impl<T, U> TryRotation<Tod<Iers2010>, Pef<Iers2010>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Tod<Iers2010>,
        _target: Pef<Iers2010>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.tod_to_pef(time, ReferenceSystem::Iers2010)
    }
}

impl<T, U> TryRotation<Pef<Iers2010>, Tod<Iers2010>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Pef<Iers2010>,
        _target: Tod<Iers2010>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.pef_to_tod(time, ReferenceSystem::Iers2010)
    }
}

// PEF <-> ITRF

impl<T, U> TryRotation<Pef<Iers1996>, Itrf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Pef<Iers1996>,
        _target: Itrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.pef_to_itrf(time, ReferenceSystem::Iers1996)
    }
}

impl<T, U> TryRotation<Itrf, Pef<Iers1996>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Itrf,
        _target: Pef<Iers1996>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.itrf_to_pef(time, ReferenceSystem::Iers1996)
    }
}

impl<T, U> TryRotation<Pef<Iers2003>, Itrf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        origin: Pef<Iers2003>,
        _target: Itrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.pef_to_itrf(time, ReferenceSystem::Iers2003(origin.0.0))
    }
}

impl<T, U> TryRotation<Itrf, Pef<Iers2003>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Itrf,
        target: Pef<Iers2003>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.itrf_to_pef(time, ReferenceSystem::Iers2003(target.0.0))
    }
}

impl<T, U> TryRotation<Pef<Iers2010>, Itrf, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Pef<Iers2010>,
        _target: Itrf,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.pef_to_itrf(time, ReferenceSystem::Iers2010)
    }
}

impl<T, U> TryRotation<Itrf, Pef<Iers2010>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Itrf,
        _target: Pef<Iers2010>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.itrf_to_pef(time, ReferenceSystem::Iers2010)
    }
}

// PEF <-> TEME

impl<T, U> TryRotation<Pef<Iers1996>, Teme, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Pef<Iers1996>,
        _target: Teme,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.pef_to_teme(time)
    }
}

impl<T, U> TryRotation<Teme, Pef<Iers1996>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Teme,
        _target: Pef<Iers1996>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.teme_to_pef(time)
    }
}

impl<T, U> TryRotation<Pef<Iers2003>, Teme, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Pef<Iers2003>,
        _target: Teme,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.pef_to_teme(time)
    }
}

impl<T, U> TryRotation<Teme, Pef<Iers2003>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Teme,
        _target: Pef<Iers2003>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.teme_to_pef(time)
    }
}

impl<T, U> TryRotation<Pef<Iers2010>, Teme, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Pef<Iers2010>,
        _target: Teme,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.pef_to_teme(time)
    }
}

impl<T, U> TryRotation<Teme, Pef<Iers2010>, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn try_rotation(
        &self,
        _origin: Teme,
        _target: Pef<Iers2010>,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        self.teme_to_pef(time)
    }
}

// Composed Rotations

macro_rules! impl_composed {
    ($($origin:ty => $target:ty: [$($via:expr),+]),* $(,)?) => {
        $(
            impl<T, U> TryRotation<$origin, $target, T> for U
            where
                T: TimeScale + Copy,
                U: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Tdb> + TryOffset<T, Ut1>,
            {
                type Error = RotationError;

                fn try_rotation(
                    &self,
                    origin: $origin,
                    target: $target,
                    time: Time<T>,
                ) -> Result<Rotation, Self::Error> {
                    (origin, $($via),+, target).try_composed_rotation(self, time)
                }
            }
        )*
    };
}

impl_composed!(
    Cirf => Iau<DynOrigin>: [Icrf],
    Cirf => Itrf: [Tirf],
    Cirf => Mod<Iers1996>: [Icrf],
    Cirf => Mod<Iers2003>: [Icrf],
    Cirf => Mod<Iers2010>: [Icrf],
    Cirf => Pef<Iers1996>: [Icrf, Mod(Iers1996), Tod(Iers1996)],
    Cirf => Pef<Iers2003>: [Icrf, Mod(Iers2003::default()), Tod(Iers2003::default())],
    Cirf => Pef<Iers2010>: [Icrf, Mod(Iers2010), Tod(Iers2010)],
    Cirf => Teme: [Icrf, Mod(Iers1996), Tod(Iers1996), Pef(Iers1996)],
    Cirf => Tod<Iers1996>: [Icrf, Mod(Iers1996)],
    Cirf => Tod<Iers2003>: [Icrf, Mod(Iers2003::default())],
    Cirf => Tod<Iers2010>: [Icrf, Mod(Iers2010)],
    Iau<DynOrigin> => Cirf: [Icrf],
    Iau<DynOrigin> => Itrf: [Icrf, Cirf, Tirf],
    Iau<DynOrigin> => Tirf: [Icrf, Cirf],
    Iau<DynOrigin> => Mod<Iers1996>: [Icrf],
    Iau<DynOrigin> => Mod<Iers2003>: [Icrf],
    Iau<DynOrigin> => Mod<Iers2010>: [Icrf],
    Iau<DynOrigin> => Tod<Iers1996>: [Icrf, Mod(Iers1996)],
    Iau<DynOrigin> => Tod<Iers2003>: [Icrf, Mod(Iers2003::default())],
    Iau<DynOrigin> => Tod<Iers2010>: [Icrf, Mod(Iers2010)],
    Iau<DynOrigin> => Pef<Iers1996>: [Icrf, Mod(Iers1996), Tod(Iers1996)],
    Iau<DynOrigin> => Pef<Iers2003>: [Icrf, Mod(Iers2003::default()), Tod(Iers2003::default())],
    Iau<DynOrigin> => Pef<Iers2010>: [Icrf, Mod(Iers2010), Tod(Iers2010)],
    Iau<DynOrigin> => Teme: [Icrf, Mod(Iers1996), Tod(Iers1996), Pef(Iers1996)],
    Icrf => Pef<Iers1996>: [Mod(Iers1996), Tod(Iers1996)],
    Icrf => Pef<Iers2003>: [Mod(Iers2003::default()), Tod(Iers2003::default())],
    Icrf => Pef<Iers2010>: [Mod(Iers2010), Tod(Iers2010)],
    Icrf => Teme: [Mod(Iers1996), Tod(Iers1996), Pef(Iers1996)],
    Icrf => Tirf: [Cirf],
    Icrf => Tod<Iers1996>: [Mod(Iers1996)],
    Icrf => Tod<Iers2003>: [Mod(Iers2003::default())],
    Icrf => Tod<Iers2010>: [Mod(Iers2010)],
    Itrf => Cirf: [Tirf],
    Itrf => Iau<DynOrigin>: [Tirf, Cirf, Icrf],
    Itrf => Mod<Iers1996>: [Pef(Iers1996), Tod(Iers1996)],
    Itrf => Mod<Iers2003>: [Pef(Iers2003::default()), Tod(Iers2003::default())],
    Itrf => Mod<Iers2010>: [Pef(Iers2010), Tod(Iers2010)],
    Itrf => Tod<Iers1996>: [Pef(Iers1996)],
    Itrf => Tod<Iers2003>: [Pef(Iers2003::default())],
    Itrf => Tod<Iers2010>: [Pef(Iers2010)],
    Itrf => Teme: [Pef(Iers1996)],
    Mod<Iers1996> => Cirf: [Icrf],
    Mod<Iers2003> => Cirf: [Icrf],
    Mod<Iers2010> => Cirf: [Icrf],
    Mod<Iers1996> => Tirf: [Icrf, Cirf],
    Mod<Iers2003> => Tirf: [Icrf, Cirf],
    Mod<Iers2010> => Tirf: [Icrf, Cirf],
    Mod<Iers1996> => Itrf: [Tod(Iers1996), Pef(Iers1996)],
    Mod<Iers2003> => Itrf: [Tod(Iers2003::default()), Pef(Iers2003::default())],
    Mod<Iers2010> => Itrf: [Tod(Iers2010), Pef(Iers2010)],
    Mod<Iers1996> => Iau<DynOrigin>: [Icrf],
    Mod<Iers2003> => Iau<DynOrigin>: [Icrf],
    Mod<Iers2010> => Iau<DynOrigin>: [Icrf],
    Mod<Iers1996> => Pef<Iers1996>: [Tod(Iers1996)],
    Mod<Iers2003> => Pef<Iers2003>: [Tod(Iers2003::default())],
    Mod<Iers2010> => Pef<Iers2010>: [Tod(Iers2010)],
    Mod<Iers1996> => Teme: [Tod(Iers1996), Pef(Iers1996)],
    Mod<Iers2003> => Teme: [Tod(Iers2003::default()), Pef(Iers2003::default())],
    Mod<Iers2010> => Teme: [Tod(Iers2010), Pef(Iers2010)],
    Pef<Iers1996> => Cirf: [Tod(Iers1996), Mod(Iers1996), Icrf],
    Pef<Iers2003> => Cirf: [Tod(Iers2003::default()), Mod(Iers2003::default()), Icrf],
    Pef<Iers2010> => Cirf: [Tod(Iers2010), Mod(Iers2010), Icrf],
    Pef<Iers1996> => Icrf: [Tod(Iers1996), Mod(Iers1996)],
    Pef<Iers2003> => Icrf: [Tod(Iers2003::default()), Mod(Iers2003::default())],
    Pef<Iers2010> => Icrf: [Tod(Iers2010), Mod(Iers2010)],
    Pef<Iers1996> => Tirf: [Itrf],
    Pef<Iers2003> => Tirf: [Itrf],
    Pef<Iers2010> => Tirf: [Itrf],
    Pef<Iers1996> => Iau<DynOrigin>: [Tod(Iers1996), Mod(Iers1996), Icrf],
    Pef<Iers2003> => Iau<DynOrigin>: [Tod(Iers2003::default()), Mod(Iers2003::default()), Icrf],
    Pef<Iers2010> => Iau<DynOrigin>: [Tod(Iers2010), Mod(Iers2010), Icrf],
    Pef<Iers1996> => Mod<Iers1996>: [Tod(Iers1996)],
    Pef<Iers2003> => Mod<Iers2003>: [Tod(Iers2003::default())],
    Pef<Iers2010> => Mod<Iers2010>: [Tod(Iers2010)],
    Teme => Cirf: [Pef(Iers1996), Tod(Iers1996), Mod(Iers1996), Icrf],
    Teme => Icrf: [Pef(Iers1996), Tod(Iers1996), Mod(Iers1996)],
    Teme => Tirf: [Pef(Iers1996), Itrf],
    Teme => Itrf: [Pef(Iers1996)],
    Teme => Iau<DynOrigin>: [Pef(Iers1996), Tod(Iers1996), Mod(Iers1996), Icrf],
    Teme => Mod<Iers1996>: [Pef(Iers1996), Tod(Iers1996)],
    Teme => Mod<Iers2003>: [Pef(Iers2003::default()), Tod(Iers2003::default())],
    Teme => Mod<Iers2010>: [Pef(Iers2010), Tod(Iers2010)],
    Teme => Tod<Iers1996>: [Pef(Iers1996)],
    Teme => Tod<Iers2003>: [Pef(Iers2003::default())],
    Teme => Tod<Iers2010>: [Pef(Iers2010)],
    Tirf => Iau<DynOrigin>: [Cirf, Icrf],
    Tirf => Icrf: [Cirf],
    Tirf => Mod<Iers1996>: [Cirf, Icrf],
    Tirf => Mod<Iers2003>: [Cirf, Icrf],
    Tirf => Mod<Iers2010>: [Cirf, Icrf],
    Tirf => Pef<Iers1996>: [Itrf],
    Tirf => Pef<Iers2003>: [Itrf],
    Tirf => Pef<Iers2010>: [Itrf],
    Tirf => Teme: [Itrf, Pef(Iers1996)],
    Tirf => Tod<Iers1996>: [Cirf, Icrf, Mod(Iers1996)],
    Tirf => Tod<Iers2003>: [Cirf, Icrf, Mod(Iers2003::default())],
    Tirf => Tod<Iers2010>: [Cirf, Icrf, Mod(Iers2010)],
    Tod<Iers1996> => Cirf: [Mod(Iers1996), Icrf],
    Tod<Iers2003> => Cirf: [Mod(Iers2003::default()), Icrf],
    Tod<Iers2010> => Cirf: [Mod(Iers2010), Icrf],
    Tod<Iers1996> => Icrf: [Mod(Iers1996)],
    Tod<Iers2003> => Icrf: [Mod(Iers2003::default())],
    Tod<Iers2010> => Icrf: [Mod(Iers2010)],
    Tod<Iers1996> => Tirf: [Mod(Iers1996), Icrf, Cirf],
    Tod<Iers2003> => Tirf: [Mod(Iers2003::default()), Icrf, Cirf],
    Tod<Iers2010> => Tirf: [Mod(Iers2010), Icrf, Cirf],
    Tod<Iers1996> => Itrf: [Pef(Iers1996)],
    Tod<Iers2003> => Itrf: [Pef(Iers2003::default())],
    Tod<Iers2010> => Itrf: [Pef(Iers2010)],
    Tod<Iers1996> => Iau<DynOrigin>: [Mod(Iers1996), Icrf],
    Tod<Iers2003> => Iau<DynOrigin>: [Mod(Iers2003::default()), Icrf],
    Tod<Iers2010> => Iau<DynOrigin>: [Mod(Iers2010), Icrf],
    Tod<Iers1996> => Teme: [Pef(Iers1996)],
    Tod<Iers2003> => Teme: [Pef(Iers2003::default())],
    Tod<Iers2010> => Teme: [Pef(Iers2010)],
);

// Dynamic

impl<T, U> TryRotation<DynFrame, DynFrame, T> for U
where
    T: TimeScale + Copy,
    U: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Tdb> + TryOffset<T, Ut1>,
{
    type Error = DynRotationError;

    fn try_rotation(
        &self,
        origin: DynFrame,
        target: DynFrame,
        time: Time<T>,
    ) -> Result<Rotation, Self::Error> {
        match (origin, target) {
            (DynFrame::Icrf, DynFrame::Cirf) => Ok(self.try_rotation(Icrf, Cirf, time)?),
            (DynFrame::Icrf, DynFrame::Tirf) => Ok(self.try_rotation(Icrf, Tirf, time)?),
            (DynFrame::Icrf, DynFrame::Itrf) => Ok(self.try_rotation(Icrf, Itrf, time)?),
            (DynFrame::Icrf, DynFrame::Iau(body)) => {
                Ok(self.try_rotation(Icrf, Iau::try_new(body)?, time)?)
            }
            (DynFrame::Icrf, DynFrame::Mod(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Icrf, Mod(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Icrf, Mod(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Icrf, Mod(Iers2010), time)?),
            },
            (DynFrame::Icrf, DynFrame::Tod(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Icrf, Tod(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Icrf, Tod(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Icrf, Tod(Iers2010), time)?),
            },
            (DynFrame::Icrf, DynFrame::Pef(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Icrf, Pef(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Icrf, Pef(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Icrf, Pef(Iers2010), time)?),
            },
            (DynFrame::Icrf, DynFrame::Teme) => Ok(self.try_rotation(Icrf, Teme, time)?),
            (DynFrame::Cirf, DynFrame::Icrf) => Ok(self.try_rotation(Cirf, Icrf, time)?),
            (DynFrame::Cirf, DynFrame::Tirf) => Ok(self.try_rotation(Cirf, Tirf, time)?),
            (DynFrame::Cirf, DynFrame::Itrf) => Ok(self.try_rotation(Cirf, Itrf, time)?),
            (DynFrame::Cirf, DynFrame::Iau(body)) => {
                Ok(self.try_rotation(Cirf, Iau::try_new(body)?, time)?)
            }
            (DynFrame::Cirf, DynFrame::Mod(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Cirf, Mod(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Cirf, Mod(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Cirf, Mod(Iers2010), time)?),
            },
            (DynFrame::Cirf, DynFrame::Tod(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Cirf, Tod(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Cirf, Tod(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Cirf, Tod(Iers2010), time)?),
            },
            (DynFrame::Cirf, DynFrame::Pef(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Cirf, Pef(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Cirf, Pef(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Cirf, Pef(Iers2010), time)?),
            },
            (DynFrame::Cirf, DynFrame::Teme) => Ok(self.try_rotation(Cirf, Teme, time)?),
            (DynFrame::Tirf, DynFrame::Icrf) => Ok(self.try_rotation(Tirf, Icrf, time)?),
            (DynFrame::Tirf, DynFrame::Cirf) => Ok(self.try_rotation(Tirf, Cirf, time)?),
            (DynFrame::Tirf, DynFrame::Itrf) => Ok(self.try_rotation(Tirf, Itrf, time)?),
            (DynFrame::Tirf, DynFrame::Iau(body)) => {
                Ok(self.try_rotation(Tirf, Iau::try_new(body)?, time)?)
            }
            (DynFrame::Tirf, DynFrame::Mod(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Tirf, Mod(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Tirf, Mod(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Tirf, Mod(Iers2010), time)?),
            },
            (DynFrame::Tirf, DynFrame::Tod(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Tirf, Tod(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Tirf, Tod(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Tirf, Tod(Iers2010), time)?),
            },
            (DynFrame::Tirf, DynFrame::Pef(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Tirf, Pef(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Tirf, Pef(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Tirf, Pef(Iers2010), time)?),
            },
            (DynFrame::Tirf, DynFrame::Teme) => Ok(self.try_rotation(Tirf, Teme, time)?),
            (DynFrame::Itrf, DynFrame::Icrf) => Ok(self.try_rotation(Itrf, Icrf, time)?),
            (DynFrame::Itrf, DynFrame::Cirf) => Ok(self.try_rotation(Itrf, Cirf, time)?),
            (DynFrame::Itrf, DynFrame::Tirf) => Ok(self.try_rotation(Itrf, Tirf, time)?),
            (DynFrame::Itrf, DynFrame::Iau(body)) => {
                Ok(self.try_rotation(Itrf, Iau::try_new(body)?, time)?)
            }
            (DynFrame::Itrf, DynFrame::Mod(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Itrf, Mod(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Itrf, Mod(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Itrf, Mod(Iers2010), time)?),
            },
            (DynFrame::Itrf, DynFrame::Tod(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Itrf, Tod(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Itrf, Tod(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Itrf, Tod(Iers2010), time)?),
            },
            (DynFrame::Itrf, DynFrame::Pef(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Itrf, Pef(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Itrf, Pef(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Itrf, Pef(Iers2010), time)?),
            },
            (DynFrame::Itrf, DynFrame::Teme) => Ok(self.try_rotation(Itrf, Teme, time)?),
            (DynFrame::Iau(body), DynFrame::Icrf) => {
                Ok(self.try_rotation(Iau::try_new(body)?, Icrf, time)?)
            }
            (DynFrame::Iau(body), DynFrame::Cirf) => {
                Ok(self.try_rotation(Iau::try_new(body)?, Cirf, time)?)
            }
            (DynFrame::Iau(body), DynFrame::Tirf) => {
                Ok(self.try_rotation(Iau::try_new(body)?, Tirf, time)?)
            }
            (DynFrame::Iau(body), DynFrame::Itrf) => {
                Ok(self.try_rotation(Iau::try_new(body)?, Itrf, time)?)
            }
            (DynFrame::Iau(body), DynFrame::Mod(sys)) => match sys {
                ReferenceSystem::Iers1996 => {
                    Ok(self.try_rotation(Iau::try_new(body)?, Mod(Iers1996), time)?)
                }
                ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                    Iau::try_new(body)?,
                    Mod(Iers2003(iau2000_model)),
                    time,
                )?),
                ReferenceSystem::Iers2010 => {
                    Ok(self.try_rotation(Iau::try_new(body)?, Mod(Iers2010), time)?)
                }
            },
            (DynFrame::Iau(body), DynFrame::Tod(sys)) => match sys {
                ReferenceSystem::Iers1996 => {
                    Ok(self.try_rotation(Iau::try_new(body)?, Tod(Iers1996), time)?)
                }
                ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                    Iau::try_new(body)?,
                    Tod(Iers2003(iau2000_model)),
                    time,
                )?),
                ReferenceSystem::Iers2010 => {
                    Ok(self.try_rotation(Iau::try_new(body)?, Tod(Iers2010), time)?)
                }
            },
            (DynFrame::Iau(body), DynFrame::Pef(sys)) => match sys {
                ReferenceSystem::Iers1996 => {
                    Ok(self.try_rotation(Iau::try_new(body)?, Pef(Iers1996), time)?)
                }
                ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                    Iau::try_new(body)?,
                    Pef(Iers2003(iau2000_model)),
                    time,
                )?),
                ReferenceSystem::Iers2010 => {
                    Ok(self.try_rotation(Iau::try_new(body)?, Pef(Iers2010), time)?)
                }
            },
            (DynFrame::Iau(body), DynFrame::Teme) => {
                Ok(self.try_rotation(Iau::try_new(body)?, Teme, time)?)
            }
            (DynFrame::Mod(sys), DynFrame::Icrf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Mod(Iers1996), Icrf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Mod(Iers2003(iau2000_model)), Icrf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Mod(Iers2010), Icrf, time)?),
            },
            (DynFrame::Mod(sys), DynFrame::Cirf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Mod(Iers1996), Cirf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Mod(Iers2003(iau2000_model)), Cirf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Mod(Iers2010), Cirf, time)?),
            },
            (DynFrame::Mod(sys), DynFrame::Tirf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Mod(Iers1996), Tirf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Mod(Iers2003(iau2000_model)), Tirf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Mod(Iers2010), Tirf, time)?),
            },
            (DynFrame::Mod(sys), DynFrame::Itrf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Mod(Iers1996), Itrf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Mod(Iers2003(iau2000_model)), Itrf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Mod(Iers2010), Itrf, time)?),
            },
            (DynFrame::Mod(sys), DynFrame::Iau(body)) => match sys {
                ReferenceSystem::Iers1996 => {
                    Ok(self.try_rotation(Mod(Iers1996), Iau::try_new(body)?, time)?)
                }
                ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                    Mod(Iers2003(iau2000_model)),
                    Iau::try_new(body)?,
                    time,
                )?),
                ReferenceSystem::Iers2010 => {
                    Ok(self.try_rotation(Mod(Iers2010), Iau::try_new(body)?, time)?)
                }
            },
            (DynFrame::Mod(sys1), DynFrame::Tod(sys2)) => {
                if sys1 != sys2 {
                    return Err(DynRotationError::IncompatibleReferenceSystems);
                }
                match sys1 {
                    ReferenceSystem::Iers1996 => {
                        Ok(self.try_rotation(Mod(Iers1996), Tod(Iers1996), time)?)
                    }
                    ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                        Mod(Iers2003(iau2000_model)),
                        Tod(Iers2003(iau2000_model)),
                        time,
                    )?),
                    ReferenceSystem::Iers2010 => {
                        Ok(self.try_rotation(Mod(Iers2010), Tod(Iers2010), time)?)
                    }
                }
            }
            (DynFrame::Mod(sys1), DynFrame::Pef(sys2)) => {
                if sys1 != sys2 {
                    return Err(DynRotationError::IncompatibleReferenceSystems);
                }
                match sys1 {
                    ReferenceSystem::Iers1996 => {
                        Ok(self.try_rotation(Mod(Iers1996), Pef(Iers1996), time)?)
                    }
                    ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                        Mod(Iers2003(iau2000_model)),
                        Pef(Iers2003(iau2000_model)),
                        time,
                    )?),
                    ReferenceSystem::Iers2010 => {
                        Ok(self.try_rotation(Mod(Iers2010), Pef(Iers2010), time)?)
                    }
                }
            }
            (DynFrame::Mod(sys), DynFrame::Teme) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Mod(Iers1996), Teme, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Mod(Iers2003(iau2000_model)), Teme, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Mod(Iers2010), Teme, time)?),
            },
            (DynFrame::Tod(sys), DynFrame::Icrf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Tod(Iers1996), Icrf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Tod(Iers2003(iau2000_model)), Icrf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Tod(Iers2010), Icrf, time)?),
            },
            (DynFrame::Tod(sys), DynFrame::Cirf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Tod(Iers1996), Cirf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Tod(Iers2003(iau2000_model)), Cirf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Tod(Iers2010), Cirf, time)?),
            },
            (DynFrame::Tod(sys), DynFrame::Tirf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Tod(Iers1996), Tirf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Tod(Iers2003(iau2000_model)), Tirf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Tod(Iers2010), Tirf, time)?),
            },
            (DynFrame::Tod(sys), DynFrame::Itrf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Tod(Iers1996), Itrf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Tod(Iers2003(iau2000_model)), Itrf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Tod(Iers2010), Itrf, time)?),
            },
            (DynFrame::Tod(sys), DynFrame::Iau(body)) => match sys {
                ReferenceSystem::Iers1996 => {
                    Ok(self.try_rotation(Tod(Iers1996), Iau::try_new(body)?, time)?)
                }
                ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                    Tod(Iers2003(iau2000_model)),
                    Iau::try_new(body)?,
                    time,
                )?),
                ReferenceSystem::Iers2010 => {
                    Ok(self.try_rotation(Tod(Iers2010), Iau::try_new(body)?, time)?)
                }
            },
            (DynFrame::Tod(sys1), DynFrame::Mod(sys2)) => {
                if sys1 != sys2 {
                    return Err(DynRotationError::IncompatibleReferenceSystems);
                }
                match sys1 {
                    ReferenceSystem::Iers1996 => {
                        Ok(self.try_rotation(Tod(Iers1996), Mod(Iers1996), time)?)
                    }
                    ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                        Tod(Iers2003(iau2000_model)),
                        Mod(Iers2003(iau2000_model)),
                        time,
                    )?),
                    ReferenceSystem::Iers2010 => {
                        Ok(self.try_rotation(Tod(Iers2010), Mod(Iers2010), time)?)
                    }
                }
            }
            (DynFrame::Tod(sys1), DynFrame::Pef(sys2)) => {
                if sys1 != sys2 {
                    return Err(DynRotationError::IncompatibleReferenceSystems);
                }
                match sys1 {
                    ReferenceSystem::Iers1996 => {
                        Ok(self.try_rotation(Tod(Iers1996), Pef(Iers1996), time)?)
                    }
                    ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                        Tod(Iers2003(iau2000_model)),
                        Pef(Iers2003(iau2000_model)),
                        time,
                    )?),
                    ReferenceSystem::Iers2010 => {
                        Ok(self.try_rotation(Tod(Iers2010), Pef(Iers2010), time)?)
                    }
                }
            }
            (DynFrame::Tod(sys), DynFrame::Teme) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Tod(Iers1996), Teme, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Tod(Iers2003(iau2000_model)), Teme, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Tod(Iers2010), Teme, time)?),
            },
            (DynFrame::Pef(sys), DynFrame::Icrf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Pef(Iers1996), Icrf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Pef(Iers2003(iau2000_model)), Icrf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Pef(Iers2010), Icrf, time)?),
            },
            (DynFrame::Pef(sys), DynFrame::Cirf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Pef(Iers1996), Cirf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Pef(Iers2003(iau2000_model)), Cirf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Pef(Iers2010), Cirf, time)?),
            },
            (DynFrame::Pef(sys), DynFrame::Tirf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Pef(Iers1996), Tirf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Pef(Iers2003(iau2000_model)), Tirf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Pef(Iers2010), Tirf, time)?),
            },
            (DynFrame::Pef(sys), DynFrame::Itrf) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Pef(Iers1996), Itrf, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Pef(Iers2003(iau2000_model)), Itrf, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Pef(Iers2010), Itrf, time)?),
            },
            (DynFrame::Pef(sys), DynFrame::Iau(body)) => match sys {
                ReferenceSystem::Iers1996 => {
                    Ok(self.try_rotation(Pef(Iers1996), Iau::try_new(body)?, time)?)
                }
                ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                    Pef(Iers2003(iau2000_model)),
                    Iau::try_new(body)?,
                    time,
                )?),
                ReferenceSystem::Iers2010 => {
                    Ok(self.try_rotation(Pef(Iers2010), Iau::try_new(body)?, time)?)
                }
            },
            (DynFrame::Pef(sys1), DynFrame::Mod(sys2)) => {
                if sys1 != sys2 {
                    return Err(DynRotationError::IncompatibleReferenceSystems);
                }
                match sys1 {
                    ReferenceSystem::Iers1996 => {
                        Ok(self.try_rotation(Pef(Iers1996), Mod(Iers1996), time)?)
                    }
                    ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                        Pef(Iers2003(iau2000_model)),
                        Mod(Iers2003(iau2000_model)),
                        time,
                    )?),
                    ReferenceSystem::Iers2010 => {
                        Ok(self.try_rotation(Pef(Iers2010), Mod(Iers2010), time)?)
                    }
                }
            }
            (DynFrame::Pef(sys1), DynFrame::Tod(sys2)) => {
                if sys1 != sys2 {
                    return Err(DynRotationError::IncompatibleReferenceSystems);
                }
                match sys1 {
                    ReferenceSystem::Iers1996 => {
                        Ok(self.try_rotation(Pef(Iers1996), Tod(Iers1996), time)?)
                    }
                    ReferenceSystem::Iers2003(iau2000_model) => Ok(self.try_rotation(
                        Pef(Iers2003(iau2000_model)),
                        Tod(Iers2003(iau2000_model)),
                        time,
                    )?),
                    ReferenceSystem::Iers2010 => {
                        Ok(self.try_rotation(Pef(Iers2010), Tod(Iers2010), time)?)
                    }
                }
            }
            (DynFrame::Pef(sys), DynFrame::Teme) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Pef(Iers1996), Teme, time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Pef(Iers2003(iau2000_model)), Teme, time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Pef(Iers2010), Teme, time)?),
            },
            (DynFrame::Teme, DynFrame::Icrf) => Ok(self.try_rotation(Teme, Icrf, time)?),
            (DynFrame::Teme, DynFrame::Cirf) => Ok(self.try_rotation(Teme, Cirf, time)?),
            (DynFrame::Teme, DynFrame::Tirf) => Ok(self.try_rotation(Teme, Tirf, time)?),
            (DynFrame::Teme, DynFrame::Itrf) => Ok(self.try_rotation(Teme, Itrf, time)?),
            (DynFrame::Teme, DynFrame::Iau(body)) => {
                Ok(self.try_rotation(Teme, Iau::try_new(body)?, time)?)
            }
            (DynFrame::Teme, DynFrame::Mod(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Teme, Mod(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Teme, Mod(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Teme, Mod(Iers2010), time)?),
            },
            (DynFrame::Teme, DynFrame::Tod(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Teme, Tod(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Teme, Tod(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Teme, Tod(Iers2010), time)?),
            },
            (DynFrame::Teme, DynFrame::Pef(sys)) => match sys {
                ReferenceSystem::Iers1996 => Ok(self.try_rotation(Teme, Pef(Iers1996), time)?),
                ReferenceSystem::Iers2003(iau2000_model) => {
                    Ok(self.try_rotation(Teme, Pef(Iers2003(iau2000_model)), time)?)
                }
                ReferenceSystem::Iers2010 => Ok(self.try_rotation(Teme, Pef(Iers2010), time)?),
            },
            (_, _) => Ok(Rotation::IDENTITY),
        }
    }
}
