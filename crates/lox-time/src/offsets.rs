// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::convert::Infallible;

use crate::time_scales::TimeScale;
use crate::{deltas::TimeDelta, julian_dates::J77, subsecond::Subsecond};
use lox_derive::OffsetProvider;
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

#[derive(Debug, Error, Default)]
#[error("an EOP provider is required for transformations from/to UT1")]
pub struct MissingEopProviderError;

// FIXME: Remove once `!` lands on stable.
impl From<Infallible> for MissingEopProviderError {
    fn from(_: Infallible) -> Self {
        MissingEopProviderError
    }
}

#[derive(Debug, Clone, Copy, OffsetProvider)]
pub struct DefaultOffsetProvider;

// TAI <-> TT

/// The constant offset between TAI and TT.
pub const D_TAI_TT: TimeDelta = TimeDelta {
    seconds: 32,
    subsecond: Subsecond(0.184),
};

// TT <-> TCG

/// The difference between J2000 TT and 1977 January 1.0 TAI as TT.
const J77_TT: f64 = -7.25803167816e8;

/// The rate of change of TCG with respect to TT.
const LG: f64 = 6.969290134e-10;

/// The rate of change of TT with respect to TCG.
const INV_LG: f64 = LG / (1.0 - LG);

pub fn tt_to_tcg(delta: TimeDelta) -> TimeDelta {
    let tt = delta.to_decimal_seconds();
    TimeDelta::from_decimal_seconds(INV_LG * (tt - J77_TT))
}

pub fn tcg_to_tt(delta: TimeDelta) -> TimeDelta {
    let tcg = delta.to_decimal_seconds();
    TimeDelta::from_decimal_seconds(-LG * (tcg - J77_TT))
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

pub fn tdb_to_tcb(delta: TimeDelta) -> TimeDelta {
    let tdb = delta.to_decimal_seconds();
    TimeDelta::from_decimal_seconds(-TCB_77 / (1.0 - LB) + INV_LB * tdb)
}

pub fn tcb_to_tdb(delta: TimeDelta) -> TimeDelta {
    let tcb = delta.to_decimal_seconds();
    TimeDelta::from_decimal_seconds(TCB_77 - LB * tcb)
}

// TT <-> TDB

const K: f64 = 1.657e-3;
const EB: f64 = 1.671e-2;
const M_0: f64 = 6.239996;
const M_1: f64 = 1.99096871e-7;

pub fn tt_to_tdb(delta: TimeDelta) -> TimeDelta {
    let tt = delta.to_decimal_seconds();
    let g = M_0 + M_1 * tt;
    TimeDelta::from_decimal_seconds(K * (g + EB * g.sin()).sin())
}

pub fn tdb_to_tt(delta: TimeDelta) -> TimeDelta {
    let tdb = delta.to_decimal_seconds();
    let mut offset = 0.0;
    for _ in 1..3 {
        let g = M_0 + M_1 * (tdb + offset);
        offset = -K * (g + EB * g.sin()).sin();
    }
    TimeDelta::from_decimal_seconds(offset)
}

// Two-step transformations

pub fn two_step_offset<P, T1, T2, T3>(
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

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;
    use rstest::rstest;

    use super::*;
    use crate::offsets::TryOffset;
    use crate::time_scales::DynTimeScale;
    use crate::{DynTime, calendar_dates::Date, deltas::ToDelta, time_of_day::TimeOfDay};

    const DEFAULT_TOL: f64 = 1e-7;
    const TCB_TOL: f64 = 1e-4;

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
    #[case::tcb_tai("TCB", "TAI", -55.668513317090046, Some(TCB_TOL))]
    #[case::tcb_tcb("TCB", "TCB", 0.0, Some(TCB_TOL))]
    #[case::tcb_tcg("TCB", "TCG", -22.4289240199929, Some(TCB_TOL))]
    #[case::tcb_tdb("TCB", "TDB", -23.484631010747805, Some(TCB_TOL))]
    #[case::tcb_tt("TCB", "TT", -23.484513317090048, Some(TCB_TOL))]
    #[case::tcg_tai("TCG", "TAI", -33.23958931272851, None)]
    #[case::tcg_tcb("TCG", "TCB", 22.428924359636042, Some(TCB_TOL))]
    #[case::tcg_tcg("TCG", "TCG", 0.0, None)]
    #[case::tcg_tdb("TCG", "TDB", -1.0557069988766656, None)]
    #[case::tcg_tt("TCG", "TT", -1.0555893127285145, None)]
    #[case::tdb_tai("TDB", "TAI", -32.18388231420531, None)]
    #[case::tdb_tcb("TDB", "TCB", 23.48463137488165, Some(TCB_TOL))]
    #[case::tdb_tcg("TDB", "TCG", 1.0557069992589518, None)]
    #[case::tdb_tdb("TDB", "TDB", 0.0, None)]
    #[case::tdb_tt("TDB", "TT", 1.176857946845189E-4, None)]
    #[case::tt_tai("TT", "TAI", -32.184, None)]
    #[case::tt_tcb("TT", "TCB", 23.484513689085105, Some(TCB_TOL))]
    #[case::tt_tcg("TT", "TCG", 1.055589313464182, None)]
    #[case::tt_tdb("TT", "TDB", -1.1768579472004603E-4, None)]
    #[case::tt_tt("TT", "TT", 0.0, None)]
    fn test_dyn_time_scale_offsets_new(
        #[case] scale1: &str,
        #[case] scale2: &str,
        #[case] exp: f64,
        #[case] tol: Option<f64>,
    ) {
        let provider = &DefaultOffsetProvider;
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
        assert_approx_eq!(act, exp, atol <= tol.unwrap_or(DEFAULT_TOL));
    }
}
