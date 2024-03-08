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
use mockall::automock;

use crate::continuous::{BaseTime, Time, TimeDelta, TimeScale, TAI, TCB, TCG, TDB, TT};
use crate::Subsecond;

/// TransformTimeScale transforms a [Time] in [TimeScale] `T` to the corresponding [Time] in
/// [TimeScale] `U`.
#[automock]
pub trait TransformTimeScale<T, U>
where
    T: TimeScale + Copy,
    U: TimeScale + Copy,
{
    fn transform(&self, time: Time<T>) -> Time<U>;
}

/// TimeScaleTransformer provides default implementations TransformTimeScale for all commonly used
/// time scale pairs.
///
/// Users with custom time scales, pairings, data sources, or who require specific transformation
/// algorithms should implement `TransformTimeScale` for their particular use case.
pub struct TimeScaleTransformer {}

/// The constant offset between TAI and TT.
pub const D_TAI_TT: TimeDelta = TimeDelta {
    seconds: 32,
    subsecond: Subsecond(0.184),
};

impl TransformTimeScale<TAI, TT> for &TimeScaleTransformer {
    fn transform(&self, time: Time<TAI>) -> Time<TT> {
        let base_time = time.base_time() + D_TAI_TT;
        Time::from_base_time(TT, base_time)
    }
}

impl TransformTimeScale<TT, TAI> for &TimeScaleTransformer {
    fn transform(&self, time: Time<TT>) -> Time<TAI> {
        let base_time = time.base_time() - D_TAI_TT;
        Time::from_base_time(TAI, base_time)
    }
}

/// The difference between J2000 TT and 1977 January 1.0 TAI as TT.
const J77_TT: f64 = -7.25803167816e8;

/// The rate of change of TCG with respect to TT.
const LG: f64 = 6.969290134e-10;

/// The rate of change of TT with respect to TCG.
const INV_LG: f64 = LG / (1.0 - LG);

impl TransformTimeScale<TT, TCG> for &TimeScaleTransformer {
    fn transform(&self, time: Time<TT>) -> Time<TCG> {
        Time::from_base_time(TCG, time.base_time() + delta_tt_tcg(time))
    }
}

impl TransformTimeScale<TCG, TT> for &TimeScaleTransformer {
    fn transform(&self, time: Time<TCG>) -> Time<TT> {
        Time::from_base_time(TT, time.base_time() + delta_tcg_tt(time))
    }
}

impl TransformTimeScale<TCB, TDB> for &TimeScaleTransformer {
    fn transform(&self, time: Time<TCB>) -> Time<TDB> {
        Time::from_base_time(TDB, time.base_time() + delta_tcb_tdb(time))
    }
}

impl TransformTimeScale<TDB, TCB> for &TimeScaleTransformer {
    fn transform(&self, time: Time<TDB>) -> Time<TCB> {
        Time::from_base_time(TCB, time.base_time() + delta_tdb_tcb(time))
    }
}

fn delta_tt_tcg(time: Time<TT>) -> TimeDelta {
    let time = time.base_time().to_f64();
    let raw_delta = INV_LG * (time - J77_TT);
    TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
        panic!(
            "Calculated TT to TCG offset `{}` could not be converted to `TimeDelta`: {}",
            raw_delta, err
        );
    })
}

fn delta_tcg_tt(time: Time<TCG>) -> TimeDelta {
    let time = time.base_time().to_f64();
    let raw_delta = -LG * (time - J77_TT);
    TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
        panic!(
            "Calculated TCG to TT offset `{}` could not be converted to `TimeDelta`: {}",
            raw_delta, err
        );
    })
}

/// 1977 January 1.0 TAI as TT.
const TT_0: Time<TT> = Time::from_base_time(
    TT,
    BaseTime {
        seconds: J77.seconds + D_TAI_TT.seconds,
        subsecond: D_TAI_TT.subsecond,
    },
);

/// The rate of change of TDB with respect to TCB.
const LB: f64 = 1.550519768e-8;

/// The rate of change of TCB with respect to TDB.
const INV_LB: f64 = LB / (1.0 - LB);

/// Constant term of TDB − TT formula of Fairhead & Bretagnon (1990).
const TDB_0: TimeDelta = TimeDelta {
    seconds: -1,
    subsecond: Subsecond(1.0 - 6.55e-5),
};

fn delta_tcb_tdb(time: Time<TCB>) -> TimeDelta {
    let time = time.base_time();
    let tt0 = &TT_0.base_time();
    time.delta(tt0).scale(-LB) + TDB_0
}

fn delta_tdb_tcb(time: Time<TDB>) -> TimeDelta {
    let time = time.base_time();
    let tt0 = &TT_0.base_time();
    time.delta(tt0).scale(INV_LB) - TDB_0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::julian_dates::{J0, SECONDS_BETWEEN_JD_AND_J2000};
    use crate::continuous::BaseTime;
    use crate::Subsecond;
    use float_eq::assert_float_eq;
    use rstest::rstest;

    // Transformations are tested for agreement with both ERFA and AstroTime.jl.

    #[test]
    fn test_transform_tai_tt() {
        let transformer = &TimeScaleTransformer {};
        let tai = Time::new(TAI, 0, Subsecond::default());
        let tt = transformer.transform(tai);
        let expected = Time::new(TT, 32, Subsecond(0.184));
        assert_eq!(expected, tt);
    }

    #[test]
    fn test_transform_tt_tai() {
        let transformer = &TimeScaleTransformer {};
        let tt = Time::new(TT, 32, Subsecond(0.184));
        let tai = transformer.transform(tt);
        let expected = Time::new(TAI, 0, Subsecond::default());
        assert_eq!(expected, tai);
    }

    #[rstest]
    #[case::j0(
        Time::from_base_time(TT, J0),
        Time::from_base_time(TCG, BaseTime::new(-211813488148, Subsecond(0.886867966488467)))
    )]
    #[case::j2000(
        Time::new(TT, 0, Subsecond::default()),
        Time::new(TCG, 0, Subsecond(0.505833286021129))
    )]
    fn test_transform_tt_tcg(#[case] tt: Time<TT>, #[case] expected: Time<TCG>) {
        let transformer = &TimeScaleTransformer {};
        let tcg = transformer.transform(tt);
        assert_eq!(expected, tcg);
    }

    #[rstest]
    #[case::j0(
        Time::from_base_time(TCG, J0),
        Time::from_base_time(TT, BaseTime::new(-211813487853, Subsecond(0.11313193098413876)))
    )]
    #[case::j2000(Time::new(TCG, 0, Subsecond::default()), Time::new(TT, -1, Subsecond(0.49416671433140047)))]
    fn test_transform_tcg_tt(#[case] tcg: Time<TCG>, #[case] expected: Time<TT>) {
        let transformer = &TimeScaleTransformer {};
        let tt = transformer.transform(tcg);
        assert_eq!(expected, tt);
    }

    #[rstest]
    #[case::j0(
        Time::from_base_time(TCB, J0),
        Time::from_base_time(TDB, BaseTime::new(-SECONDS_BETWEEN_JD_AND_J2000 + 3272, Subsecond(0.956_215_636_550_950)))
    )]
    #[case::j2000(Time::j2000(TCB), Time::new(TDB, -12, Subsecond(0.7462129062427061)))]
    fn test_transform_tcb_tdb(#[case] tcb: Time<TCB>, #[case] expected: Time<TDB>) {
        let transformer = &TimeScaleTransformer {};
        let tdb = transformer.transform(tcb);
        assert_eq!(expected.seconds(), tdb.seconds());
        // Lox and ERFA agree to the picosecond. If the use case arises, it may be worth
        // investigating the size of the errors in both libraries and whether greater accuracy is
        // possible.
        assert_float_eq!(expected.subsecond(), tdb.subsecond(), abs <= 1e-12);
    }

    #[rstest]
    #[case::j0(
        Time::from_base_time(TDB, J0),
        Time::from_base_time(TCB, BaseTime::new(-SECONDS_BETWEEN_JD_AND_J2000 - 3273, Subsecond(0.04373361561511046)))
    )]
    #[case::j2000(Time::j2000(TDB), Time::new(TCB, 11, Subsecond(0.2537872682494892)))]
    fn test_transform_tdb_tcb(#[case] tdb: Time<TDB>, #[case] expected: Time<TCB>) {
        let transformer = &TimeScaleTransformer {};
        let tcb = transformer.transform(tdb);
        assert_eq!(expected.seconds(), tcb.seconds());
        assert_float_eq!(expected.subsecond(), tcb.subsecond(), abs <= 1e-11)
    }
}
