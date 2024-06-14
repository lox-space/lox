/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/*!
    Module `transformations` provides traits for transforming between pairs of [TimeScale]s, together
    with default implementations for the most commonly used time scale pairs.
*/

use std::convert::Infallible;

use crate::calendar_dates::Date;
use crate::constants::julian_dates::J77;
use crate::deltas::{TimeDelta, ToDelta};
use crate::subsecond::Subsecond;
use crate::time_scales::{Tai, Tcb, Tcg, Tdb, TimeScale, Tt, Ut1};
use crate::ut1::DeltaUt1TaiProvider;
use crate::utc::Utc;
use crate::Time;

/// Marker trait denoting a type that returns an offset between a pair of [TimeScale]s.
pub trait OffsetProvider {
    type Error: std::error::Error;
}

/// A no-op [OffsetProvider] equivalent to `()`, used to guide the type system when implementing
/// transformations with constant offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NoOpOffsetProvider;

impl OffsetProvider for NoOpOffsetProvider {
    type Error = Infallible;
}

/// The base trait underlying all time scale transformations.
///
/// By default, `TryToScale` assumes that no [OffsetProvider] is required and that the
/// transformation is infallible.
pub trait TryToScale<T: TimeScale, U: OffsetProvider>: ToDelta {
    fn try_to_scale(&self, scale: T, provider: &U) -> Result<Time<T>, U::Error>;
}

/// `ToScale` narrows [TryToScale] for the case where no [OffsetProvider] is required and the
/// transformation is infallible.
pub trait ToScale<T: TimeScale>: TryToScale<T, NoOpOffsetProvider> {
    fn to_scale(&self, scale: T) -> Time<T> {
        self.try_to_scale(scale, &NoOpOffsetProvider).unwrap()
    }
}

/// Blanket implementation of [ToScale] for all types that implement [TryToScale] infallibly with
/// no [OffsetProvider].
impl<T: TimeScale, U: TryToScale<T, NoOpOffsetProvider>> ToScale<T> for U {}

/// Convenience trait and default implementation for infallible conversions to [Tai] in terms of
/// [ToScale].
pub trait ToTai: ToScale<Tai> {
    fn to_tai(&self) -> Time<Tai> {
        self.to_scale(Tai)
    }
}

/// Convenience trait and default implementation for infallible conversions to [Tt] in terms of
/// [ToScale].
pub trait ToTt: ToScale<Tt> {
    fn to_tt(&self) -> Time<Tt> {
        self.to_scale(Tt)
    }
}

/// Convenience trait and default implementation for infallible conversions to [Tcg] in terms of
/// [ToScale].
pub trait ToTcg: ToScale<Tcg> {
    fn to_tcg(&self) -> Time<Tcg> {
        self.to_scale(Tcg)
    }
}

/// Convenience trait and default implementation for infallible conversions to [Tcb] in terms of
/// [ToScale].
pub trait ToTcb: ToScale<Tcb> {
    fn to_tcb(&self) -> Time<Tcb> {
        self.to_scale(Tcb)
    }
}

/// Convenience trait and default implementation for infallible conversions to [Tdb] in terms of
/// [ToScale].
pub trait ToTdb: ToScale<Tdb> {
    fn to_tdb(&self) -> Time<Tdb> {
        self.to_scale(Tdb)
    }
}

/// Convenience trait and default implementation for conversions to [Ut1] in terms of [TryToScale].
pub trait ToUt1<T: DeltaUt1TaiProvider>: TryToScale<Ut1, T> {
    fn try_to_ut1(&self, provider: &T) -> Result<Time<Ut1>, T::Error> {
        self.try_to_scale(Ut1, provider)
    }
}

// No-ops

impl<T: OffsetProvider> TryToScale<Tai, T> for Time<Tai> {
    fn try_to_scale(&self, _scale: Tai, _provider: &T) -> Result<Time<Tai>, T::Error> {
        Ok(*self)
    }
}

impl ToTai for Time<Tai> {}

impl<T: OffsetProvider> TryToScale<Tcb, T> for Time<Tcb> {
    fn try_to_scale(&self, _scale: Tcb, _provider: &T) -> Result<Time<Tcb>, T::Error> {
        Ok(*self)
    }
}

impl ToTcb for Time<Tcb> {}

impl<T: OffsetProvider> TryToScale<Tcg, T> for Time<Tcg> {
    fn try_to_scale(&self, _scale: Tcg, _provider: &T) -> Result<Time<Tcg>, T::Error> {
        Ok(*self)
    }
}

impl ToTcg for Time<Tcg> {}

impl<T: OffsetProvider> TryToScale<Tdb, T> for Time<Tdb> {
    fn try_to_scale(&self, _scale: Tdb, _provider: &T) -> Result<Time<Tdb>, T::Error> {
        Ok(*self)
    }
}

impl ToTdb for Time<Tdb> {}

impl<T: OffsetProvider> TryToScale<Tt, T> for Time<Tt> {
    fn try_to_scale(&self, _scale: Tt, _provider: &T) -> Result<Time<Tt>, T::Error> {
        Ok(*self)
    }
}

impl ToTt for Time<Tt> {}

impl<T: DeltaUt1TaiProvider> TryToScale<Ut1, T> for Time<Ut1> {
    fn try_to_scale(&self, _scale: Ut1, _provider: &T) -> Result<Time<Ut1>, T::Error> {
        Ok(*self)
    }
}

impl<T: DeltaUt1TaiProvider> ToUt1<T> for Time<Ut1> {}

// TAI <-> TT

/// The constant offset between TAI and TT.
pub const D_TAI_TT: TimeDelta = TimeDelta {
    seconds: 32,
    subsecond: Subsecond(0.184),
};

impl<T: OffsetProvider> TryToScale<Tt, T> for Time<Tai> {
    fn try_to_scale(&self, scale: Tt, _provider: &T) -> Result<Time<Tt>, T::Error> {
        Ok(self.with_scale_and_delta(scale, D_TAI_TT))
    }
}

impl ToTt for Time<Tai> {}

impl<T: OffsetProvider> TryToScale<Tai, T> for Time<Tt> {
    fn try_to_scale(&self, scale: Tai, _provider: &T) -> Result<Time<Tai>, T::Error> {
        Ok(self.with_scale_and_delta(scale, -D_TAI_TT))
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

impl<T: OffsetProvider> TryToScale<Tcg, T> for Time<Tt> {
    fn try_to_scale(&self, scale: Tcg, _provider: &T) -> Result<Time<Tcg>, T::Error> {
        let time = self.to_delta().to_decimal_seconds();
        let raw_delta = INV_LG * (time - J77_TT);
        let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TT to TCG offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err
            );
        });
        Ok(self.with_scale_and_delta(scale, delta))
    }
}

impl ToTcg for Time<Tt> {}

impl<T: OffsetProvider> TryToScale<Tt, T> for Time<Tcg> {
    fn try_to_scale(&self, scale: Tt, _provider: &T) -> Result<Time<Tt>, T::Error> {
        let time = self.to_delta().to_decimal_seconds();
        let raw_delta = -LG * (time - J77_TT);
        let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TCG to TT offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err
            );
        });
        Ok(self.with_scale_and_delta(scale, delta))
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
const TDB_0: f64 = -6.55e-5;

const TCB_77: f64 = TDB_0 + LB * TT_0;

impl<T: OffsetProvider> TryToScale<Tcb, T> for Time<Tdb> {
    fn try_to_scale(&self, scale: Tcb, _provider: &T) -> Result<Time<Tcb>, T::Error> {
        let dt = self.to_delta().to_decimal_seconds();
        let raw_delta = -TCB_77 / (1.0 - LB) + INV_LB * dt;
        let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TDB to TCB offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err
            );
        });
        Ok(self.with_scale_and_delta(scale, delta))
    }
}

impl ToTcb for Time<Tdb> {}

impl<T: OffsetProvider> TryToScale<Tdb, T> for Time<Tcb> {
    fn try_to_scale(&self, scale: Tdb, _provider: &T) -> Result<Time<Tdb>, T::Error> {
        let dt = self.to_delta().to_decimal_seconds();
        let raw_delta = TCB_77 - LB * dt;
        let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TCB to TDB offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err
            );
        });
        Ok(self.with_scale_and_delta(scale, delta))
    }
}

impl ToTdb for Time<Tcb> {}

// TT <-> TDB

const K: f64 = 1.657e-3;
const EB: f64 = 1.671e-2;
const M_0: f64 = 6.239996;
const M_1: f64 = 1.99096871e-7;

impl<T: OffsetProvider> TryToScale<Tdb, T> for Time<Tt> {
    fn try_to_scale(&self, scale: Tdb, _provider: &T) -> Result<Time<Tdb>, T::Error> {
        let tt = self.to_delta().to_decimal_seconds();
        let g = M_0 + M_1 * tt;
        let raw_delta = K * (g + EB * g.sin()).sin();
        let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TT to TDB offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err,
            )
        });
        Ok(self.with_scale_and_delta(scale, delta))
    }
}

impl ToTdb for Time<Tt> {}

impl<T: OffsetProvider> TryToScale<Tt, T> for Time<Tdb> {
    fn try_to_scale(&self, scale: Tt, _provider: &T) -> Result<Time<Tt>, T::Error> {
        let tdb = self.to_delta().to_decimal_seconds();
        let mut tt = tdb;
        let mut raw_delta = 0.0;
        for _ in 1..3 {
            let g = M_0 + M_1 * tt;
            raw_delta = -K * (g + EB * g.sin()).sin();
            tt = tdb + raw_delta;
        }

        let delta = TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TDB to TT offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err,
            )
        });
        Ok(self.with_scale_and_delta(scale, delta))
    }
}

impl ToTt for Time<Tdb> {}

// TAI <-> UT1

impl<T: DeltaUt1TaiProvider> TryToScale<Ut1, T> for Time<Tai> {
    fn try_to_scale(&self, scale: Ut1, provider: &T) -> Result<Time<Ut1>, T::Error> {
        let delta_ut1_tai = provider.delta_ut1_tai(self)?;
        Ok(self.with_scale_and_delta(scale, delta_ut1_tai))
    }
}

impl<T: DeltaUt1TaiProvider> ToUt1<T> for Time<Tai> {}

impl<T: DeltaUt1TaiProvider> TryToScale<Tai, T> for Time<Ut1> {
    fn try_to_scale(&self, scale: Tai, provider: &T) -> Result<Time<Tai>, T::Error> {
        let delta_tai_ut1 = provider.delta_tai_ut1(self)?;
        Ok(self.with_scale_and_delta(scale, delta_tai_ut1))
    }
}

// Multi-step transformations

impl<T: OffsetProvider> TryToScale<Tai, T> for Time<Tdb> {
    fn try_to_scale(&self, scale: Tai, provider: &T) -> Result<Time<Tai>, T::Error> {
        self.to_tt().try_to_scale(scale, provider)
    }
}

impl ToTai for Time<Tdb> {}

impl<T: OffsetProvider> TryToScale<Tdb, T> for Time<Tai> {
    fn try_to_scale(&self, scale: Tdb, provider: &T) -> Result<Time<Tdb>, T::Error> {
        self.to_tt().try_to_scale(scale, provider)
    }
}

impl ToTdb for Time<Tai> {}

impl<T: OffsetProvider> TryToScale<Tcg, T> for Time<Tdb> {
    fn try_to_scale(&self, scale: Tcg, provider: &T) -> Result<Time<Tcg>, T::Error> {
        self.to_tt().try_to_scale(scale, provider)
    }
}

impl ToTcg for Time<Tdb> {}

impl<T: OffsetProvider> TryToScale<Tcg, T> for Time<Tai> {
    fn try_to_scale(&self, scale: Tcg, provider: &T) -> Result<Time<Tcg>, T::Error> {
        self.to_tt().try_to_scale(scale, provider)
    }
}

impl ToTcg for Time<Tai> {}

impl<T: OffsetProvider> TryToScale<Tai, T> for Time<Tcg> {
    fn try_to_scale(&self, scale: Tai, provider: &T) -> Result<Time<Tai>, T::Error> {
        self.to_tt().try_to_scale(scale, provider)
    }
}

impl ToTai for Time<Tcg> {}

impl<T: OffsetProvider> TryToScale<Tdb, T> for Time<Tcg> {
    fn try_to_scale(&self, scale: Tdb, provider: &T) -> Result<Time<Tdb>, T::Error> {
        self.to_tt().try_to_scale(scale, provider)
    }
}

impl ToTdb for Time<Tcg> {}

impl<T: OffsetProvider> TryToScale<Tcb, T> for Time<Tt> {
    fn try_to_scale(&self, scale: Tcb, provider: &T) -> Result<Time<Tcb>, T::Error> {
        self.to_tdb().try_to_scale(scale, provider)
    }
}

impl ToTcb for Time<Tt> {}

impl<T: OffsetProvider> TryToScale<Tcb, T> for Time<Tai> {
    fn try_to_scale(&self, scale: Tcb, provider: &T) -> Result<Time<Tcb>, T::Error> {
        self.to_tdb().try_to_scale(scale, provider)
    }
}

impl ToTcb for Time<Tai> {}

impl<T: OffsetProvider> TryToScale<Tcb, T> for Time<Tcg> {
    fn try_to_scale(&self, scale: Tcb, provider: &T) -> Result<Time<Tcb>, T::Error> {
        self.to_tdb().try_to_scale(scale, provider)
    }
}

impl ToTcb for Time<Tcg> {}

impl<T: OffsetProvider> TryToScale<Tt, T> for Time<Tcb> {
    fn try_to_scale(&self, scale: Tt, provider: &T) -> Result<Time<Tt>, T::Error> {
        self.to_tdb().try_to_scale(scale, provider)
    }
}

impl ToTt for Time<Tcb> {}

impl<T: OffsetProvider> TryToScale<Tcg, T> for Time<Tcb> {
    fn try_to_scale(&self, scale: Tcg, provider: &T) -> Result<Time<Tcg>, T::Error> {
        self.to_tdb().try_to_scale(scale, provider)
    }
}

impl ToTcg for Time<Tcb> {}

impl<T: OffsetProvider> TryToScale<Tai, T> for Time<Tcb> {
    fn try_to_scale(&self, scale: Tai, provider: &T) -> Result<Time<Tai>, T::Error> {
        self.to_tdb().to_tt().try_to_scale(scale, provider)
    }
}

impl ToTai for Time<Tcb> {}

impl<T: DeltaUt1TaiProvider> TryToScale<Ut1, T> for Time<Tcb> {
    fn try_to_scale(&self, scale: Ut1, provider: &T) -> Result<Time<Ut1>, T::Error> {
        let tai = self.to_tai();
        let delta_ut1_tai = provider.delta_ut1_tai(&tai)?;
        Ok(tai.with_scale_and_delta(scale, delta_ut1_tai))
    }
}

impl<T: DeltaUt1TaiProvider> ToUt1<T> for Time<Tcb> {}

impl<T: DeltaUt1TaiProvider> TryToScale<Ut1, T> for Time<Tcg> {
    fn try_to_scale(&self, scale: Ut1, provider: &T) -> Result<Time<Ut1>, T::Error> {
        let tai = self.to_tai();
        let delta_ut1_tai = provider.delta_ut1_tai(&tai)?;
        Ok(tai.with_scale_and_delta(scale, delta_ut1_tai))
    }
}

impl<T: DeltaUt1TaiProvider> ToUt1<T> for Time<Tcg> {}

impl<T: DeltaUt1TaiProvider> TryToScale<Ut1, T> for Time<Tdb> {
    fn try_to_scale(&self, scale: Ut1, provider: &T) -> Result<Time<Ut1>, T::Error> {
        let tai = self.to_tai();
        let delta_ut1_tai = provider.delta_ut1_tai(&tai)?;
        Ok(tai.with_scale_and_delta(scale, delta_ut1_tai))
    }
}

impl<T: DeltaUt1TaiProvider> ToUt1<T> for Time<Tdb> {}

impl<T: DeltaUt1TaiProvider> TryToScale<Ut1, T> for Time<Tt> {
    fn try_to_scale(&self, scale: Ut1, provider: &T) -> Result<Time<Ut1>, T::Error> {
        let tai = self.to_tai();
        let delta_ut1_tai = provider.delta_ut1_tai(&tai)?;
        Ok(tai.with_scale_and_delta(scale, delta_ut1_tai))
    }
}

impl<T: DeltaUt1TaiProvider> ToUt1<T> for Time<Tt> {}

impl<T: DeltaUt1TaiProvider> TryToScale<Tt, T> for Time<Ut1> {
    fn try_to_scale(&self, _scale: Tt, provider: &T) -> Result<Time<Tt>, T::Error> {
        let tai = self.try_to_scale(Tai, provider)?;
        Ok(tai.to_tt())
    }
}

impl<T: DeltaUt1TaiProvider> TryToScale<Tcg, T> for Time<Ut1> {
    fn try_to_scale(&self, _scale: Tcg, provider: &T) -> Result<Time<Tcg>, T::Error> {
        let tai = self.try_to_scale(Tai, provider)?;
        Ok(tai.to_tcg())
    }
}

impl<T: DeltaUt1TaiProvider> TryToScale<Tcb, T> for Time<Ut1> {
    fn try_to_scale(&self, _scale: Tcb, provider: &T) -> Result<Time<Tcb>, T::Error> {
        let tai = self.try_to_scale(Tai, provider)?;
        Ok(tai.to_tcb())
    }
}

impl<T: DeltaUt1TaiProvider> TryToScale<Tdb, T> for Time<Ut1> {
    fn try_to_scale(&self, _scale: Tdb, provider: &T) -> Result<Time<Tdb>, T::Error> {
        let tai = self.try_to_scale(Tai, provider)?;
        Ok(tai.to_tdb())
    }
}

/// Implementers of `LeapSecondsProvider` provide the offset between TAI and UTC in leap seconds at
/// an instant in either time scale.
pub trait LeapSecondsProvider: OffsetProvider {
    /// The difference in leap seconds between TAI and UTC at the given TAI instant.
    fn delta_tai_utc(&self, tai: Time<Tai>) -> Option<TimeDelta>;

    /// The difference in leap seconds between UTC and TAI at the given UTC instant.
    fn delta_utc_tai(&self, utc: Utc) -> Option<TimeDelta>;

    /// Returns `true` if a leap second occurs on `date`.
    fn is_leap_second_date(&self, date: Date) -> bool;

    /// Returns `true` if a leap second occurs at `tai`.
    fn is_leap_second(&self, tai: Time<Tai>) -> bool;
}

#[cfg(test)]
mod tests {

    use float_eq::assert_float_eq;
    use rstest::rstest;

    use crate::constants::julian_dates::{J0, SECONDS_BETWEEN_JD_AND_J2000};
    use crate::subsecond::Subsecond;
    use crate::test_helpers::delta_ut1_tai;
    use crate::time;

    use super::*;

    // Transformations are tested for agreement with both ERFA and AstroTime.jl.

    const PANIC_INDUCING_DELTA: TimeDelta = TimeDelta {
        seconds: 0,
        subsecond: Subsecond(f64::NAN),
    };

    #[test]
    fn test_transform_all() {
        let tai_exp: Time<Tai> = Time::default();
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
    fn test_time_no_ops() {
        let tai = time!(Tai, 2000, 1, 1).unwrap();
        assert_eq!(tai, tai.to_tai());
        let tcb = time!(Tcb, 2000, 1, 1).unwrap();
        assert_eq!(tcb, tcb.to_tcb());
        let tcg = time!(Tcg, 2000, 1, 1).unwrap();
        assert_eq!(tcg, tcg.to_tcg());
        let tdb = time!(Tdb, 2000, 1, 1).unwrap();
        assert_eq!(tdb, tdb.to_tdb());
        let tt = time!(Tt, 2000, 1, 1).unwrap();
        assert_eq!(tt, tt.to_tt());
        let ut1 = time!(Ut1, 2000, 1, 1).unwrap();
        assert_eq!(ut1, ut1.try_to_ut1(delta_ut1_tai()).unwrap());
    }

    #[test]
    fn test_all_scales_to_ut1() {
        let provider = delta_ut1_tai();

        let tai = time!(Tai, 2024, 5, 17, 12, 13, 14.0).unwrap();
        let exp = tai.try_to_ut1(provider).unwrap();

        let tt = tai.to_tt();
        let act = tt.try_to_ut1(provider).unwrap();
        assert_eq!(act, exp);
        let tcg = tai.to_tcg();
        let act = tcg.try_to_ut1(provider).unwrap();
        assert_eq!(act, exp);
        let tcb = tai.to_tcb();
        let act = tcb.try_to_ut1(provider).unwrap();
        assert_eq!(act, exp);
        let tdb = tai.to_tdb();
        let act = tdb.try_to_ut1(provider).unwrap();
        assert_eq!(act, exp);
    }

    #[test]
    fn test_ut1_to_tai() {
        let provider = delta_ut1_tai();
        let expected = time!(Tai, 2024, 5, 17, 12, 13, 14.0).unwrap();
        let actual = expected
            .try_to_ut1(provider)
            .unwrap()
            .try_to_scale(Tai, provider)
            .unwrap();
        assert_eq!(expected, actual)
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
