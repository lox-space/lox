/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module transform provides a trait for transforming between pairs of timescales, together
//! with a default implementation for the most commonly used time scale pairs.

use crate::constants::julian_dates::J77;
use crate::deltas::{TimeDelta, ToDelta};
use crate::subsecond::Subsecond;
use crate::time_scales::{Tai, Tcb, Tcg, Tdb, TimeScale, Tt};
use crate::utc::Utc;
use crate::Time;

pub trait ToScale<T: TimeScale + Copy>: ToDelta {
    fn offset(&self, scale: T) -> TimeDelta;

    fn to_scale(&self, scale: T) -> Time<T> {
        let delta_from_epoch = self.to_delta();
        Time::from_delta(scale, delta_from_epoch + self.offset(scale))
    }
}

pub trait ToTai: ToScale<Tai> {
    fn to_tai(&self) -> Time<Tai> {
        self.to_scale(Tai)
    }
}

pub trait ToTt: ToScale<Tt> {
    fn to_tt(&self) -> Time<Tt> {
        self.to_scale(Tt)
    }
}

pub trait ToTcg: ToScale<Tcg> {
    fn to_tcg(&self) -> Time<Tcg> {
        self.to_scale(Tcg)
    }
}

pub trait ToTcb: ToScale<Tcb> {
    fn to_tcb(&self) -> Time<Tcb> {
        self.to_scale(Tcb)
    }
}

pub trait ToTdb: ToScale<Tdb> {
    fn to_tdb(&self) -> Time<Tdb> {
        self.to_scale(Tdb)
    }
}

// TAI <-> TT

/// The constant offset between TAI and TT.
pub const D_TAI_TT: TimeDelta = TimeDelta {
    seconds: 32,
    subsecond: Subsecond(0.184),
};

impl ToScale<Tt> for Time<Tai> {
    fn offset(&self, _scale: Tt) -> TimeDelta {
        D_TAI_TT
    }
}

impl ToTt for Time<Tai> {}

impl ToScale<Tai> for Time<Tt> {
    fn offset(&self, _scale: Tai) -> TimeDelta {
        -D_TAI_TT
    }
}

impl ToTai for Time<Tt> {}

// TT <-> TCG

/// The difference between J2000 TT and 1977 January 1.0 TAI as TT.
const J77_TT: f64 = -7.25803167816e8;

/// The rate of change of TCG with respect to TT.
const LG: f64 = 6.969290134e-10;

/// The rate of change of TT with respect to TCG.
const INV_LG: f64 = LG / (1.0 - LG);

impl ToScale<Tcg> for Time<Tt> {
    fn offset(&self, _scale: Tcg) -> TimeDelta {
        let time = self.to_delta().to_decimal_seconds();
        let raw_delta = INV_LG * (time - J77_TT);
        TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TT to TCG offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err
            );
        })
    }
}

impl ToTcg for Time<Tt> {}

impl ToScale<Tt> for Time<Tcg> {
    fn offset(&self, _scale: Tt) -> TimeDelta {
        let time = self.to_delta().to_decimal_seconds();
        let raw_delta = -LG * (time - J77_TT);
        TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TCG to TT offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err
            );
        })
    }
}

impl ToTt for Time<Tcg> {}

// TDB <-> TCB

/// 1977 January 1.0 TAI
const TT_0: f64 = J77.seconds as f64 + D_TAI_TT.seconds as f64 + D_TAI_TT.subsecond.0;

/// The rate of change of TDB with respect to TCB.
const LB: f64 = 1.550519768e-8;

/// The rate of change of TCB with respect to TDB.
const INV_LB: f64 = LB / (1.0 - LB);

/// Constant term of TDB − TT formula of Fairhead & Bretagnon (1990).
const TDB_00: f64 = -6.55e-5;

const TCB_77: f64 = TDB_00 + LB * TT_0;

impl ToScale<Tcb> for Time<Tdb> {
    fn offset(&self, _scale: Tcb) -> TimeDelta {
        let dt = self.to_delta().to_decimal_seconds();
        TimeDelta::from_decimal_seconds(-TCB_77 / (1.0 - LB) + INV_LB * dt).unwrap()
    }
}

impl ToTcb for Time<Tdb> {}

impl ToScale<Tdb> for Time<Tcb> {
    fn offset(&self, _scale: Tdb) -> TimeDelta {
        let dt = self.to_delta().to_decimal_seconds();
        TimeDelta::from_decimal_seconds(TCB_77 - LB * dt).unwrap()
    }
}

impl ToTdb for Time<Tcb> {}

// TT <-> TDB

const K: f64 = 1.657e-3;
const EB: f64 = 1.671e-2;
const M_0: f64 = 6.239996;
const M_1: f64 = 1.99096871e-7;

impl ToScale<Tdb> for Time<Tt> {
    fn offset(&self, _scale: Tdb) -> TimeDelta {
        let tt = self.to_delta().to_decimal_seconds();
        let g = M_0 + M_1 * tt;
        let raw_delta = K * (g + EB * g.sin()).sin();
        TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TT to TDB offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err,
            )
        })
    }
}

impl ToTdb for Time<Tt> {}

impl ToScale<Tt> for Time<Tdb> {
    fn offset(&self, _scale: Tt) -> TimeDelta {
        let tdb = self.to_delta().to_decimal_seconds();
        let mut tt = tdb;
        let mut raw_delta = 0.0;
        for _ in 1..3 {
            let g = M_0 + M_1 * tt;
            raw_delta = -K * (g + EB * g.sin()).sin();
            tt = tdb + raw_delta;
        }

        TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TDB to TT offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err,
            )
        })
    }
}

impl ToTt for Time<Tdb> {}

// Multi-step transformations

// Concrete implementation to avoid conflicting implementation errors
impl ToScale<Tdb> for Time<Tai> {
    fn offset(&self, _scale: Tdb) -> TimeDelta {
        let tdb = ToScale::<Tdb>::to_scale(self, Tdb);
        tdb.to_delta() - self.to_delta()
    }

    fn to_scale(&self, _scale: Tdb) -> Time<Tdb> {
        self.to_tt().to_tdb()
    }
}

impl ToTdb for Time<Tai> {}

// Concrete implementation to avoid conflicting implementation errors
impl ToScale<Tdb> for Time<Tcg> {
    fn offset(&self, _scale: Tdb) -> TimeDelta {
        let tdb = ToScale::<Tdb>::to_scale(self, Tdb);
        tdb.to_delta() - self.to_delta()
    }

    fn to_scale(&self, _scale: Tdb) -> Time<Tdb> {
        self.to_tt().to_tdb()
    }
}

impl ToTdb for Time<Tcg> {}

impl<U: ToTt + ToDelta> ToScale<Tai> for U {
    fn offset(&self, _scale: Tai) -> TimeDelta {
        let tdb = ToScale::<Tai>::to_scale(self, Tai);
        tdb.to_delta() - self.to_delta()
    }

    fn to_scale(&self, _scale: Tai) -> Time<Tai> {
        self.to_tt().to_tai()
    }
}

impl ToTai for Time<Tcg> {}
impl ToTai for Time<Tdb> {}

impl<U: ToTt + ToDelta> ToScale<Tcg> for U {
    fn offset(&self, _scale: Tcg) -> TimeDelta {
        let tdb = ToScale::<Tcg>::to_scale(self, Tcg);
        tdb.to_delta() - self.to_delta()
    }

    fn to_scale(&self, _scale: Tcg) -> Time<Tcg> {
        self.to_tt().to_tcg()
    }
}

impl ToTcg for Time<Tai> {}
impl ToTcg for Time<Tdb> {}

impl<U: ToTdb + ToDelta> ToScale<Tcb> for U {
    fn offset(&self, _scale: Tcb) -> TimeDelta {
        let tdb = ToScale::<Tcb>::to_scale(self, Tcb);
        tdb.to_delta() - self.to_delta()
    }

    fn to_scale(&self, _scale: Tcb) -> Time<Tcb> {
        self.to_tdb().to_tcb()
    }
}

impl ToTcb for Time<Tai> {}
impl ToTcb for Time<Tcg> {}
impl ToTcb for Time<Tt> {}

// Concrete implementation to avoid conflicting implementation errors
impl ToScale<Tt> for Time<Tcb> {
    fn offset(&self, _scale: Tt) -> TimeDelta {
        let tt = self.to_scale(Tt);
        tt.to_delta() - self.to_delta()
    }

    fn to_scale(&self, _scale: Tt) -> Time<Tt> {
        self.to_tdb().to_tt()
    }
}

impl ToTai for Time<Tcb> {}
impl ToTt for Time<Tcb> {}
impl ToTcg for Time<Tcb> {}

pub trait LeapSecondsProvider {
    fn delta_tai_utc(&self, tai: Time<Tai>) -> Option<TimeDelta>;

    fn delta_utc_tai(&self, utc: Utc) -> Option<TimeDelta>;
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use rstest::rstest;

    use crate::constants::julian_dates::{J0, SECONDS_BETWEEN_JD_AND_J2000};
    use crate::subsecond::Subsecond;

    use super::*;

    // Transformations are tested for agreement with both ERFA and AstroTime.jl.

    const PANIC_INDUCING_DELTA: TimeDelta = TimeDelta {
        seconds: 0,
        subsecond: Subsecond(f64::NAN),
    };

    #[test]
    fn test_transform_all() {
        let tai_exp = Time::new(Tai, 0, Subsecond::default());
        let tt_exp = tai_exp.to_tt();
        let tcg_exp = tt_exp.to_tcg();
        let tdb_exp = tt_exp.to_tdb();
        let tcb_exp = tdb_exp.to_tcb();

        let tt_act = tai_exp.to_tt();
        let tcg_act = tai_exp.to_tcg();
        let tdb_act = tai_exp.to_tdb();
        let tcb_act = tai_exp.to_tcb();

        assert_eq!(tt_exp, tt_act);
        assert_eq!(tcg_exp, tcg_act);
        assert_eq!(tdb_exp, tdb_act);
        assert_eq!(tcb_exp, tcb_act);

        let tai_act = tt_exp.to_tai();
        let tcg_act = tt_exp.to_tcg();
        let tdb_act = tt_exp.to_tdb();
        let tcb_act = tt_exp.to_tcb();

        assert_eq!(tai_exp, tai_act);
        assert_eq!(tcg_exp, tcg_act);
        assert_eq!(tdb_exp, tdb_act);
        assert_eq!(tcb_exp, tcb_act);

        let tai_act = tcg_exp.to_tai();
        let tt_act = tcg_exp.to_tt();
        let tdb_act = tcg_exp.to_tdb();
        let tcb_act = tcg_exp.to_tcb();

        assert_eq!(tai_exp, tai_act);
        assert_eq!(tt_exp, tt_act);
        assert_eq!(tdb_exp, tdb_act);
        assert_eq!(tcb_exp, tcb_act);

        let tai_act = tdb_exp.to_tai();
        let tt_act = tdb_exp.to_tt();
        let tcg_act = tdb_exp.to_tcg();
        let tcb_act = tdb_exp.to_tcb();

        assert_eq!(tai_exp, tai_act);
        assert_eq!(tt_exp, tt_act);
        assert_eq!(tcg_exp, tcg_act);
        assert_eq!(tcb_exp, tcb_act);

        let tai_act = tcb_exp.to_tai();
        let tt_act = tcb_exp.to_tt();
        let tcg_act = tcb_exp.to_tcg();
        let tdb_act = tcb_exp.to_tdb();

        assert_eq!(tai_exp, tai_act);
        assert_eq!(tt_exp, tt_act);
        assert_eq!(tcg_exp, tcg_act);
        assert_eq!(tdb_exp, tdb_act);
    }

    #[test]
    fn test_transform_tai_tt() {
        let tai = Time::new(Tai, 0, Subsecond::default());
        let tt = tai.to_tt();
        let expected = Time::new(Tt, 32, Subsecond(0.184));
        assert_eq!(expected, tt);
    }

    #[test]
    fn test_transform_tt_tai() {
        let tt = Time::new(Tt, 32, Subsecond(0.184));
        let tai = tt.to_tai();
        let expected = Time::new(Tai, 0, Subsecond::default());
        assert_eq!(expected, tai);
    }

    #[rstest]
    #[case::j0(
        Time::from_delta(Tt, J0),
        Time::from_delta(Tcg, TimeDelta::new(-211813488148, Subsecond(0.886_867_966_488_467)))
    )]
    #[case::j2000(
        Time::new(Tt, 0, Subsecond::default()),
        Time::new(Tcg, 0, Subsecond(0.505_833_286_021_129))
    )]
    #[should_panic]
    #[case::unrepresentable(Time::from_delta(Tt, PANIC_INDUCING_DELTA), Time::default())]
    fn test_transform_tt_tcg(#[case] tt: Time<Tt>, #[case] expected: Time<Tcg>) {
        let tcg = tt.to_tcg();
        assert_eq!(expected, tcg);
    }

    #[rstest]
    #[case::j0(
        Time::from_delta(Tcg, J0),
        Time::from_delta(Tt, TimeDelta::new(-211813487853, Subsecond(0.113_131_930_984_139)))
    )]
    #[case::j2000(Time::new(Tcg, 0, Subsecond::default()), Time::new(Tt, -1, Subsecond(0.494_166_714_331_400)))]
    #[should_panic]
    #[case::unrepresentable(Time::from_delta(Tcg, PANIC_INDUCING_DELTA), Time::default())]
    fn test_transform_tcg_tt(#[case] tcg: Time<Tcg>, #[case] expected: Time<Tt>) {
        let tt = tcg.to_tt();
        assert_eq!(expected.seconds(), tt.seconds());
        assert_float_eq!(expected.subsecond(), tt.subsecond(), abs <= 1e-12);
    }

    #[rstest]
    #[case::j0(
        Time::from_delta(Tcb, J0),
        Time::from_delta(Tdb, TimeDelta::new(-SECONDS_BETWEEN_JD_AND_J2000 + 3272, Subsecond(0.956_215_636_550_950)))
    )]
    #[case::j2000(Time::j2000(Tcb), Time::new(Tdb, -12, Subsecond(0.746_212_906_242_706)))]
    fn test_transform_tcb_tdb(#[case] tcb: Time<Tcb>, #[case] expected: Time<Tdb>) {
        let tdb = tcb.to_tdb();
        assert_eq!(expected.seconds(), tdb.seconds());
        // Lox and ERFA agree to the picosecond. However, the paper from which these formulae derive
        // (Fairhead & Bretagnon, 1990) provide coefficients for transformations with only
        // nanosecond accuracy. Chasing greater accuracy may not be practical or useful.
        assert_float_eq!(expected.subsecond(), tdb.subsecond(), abs <= 1e-15);
    }

    #[rstest]
    #[case::j0(
        Time::from_delta(Tdb, J0),
        Time::from_delta(Tcb, TimeDelta::new(-SECONDS_BETWEEN_JD_AND_J2000 - 3273, Subsecond(0.043_733_615_615_110)))
    )]
    #[case::j2000(Time::j2000(Tdb), Time::new(Tcb, 11, Subsecond(0.253_787_268_249_489)))]
    fn test_transform_tdb_tcb(#[case] tdb: Time<Tdb>, #[case] expected: Time<Tcb>) {
        let tcb = tdb.to_tcb();
        assert_eq!(expected.seconds(), tcb.seconds());
        assert_float_eq!(expected.subsecond(), tcb.subsecond(), abs <= 1e-12);
    }

    #[rstest]
    #[case::j0(Time::from_delta(Tt, J0), Time::from_delta(Tdb, TimeDelta::new(-SECONDS_BETWEEN_JD_AND_J2000, Subsecond(0.001_600_955_458_249))))]
    #[case::j2000(Time::j2000(Tt), Time::from_delta(Tdb, TimeDelta::new(-1, Subsecond(0.999_927_263_223_809))))]
    #[should_panic]
    #[case::unrepresentable(Time::from_delta(Tt, PANIC_INDUCING_DELTA), Time::default())]
    fn test_transform_tt_tdb(#[case] tt: Time<Tt>, #[case] expected: Time<Tdb>) {
        let tdb = tt.to_tdb();
        assert_eq!(expected, tdb);
    }

    #[rstest]
    #[case::j0(Time::from_delta(Tdb, J0), Time::from_delta(Tt, TimeDelta::new(-SECONDS_BETWEEN_JD_AND_J2000 - 1, Subsecond(0.998_399_044_541_884))))]
    #[case::j2000(
        Time::j2000(Tdb),
        Time::from_delta(Tt, TimeDelta::new(0, Subsecond(0.000_072_736_776_166)))
    )]
    #[should_panic]
    #[case::unrepresentable(Time::from_delta(Tdb, PANIC_INDUCING_DELTA), Time::default())]
    fn test_transform_tdb_tt(#[case] tdb: Time<Tdb>, #[case] expected: Time<Tt>) {
        let tt = tdb.to_tt();
        assert_eq!(expected, tt);
    }
}
