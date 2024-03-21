/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module transform provides a trait for transforming between pairs of timescales, together
//! with a default implementation for the most commonly used time scale pairs.

use mockall::automock;

use lox_eop::DeltaUt1UtcProvider;

use crate::base_time::BaseTime;
use crate::calendar_dates::Date;
use crate::constants::julian_dates::J77;
use crate::deltas::TimeDelta;
use crate::julian_dates::Epoch::ModifiedJulianDate;
use crate::julian_dates::JulianDate;
use crate::julian_dates::Unit::Days;
use crate::subsecond::Subsecond;
use crate::time_scales::{TimeScale, TAI, TCB, TCG, TDB, TT};
use crate::utc::{UTCDateTime, UTC};
use crate::wall_clock::WallClock;
use crate::Time;

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

/// An accurate transformation between TDB and TT depends on the trajectory of the observer. For two
/// observers fixed on Earth's surface, the quantity TDB-TT can differ by as much as ~4 µs.
/// Users requiring greater accuracy should implement TransformTimeScale<TT, TDB> manually.
impl TransformTimeScale<TT, TDB> for &TimeScaleTransformer {
    fn transform(&self, time: Time<TT>) -> Time<TDB> {
        Time::from_base_time(TDB, time.base_time() + delta_tt_tdb(time))
    }
}

/// An accurate transformation between TDB and TT depends on the trajectory of the observer. For two
/// observers fixed on Earth's surface, the quantity TDB-TT can differ by as much as ~4 µs.
/// Users requiring greater accuracy should implement TransformTimeScale<TT, TDB> manually.
impl TransformTimeScale<TDB, TT> for &TimeScaleTransformer {
    fn transform(&self, time: Time<TDB>) -> Time<TT> {
        Time::from_base_time(TT, time.base_time() + delta_tdb_tt(time))
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

// TT <-> TDB constants.
const K: f64 = 1.657e-3;
const EB: f64 = 1.671e-2;
const M_0: f64 = 6.239996;
const M_1: f64 = 1.99096871e-7;

fn delta_tt_tdb(time: Time<TT>) -> TimeDelta {
    let tt = time.timestamp.to_f64();
    let g = M_0 + M_1 * tt;
    let raw_delta = K * (g + EB * g.sin()).sin();
    TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
        panic!(
            "Calculated TT to TDB offset `{}` could not be converted to `TimeDelta`: {}",
            raw_delta, err,
        )
    })
}

fn delta_tdb_tt(time: Time<TDB>) -> TimeDelta {
    let tdb = time.timestamp.to_f64();
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

// impl TryFrom<UTCDateTime> for Time<TAI> {
//     type Error = UTCTransformationError;
//
//     fn try_from(utc: UTCDateTime) -> Result<Self, Self::Error> {
//         let base = BaseTime::from_utc_datetime(utc);
//         let mjd = base.julian_date(ModifiedJulianDate, Days);
//         let delta = if (utc.time().second() == 60) {
//             delta_tai_leap_second_utc(mjd)
//         } else {
//             delta_tai_utc(mjd)
//         }?;
//         let tai = Time::from_base_time(TAI, base + delta);
//         Ok(tai)
//     }
// }

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use rstest::rstest;

    use crate::constants::julian_dates::{J0, SECONDS_BETWEEN_JD_AND_J2000};
    use crate::subsecond::Subsecond;
    use crate::BaseTime;

    use super::*;

    // Transformations are tested for agreement with both ERFA and AstroTime.jl.

    const PANIC_INDUCING_BASE_TIME: BaseTime = BaseTime {
        seconds: 0,
        subsecond: Subsecond(f64::NAN),
    };

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
        Time::from_base_time(TCG, BaseTime::new(-211813488148, Subsecond(0.886_867_966_488_467)))
    )]
    #[case::j2000(
        Time::new(TT, 0, Subsecond::default()),
        Time::new(TCG, 0, Subsecond(0.505_833_286_021_129))
    )]
    #[should_panic]
    #[case::unrepresentable(
        Time {
            timestamp: PANIC_INDUCING_BASE_TIME,
            scale: TT,
        },
        Time::default(),
    )]
    fn test_transform_tt_tcg(#[case] tt: Time<TT>, #[case] expected: Time<TCG>) {
        let transformer = &TimeScaleTransformer {};
        let tcg = transformer.transform(tt);
        assert_eq!(expected, tcg);
    }

    #[rstest]
    #[case::j0(
        Time::from_base_time(TCG, J0),
        Time::from_base_time(TT, BaseTime::new(-211813487853, Subsecond(0.113_131_930_984_139)))
    )]
    #[case::j2000(Time::new(TCG, 0, Subsecond::default()), Time::new(TT, -1, Subsecond(0.494_166_714_331_400)))]
    #[should_panic]
    #[case::unrepresentable(
        Time {
            timestamp: PANIC_INDUCING_BASE_TIME,
            scale: TCG,
        },
        Time::default(),
    )]
    fn test_transform_tcg_tt(#[case] tcg: Time<TCG>, #[case] expected: Time<TT>) {
        let transformer = &TimeScaleTransformer {};
        let tt = transformer.transform(tcg);
        assert_eq!(expected.seconds(), tt.seconds());
        assert_float_eq!(expected.subsecond(), tt.subsecond(), abs <= 1e-12)
    }

    #[rstest]
    #[case::j0(
        Time::from_base_time(TCB, J0),
        Time::from_base_time(TDB, BaseTime::new(-SECONDS_BETWEEN_JD_AND_J2000 + 3272, Subsecond(0.956_215_636_550_950)))
    )]
    #[case::j2000(Time::j2000(TCB), Time::new(TDB, -12, Subsecond(0.746_212_906_242_706)))]
    fn test_transform_tcb_tdb(#[case] tcb: Time<TCB>, #[case] expected: Time<TDB>) {
        let transformer = &TimeScaleTransformer {};
        let tdb = transformer.transform(tcb);
        assert_eq!(expected.seconds(), tdb.seconds());
        // Lox and ERFA agree to the picosecond. However, the paper from which these formulae derive
        // (Fairhead & Bretagnon, 1990) provide coefficients for transformations with only
        // nanosecond accuracy. Chasing greater accuracy may not be practical or useful.
        assert_float_eq!(expected.subsecond(), tdb.subsecond(), abs <= 1e-12);
    }

    #[rstest]
    #[case::j0(
        Time::from_base_time(TDB, J0),
        Time::from_base_time(TCB, BaseTime::new(-SECONDS_BETWEEN_JD_AND_J2000 - 3273, Subsecond(0.043_733_615_615_110)))
    )]
    #[case::j2000(Time::j2000(TDB), Time::new(TCB, 11, Subsecond(0.253_787_268_249_489)))]
    fn test_transform_tdb_tcb(#[case] tdb: Time<TDB>, #[case] expected: Time<TCB>) {
        let transformer = &TimeScaleTransformer {};
        let tcb: Time<TCB> = transformer.transform(tdb);
        assert_eq!(expected.seconds(), tcb.seconds());
        assert_float_eq!(expected.subsecond(), tcb.subsecond(), abs <= 1e-11)
    }

    #[rstest]
    #[case::j0(Time::from_base_time(TT, J0), Time::from_base_time(TDB, BaseTime::new(-SECONDS_BETWEEN_JD_AND_J2000, Subsecond(0.001_600_955_458_249))))]
    #[case::j2000(Time::j2000(TT), Time::from_base_time(TDB, BaseTime::new(-1, Subsecond(0.999_927_263_223_809))))]
    #[should_panic]
    #[case::unrepresentable(
        Time {
            timestamp: PANIC_INDUCING_BASE_TIME,
            scale: TT,
        },
    Time::default(),
    )]
    fn test_transform_tt_tdb(#[case] tt: Time<TT>, #[case] expected: Time<TDB>) {
        let transformer = &TimeScaleTransformer {};
        let tdb: Time<TDB> = transformer.transform(tt);
        assert_eq!(expected, tdb)
    }

    #[rstest]
    #[case::j0(Time::from_base_time(TDB, J0), Time::from_base_time(TT, BaseTime::new(-SECONDS_BETWEEN_JD_AND_J2000 - 1, Subsecond(0.998_399_044_541_884))))]
    #[case::j2000(
        Time::j2000(TDB),
        Time::from_base_time(TT, BaseTime::new(0, Subsecond(0.000_072_736_776_166)))
    )]
    #[should_panic]
    #[case::unrepresentable(
        Time {
            timestamp: PANIC_INDUCING_BASE_TIME,
            scale: TDB,
        },
    Time::default(),
    )]
    fn test_transform_tdb_tt(#[case] tdb: Time<TDB>, #[case] expected: Time<TT>) {
        let transformer = &TimeScaleTransformer {};
        let tt: Time<TT> = transformer.transform(tdb);
        assert_eq!(expected, tt)
    }

    // #[rstest]
    // #[case::before_leap_second(
    //     UTCDateTime::new(
    //         Date::new(2016, 12, 31).unwrap(),
    //         UTC::new(23, 59, 59, Subsecond(0.0)).unwrap()
    //     ),
    // // 2017-01-01T00:00:35.000 TAI
    // Ok(Time::new(TAI, 536500835, Subsecond::default()))
    // )]
    // #[case::during_leap_second(
    //     UTCDateTime::new(
    //         Date::new(2016, 12, 31).unwrap(),
    //         UTC::new(23, 59, 60, Subsecond(0.0)).unwrap()
    //     ),
    //     // TODO: it doesn't seem possible to create a TAI time of 36 seconds. But we have two 37-second times...
    //     // 2017-01-01T00:00:36.000 TAI
    //     Ok(Time::new(TAI, 536500836, Subsecond::default()))
    // )]
    // #[case::after_leap_second(
    //     UTCDateTime::new(
    //         Date::new(2017, 1, 1).unwrap(),
    //         UTC::new(0, 0, 0, Subsecond(0.0)).unwrap()
    //     ),
    //     // 2017-01-01T00:00:37.000 TAI
    //     Ok(Time::new(TAI, 536500837, Subsecond::default()))
    // )]
    // fn test_transform_tai_try_from_utc(
    //     #[case] utc: UTCDateTime,
    //     #[case] expected: Result<Time<TAI>, UTCTransformationError>,
    // ) {
    //     let tai: Result<Time<TAI>, UTCTransformationError> = utc.try_into();
    //     assert_eq!(expected, tai);
    // }
}
