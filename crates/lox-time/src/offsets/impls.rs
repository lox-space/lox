// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use core::convert::Infallible;

use lox_core::time::deltas::TimeDelta;

use crate::{
    offsets::{Offset, OffsetProvider, TryOffset},
    time_scales::{Gps, Tai, Tcb, Tcg, Tdb, TimeScale, Tt, Ut1},
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

impl_noop!(Gps, Tai, Tcb, Tcg, Tdb, Tt, Ut1);

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

// TAI <-> GPS

impl<T> TryOffset<Tai, Gps> for T
where
    T: OffsetProvider,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: Tai,
        _target: Gps,
        _delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.tai_to_gps())
    }
}

impl<T> TryOffset<Gps, Tai> for T
where
    T: OffsetProvider,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        _origin: Gps,
        _target: Tai,
        _delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.gps_to_tai())
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
    (Tcb, Tdb, Tcg),
    (Gps, Tai, Tt),
    (Gps, Tai, Tcg),
    (Gps, Tai, Tdb),
    (Gps, Tai, Tcb)
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

impl_two_step_ut1!(Gps, Tcb, Tcg, Tdb, Tt);

// Dynamic

impl<T> TryOffset<TimeScale, TimeScale> for T
where
    T: OffsetProvider,
{
    type Error = <Self as OffsetProvider>::Error;

    fn try_offset(
        &self,
        origin: TimeScale,
        target: TimeScale,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        if origin == target {
            return Ok(TimeDelta::default());
        }
        match (origin, target) {
            (TimeScale::Gps, TimeScale::Tai) => Ok(self.offset(Gps, Tai, delta)),
            (TimeScale::Gps, TimeScale::Tcb) => Ok(self.offset(Gps, Tcb, delta)),
            (TimeScale::Gps, TimeScale::Tcg) => Ok(self.offset(Gps, Tcg, delta)),
            (TimeScale::Gps, TimeScale::Tdb) => Ok(self.offset(Gps, Tdb, delta)),
            (TimeScale::Gps, TimeScale::Tt) => Ok(self.offset(Gps, Tt, delta)),
            (TimeScale::Gps, TimeScale::Ut1) => self.try_offset(Gps, Ut1, delta),
            (TimeScale::Tai, TimeScale::Gps) => Ok(self.offset(Tai, Gps, delta)),
            (TimeScale::Tai, TimeScale::Tcb) => Ok(self.offset(Tai, Tcb, delta)),
            (TimeScale::Tai, TimeScale::Tcg) => Ok(self.offset(Tai, Tcg, delta)),
            (TimeScale::Tai, TimeScale::Tdb) => Ok(self.offset(Tai, Tdb, delta)),
            (TimeScale::Tai, TimeScale::Tt) => Ok(self.offset(Tai, Tt, delta)),
            (TimeScale::Tai, TimeScale::Ut1) => self.try_offset(Tai, Ut1, delta),
            (TimeScale::Tcb, TimeScale::Gps) => Ok(self.offset(Tcb, Gps, delta)),
            (TimeScale::Tcb, TimeScale::Tai) => Ok(self.offset(Tcb, Tai, delta)),
            (TimeScale::Tcb, TimeScale::Tcg) => Ok(self.offset(Tcb, Tcg, delta)),
            (TimeScale::Tcb, TimeScale::Tdb) => Ok(self.offset(Tcb, Tdb, delta)),
            (TimeScale::Tcb, TimeScale::Tt) => Ok(self.offset(Tcb, Tt, delta)),
            (TimeScale::Tcb, TimeScale::Ut1) => self.try_offset(Tcb, Ut1, delta),
            (TimeScale::Tcg, TimeScale::Gps) => Ok(self.offset(Tcg, Gps, delta)),
            (TimeScale::Tcg, TimeScale::Tai) => Ok(self.offset(Tcg, Tai, delta)),
            (TimeScale::Tcg, TimeScale::Tcb) => Ok(self.offset(Tcg, Tcb, delta)),
            (TimeScale::Tcg, TimeScale::Tdb) => Ok(self.offset(Tcg, Tdb, delta)),
            (TimeScale::Tcg, TimeScale::Tt) => Ok(self.offset(Tcg, Tt, delta)),
            (TimeScale::Tcg, TimeScale::Ut1) => self.try_offset(Tcg, Ut1, delta),
            (TimeScale::Tdb, TimeScale::Gps) => Ok(self.offset(Tdb, Gps, delta)),
            (TimeScale::Tdb, TimeScale::Tai) => Ok(self.offset(Tdb, Tai, delta)),
            (TimeScale::Tdb, TimeScale::Tcb) => Ok(self.offset(Tdb, Tcb, delta)),
            (TimeScale::Tdb, TimeScale::Tcg) => Ok(self.offset(Tdb, Tcg, delta)),
            (TimeScale::Tdb, TimeScale::Tt) => Ok(self.offset(Tdb, Tt, delta)),
            (TimeScale::Tdb, TimeScale::Ut1) => self.try_offset(Tdb, Ut1, delta),
            (TimeScale::Tt, TimeScale::Gps) => Ok(self.offset(Tt, Gps, delta)),
            (TimeScale::Tt, TimeScale::Tai) => Ok(self.offset(Tt, Tai, delta)),
            (TimeScale::Tt, TimeScale::Tcb) => Ok(self.offset(Tt, Tcb, delta)),
            (TimeScale::Tt, TimeScale::Tcg) => Ok(self.offset(Tt, Tcg, delta)),
            (TimeScale::Tt, TimeScale::Tdb) => Ok(self.offset(Tt, Tdb, delta)),
            (TimeScale::Tt, TimeScale::Ut1) => self.try_offset(Tt, Ut1, delta),
            (TimeScale::Ut1, TimeScale::Gps) => self.try_offset(Ut1, Gps, delta),
            (TimeScale::Ut1, TimeScale::Tai) => self.try_offset(Ut1, Tai, delta),
            (TimeScale::Ut1, TimeScale::Tcb) => self.try_offset(Ut1, Tcb, delta),
            (TimeScale::Ut1, TimeScale::Tcg) => self.try_offset(Ut1, Tcg, delta),
            (TimeScale::Ut1, TimeScale::Tdb) => self.try_offset(Ut1, Tdb, delta),
            (TimeScale::Ut1, TimeScale::Tt) => self.try_offset(Ut1, Tt, delta),
            (TimeScale::Gps, TimeScale::Gps)
            | (TimeScale::Tai, TimeScale::Tai)
            | (TimeScale::Tcb, TimeScale::Tcb)
            | (TimeScale::Tcg, TimeScale::Tcg)
            | (TimeScale::Tdb, TimeScale::Tdb)
            | (TimeScale::Tt, TimeScale::Tt)
            | (TimeScale::Ut1, TimeScale::Ut1) => Ok(TimeDelta::default()),
        }
    }
}

macro_rules! impl_dyn {
    ($($scale:ident),*) => {
        $(
            impl<T> TryOffset<$scale, TimeScale> for T
            where
                T: OffsetProvider,
            {
                type Error = <Self as OffsetProvider>::Error;

                fn try_offset(
                    &self,
                    origin: $scale,
                    target: TimeScale,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    let origin: TimeScale = origin.into();
                    self.try_offset(origin, target, delta)
                }
            }

            impl<T> TryOffset<TimeScale, $scale> for T
            where
                T: OffsetProvider,
            {
                type Error = <Self as OffsetProvider>::Error;

                fn try_offset(
                    &self,
                    origin: TimeScale,
                    target: $scale,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    let target: TimeScale = target.into();
                    self.try_offset(origin, target, delta)
                }
            }
        )*
    };
}

impl_dyn!(Gps, Tai, Tcb, Tcg, Tdb, Tt, Ut1);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offsets::DefaultOffsetProvider;

    #[test]
    fn tai_to_gps_is_negative_19_seconds() {
        let p = DefaultOffsetProvider;
        let delta = p.offset(Tai, Gps, TimeDelta::default());
        assert_eq!(delta, TimeDelta::builder().seconds(-19).build());
    }

    #[test]
    fn gps_to_tai_is_positive_19_seconds() {
        let p = DefaultOffsetProvider;
        let delta = p.offset(Gps, Tai, TimeDelta::default());
        assert_eq!(delta, TimeDelta::builder().seconds(19).build());
    }

    #[test]
    fn tai_gps_round_trip_is_zero() {
        let p = DefaultOffsetProvider;
        let to_gps = p.offset(Tai, Gps, TimeDelta::default());
        let back_to_tai = p.offset(Gps, Tai, to_gps);
        assert_eq!(to_gps + back_to_tai, TimeDelta::default());
    }
}
