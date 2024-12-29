use std::convert::Infallible;

use thiserror::Error;

use crate::{
    constants::julian_dates::J77, deltas::TimeDelta, subsecond::Subsecond, ut1::DeltaUt1TaiProvider,
};

use super::{DynTimeScale, Tai, Tcb, Tcg, Tdb, TimeScale, Tt, Ut1};

pub trait TryToScale<T, P, E>
where
    T: TimeScale,
{
    fn try_offset(&self, scale: T, dt: TimeDelta, provider: Option<&P>) -> Result<TimeDelta, E>;
}

pub trait ToScale<T: TimeScale> {
    fn offset(&self, scale: T, dt: TimeDelta) -> TimeDelta;
}

impl<T> ToScale<T> for T
where
    T: TimeScale,
{
    fn offset(&self, _scale: T, _dt: TimeDelta) -> TimeDelta {
        TimeDelta::default()
    }
}

pub trait FromScale<T: TimeScale> {
    fn offset_from(&self, scale: T, dt: TimeDelta) -> TimeDelta;
}

impl<T, U> FromScale<U> for T
where
    T: TimeScale + Copy,
    U: TimeScale + ToScale<T>,
{
    fn offset_from(&self, scale: U, dt: TimeDelta) -> TimeDelta {
        scale.offset(*self, dt)
    }
}

impl<T, P, U> TryToScale<T, P, Infallible> for U
where
    T: TimeScale,
    U: ToScale<T>,
{
    fn try_offset(
        &self,
        scale: T,
        delta: TimeDelta,
        _provider: Option<&P>,
    ) -> Result<TimeDelta, Infallible> {
        Ok(self.offset(scale, delta))
    }
}

// TAI <-> TT

/// The constant offset between TAI and TT.
pub const D_TAI_TT: TimeDelta = TimeDelta {
    seconds: 32,
    subsecond: Subsecond(0.184),
};

impl ToScale<Tt> for Tai {
    fn offset(&self, _scale: Tt, _dt: TimeDelta) -> TimeDelta {
        D_TAI_TT
    }
}

impl ToScale<Tai> for Tt {
    fn offset(&self, _scale: Tai, _dt: TimeDelta) -> TimeDelta {
        -D_TAI_TT
    }
}

// TT <-> TCG

/// The difference between J2000 TT and 1977 January 1.0 TAI as TT.
const J77_TT: f64 = -7.25803167816e8;

/// The rate of change of TCG with respect to TT.
const LG: f64 = 6.969290134e-10;

/// The rate of change of TT with respect to TCG.
const INV_LG: f64 = LG / (1.0 - LG);

impl ToScale<Tcg> for Tt {
    fn offset(&self, _scale: Tcg, dt: TimeDelta) -> TimeDelta {
        let tt = dt.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(INV_LG * (tt - J77_TT))
    }
}

impl ToScale<Tt> for Tcg {
    fn offset(&self, _scale: Tt, dt: TimeDelta) -> TimeDelta {
        let tcg = dt.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(-LG * (tcg - J77_TT))
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

impl ToScale<Tcb> for Tdb {
    fn offset(&self, _scale: Tcb, dt: TimeDelta) -> TimeDelta {
        let tdb = dt.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(-TCB_77 / (1.0 - LB) + INV_LB * tdb)
    }
}

impl ToScale<Tdb> for Tcb {
    fn offset(&self, _scale: Tdb, dt: TimeDelta) -> TimeDelta {
        let tcb = dt.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(TCB_77 - LB * tcb)
    }
}

// TT <-> TDB

const K: f64 = 1.657e-3;
const EB: f64 = 1.671e-2;
const M_0: f64 = 6.239996;
const M_1: f64 = 1.99096871e-7;

impl ToScale<Tdb> for Tt {
    fn offset(&self, _scale: Tdb, dt: TimeDelta) -> TimeDelta {
        let tt = dt.to_decimal_seconds();
        let g = M_0 + M_1 * tt;
        TimeDelta::from_decimal_seconds(K * (g + EB * g.sin()).sin())
    }
}

impl ToScale<Tt> for Tdb {
    fn offset(&self, _scale: Tt, dt: TimeDelta) -> TimeDelta {
        let tdb = dt.to_decimal_seconds();
        let mut tt = tdb;
        for _ in 1..3 {
            let g = M_0 + M_1 * tt;
            tt = tdb - K * (g + EB * g.sin()).sin();
        }
        TimeDelta::from_decimal_seconds(tt)
    }
}

// TAI <-> UT1

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum Ut1Error<T> {
    #[error("a UT1-TAI provider is required but was not provided")]
    MissingProvider,
    #[error(transparent)]
    ProviderError(#[from] T),
}

impl<P> TryToScale<Ut1, P, Ut1Error<P::Error>> for Tai
where
    P: DeltaUt1TaiProvider,
{
    fn try_offset(
        &self,
        _scale: Ut1,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Ut1Error<P::Error>> {
        Ok(provider
            .ok_or(Ut1Error::MissingProvider)?
            .delta_ut1_tai_dt(dt)?)
    }
}

impl<P> TryToScale<Tai, P, Ut1Error<P::Error>> for Ut1
where
    P: DeltaUt1TaiProvider,
{
    fn try_offset(
        &self,
        _scale: Tai,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Ut1Error<P::Error>> {
        Ok(provider
            .ok_or(Ut1Error::MissingProvider)?
            .delta_tai_ut1_dt(dt)?)
    }
}

// Multi-step transformations

#[inline]
fn multi_step_offset<
    T1: TimeScale + ToScale<T2>,
    T2: TimeScale + ToScale<T3> + Copy,
    T3: TimeScale,
>(
    origin: T1,
    via: T2,
    target: T3,
    dt: TimeDelta,
) -> TimeDelta {
    let mut dt = dt;
    dt += origin.offset(via, dt);
    dt += via.offset(target, dt);
    dt
}

// TAI <-> TDB

impl ToScale<Tdb> for Tai {
    fn offset(&self, scale: Tdb, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

impl ToScale<Tai> for Tdb {
    fn offset(&self, scale: Tai, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

// TDB <-> TCG

impl ToScale<Tcg> for Tdb {
    fn offset(&self, scale: Tcg, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

impl ToScale<Tdb> for Tcg {
    fn offset(&self, scale: Tdb, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

// TAI <-> TCG

impl ToScale<Tcg> for Tai {
    fn offset(&self, scale: Tcg, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

impl ToScale<Tai> for Tcg {
    fn offset(&self, scale: Tai, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tt, scale, dt)
    }
}

// TAI <-> TCB

impl ToScale<Tcb> for Tai {
    fn offset(&self, scale: Tcb, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

impl ToScale<Tai> for Tcb {
    fn offset(&self, scale: Tai, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

// TT <-> TCB

impl ToScale<Tcb> for Tt {
    fn offset(&self, scale: Tcb, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

impl ToScale<Tt> for Tcb {
    fn offset(&self, scale: Tt, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

// TCB <-> TCG

impl ToScale<Tcg> for Tcb {
    fn offset(&self, scale: Tcg, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

impl ToScale<Tcb> for Tcg {
    fn offset(&self, scale: Tcb, dt: TimeDelta) -> TimeDelta {
        multi_step_offset(*self, Tdb, scale, dt)
    }
}

// UT1

macro_rules! impl_ut1 {
    ($scale:ident) => {
        impl<P> TryToScale<Ut1, P, Ut1Error<P::Error>> for $scale
        where
            P: DeltaUt1TaiProvider,
        {
            fn try_offset(
                &self,
                _scale: Ut1,
                dt: TimeDelta,
                provider: Option<&P>,
            ) -> Result<TimeDelta, Ut1Error<P::Error>> {
                let mut dt = dt;
                dt += $scale.offset(Tai, dt);
                dt += Tai.try_offset(Ut1, dt, provider)?;
                Ok(dt)
            }
        }

        impl<P> TryToScale<$scale, P, Ut1Error<P::Error>> for Ut1
        where
            P: DeltaUt1TaiProvider,
        {
            fn try_offset(
                &self,
                scale: $scale,
                dt: TimeDelta,
                provider: Option<&P>,
            ) -> Result<TimeDelta, Ut1Error<P::Error>> {
                let mut dt = dt;
                dt += Ut1.try_offset(Tai, dt, provider)?;
                dt += scale.offset_from(Tai, dt);
                Ok(dt)
            }
        }
    };
}

impl_ut1!(Tcb);
impl_ut1!(Tcg);
impl_ut1!(Tdb);
impl_ut1!(Tt);

// DynTimeScale

impl<P> TryToScale<DynTimeScale, P, Ut1Error<P::Error>> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    fn try_offset(
        &self,
        scale: DynTimeScale,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Ut1Error<P::Error>> {
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
                DynTimeScale::Tt => Ok(Tt.offset(Tai, dt)),
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

impl<P> TryToScale<Tai, P, Ut1Error<P::Error>> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    fn try_offset(
        &self,
        _scale: Tai,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Ut1Error<P::Error>> {
        self.try_offset(DynTimeScale::Tai, dt, provider)
    }
}

impl<P> TryToScale<Tcb, P, Ut1Error<P::Error>> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    fn try_offset(
        &self,
        _scale: Tcb,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Ut1Error<P::Error>> {
        self.try_offset(DynTimeScale::Tcb, dt, provider)
    }
}

impl<P> TryToScale<Tcg, P, Ut1Error<P::Error>> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    fn try_offset(
        &self,
        _scale: Tcg,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Ut1Error<P::Error>> {
        self.try_offset(DynTimeScale::Tcg, dt, provider)
    }
}

impl<P> TryToScale<Tdb, P, Ut1Error<P::Error>> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    fn try_offset(
        &self,
        _scale: Tdb,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Ut1Error<P::Error>> {
        self.try_offset(DynTimeScale::Tdb, dt, provider)
    }
}

impl<P> TryToScale<Tt, P, Ut1Error<P::Error>> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    fn try_offset(
        &self,
        _scale: Tt,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Ut1Error<P::Error>> {
        self.try_offset(DynTimeScale::Tt, dt, provider)
    }
}

impl<P> TryToScale<Ut1, P, Ut1Error<P::Error>> for DynTimeScale
where
    P: DeltaUt1TaiProvider,
{
    fn try_offset(
        &self,
        _scale: Ut1,
        dt: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Ut1Error<P::Error>> {
        self.try_offset(DynTimeScale::Ut1, dt, provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_scale() {
        let dt = TimeDelta::default();
        assert_eq!(Tai.offset(Tt, dt), Tt.offset_from(Tai, dt))
    }
}
