// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Conversions between `lox-time` and `hifitime` types.
//!
//! A [`Time`] converts to a `hifitime::Epoch` in the same time scale and back, keeping its
//! reading in that scale exactly, down to whole nanoseconds. The conversion involves no time
//! scale model and therefore does not depend on how either library realises a scale.
//!
//! The libraries do differ in their models for TDB and TCB: converting the same TDB or TCB
//! epoch to TAI can give results that are tens of microseconds apart. hifitime's `ET` scale
//! uses the same model as `lox-time`'s TDB.
//!
//! hifitime 4.3 prints and parses TCB and TCG dates 32 s off from its own scale conversions.
//! The conversions here follow hifitime's scale conversions, so a TCG instant agrees with
//! hifitime on the physical instant, but its date as hifitime prints it is 32 s early.
//!
//! | `lox-time`  | `hifitime`        |
//! |-------------|-------------------|
//! | `Gps`       | `GPST`, `QZSST`   |
//! | `Tai`       | `TAI`             |
//! | `Tcb`       | `TCB`             |
//! | `Tcg`       | `TCG`             |
//! | `Tdb`       | `TDB`, `ET`       |
//! | `Tt`        | `TT`              |
//! | `Ut1`       | —                 |
//!
//! An epoch in one of hifitime's other scales, including `UTC`, converts to TAI first, using
//! hifitime's own leap seconds and models.

use hifitime::{Duration, Epoch, TimeScale as HifitimeScale};
use thiserror::Error;

pub use lox_core::time::ToHifitimeDurationError;

use crate::Time;
use crate::time_scales::{Gps, Tai, Tcb, Tcg, Tdb, TimeScale, Tt};
use lox_core::time::TimeDelta;

/// Error returned when a [`Time`] cannot be represented as a `hifitime::Epoch`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ToHifitimeEpochError {
    /// The instant lies outside the ±32,768 centuries a `hifitime::Epoch` can represent.
    #[error("{0} cannot be represented as a `hifitime::Epoch`")]
    OutOfRange(Time),
    /// hifitime has no equivalent of the time scale.
    #[error("hifitime has no equivalent of the {0} time scale")]
    UnsupportedTimeScale(TimeScale),
}

// hifitime counts each scale from its own reference epoch, while `lox-time` counts from J2000
// in every scale. Each offset is the reading of J2000 in hifitime's count, which is the time
// from the reference epoch to J2000 on that scale's clock.

/// 1900-01-01T00:00:00 to J2000.
const J1900_TO_J2000: TimeDelta = TimeDelta::from_seconds(3_155_716_800);
/// 1980-01-06T00:00:00 to J2000.
const GPST_TO_J2000: TimeDelta = TimeDelta::from_seconds(630_763_200);
/// 1977-01-01T00:00:32.184 to J2000.
const TCG_TO_J2000: TimeDelta = TimeDelta::new(725_803_167, 816_000_000_000_000_000);
/// 1977-01-01T00:00:32.1840655 to J2000: hifitime counts TCB from 65.5 µs after the TCG
/// reference epoch.
const TCB_TO_J2000: TimeDelta = TimeDelta::new(725_803_167, 815_934_500_000_000_000);

const fn offset(scale: TimeScale) -> Option<(HifitimeScale, TimeDelta)> {
    match scale {
        TimeScale::Gps => Some((HifitimeScale::GPST, GPST_TO_J2000)),
        TimeScale::Tai => Some((HifitimeScale::TAI, J1900_TO_J2000)),
        TimeScale::Tcb => Some((HifitimeScale::TCB, TCB_TO_J2000)),
        TimeScale::Tcg => Some((HifitimeScale::TCG, TCG_TO_J2000)),
        TimeScale::Tdb => Some((HifitimeScale::TDB, TimeDelta::ZERO)),
        TimeScale::Tt => Some((HifitimeScale::TT, J1900_TO_J2000)),
        TimeScale::Ut1 => None,
    }
}

impl TryFrom<Time> for Epoch {
    type Error = ToHifitimeEpochError;

    /// Converts the instant to an epoch in the same time scale, rounding it down to whole
    /// nanoseconds.
    fn try_from(time: Time) -> Result<Self, Self::Error> {
        let (scale, offset) =
            offset(time.scale()).ok_or(ToHifitimeEpochError::UnsupportedTimeScale(time.scale()))?;
        time.to_delta()
            .checked_add(offset)
            .and_then(|delta| Duration::try_from(delta).ok())
            .map(|duration| Epoch::from_duration(duration, scale))
            .ok_or(ToHifitimeEpochError::OutOfRange(time))
    }
}

macro_rules! impl_to_epoch {
    ($($scale:ident),*) => {
        $(
            impl TryFrom<Time<$scale>> for Epoch {
                type Error = ToHifitimeEpochError;

                /// Converts the instant to an epoch in the same time scale, rounding it down to
                /// whole nanoseconds.
                fn try_from(time: Time<$scale>) -> Result<Self, Self::Error> {
                    time.into_dynamic().try_into()
                }
            }
        )*
    };
}

impl_to_epoch!(Gps, Tai, Tcb, Tcg, Tdb, Tt);

impl From<Epoch> for Time {
    /// Converts the epoch to an instant in the equivalent time scale, or in TAI if `lox-time`
    /// has no equivalent.
    fn from(epoch: Epoch) -> Self {
        let (scale, epoch) = match epoch.time_scale {
            HifitimeScale::GPST | HifitimeScale::QZSST => (TimeScale::Gps, epoch),
            HifitimeScale::TAI => (TimeScale::Tai, epoch),
            HifitimeScale::TCB => (TimeScale::Tcb, epoch),
            HifitimeScale::TCG => (TimeScale::Tcg, epoch),
            HifitimeScale::TDB | HifitimeScale::ET => (TimeScale::Tdb, epoch),
            HifitimeScale::TT => (TimeScale::Tt, epoch),
            // UTC, GST, BDT, TL, TCL and any scale hifitime adds later.
            _ => (TimeScale::Tai, epoch.to_time_scale(HifitimeScale::TAI)),
        };
        let (_, offset) = offset(scale).expect("the scale should have a hifitime equivalent");
        // A `Duration` spans at most ±32,768 centuries, so this cannot overflow.
        Time::from_delta(scale, TimeDelta::from(epoch.duration) - offset)
    }
}

impl From<Epoch> for Time<Tai> {
    /// Converts the epoch to TAI, using hifitime's models for scales other than TAI.
    fn from(epoch: Epoch) -> Self {
        Time::from_delta(
            Tai,
            TimeDelta::from(epoch.to_tai_duration()) - J1900_TO_J2000,
        )
    }
}

#[cfg(test)]
mod tests {
    use hifitime::Unit;
    use rstest::rstest;

    use crate::time_scales::Ut1;

    use super::*;

    fn epoch(iso: &str, scale: HifitimeScale) -> Epoch {
        Epoch::from_gregorian_str(&format!("{iso} {scale}")).unwrap()
    }

    #[rstest]
    #[case(TimeScale::Gps, HifitimeScale::GPST)]
    #[case(TimeScale::Tai, HifitimeScale::TAI)]
    #[case(TimeScale::Tdb, HifitimeScale::TDB)]
    #[case(TimeScale::Tt, HifitimeScale::TT)]
    // TCB and TCG are left out: hifitime 4.3.2 parses and prints their dates 32 s off from
    // its own conversions, so `test_hifitime_epoch_matches_tai` covers them instead.
    fn test_hifitime_epoch_keeps_the_reading(
        #[case] scale: TimeScale,
        #[values(
            "1900-01-01T00:00:00",
            "1977-01-01T00:00:32.184",
            "2000-01-01T12:00:00",
            "2024-06-01T01:02:03.123456789"
        )]
        iso: &str,
        #[case] hifitime_scale: HifitimeScale,
    ) {
        let time = Time::from_iso(scale, iso).unwrap();
        let epoch = epoch(iso, hifitime_scale);
        assert_eq!(Epoch::try_from(time).unwrap(), epoch);
        assert_eq!(Time::from(epoch), time);
    }

    #[rstest]
    #[case(HifitimeScale::QZSST, TimeScale::Gps)]
    #[case(HifitimeScale::ET, TimeScale::Tdb)]
    fn test_hifitime_epoch_aliases(
        #[case] hifitime_scale: HifitimeScale,
        #[case] scale: TimeScale,
    ) {
        let iso = "2024-06-01T01:02:03.123456789";
        let time = Time::from(epoch(iso, hifitime_scale));
        assert_eq!(time, Time::from_iso(scale, iso).unwrap());
    }

    #[test]
    fn test_hifitime_epoch_static_scales() {
        let iso = "2024-06-01T01:02:03.123456789";
        let tt = Time::from_iso(Tt, iso).unwrap();
        assert_eq!(Epoch::try_from(tt).unwrap(), epoch(iso, HifitimeScale::TT));
        let gps = Time::from_iso(Gps, iso).unwrap();
        assert_eq!(
            Epoch::try_from(gps).unwrap(),
            epoch(iso, HifitimeScale::GPST)
        );
    }

    #[rstest]
    // These scales are related to TAI by definition, so both libraries must agree on the
    // instant, not just the reading, to within hifitime's rounding.
    #[case(TimeScale::Gps, 1e-9)]
    #[case(TimeScale::Tcg, 1e-7)]
    #[case(TimeScale::Tt, 1e-9)]
    // The libraries model TDB, and hence TCB, differently, by up to about 40 µs.
    #[case(TimeScale::Tcb, 1e-4)]
    fn test_hifitime_epoch_matches_tai(
        #[case] scale: TimeScale,
        #[values("1977-01-01T00:00:00", "2000-01-01T12:00:00", "2024-06-01T00:00:00")] iso: &str,
        #[case] tolerance: f64,
    ) {
        let time = Time::from_iso(Tai, iso).unwrap();
        let epoch = Epoch::try_from(time.to_scale(scale)).unwrap();
        let act = Time::<Tai>::from(epoch);
        assert!((act - time).as_seconds_f64().abs() < tolerance);
    }

    #[rstest]
    #[case(TimeScale::Tcb, TCB_TO_J2000)]
    #[case(TimeScale::Tcg, TCG_TO_J2000)]
    fn test_hifitime_epoch_j2000_reading(#[case] scale: TimeScale, #[case] offset: TimeDelta) {
        let epoch = Epoch::try_from(Time::j2000(scale)).unwrap();
        assert_eq!(TimeDelta::from(epoch.duration), offset);
        assert_eq!(Time::from(epoch), Time::j2000(scale));
    }

    #[test]
    fn test_hifitime_epoch_et_matches_tdb_model() {
        let tdb = Time::from_iso(Tdb, "2024-06-01T00:00:00").unwrap();
        let et = Epoch::from_duration(Epoch::try_from(tdb).unwrap().duration, HifitimeScale::ET);
        let act = Time::<Tai>::from(et);
        assert!((act - tdb.to_scale(Tai)).as_seconds_f64().abs() < 1e-8);
    }

    #[test]
    fn test_hifitime_epoch_utc() {
        // TAI - UTC = 37 s since 2017.
        let epoch = epoch("2024-06-01T00:00:00", HifitimeScale::UTC);
        let exp = Time::from_iso(TimeScale::Tai, "2024-06-01T00:00:37").unwrap();
        assert_eq!(Time::from(epoch), exp);
        assert_eq!(Time::<Tai>::from(epoch), exp.with_scale(Tai));
    }

    #[test]
    fn test_hifitime_epoch_gst() {
        // GST, like GPS time, is TAI - 19 s.
        let epoch = epoch("2024-06-01T00:00:00", HifitimeScale::GST);
        let exp = Time::from_iso(TimeScale::Tai, "2024-06-01T00:00:19").unwrap();
        assert_eq!(Time::from(epoch), exp);
    }

    #[test]
    fn test_hifitime_epoch_tai_from_other_scale() {
        let epoch = epoch("2024-06-01T00:00:00", HifitimeScale::TT);
        let exp = Time::from_iso(Tai, "2024-05-31T23:59:27.816").unwrap();
        assert_eq!(Time::<Tai>::from(epoch), exp);
    }

    #[test]
    fn test_hifitime_epoch_rounds_down_to_nanoseconds() {
        let time = Time::from_iso(Tai, "2000-01-01T12:00:00.000000000999").unwrap();
        assert_eq!(
            Epoch::try_from(time).unwrap(),
            epoch("2000-01-01T12:00:00", HifitimeScale::TAI)
        );
        let time = Time::from_iso(Tai, "1999-12-31T23:59:59.999999999999").unwrap();
        assert_eq!(
            Epoch::try_from(time).unwrap(),
            epoch("1999-12-31T23:59:59.999999999", HifitimeScale::TAI)
        );
    }

    #[rstest]
    #[case(Duration::MIN)]
    #[case(Duration::MAX)]
    fn test_hifitime_epoch_limits(#[case] duration: Duration) {
        let epoch = Epoch::from_tai_duration(duration);
        let time: Time = epoch.into();
        assert_eq!(Epoch::try_from(time).unwrap(), epoch);
    }

    #[test]
    fn test_hifitime_epoch_just_after_max() {
        let max: Time = Epoch::from_tai_duration(Duration::MAX).into();
        let time = max + TimeDelta::from_nanoseconds(1);
        assert_eq!(
            Epoch::try_from(time),
            Err(ToHifitimeEpochError::OutOfRange(time))
        );
    }

    #[test]
    fn test_hifitime_epoch_out_of_range() {
        let time = Time::from_delta(Tai, TimeDelta::new(i64::MAX, 0));
        assert_eq!(
            Epoch::try_from(time),
            Err(ToHifitimeEpochError::OutOfRange(time.into_dynamic()))
        );
    }

    #[test]
    fn test_hifitime_epoch_ut1_is_unsupported() {
        let time = Time::from_delta(Ut1, TimeDelta::ZERO).into_dynamic();
        assert_eq!(
            Epoch::try_from(time),
            Err(ToHifitimeEpochError::UnsupportedTimeScale(TimeScale::Ut1))
        );
    }

    #[test]
    fn test_hifitime_epoch_duration_arithmetic() {
        let time = Time::from_iso(Tai, "2024-06-01T00:00:00").unwrap();
        let epoch = Epoch::try_from(time).unwrap() + Unit::Second * 1.5;
        assert_eq!(
            Time::<Tai>::from(epoch),
            time + TimeDelta::from_milliseconds(1_500)
        );
    }
}
