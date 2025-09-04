use super::{DynTimeScale, FromScale, Tai, Tcb, Tcg, Tdb, TimeScale, ToScale, TryToScale, Tt, Ut1};
use crate::{
    constants::julian_dates::J77,
    deltas::TimeDelta,
    subsecond::Subsecond,
    ut1::{DeltaUt1Tai, DeltaUt1TaiProvider, ExtrapolatedDeltaUt1Tai},
};
use std::convert::Infallible;
use thiserror::Error;

pub trait TryOffset<Origin, Target, Error>
where
    Origin: TimeScale,
    Target: TimeScale,
    Error: std::error::Error + Send + Sync + 'static,
{
    fn try_offset(
        &self,
        origin: Origin,
        target: Target,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Error>;
}

pub trait Offset<Origin, Target>
where
    Origin: TimeScale,
    Target: TimeScale,
{
    fn offset(&self, origin: Origin, target: Target, delta: TimeDelta) -> TimeDelta;
}

impl<Origin, Target, T> Offset<Origin, Target> for T
where
    Origin: TimeScale,
    Target: TimeScale,
    T: TryOffset<Origin, Target, Infallible>,
{
    fn offset(&self, origin: Origin, target: Target, delta: TimeDelta) -> TimeDelta {
        self.try_offset(origin, target, delta).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultOffsetProvider;

macro_rules! impl_fallible {
    ($in:ident, $out:ident) => {
        impl<P> TryToScale<$out, P> for $in {
            type Error = Infallible;

            fn try_offset(
                &self,
                scale: $out,
                delta: TimeDelta,
                _provider: Option<&P>,
            ) -> Result<TimeDelta, Self::Error> {
                Ok(self.offset(scale, delta))
            }
        }
    };
}

macro_rules! impl_noops {
    ($scale:ident) => {
        impl<P> TryToScale<$scale, P> for $scale {
            type Error = Infallible;

            fn try_offset(
                &self,
                _scale: $scale,
                _delta: TimeDelta,
                _provider: Option<&P>,
            ) -> Result<TimeDelta, Self::Error> {
                Ok(TimeDelta::default())
            }
        }

        impl ToScale<$scale> for $scale {
            fn offset(&self, _scale: $scale, _delta: TimeDelta) -> TimeDelta {
                TimeDelta::default()
            }
        }
    };
}

impl_noops!(Tai);
impl_noops!(Tcb);
impl_noops!(Tcg);
impl_noops!(Tdb);
impl_noops!(Tt);
impl_noops!(Ut1);

// TAI <-> TT

/// The constant offset between TAI and TT.
pub const D_TAI_TT: TimeDelta = TimeDelta {
    seconds: 32,
    subsecond: Subsecond(0.184),
};

impl TryOffset<Tai, Tt, Infallible> for DefaultOffsetProvider {
    fn try_offset(
        &self,
        _origin: Tai,
        _target: Tt,
        _delta: TimeDelta,
    ) -> Result<TimeDelta, Infallible> {
        Ok(D_TAI_TT)
    }
}

impl TryOffset<Tt, Tai, Infallible> for DefaultOffsetProvider {
    fn try_offset(
        &self,
        _origin: Tt,
        _target: Tai,
        _delta: TimeDelta,
    ) -> Result<TimeDelta, Infallible> {
        Ok(-D_TAI_TT)
    }
}

impl ToScale<Tt> for Tai {
    fn offset(&self, _scale: Tt, _dt: TimeDelta) -> TimeDelta {
        D_TAI_TT
    }
}

impl_fallible!(Tai, Tt);

impl ToScale<Tai> for Tt {
    fn offset(&self, _scale: Tai, _dt: TimeDelta) -> TimeDelta {
        -D_TAI_TT
    }
}

impl_fallible!(Tt, Tai);

// TT <-> TCG

/// The difference between J2000 TT and 1977 January 1.0 TAI as TT.
const J77_TT: f64 = -7.25803167816e8;

/// The rate of change of TCG with respect to TT.
const LG: f64 = 6.969290134e-10;

/// The rate of change of TT with respect to TCG.
const INV_LG: f64 = LG / (1.0 - LG);

impl TryOffset<Tt, Tcg, Infallible> for DefaultOffsetProvider {
    fn try_offset(
        &self,
        _origin: Tt,
        _target: Tcg,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Infallible> {
        let tt = delta.to_decimal_seconds();
        Ok(TimeDelta::from_decimal_seconds(INV_LG * (tt - J77_TT)))
    }
}

impl TryOffset<Tcg, Tt, Infallible> for DefaultOffsetProvider {
    fn try_offset(
        &self,
        _origin: Tcg,
        _target: Tt,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Infallible> {
        let tcg = delta.to_decimal_seconds();
        Ok(TimeDelta::from_decimal_seconds(-LG * (tcg - J77_TT)))
    }
}

impl ToScale<Tcg> for Tt {
    fn offset(&self, _scale: Tcg, dt: TimeDelta) -> TimeDelta {
        let tt = dt.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(INV_LG * (tt - J77_TT))
    }
}

impl_fallible!(Tt, Tcg);

impl ToScale<Tt> for Tcg {
    fn offset(&self, _scale: Tt, dt: TimeDelta) -> TimeDelta {
        let tcg = dt.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(-LG * (tcg - J77_TT))
    }
}

impl_fallible!(Tcg, Tt);

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

impl TryOffset<Tdb, Tcb, Infallible> for DefaultOffsetProvider {
    fn try_offset(
        &self,
        _origin: Tdb,
        _target: Tcb,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Infallible> {
        let tdb = delta.to_decimal_seconds();
        Ok(TimeDelta::from_decimal_seconds(
            -TCB_77 / (1.0 - LB) + INV_LB * tdb,
        ))
    }
}

impl TryOffset<Tcb, Tdb, Infallible> for DefaultOffsetProvider {
    fn try_offset(
        &self,
        _origin: Tcb,
        _target: Tdb,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Infallible> {
        let tcb = delta.to_decimal_seconds();
        Ok(TimeDelta::from_decimal_seconds(TCB_77 - LB * tcb))
    }
}

impl ToScale<Tcb> for Tdb {
    fn offset(&self, _scale: Tcb, dt: TimeDelta) -> TimeDelta {
        let tdb = dt.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(-TCB_77 / (1.0 - LB) + INV_LB * tdb)
    }
}

impl_fallible!(Tdb, Tcb);

impl ToScale<Tdb> for Tcb {
    fn offset(&self, _scale: Tdb, dt: TimeDelta) -> TimeDelta {
        let tcb = dt.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(TCB_77 - LB * tcb)
    }
}

impl_fallible!(Tcb, Tdb);

// TT <-> TDB

const K: f64 = 1.657e-3;
const EB: f64 = 1.671e-2;
const M_0: f64 = 6.239996;
const M_1: f64 = 1.99096871e-7;

impl TryOffset<Tt, Tdb, Infallible> for DefaultOffsetProvider {
    fn try_offset(
        &self,
        _origin: Tt,
        _target: Tdb,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Infallible> {
        let tt = delta.to_decimal_seconds();
        let g = M_0 + M_1 * tt;
        Ok(TimeDelta::from_decimal_seconds(
            K * (g + EB * g.sin()).sin(),
        ))
    }
}

impl TryOffset<Tdb, Tt, Infallible> for DefaultOffsetProvider {
    fn try_offset(
        &self,
        _origin: Tdb,
        _target: Tt,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Infallible> {
        let tdb = delta.to_decimal_seconds();
        let mut offset = 0.0;
        for _ in 1..3 {
            let g = M_0 + M_1 * (tdb + offset);
            offset = -K * (g + EB * g.sin()).sin();
        }
        Ok(TimeDelta::from_decimal_seconds(offset))
    }
}

impl ToScale<Tdb> for Tt {
    fn offset(&self, _scale: Tdb, dt: TimeDelta) -> TimeDelta {
        let tt = dt.to_decimal_seconds();
        let g = M_0 + M_1 * tt;
        TimeDelta::from_decimal_seconds(K * (g + EB * g.sin()).sin())
    }
}

impl_fallible!(Tt, Tdb);

impl ToScale<Tt> for Tdb {
    fn offset(&self, _scale: Tt, dt: TimeDelta) -> TimeDelta {
        let tdb = dt.to_decimal_seconds();
        let mut offset = 0.0;
        for _ in 1..3 {
            let g = M_0 + M_1 * (tdb + offset);
            offset = -K * (g + EB * g.sin()).sin();
        }
        TimeDelta::from_decimal_seconds(offset)
    }
}

impl_fallible!(Tdb, Tt);

// TAI <-> UT1

impl<T: TimeScale> TryOffset<T, Ut1, ExtrapolatedDeltaUt1Tai> for DeltaUt1Tai
where
    DefaultOffsetProvider: TryOffset<T, Tai, Infallible>,
{
    fn try_offset(
        &self,
        origin: T,
        _target: Ut1,
        delta: TimeDelta,
    ) -> Result<TimeDelta, ExtrapolatedDeltaUt1Tai> {
        let tai = delta
            + DefaultOffsetProvider
                .try_offset(origin, Tai, delta)
                .unwrap();
        self.delta_ut1_tai(tai)
    }
}

impl<T: TimeScale> TryOffset<Ut1, T, ExtrapolatedDeltaUt1Tai> for DeltaUt1Tai
where
    DefaultOffsetProvider: TryOffset<Tai, T, Infallible>,
{
    fn try_offset(
        &self,
        _origin: Ut1,
        target: T,
        delta: TimeDelta,
    ) -> Result<TimeDelta, ExtrapolatedDeltaUt1Tai> {
        let tai = delta + self.delta_tai_ut1(delta)?;
        Ok(DefaultOffsetProvider.try_offset(Tai, target, tai).unwrap())
    }
}

impl<T: TimeScale> TryOffset<T, T, Infallible> for DeltaUt1Tai
where
    DefaultOffsetProvider: TryOffset<Tai, T, Infallible>,
{
    fn try_offset(
        &self,
        _origin: T,
        _target: T,
        _delta: TimeDelta,
    ) -> Result<TimeDelta, Infallible> {
        Ok(TimeDelta::default())
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum Ut1Error {
    #[error("a UT1-TAI provider is required but was not provided")]
    MissingProvider,
    #[error("failed provider: {0}")]
    FailedProvider(String),
}

impl<P> TryToScale<Ut1, P> for Tai
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        _scale: Ut1,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        provider
            .ok_or(Ut1Error::MissingProvider)?
            .delta_ut1_tai(dt)
            .map_err(|err| Ut1Error::FailedProvider(err.to_string()))
    }
}

impl<P> TryToScale<Tai, P> for Ut1
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        _scale: Tai,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        provider
            .ok_or(Ut1Error::MissingProvider)?
            .delta_tai_ut1(dt)
            .map_err(|err| Ut1Error::FailedProvider(err.to_string()))
    }
}

// Multi-step transformations

fn two_step_offset<T1, T2, T3>(origin: T1, via: T2, target: T3, delta: TimeDelta) -> TimeDelta
where
    T1: TimeScale,
    T2: TimeScale + Copy,
    T3: TimeScale,
    DefaultOffsetProvider: TryOffset<T1, T2, Infallible>,
    DefaultOffsetProvider: TryOffset<T2, T3, Infallible>,
{
    let mut offset = DefaultOffsetProvider
        .try_offset(origin, via, delta)
        .unwrap();
    offset += DefaultOffsetProvider
        .try_offset(via, target, delta + offset)
        .unwrap();
    offset
}

macro_rules! two_step_impl {
    ($origin:ident, $via:ident, $target:ident) => {
        impl TryOffset<$origin, $target, Infallible> for DefaultOffsetProvider {
            fn try_offset(
                &self,
                origin: $origin,
                target: $target,
                delta: TimeDelta,
            ) -> Result<TimeDelta, Infallible> {
                Ok(two_step_offset(origin, $via, target, delta))
            }
        }
    };
}

fn multi_step_offset<
    T1: TimeScale + ToScale<T2>,
    T2: TimeScale + ToScale<T3> + Copy,
    T3: TimeScale + Copy,
>(
    origin: T1,
    via: T2,
    target: T3,
    dt: TimeDelta,
) -> TimeDelta {
    let mut offset = origin.offset(via, dt);
    offset += via.offset(target, dt + offset);
    offset
}

// TAI <-> TDB
two_step_impl!(Tai, Tt, Tdb);
two_step_impl!(Tdb, Tt, Tai);

impl ToScale<Tdb> for Tai {
    fn offset(&self, scale: Tdb, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

impl_fallible!(Tai, Tdb);

impl ToScale<Tai> for Tdb {
    fn offset(&self, scale: Tai, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

impl_fallible!(Tdb, Tai);

// TDB <-> TCG
two_step_impl!(Tdb, Tt, Tcg);
two_step_impl!(Tcg, Tt, Tdb);

impl ToScale<Tcg> for Tdb {
    fn offset(&self, scale: Tcg, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

impl_fallible!(Tdb, Tcg);

impl ToScale<Tdb> for Tcg {
    fn offset(&self, scale: Tdb, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

impl_fallible!(Tcg, Tdb);

// TAI <-> TCG
two_step_impl!(Tai, Tt, Tcg);
two_step_impl!(Tcg, Tt, Tai);

impl ToScale<Tcg> for Tai {
    fn offset(&self, scale: Tcg, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

impl_fallible!(Tai, Tcg);

impl ToScale<Tai> for Tcg {
    fn offset(&self, scale: Tai, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

impl_fallible!(Tcg, Tai);

// TAI <-> TCB
two_step_impl!(Tai, Tdb, Tcb);
two_step_impl!(Tcb, Tdb, Tai);

impl ToScale<Tcb> for Tai {
    fn offset(&self, scale: Tcb, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

impl_fallible!(Tai, Tcb);

impl ToScale<Tai> for Tcb {
    fn offset(&self, scale: Tai, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

impl_fallible!(Tcb, Tai);

// TT <-> TCB
two_step_impl!(Tt, Tdb, Tcb);
two_step_impl!(Tcb, Tdb, Tt);

impl ToScale<Tcb> for Tt {
    fn offset(&self, scale: Tcb, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

impl_fallible!(Tt, Tcb);

impl ToScale<Tt> for Tcb {
    fn offset(&self, scale: Tt, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

impl_fallible!(Tcb, Tt);

// TCB <-> TCG
two_step_impl!(Tcb, Tdb, Tcg);
two_step_impl!(Tcg, Tdb, Tcb);

impl ToScale<Tcg> for Tcb {
    fn offset(&self, scale: Tcg, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

impl_fallible!(Tcb, Tcg);

impl ToScale<Tcb> for Tcg {
    fn offset(&self, scale: Tcb, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

impl_fallible!(Tcg, Tcb);

// UT1

macro_rules! impl_ut1 {
    ($scale:ident) => {
        impl<P> TryToScale<Ut1, P> for $scale
        where
            P: DeltaUt1TaiProvider,
        {
            type Error = Ut1Error;

            fn try_offset(
                &self,
                _scale: Ut1,
                dt: TimeDelta,
                provider: Option<&P>,
            ) -> Result<TimeDelta, Self::Error> {
                let mut offset = $scale.offset(Tai, dt);
                offset += Tai.try_offset(Ut1, dt + offset, provider)?;
                Ok(offset)
            }
        }

        impl<P> TryToScale<$scale, P> for Ut1
        where
            P: DeltaUt1TaiProvider,
        {
            type Error = Ut1Error;

            fn try_offset(
                &self,
                scale: $scale,
                dt: TimeDelta,
                provider: Option<&P>,
            ) -> Result<TimeDelta, Self::Error> {
                let mut offset = Ut1.try_offset(Tai, dt, provider)?;
                offset += scale.offset_from(Tai, dt + offset);
                Ok(offset)
            }
        }
    };
}

impl_ut1!(Tcb);
impl_ut1!(Tcg);
impl_ut1!(Tdb);
impl_ut1!(Tt);

// DynTimeScale

#[derive(Debug, Error, Clone, Copy, Eq, PartialEq)]
#[error("an EOP provider is required for transformations from/to UT1")]
pub struct MissingEopProviderError;

impl<T: TimeScale> TryOffset<DynTimeScale, T, MissingEopProviderError> for DefaultOffsetProvider
where
    DefaultOffsetProvider: Offset<Tai, T>,
    DefaultOffsetProvider: Offset<Tcb, T>,
    DefaultOffsetProvider: Offset<Tcg, T>,
    DefaultOffsetProvider: Offset<Tdb, T>,
    DefaultOffsetProvider: Offset<Tt, T>,
{
    fn try_offset(
        &self,
        origin: DynTimeScale,
        target: T,
        delta: TimeDelta,
    ) -> Result<TimeDelta, MissingEopProviderError> {
        match origin {
            DynTimeScale::Tai => Ok(DefaultOffsetProvider.offset(Tai, target, delta)),
            DynTimeScale::Tcb => Ok(DefaultOffsetProvider.offset(Tcb, target, delta)),
            DynTimeScale::Tcg => Ok(DefaultOffsetProvider.offset(Tcg, target, delta)),
            DynTimeScale::Tdb => Ok(DefaultOffsetProvider.offset(Tdb, target, delta)),
            DynTimeScale::Tt => Ok(DefaultOffsetProvider.offset(Tt, target, delta)),
            DynTimeScale::Ut1 => Err(MissingEopProviderError),
        }
    }
}

impl<T: TimeScale> TryOffset<T, DynTimeScale, MissingEopProviderError> for DefaultOffsetProvider
where
    DefaultOffsetProvider: Offset<T, Tai>,
    DefaultOffsetProvider: Offset<T, Tcb>,
    DefaultOffsetProvider: Offset<T, Tcg>,
    DefaultOffsetProvider: Offset<T, Tdb>,
    DefaultOffsetProvider: Offset<T, Tt>,
{
    fn try_offset(
        &self,
        origin: T,
        target: DynTimeScale,
        delta: TimeDelta,
    ) -> Result<TimeDelta, MissingEopProviderError> {
        match target {
            DynTimeScale::Tai => Ok(DefaultOffsetProvider.offset(origin, Tai, delta)),
            DynTimeScale::Tcb => Ok(DefaultOffsetProvider.offset(origin, Tcb, delta)),
            DynTimeScale::Tcg => Ok(DefaultOffsetProvider.offset(origin, Tcg, delta)),
            DynTimeScale::Tdb => Ok(DefaultOffsetProvider.offset(origin, Tdb, delta)),
            DynTimeScale::Tt => Ok(DefaultOffsetProvider.offset(origin, Tt, delta)),
            DynTimeScale::Ut1 => Err(MissingEopProviderError),
        }
    }
}

impl TryOffset<DynTimeScale, DynTimeScale, MissingEopProviderError> for DefaultOffsetProvider {
    fn try_offset(
        &self,
        origin: DynTimeScale,
        target: DynTimeScale,
        delta: TimeDelta,
    ) -> Result<TimeDelta, MissingEopProviderError> {
        match (origin, target) {
            (DynTimeScale::Tai, DynTimeScale::Tai) => Ok(TimeDelta::default()),
            (DynTimeScale::Tai, DynTimeScale::Tcb) => Ok(self.offset(Tai, Tcb, delta)),
            (DynTimeScale::Tai, DynTimeScale::Tcg) => Ok(self.offset(Tai, Tcg, delta)),
            (DynTimeScale::Tai, DynTimeScale::Tdb) => todo!(),
            (DynTimeScale::Tai, DynTimeScale::Tt) => todo!(),
            (DynTimeScale::Tai, DynTimeScale::Ut1) => todo!(),
            (DynTimeScale::Tcb, DynTimeScale::Tai) => todo!(),
            (DynTimeScale::Tcb, DynTimeScale::Tcb) => Ok(TimeDelta::default()),
            (DynTimeScale::Tcb, DynTimeScale::Tcg) => todo!(),
            (DynTimeScale::Tcb, DynTimeScale::Tdb) => todo!(),
            (DynTimeScale::Tcb, DynTimeScale::Tt) => todo!(),
            (DynTimeScale::Tcb, DynTimeScale::Ut1) => todo!(),
            (DynTimeScale::Tcg, DynTimeScale::Tai) => todo!(),
            (DynTimeScale::Tcg, DynTimeScale::Tcb) => todo!(),
            (DynTimeScale::Tcg, DynTimeScale::Tcg) => Ok(TimeDelta::default()),
            (DynTimeScale::Tcg, DynTimeScale::Tdb) => todo!(),
            (DynTimeScale::Tcg, DynTimeScale::Tt) => todo!(),
            (DynTimeScale::Tcg, DynTimeScale::Ut1) => todo!(),
            (DynTimeScale::Tdb, DynTimeScale::Tai) => todo!(),
            (DynTimeScale::Tdb, DynTimeScale::Tcb) => todo!(),
            (DynTimeScale::Tdb, DynTimeScale::Tcg) => todo!(),
            (DynTimeScale::Tdb, DynTimeScale::Tdb) => Ok(TimeDelta::default()),
            (DynTimeScale::Tdb, DynTimeScale::Tt) => todo!(),
            (DynTimeScale::Tdb, DynTimeScale::Ut1) => todo!(),
            (DynTimeScale::Tt, DynTimeScale::Tai) => todo!(),
            (DynTimeScale::Tt, DynTimeScale::Tcb) => todo!(),
            (DynTimeScale::Tt, DynTimeScale::Tcg) => todo!(),
            (DynTimeScale::Tt, DynTimeScale::Tdb) => todo!(),
            (DynTimeScale::Tt, DynTimeScale::Tt) => Ok(TimeDelta::default()),
            (DynTimeScale::Tt, DynTimeScale::Ut1) => todo!(),
            (DynTimeScale::Ut1, DynTimeScale::Tai) => todo!(),
            (DynTimeScale::Ut1, DynTimeScale::Tcb) => todo!(),
            (DynTimeScale::Ut1, DynTimeScale::Tcg) => todo!(),
            (DynTimeScale::Ut1, DynTimeScale::Tdb) => todo!(),
            (DynTimeScale::Ut1, DynTimeScale::Tt) => todo!(),
            (DynTimeScale::Ut1, DynTimeScale::Ut1) => Ok(TimeDelta::default()),
        }
    }
}

impl TryOffset<DynTimeScale, DynTimeScale, ExtrapolatedDeltaUt1Tai> for DeltaUt1Tai {
    fn try_offset(
        &self,
        origin: DynTimeScale,
        target: DynTimeScale,
        delta: TimeDelta,
    ) -> Result<TimeDelta, ExtrapolatedDeltaUt1Tai>
    where
        DefaultOffsetProvider: TryOffset<DynTimeScale, DynTimeScale, MissingEopProviderError>,
    {
        todo!()
    }
}

impl<P> TryToScale<DynTimeScale, P> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        scale: DynTimeScale,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        match self {
            DynTimeScale::Tai => match scale {
                DynTimeScale::Tai => Ok(TimeDelta::default()),
                DynTimeScale::Tcb => Ok(Tai.offset(Tcb, dt)),
                DynTimeScale::Tcg => Ok(Tai.offset(Tcg, dt)),
                DynTimeScale::Tdb => Ok(Tai.offset(Tdb, dt)),
                DynTimeScale::Tt => Ok(Tai.offset(Tt, dt)),
                DynTimeScale::Ut1 => Tai.try_offset(Ut1, dt, provider),
            },
            DynTimeScale::Tcb => match scale {
                DynTimeScale::Tai => Ok(Tcb.offset(Tai, dt)),
                DynTimeScale::Tcb => Ok(TimeDelta::default()),
                DynTimeScale::Tcg => Ok(Tcb.offset(Tcg, dt)),
                DynTimeScale::Tdb => Ok(Tcb.offset(Tdb, dt)),
                DynTimeScale::Tt => Ok(Tcb.offset(Tt, dt)),
                DynTimeScale::Ut1 => Tcb.try_offset(Ut1, dt, provider),
            },
            DynTimeScale::Tcg => match scale {
                DynTimeScale::Tai => Ok(Tcg.offset(Tai, dt)),
                DynTimeScale::Tcb => Ok(Tcg.offset(Tcb, dt)),
                DynTimeScale::Tcg => Ok(TimeDelta::default()),
                DynTimeScale::Tdb => Ok(Tcg.offset(Tdb, dt)),
                DynTimeScale::Tt => Ok(Tcg.offset(Tt, dt)),
                DynTimeScale::Ut1 => Tcg.try_offset(Ut1, dt, provider),
            },
            DynTimeScale::Tdb => match scale {
                DynTimeScale::Tai => Ok(Tdb.offset(Tai, dt)),
                DynTimeScale::Tcb => Ok(Tdb.offset(Tcb, dt)),
                DynTimeScale::Tcg => Ok(Tdb.offset(Tcg, dt)),
                DynTimeScale::Tdb => Ok(TimeDelta::default()),
                DynTimeScale::Tt => Ok(Tdb.offset(Tt, dt)),
                DynTimeScale::Ut1 => Tdb.try_offset(Ut1, dt, provider),
            },
            DynTimeScale::Tt => match scale {
                DynTimeScale::Tai => Ok(Tt.offset(Tai, dt)),
                DynTimeScale::Tcb => Ok(Tt.offset(Tcb, dt)),
                DynTimeScale::Tcg => Ok(Tt.offset(Tcg, dt)),
                DynTimeScale::Tdb => Ok(Tt.offset(Tdb, dt)),
                DynTimeScale::Tt => Ok(TimeDelta::default()),
                DynTimeScale::Ut1 => Tt.try_offset(Ut1, dt, provider),
            },
            DynTimeScale::Ut1 => match scale {
                DynTimeScale::Tai => Ut1.try_offset(Tai, dt, provider),
                DynTimeScale::Tcb => Ut1.try_offset(Tcb, dt, provider),
                DynTimeScale::Tcg => Ut1.try_offset(Tcg, dt, provider),
                DynTimeScale::Tdb => Ut1.try_offset(Tdb, dt, provider),
                DynTimeScale::Tt => Ut1.try_offset(Tt, dt, provider),
                DynTimeScale::Ut1 => Ok(TimeDelta::default()),
            },
        }
    }
}

impl<P> TryToScale<Tai, P> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        _scale: Tai,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        self.try_offset(DynTimeScale::Tai, dt, provider)
    }
}

impl<P> TryToScale<DynTimeScale, P> for Tai
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        scale: DynTimeScale,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        DynTimeScale::Tai.try_offset(scale, dt, provider)
    }
}

impl<P> TryToScale<Tcb, P> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        _scale: Tcb,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        self.try_offset(DynTimeScale::Tcb, dt, provider)
    }
}

impl<P> TryToScale<DynTimeScale, P> for Tcb
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        scale: DynTimeScale,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        DynTimeScale::Tcb.try_offset(scale, dt, provider)
    }
}

impl<P> TryToScale<Tcg, P> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        _scale: Tcg,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        self.try_offset(DynTimeScale::Tcg, dt, provider)
    }
}

impl<P> TryToScale<DynTimeScale, P> for Tcg
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        scale: DynTimeScale,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        DynTimeScale::Tcg.try_offset(scale, dt, provider)
    }
}

impl<P> TryToScale<Tdb, P> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        _scale: Tdb,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        self.try_offset(DynTimeScale::Tdb, dt, provider)
    }
}

impl<P> TryToScale<DynTimeScale, P> for Tdb
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        scale: DynTimeScale,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        DynTimeScale::Tdb.try_offset(scale, dt, provider)
    }
}

impl<P> TryToScale<Tt, P> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        _scale: Tt,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        self.try_offset(DynTimeScale::Tt, dt, provider)
    }
}

impl<P> TryToScale<DynTimeScale, P> for Tt
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        scale: DynTimeScale,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        DynTimeScale::Tt.try_offset(scale, dt, provider)
    }
}

impl<P> TryToScale<Ut1, P> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        _scale: Ut1,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        self.try_offset(DynTimeScale::Ut1, dt, provider)
    }
}

impl<P> TryToScale<DynTimeScale, P> for Ut1
where
    P: DeltaUt1TaiProvider,
{
    type Error = Ut1Error;

    fn try_offset(
        &self,
        scale: DynTimeScale,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        DynTimeScale::Ut1.try_offset(scale, dt, provider)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::time_scales::{FromScale, ToScale};
    use crate::{
        DynTime, calendar_dates::Date, deltas::ToDelta, test_helpers::delta_ut1_tai,
        time_of_day::TimeOfDay,
    };
    use lox_math::is_close::IsClose;

    #[test]
    fn test_from_scale() {
        let dt = TimeDelta::default();
        assert_eq!(Tai.offset(Tt, dt), Tt.offset_from(Tai, dt))
    }

    const DEFAULT_TOL: f64 = 1e-7;
    const UT1_TOL: f64 = 1e-2;
    const TCB_TOL: f64 = 1e-5;

    // Reference values from Orekit
    //
    // Since we use different algorithms for TCB and UT1 we need to
    // adjust the tolerances accordingly.
    //
    #[rstest]
    #[case::tai_tai("TAI", "TAI", 0.0, None)]
    #[case::tai_tcb("TAI", "TCB", 55.66851419888016, Some(TCB_TOL))]
    #[case::tai_tcg("TAI", "TCG", 33.239589335894145, None)]
    #[case::tai_tdb("TAI", "TDB", 32.183882324981056, None)]
    #[case::tai_tt("TAI", "TT", 32.184, None)]
    #[case::tai_ut1("TAI", "UT1", -36.949521832072996, Some(UT1_TOL))]
    #[case::tcb_tai("TCB", "TAI", -55.668513317090046, Some(TCB_TOL))]
    #[case::tcb_tcb("TCB", "TCB", 0.0, Some(TCB_TOL))]
    #[case::tcb_tcg("TCB", "TCG", -22.4289240199929, Some(TCB_TOL))]
    #[case::tcb_tdb("TCB", "TDB", -23.484631010747805, Some(TCB_TOL))]
    #[case::tcb_tt("TCB", "TT", -23.484513317090048, Some(TCB_TOL))]
    #[case::tcb_ut1("TCB", "UT1", -92.61803559995818, Some(UT1_TOL))]
    #[case::tcg_tai("TCG", "TAI", -33.23958931272851, None)]
    #[case::tcg_tcb("TCG", "TCB", 22.428924359636042, Some(TCB_TOL))]
    #[case::tcg_tcg("TCG", "TCG", 0.0, None)]
    #[case::tcg_tdb("TCG", "TDB", -1.0557069988766656, None)]
    #[case::tcg_tt("TCG", "TT", -1.0555893127285145, None)]
    #[case::tcg_ut1("TCG", "UT1", -70.1891114139689, Some(UT1_TOL))]
    #[case::tdb_tai("TDB", "TAI", -32.18388231420531, None)]
    #[case::tdb_tcb("TDB", "TCB", 23.48463137488165, Some(TCB_TOL))]
    #[case::tdb_tcg("TDB", "TCG", 1.0557069992589518, None)]
    #[case::tdb_tdb("TDB", "TDB", 0.0, None)]
    #[case::tdb_tt("TDB", "TT", 1.176857946845189E-4, None)]
    #[case::tdb_ut1("TDB", "UT1", -69.13340440689674, Some(UT1_TOL))]
    #[case::tt_tai("TT", "TAI", -32.184, None)]
    #[case::tt_tcb("TT", "TCB", 23.484513689085105, Some(TCB_TOL))]
    #[case::tt_tcg("TT", "TCG", 1.055589313464182, None)]
    #[case::tt_tdb("TT", "TDB", -1.1768579472004603E-4, None)]
    #[case::tt_tt("TT", "TT", 0.0, None)]
    #[case::tt_ut1("TT", "UT1", -69.13352209269237, Some(UT1_TOL))]
    #[case::ut1_tai("UT1", "TAI", 36.949521532869305, Some(UT1_TOL))]
    #[case::ut1_tcb("UT1", "TCB", 92.61803631703046, Some(UT1_TOL))]
    #[case::ut1_tcg("UT1", "TCG", 70.18911089451464, Some(UT1_TOL))]
    #[case::ut1_tdb("UT1", "TDB", 69.13340387022173, Some(UT1_TOL))]
    #[case::ut1_tt("UT1", "TT", 69.13352153286931, Some(UT1_TOL))]
    #[case::ut1_ut1("UT1", "UT1", 0.0, Some(UT1_TOL))]
    fn test_dyn_time_scale_offsets(
        #[case] scale1: &str,
        #[case] scale2: &str,
        #[case] exp: f64,
        #[case] tol: Option<f64>,
    ) {
        use crate::time_scales::TryToScale;
        use lox_math::assert_close;

        let provider = Some(delta_ut1_tai());
        let scale1: DynTimeScale = scale1.parse().unwrap();
        let scale2: DynTimeScale = scale2.parse().unwrap();
        let date = Date::new(2024, 12, 30).unwrap();
        let time = TimeOfDay::from_hms(10, 27, 13.145).unwrap();
        let dt = DynTime::from_date_and_time(scale1, date, time)
            .unwrap()
            .to_delta();
        let act = scale1
            .try_offset(scale2, dt, provider)
            .unwrap()
            .to_decimal_seconds();
        assert_close!(act, exp, 1e-7, tol.unwrap_or(DEFAULT_TOL));
    }

    #[rstest]
    #[case::tai_tai("TAI", "TAI", 0.0, None)]
    #[case::tai_tcb("TAI", "TCB", 55.66851419888016, Some(TCB_TOL))]
    #[case::tai_tcg("TAI", "TCG", 33.239589335894145, None)]
    #[case::tai_tdb("TAI", "TDB", 32.183882324981056, None)]
    #[case::tai_tt("TAI", "TT", 32.184, None)]
    #[case::tai_ut1("TAI", "UT1", -36.949521832072996, Some(UT1_TOL))]
    #[case::tcb_tai("TCB", "TAI", -55.668513317090046, Some(TCB_TOL))]
    #[case::tcb_tcb("TCB", "TCB", 0.0, Some(TCB_TOL))]
    #[case::tcb_tcg("TCB", "TCG", -22.4289240199929, Some(TCB_TOL))]
    #[case::tcb_tdb("TCB", "TDB", -23.484631010747805, Some(TCB_TOL))]
    #[case::tcb_tt("TCB", "TT", -23.484513317090048, Some(TCB_TOL))]
    #[case::tcb_ut1("TCB", "UT1", -92.61803559995818, Some(UT1_TOL))]
    #[case::tcg_tai("TCG", "TAI", -33.23958931272851, None)]
    #[case::tcg_tcb("TCG", "TCB", 22.428924359636042, Some(TCB_TOL))]
    #[case::tcg_tcg("TCG", "TCG", 0.0, None)]
    #[case::tcg_tdb("TCG", "TDB", -1.0557069988766656, None)]
    #[case::tcg_tt("TCG", "TT", -1.0555893127285145, None)]
    #[case::tcg_ut1("TCG", "UT1", -70.1891114139689, Some(UT1_TOL))]
    #[case::tdb_tai("TDB", "TAI", -32.18388231420531, None)]
    #[case::tdb_tcb("TDB", "TCB", 23.48463137488165, Some(TCB_TOL))]
    #[case::tdb_tcg("TDB", "TCG", 1.0557069992589518, None)]
    #[case::tdb_tdb("TDB", "TDB", 0.0, None)]
    #[case::tdb_tt("TDB", "TT", 1.176857946845189E-4, None)]
    #[case::tdb_ut1("TDB", "UT1", -69.13340440689674, Some(UT1_TOL))]
    #[case::tt_tai("TT", "TAI", -32.184, None)]
    #[case::tt_tcb("TT", "TCB", 23.484513689085105, Some(TCB_TOL))]
    #[case::tt_tcg("TT", "TCG", 1.055589313464182, None)]
    #[case::tt_tdb("TT", "TDB", -1.1768579472004603E-4, None)]
    #[case::tt_tt("TT", "TT", 0.0, None)]
    #[case::tt_ut1("TT", "UT1", -69.13352209269237, Some(UT1_TOL))]
    #[case::ut1_tai("UT1", "TAI", 36.949521532869305, Some(UT1_TOL))]
    #[case::ut1_tcb("UT1", "TCB", 92.61803631703046, Some(UT1_TOL))]
    #[case::ut1_tcg("UT1", "TCG", 70.18911089451464, Some(UT1_TOL))]
    #[case::ut1_tdb("UT1", "TDB", 69.13340387022173, Some(UT1_TOL))]
    #[case::ut1_tt("UT1", "TT", 69.13352153286931, Some(UT1_TOL))]
    #[case::ut1_ut1("UT1", "UT1", 0.0, Some(UT1_TOL))]
    fn test_dyn_time_scale_offsets_new(
        #[case] scale1: &str,
        #[case] scale2: &str,
        #[case] exp: f64,
        #[case] tol: Option<f64>,
    ) {
        use lox_math::assert_close;

        let provider = delta_ut1_tai();
        let scale1: DynTimeScale = scale1.parse().unwrap();
        let scale2: DynTimeScale = scale2.parse().unwrap();
        let date = Date::new(2024, 12, 30).unwrap();
        let time = TimeOfDay::from_hms(10, 27, 13.145).unwrap();
        let dt = DynTime::from_date_and_time(scale1, date, time)
            .unwrap()
            .to_delta();
        let act = provider
            .try_offset(scale1, scale2, dt)
            .unwrap()
            .to_decimal_seconds();
        assert_close!(act, exp, 1e-7, tol.unwrap_or(DEFAULT_TOL));
    }
}
