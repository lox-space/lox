// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::convert::Infallible;

use crate::Time;
use crate::deltas::TimeDelta;
use crate::time_scales::{Tai, TimeScale};
use crate::utc::Utc;
use crate::utc::leap_seconds::{DefaultLeapSecondsProvider, LeapSecondsProvider};

mod impls;

pub trait TryOffset<Origin, Target>
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

pub trait Offset<Origin, Target>
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

pub trait OffsetProvider {
    type Error: std::error::Error + Send + Sync + 'static;

    fn tai_to_ut1(&self, delta: TimeDelta) -> Result<TimeDelta, Self::Error>;
    fn ut1_to_tai(&self, delta: TimeDelta) -> Result<TimeDelta, Self::Error>;

    fn tai_to_tt(&self) -> TimeDelta {
        D_TAI_TT
    }

    fn tt_to_tai(&self) -> TimeDelta {
        -D_TAI_TT
    }

    fn tt_to_tcg(&self, delta: TimeDelta) -> TimeDelta {
        tt_to_tcg(delta)
    }

    fn tcg_to_tt(&self, delta: TimeDelta) -> TimeDelta {
        tcg_to_tt(delta)
    }

    fn tdb_to_tcb(&self, delta: TimeDelta) -> TimeDelta {
        tdb_to_tcb(delta)
    }

    fn tcb_to_tdb(&self, delta: TimeDelta) -> TimeDelta {
        tcb_to_tdb(delta)
    }

    fn tt_to_tdb(&self, delta: TimeDelta) -> TimeDelta {
        tt_to_tdb(delta)
    }

    fn tdb_to_tt(&self, delta: TimeDelta) -> TimeDelta {
        tdb_to_tt(delta)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultOffsetProvider;

impl LeapSecondsProvider for DefaultOffsetProvider {}

impl OffsetProvider for DefaultOffsetProvider {
    type Error = Infallible;

    fn tai_to_ut1(&self, delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
        let tai = Time::from_delta(Tai, delta);
        Ok(DefaultLeapSecondsProvider
            .delta_tai_utc(tai)
            .unwrap_or_default())
    }

    fn ut1_to_tai(&self, delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
        Ok(Utc::from_delta(delta)
            .ok()
            .map(|utc| {
                DefaultLeapSecondsProvider
                    .delta_utc_tai(utc)
                    .unwrap_or_default()
            })
            .unwrap_or_default())
    }
}

// TAI <-> TT

/// The constant offset between TAI and TT.
pub const D_TAI_TT: TimeDelta = TimeDelta::builder().seconds(32).milliseconds(184).build();

// TT <-> TCG

/// The difference between J2000 TT and 1977 January 1.0 TAI as TT.
const J77_TT: f64 = -7.25803167816e8;

/// The rate of change of TCG with respect to TT.
const LG: f64 = 6.969290134e-10;

/// The rate of change of TT with respect to TCG.
const INV_LG: f64 = LG / (1.0 - LG);

fn tt_to_tcg(delta: TimeDelta) -> TimeDelta {
    let tt = delta.as_seconds_f64();
    TimeDelta::from_seconds_f64(INV_LG * (tt - J77_TT))
}

fn tcg_to_tt(delta: TimeDelta) -> TimeDelta {
    let tcg = delta.as_seconds_f64();
    TimeDelta::from_seconds_f64(-LG * (tcg - J77_TT))
}

// TDB <-> TCB

/// 1977 January 1 00:00, at which the following are equal:
/// * 1977-01-01T00:00:00.000 TAI
/// * 1977-01-01T00:00:32.184 TT
/// * 1977-01-01T00:00:32.184 TCG
/// * 1977-01-01T00:00:32.184 TCB
pub const J77: f64 = -725803200.0;

/// 1977 January 1.0 TAI
const TT_0: f64 = D_TAI_TT.as_seconds_f64() + J77;

/// The rate of change of TDB with respect to TCB.
const LB: f64 = 1.550519768e-8;

/// The rate of change of TCB with respect to TDB.
const INV_LB: f64 = LB / (1.0 - LB);

/// Constant term of TDB − TT formula of Fairhead & Bretagnon (1990).
const TDB_0: f64 = -6.55e-5;

const TCB_77: f64 = TDB_0 + LB * TT_0;

fn tdb_to_tcb(delta: TimeDelta) -> TimeDelta {
    let tdb = delta.as_seconds_f64();
    TimeDelta::from_seconds_f64(-TCB_77 / (1.0 - LB) + INV_LB * tdb)
}

fn tcb_to_tdb(delta: TimeDelta) -> TimeDelta {
    let tcb = delta.as_seconds_f64();
    TimeDelta::from_seconds_f64(TCB_77 - LB * tcb)
}

// TT <-> TDB

const K: f64 = 1.657e-3;
const EB: f64 = 1.671e-2;
const M_0: f64 = 6.239996;
const M_1: f64 = 1.99096871e-7;

fn tt_to_tdb(delta: TimeDelta) -> TimeDelta {
    let tt = delta.as_seconds_f64();
    let g = M_0 + M_1 * tt;
    TimeDelta::from_seconds_f64(K * (g + EB * g.sin()).sin())
}

fn tdb_to_tt(delta: TimeDelta) -> TimeDelta {
    let tdb = delta.as_seconds_f64();
    let mut offset = 0.0;
    for _ in 1..3 {
        let g = M_0 + M_1 * (tdb + offset);
        offset = -K * (g + EB * g.sin()).sin();
    }
    TimeDelta::from_seconds_f64(offset)
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
            .as_seconds_f64();
        assert_approx_eq!(act, exp, atol <= tol.unwrap_or(DEFAULT_TOL));
    }
}
