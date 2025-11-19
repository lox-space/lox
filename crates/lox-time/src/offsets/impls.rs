// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::convert::Infallible;

use lox_core::time::deltas::TimeDelta;

use crate::{
    offsets::{Offset, OffsetProvider, TryOffset},
    time_scales::{DynTimeScale, Tai, Tcb, Tcg, Tdb, Tt, Ut1},
};

// No-ops

macro_rules! impl_noop {
    ($($scale:ident),*) => {
        $(
            impl<T> TryOffset<$scale, $scale> for T
            where
                T: OffsetProvider,
            {
                type Error = Infallible;

                fn try_offset(
                    &self,
                    _origin: $scale,
                    _target: $scale,
                    _delta: TimeDelta
                ) -> Result<TimeDelta, Self::Error> {
                    Ok(TimeDelta::default())
                }
            }
        )*
    };
}

impl_noop!(Tai, Tcb, Tcg, Tdb, Tt, Ut1);

// TAI <-> TT

impl<T> TryOffset<Tai, Tt> for T
where
    T: OffsetProvider,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: Tai,
        _target: Tt,
        _delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.tai_to_tt())
    }
}

impl<T> TryOffset<Tt, Tai> for T
where
    T: OffsetProvider,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: Tt,
        _target: Tai,
        _delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.tt_to_tai())
    }
}

// TT <-> TCG

impl<T> TryOffset<Tt, Tcg> for T
where
    T: OffsetProvider,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: Tt,
        _target: Tcg,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.tt_to_tcg(delta))
    }
}

impl<T> TryOffset<Tcg, Tt> for T
where
    T: OffsetProvider,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: Tcg,
        _target: Tt,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.tcg_to_tt(delta))
    }
}

// TDB <-> TCB

impl<T> TryOffset<Tdb, Tcb> for T
where
    T: OffsetProvider,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: Tdb,
        _target: Tcb,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.tdb_to_tcb(delta))
    }
}

impl<T> TryOffset<Tcb, Tdb> for T
where
    T: OffsetProvider,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: Tcb,
        _target: Tdb,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.tcb_to_tdb(delta))
    }
}

// TT <-> TDB

impl<T> TryOffset<Tt, Tdb> for T
where
    T: OffsetProvider,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: Tt,
        _target: Tdb,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.tt_to_tdb(delta))
    }
}

impl<T> TryOffset<Tdb, Tt> for T
where
    T: OffsetProvider,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: Tdb,
        _target: Tt,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.tdb_to_tt(delta))
    }
}

// TAI <-> UT1

impl<T> TryOffset<Tai, Ut1> for T
where
    T: OffsetProvider,
{
    type Error = <Self as OffsetProvider>::Error;

    fn try_offset(
        &self,
        _origin: Tai,
        _target: Ut1,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        self.tai_to_ut1(delta)
    }
}

impl<T> TryOffset<Ut1, Tai> for T
where
    T: OffsetProvider,
{
    type Error = <Self as OffsetProvider>::Error;

    fn try_offset(
        &self,
        _origin: Ut1,
        _target: Tai,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        self.ut1_to_tai(delta)
    }
}

// Two-step

macro_rules! impl_two_step {
    ($(($origin:ident, $via:ident, $target:ident)),*) => {
        $(
            impl<T> TryOffset<$origin, $target> for T
            where
                T: OffsetProvider,
            {
                type Error = Infallible;

                fn try_offset(
                    &self,
                    origin: $origin,
                    target: $target,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    Ok(super::two_step_offset(self, origin, $via, target, delta))
                }
            }

            impl<T> TryOffset<$target, $origin> for T
            where
                T: OffsetProvider,
            {
                type Error = Infallible;

                fn try_offset(
                    &self,
                    origin: $target,
                    target: $origin,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    Ok(super::two_step_offset(self, origin, $via, target, delta))
                }
            }
        )*
    }
}

impl_two_step!(
    (Tai, Tt, Tdb),
    (Tdb, Tt, Tcg),
    (Tai, Tt, Tcg),
    (Tai, Tdb, Tcb),
    (Tt, Tdb, Tcb),
    (Tcb, Tdb, Tcg)
);

macro_rules! impl_two_step_ut1 {
    ($($scale:ident),*) => {
        $(
            impl<T> TryOffset<$scale, Ut1> for T
            where
                T: OffsetProvider,
            {
                type Error = <Self as OffsetProvider>::Error;

                fn try_offset(
                    &self,
                    _origin: $scale,
                    _target: Ut1,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    let mut offset = self.offset($scale, Tai, delta);
                    offset += self.try_offset(Tai, Ut1, delta + offset)?;
                    Ok(offset)
                }
            }

            impl<T> TryOffset<Ut1, $scale> for T
            where
                T: OffsetProvider,
            {
                type Error = <Self as OffsetProvider>::Error;

                fn try_offset(
                    &self,
                    _origin: Ut1,
                    _target: $scale,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    let mut offset = self.try_offset(Ut1, Tai, delta)?;
                    offset += self.offset(Tai, $scale, delta + offset);
                    Ok(offset)
                }
            }
        )*
    };
}

impl_two_step_ut1!(Tcb, Tcg, Tdb, Tt);

// Dynamic

impl<T> TryOffset<DynTimeScale, DynTimeScale> for T
where
    T: OffsetProvider,
{
    type Error = <Self as OffsetProvider>::Error;

    fn try_offset(
        &self,
        origin: DynTimeScale,
        target: DynTimeScale,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        match (origin, target) {
            (DynTimeScale::Tai, DynTimeScale::Tcb) => Ok(self.offset(Tai, Tcb, delta)),
            (DynTimeScale::Tai, DynTimeScale::Tcg) => Ok(self.offset(Tai, Tcg, delta)),
            (DynTimeScale::Tai, DynTimeScale::Tdb) => Ok(self.offset(Tai, Tdb, delta)),
            (DynTimeScale::Tai, DynTimeScale::Tt) => Ok(self.offset(Tai, Tt, delta)),
            (DynTimeScale::Tai, DynTimeScale::Ut1) => self.try_offset(Tai, Ut1, delta),
            (DynTimeScale::Tcb, DynTimeScale::Tai) => Ok(self.offset(Tcb, Tai, delta)),
            (DynTimeScale::Tcb, DynTimeScale::Tcg) => Ok(self.offset(Tcb, Tcg, delta)),
            (DynTimeScale::Tcb, DynTimeScale::Tdb) => Ok(self.offset(Tcb, Tdb, delta)),
            (DynTimeScale::Tcb, DynTimeScale::Tt) => Ok(self.offset(Tcb, Tt, delta)),
            (DynTimeScale::Tcb, DynTimeScale::Ut1) => self.try_offset(Tcb, Ut1, delta),
            (DynTimeScale::Tcg, DynTimeScale::Tai) => Ok(self.offset(Tcg, Tai, delta)),
            (DynTimeScale::Tcg, DynTimeScale::Tcb) => Ok(self.offset(Tcg, Tcb, delta)),
            (DynTimeScale::Tcg, DynTimeScale::Tdb) => Ok(self.offset(Tcg, Tdb, delta)),
            (DynTimeScale::Tcg, DynTimeScale::Tt) => Ok(self.offset(Tcg, Tt, delta)),
            (DynTimeScale::Tcg, DynTimeScale::Ut1) => self.try_offset(Tcg, Ut1, delta),
            (DynTimeScale::Tdb, DynTimeScale::Tai) => Ok(self.offset(Tdb, Tai, delta)),
            (DynTimeScale::Tdb, DynTimeScale::Tcb) => Ok(self.offset(Tdb, Tcb, delta)),
            (DynTimeScale::Tdb, DynTimeScale::Tcg) => Ok(self.offset(Tdb, Tcg, delta)),
            (DynTimeScale::Tdb, DynTimeScale::Tt) => Ok(self.offset(Tdb, Tt, delta)),
            (DynTimeScale::Tdb, DynTimeScale::Ut1) => self.try_offset(Tdb, Ut1, delta),
            (DynTimeScale::Tt, DynTimeScale::Tai) => Ok(self.offset(Tt, Tai, delta)),
            (DynTimeScale::Tt, DynTimeScale::Tcb) => Ok(self.offset(Tt, Tcb, delta)),
            (DynTimeScale::Tt, DynTimeScale::Tcg) => Ok(self.offset(Tt, Tcg, delta)),
            (DynTimeScale::Tt, DynTimeScale::Tdb) => Ok(self.offset(Tt, Tdb, delta)),
            (DynTimeScale::Tt, DynTimeScale::Ut1) => self.try_offset(Tt, Ut1, delta),
            (DynTimeScale::Ut1, DynTimeScale::Tai) => self.try_offset(Ut1, Tai, delta),
            (DynTimeScale::Ut1, DynTimeScale::Tcb) => self.try_offset(Ut1, Tcb, delta),
            (DynTimeScale::Ut1, DynTimeScale::Tcg) => self.try_offset(Ut1, Tcg, delta),
            (DynTimeScale::Ut1, DynTimeScale::Tdb) => self.try_offset(Ut1, Tdb, delta),
            (DynTimeScale::Ut1, DynTimeScale::Tt) => self.try_offset(Ut1, Tt, delta),
            (_, _) => Ok(TimeDelta::default()),
        }
    }
}

macro_rules! impl_dyn {
    ($($scale:ident),*) => {
        $(
            impl<T> TryOffset<$scale, DynTimeScale> for T
            where
                T: OffsetProvider,
            {
                type Error = <Self as OffsetProvider>::Error;

                fn try_offset(
                    &self,
                    origin: $scale,
                    target: DynTimeScale,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    let origin: DynTimeScale = origin.into();
                    self.try_offset(origin, target, delta)
                }
            }

            impl<T> TryOffset<DynTimeScale, $scale> for T
            where
                T: OffsetProvider,
            {
                type Error = <Self as OffsetProvider>::Error;

                fn try_offset(
                    &self,
                    origin: DynTimeScale,
                    target: $scale,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    let target: DynTimeScale = target.into();
                    self.try_offset(origin, target, delta)
                }
            }
        )*
    };
}

impl_dyn!(Tai, Tcb, Tcg, Tdb, Tt, Ut1);
