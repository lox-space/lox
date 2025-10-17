use std::convert::Infallible;

use crate::time_scales::{Tai, Tcb, Tcg, Tdb, TimeScale, Tt, Ut1};
use crate::{deltas::TimeDelta, julian_dates::J77, subsecond::Subsecond};
use thiserror::Error;

pub trait OffsetProvider {}

pub trait TryOffset<Origin, Target>: OffsetProvider
where
    Origin: TimeScale,
    Target: TimeScale,
{
    type Error: std::error::Error + Send + Sync + 'static;

    fn try_offset(
        &self,
        origin: Origin,
        target: Target,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error>;
}

pub trait Offset<Origin, Target>: OffsetProvider
where
    Origin: TimeScale,
    Target: TimeScale,
{
    fn offset(&self, origin: Origin, target: Target, delta: TimeDelta) -> TimeDelta;
}

impl<T, Origin, Target> Offset<Origin, Target> for T
where
    Origin: TimeScale,
    Target: TimeScale,
    T: TryOffset<Origin, Target, Error = Infallible>,
{
    fn offset(&self, origin: Origin, target: Target, delta: TimeDelta) -> TimeDelta {
        self.try_offset(origin, target, delta).unwrap()
    }
}

#[derive(Debug, Error, Clone, Copy, Eq, PartialEq)]
#[error("an EOP provider is required for transformations from/to UT1")]
pub struct MissingEopProviderError;

impl From<Infallible> for MissingEopProviderError {
    fn from(_: Infallible) -> Self {
        MissingEopProviderError
    }
}

macro_rules! impl_noop {
    ($($scale:ident),*) => {
        $(
            impl<T> TryOffset<$scale, $scale> for T where T: OffsetProvider {
                type Error = MissingEopProviderError;

                fn try_offset(&self, _origin: $scale, _target: $scale, _delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
                    Ok(TimeDelta::default())
                }
            }
        )*
    };
}

impl_noop!(Tai, Tcb, Tcg, Tdb, Tt, Ut1);

// TAI <-> TT

/// The constant offset between TAI and TT.
pub const D_TAI_TT: TimeDelta = TimeDelta {
    seconds: 32,
    subsecond: Subsecond(0.184),
};

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
        Ok(D_TAI_TT)
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
        Ok(-D_TAI_TT)
    }
}

// TT <-> TCG

/// The difference between J2000 TT and 1977 January 1.0 TAI as TT.
const J77_TT: f64 = -7.25803167816e8;

/// The rate of change of TCG with respect to TT.
const LG: f64 = 6.969290134e-10;

/// The rate of change of TT with respect to TCG.
const INV_LG: f64 = LG / (1.0 - LG);

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
        let tt = delta.to_decimal_seconds();
        Ok(TimeDelta::from_decimal_seconds(INV_LG * (tt - J77_TT)))
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
        let tcg = delta.to_decimal_seconds();
        Ok(TimeDelta::from_decimal_seconds(-LG * (tcg - J77_TT)))
    }
}

// TDB <-> TCB

/// 1977 January 1.0 TAI
const TT_0: f64 = J77.seconds as f64 + D_TAI_TT.seconds as f64 + D_TAI_TT.subsecond.0;

/// The rate of change of TDB with respect to TCB.
const LB: f64 = 1.550519768e-8;

/// The rate of change of TCB with respect to TDB.
const INV_LB: f64 = LB / (1.0 - LB);

/// Constant term of TDB − TT formula of Fairhead & Bretagnon (1990).
const TDB_0: f64 = -6.55e-5;

const TCB_77: f64 = TDB_0 + LB * TT_0;

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
        let tdb = delta.to_decimal_seconds();
        Ok(TimeDelta::from_decimal_seconds(
            -TCB_77 / (1.0 - LB) + INV_LB * tdb,
        ))
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
        let tcb = delta.to_decimal_seconds();
        Ok(TimeDelta::from_decimal_seconds(TCB_77 - LB * tcb))
    }
}

// TT <-> TDB

const K: f64 = 1.657e-3;
const EB: f64 = 1.671e-2;
const M_0: f64 = 6.239996;
const M_1: f64 = 1.99096871e-7;

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
        let tt = delta.to_decimal_seconds();
        let g = M_0 + M_1 * tt;
        Ok(TimeDelta::from_decimal_seconds(
            K * (g + EB * g.sin()).sin(),
        ))
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
        let tdb = delta.to_decimal_seconds();
        let mut offset = 0.0;
        for _ in 1..3 {
            let g = M_0 + M_1 * (tdb + offset);
            offset = -K * (g + EB * g.sin()).sin();
        }
        Ok(TimeDelta::from_decimal_seconds(offset))
    }
}

// Two-step transformations

fn two_step_offset<P, T1, T2, T3>(
    provider: &P,
    origin: T1,
    via: T2,
    target: T3,
    delta: TimeDelta,
) -> TimeDelta
where
    T1: TimeScale,
    T2: TimeScale + Copy,
    T3: TimeScale,
    P: Offset<T1, T2> + Offset<T2, T3>,
{
    let mut offset = provider.offset(origin, via, delta);
    offset += provider.offset(via, target, delta + offset);
    offset
}

macro_rules! impl_two_step {
    ($(($origin:ident, $via:ident, $target:ident)),*) => {
        $(
            impl<T> TryOffset<$origin, $target> for T
            where
                T: OffsetProvider,
            {
                type Error = Infallible;

                fn try_offset(&self, origin: $origin, target: $target, delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
                    Ok(two_step_offset(self, origin, $via, target, delta))
                }
            }

            impl<T> TryOffset<$target, $origin> for T
            where
                T: OffsetProvider,
            {
                type Error = Infallible;

                fn try_offset(&self, origin: $target, target: $origin, delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
                    Ok(two_step_offset(self, origin, $via, target, delta))
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
