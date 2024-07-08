/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::constants::julian_dates::J77;
use crate::deltas::TimeDelta;
use crate::prelude::{Tai, Tt};
use crate::subsecond::Subsecond;
use crate::time_scales::{DynTimeScale, Tcb, Tcg, Tdb, TimeScale, Ut1};
use thiserror::Error;

pub trait TryToScale<T: TimeScale, P> {
    type Error: std::error::Error;

    fn try_offset(
        &self,
        scale: &T,
        delta: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error>;

    fn try_to_scale(
        &self,
        scale: &T,
        delta: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        let offset = self.try_offset(scale, delta, provider)?;
        Ok(delta + offset)
    }
}

pub trait ToScale<T: TimeScale> {
    fn offset(&self, scale: &T, delta: TimeDelta) -> TimeDelta;

    fn to_scale(&self, scale: &T, delta: TimeDelta) -> TimeDelta {
        let offset = self.offset(scale, delta);
        delta + offset
    }
}

impl<T, U, P> TryToScale<T, P> for U
where
    T: TimeScale,
    U: ToScale<T>,
{
    type Error = std::convert::Infallible;

    fn try_offset(
        &self,
        scale: &T,
        delta: TimeDelta,
        _provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.offset(scale, delta))
    }
}

impl<T: TimeScale> ToScale<T> for T {
    fn offset(&self, _scale: &T, _delta: TimeDelta) -> TimeDelta {
        TimeDelta::default()
    }
}

pub trait ToTai {
    fn to_tai(&self, delta: TimeDelta) -> TimeDelta;
}

impl<T: ToScale<Tai>> ToTai for T {
    fn to_tai(&self, delta: TimeDelta) -> TimeDelta {
        self.to_scale(&Tai, delta)
    }
}

pub trait ToTcb {
    fn to_tcb(&self, delta: TimeDelta) -> TimeDelta;
}

impl<T: ToScale<Tcb>> ToTcb for T {
    fn to_tcb(&self, delta: TimeDelta) -> TimeDelta {
        self.to_scale(&Tcb, delta)
    }
}

pub trait ToTcg {
    fn to_tcg(&self, delta: TimeDelta) -> TimeDelta;
}

impl<T: ToScale<Tcg>> ToTcg for T {
    fn to_tcg(&self, delta: TimeDelta) -> TimeDelta {
        self.to_scale(&Tcg, delta)
    }
}

pub trait ToTdb {
    fn to_tdb(&self, delta: TimeDelta) -> TimeDelta;
}

impl<T: ToScale<Tdb>> ToTdb for T {
    fn to_tdb(&self, delta: TimeDelta) -> TimeDelta {
        self.to_scale(&Tdb, delta)
    }
}

pub trait ToTt {
    fn to_tt(&self, delta: TimeDelta) -> TimeDelta;
}

impl<T: ToScale<Tt>> ToTt for T {
    fn to_tt(&self, delta: TimeDelta) -> TimeDelta {
        self.to_scale(&Tt, delta)
    }
}

pub trait TryToUt1<P> {
    type Error: std::error::Error;

    fn try_to_ut1(&self, delta: TimeDelta, provider: &P) -> Result<TimeDelta, Self::Error>;
}

impl<T, E, P> TryToUt1<P> for T
where
    T: TryToScale<Ut1, P, Error = E>,
    E: std::error::Error,
{
    type Error = E;

    fn try_to_ut1(&self, delta: TimeDelta, provider: &P) -> Result<TimeDelta, Self::Error> {
        self.try_to_scale(&Ut1, delta, Some(provider))
    }
}

// TAI <-> TT

/// The constant offset between TAI and TT.
pub const D_TAI_TT: TimeDelta = TimeDelta {
    seconds: 32,
    subsecond: Subsecond(0.184),
};

impl ToScale<Tt> for Tai {
    fn offset(&self, _scale: &Tt, _delta: TimeDelta) -> TimeDelta {
        D_TAI_TT
    }
}

impl ToScale<Tai> for Tt {
    fn offset(&self, _scale: &Tai, _delta: TimeDelta) -> TimeDelta {
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
    fn offset(&self, _scale: &Tcg, delta: TimeDelta) -> TimeDelta {
        let dt = delta.to_decimal_seconds();
        let raw_delta = INV_LG * (dt - J77_TT);
        TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TT to TCG offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err
            );
        })
    }
}

impl ToScale<Tt> for Tcg {
    fn offset(&self, _scale: &Tt, delta: TimeDelta) -> TimeDelta {
        let dt = delta.to_decimal_seconds();
        let raw_delta = -LG * (dt - J77_TT);
        TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TCG to TT offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err
            );
        })
    }
}

// TDB <-> TCB

/// 1977 January 1.0 TAI
const TT_0: f64 = J77.seconds as f64
    + crate::transformations::D_TAI_TT.seconds as f64
    + crate::transformations::D_TAI_TT.subsecond.0;

/// The rate of change of TDB with respect to TCB.
const LB: f64 = 1.550519768e-8;

/// The rate of change of TCB with respect to TDB.
const INV_LB: f64 = LB / (1.0 - LB);

/// Constant term of TDB − TT formula of Fairhead & Bretagnon (1990).
const TDB_0: f64 = -6.55e-5;

const TCB_77: f64 = TDB_0 + LB * TT_0;

impl ToScale<Tcb> for Tdb {
    fn offset(&self, _scale: &Tcb, delta: TimeDelta) -> TimeDelta {
        let dt = delta.to_decimal_seconds();
        let raw_delta = -TCB_77 / (1.0 - LB) + INV_LB * dt;
        TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TDB to TCB offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err
            );
        })
    }
}

impl ToScale<Tdb> for Tcb {
    fn offset(&self, _scale: &Tdb, delta: TimeDelta) -> TimeDelta {
        let dt = delta.to_decimal_seconds();
        let raw_delta = TCB_77 - LB * dt;
        TimeDelta::from_decimal_seconds(raw_delta).unwrap_or_else(|err| {
            panic!(
                "Calculated TCB to TDB offset `{}` could not be converted to `TimeDelta`: {}",
                raw_delta, err
            );
        })
    }
}

// TT <-> TDB

const K: f64 = 1.657e-3;
const EB: f64 = 1.671e-2;
const M_0: f64 = 6.239996;
const M_1: f64 = 1.99096871e-7;

impl ToScale<Tdb> for Tt {
    fn offset(&self, _scale: &Tdb, delta: TimeDelta) -> TimeDelta {
        let tt = delta.to_decimal_seconds();
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

impl ToScale<Tt> for Tdb {
    fn offset(&self, _scale: &Tt, delta: TimeDelta) -> TimeDelta {
        let tdb = delta.to_decimal_seconds();
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

// TAI <-> UT1

#[derive(Debug, Error)]
pub enum Ut1Error<P> {
    #[error("no UT1-TAI source provided")]
    NoProvider,
    #[error("UT1 provider error: {0}")]
    ProviderError(#[source] P),
}

pub trait DeltaUt1TaiProvider {
    type Error: std::error::Error + 'static;

    fn delta_ut1_tai(&self, delta: TimeDelta) -> Result<TimeDelta, Self::Error>;
    fn delta_tai_ut1(&self, delta: TimeDelta) -> Result<TimeDelta, Self::Error>;
}

impl<P: DeltaUt1TaiProvider> TryToScale<Ut1, P> for Tai {
    type Error = Ut1Error<P::Error>;

    fn try_offset(
        &self,
        _scale: &Ut1,
        delta: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        let Some(provider) = provider else {
            return Err(Ut1Error::NoProvider);
        };
        provider
            .delta_ut1_tai(delta)
            .map_err(Ut1Error::ProviderError)
    }
}

impl<P: DeltaUt1TaiProvider> TryToScale<Tai, P> for Ut1 {
    type Error = Ut1Error<P::Error>;

    fn try_offset(
        &self,
        _scale: &Tai,
        delta: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        let Some(provider) = provider else {
            return Err(Ut1Error::NoProvider);
        };
        provider
            .delta_tai_ut1(delta)
            .map_err(Ut1Error::ProviderError)
    }
}

// Multi-step transformations

trait ToScaleMulti<T: TimeScale, U: TimeScale + ToScale<T>>: TimeScale + ToScale<U> {
    fn offset_multi(&self, scales: (&U, &T), delta: TimeDelta) -> TimeDelta {
        let (first, second) = scales;
        let mut offset = self.offset(first, delta);
        let delta = delta + offset;
        offset += first.offset(second, delta);
        offset
    }
}

impl<T: TimeScale, U: TimeScale + ToScale<T>, V: TimeScale + ToScale<U>> ToScaleMulti<T, U> for V {}

impl ToScale<Tai> for Tdb {
    fn offset(&self, scale: &Tai, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tt, scale), delta)
    }
}

impl ToScale<Tdb> for Tai {
    fn offset(&self, scale: &Tdb, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tt, scale), delta)
    }
}

impl ToScale<Tai> for Tcg {
    fn offset(&self, scale: &Tai, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tt, scale), delta)
    }
}

impl ToScale<Tcg> for Tai {
    fn offset(&self, scale: &Tcg, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tt, scale), delta)
    }
}

impl ToScale<Tdb> for Tcg {
    fn offset(&self, scale: &Tdb, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tt, scale), delta)
    }
}

impl ToScale<Tcg> for Tdb {
    fn offset(&self, scale: &Tcg, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tt, scale), delta)
    }
}

impl ToScale<Tai> for Tcb {
    fn offset(&self, scale: &Tai, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tdb, scale), delta)
    }
}

impl ToScale<Tcb> for Tai {
    fn offset(&self, scale: &Tcb, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tdb, scale), delta)
    }
}

impl ToScale<Tt> for Tcb {
    fn offset(&self, scale: &Tt, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tdb, scale), delta)
    }
}

impl ToScale<Tcb> for Tt {
    fn offset(&self, scale: &Tcb, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tdb, scale), delta)
    }
}

impl ToScale<Tcg> for Tcb {
    fn offset(&self, scale: &Tcg, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tdb, scale), delta)
    }
}

impl ToScale<Tcb> for Tcg {
    fn offset(&self, scale: &Tcb, delta: TimeDelta) -> TimeDelta {
        self.offset_multi((&Tdb, scale), delta)
    }
}

// Dynamic time scale transformations

impl<P: DeltaUt1TaiProvider> TryToScale<Tai, P> for DynTimeScale {
    type Error = Ut1Error<P::Error>;

    fn try_offset(
        &self,
        _scale: &Tai,
        delta: TimeDelta,
        provider: Option<&P>,
    ) -> Result<TimeDelta, Self::Error> {
        match self {
            DynTimeScale::Tai => Ok(TimeDelta::default()),
            DynTimeScale::Tcb => Ok(Tcb.offset(&Tai, delta)),
            DynTimeScale::Tcg => Ok(Tcg.offset(&Tai, delta)),
            DynTimeScale::Tdb => Ok(Tdb.offset(&Tai, delta)),
            DynTimeScale::Tt => Ok(Tt.offset(&Tai, delta)),
            DynTimeScale::Ut1 => match provider {
                Some(provider) => Ut1.try_offset(&Tai, delta, Some(provider)),
                None => Err(Ut1Error::NoProvider),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::julian_dates::{J0, SECONDS_BETWEEN_JD_AND_J2000};
    use float_eq::assert_float_eq;
    use rstest::rstest;

    const PANIC_INDUCING_DELTA: TimeDelta = TimeDelta {
        seconds: 0,
        subsecond: Subsecond(f64::NAN),
    };

    #[test]
    fn test_transform_all() {
        let tai_exp = TimeDelta::default();
        let tt_exp = Tai.to_tt(tai_exp);
        let tcg_exp = Tt.to_tcg(tt_exp);
        let tdb_exp = Tt.to_tdb(tt_exp);
        let tcb_exp = Tdb.to_tcb(tdb_exp);

        let tt_act = Tai.to_tt(tai_exp);
        let tcg_act = Tai.to_tcg(tai_exp);
        let tdb_act = Tai.to_tdb(tai_exp);
        let tcb_act = Tai.to_tcb(tai_exp);

        assert_eq!(tt_exp, tt_act);
        assert_eq!(tcg_exp, tcg_act);
        assert_eq!(tdb_exp, tdb_act);
        assert_eq!(tcb_exp, tcb_act);

        let tai_act = Tt.to_tai(tt_exp);
        let tcg_act = Tt.to_tcg(tt_exp);
        let tdb_act = Tt.to_tdb(tt_exp);
        let tcb_act = Tt.to_tcb(tt_exp);

        assert_eq!(tai_exp, tai_act);
        assert_eq!(tcg_exp, tcg_act);
        assert_eq!(tdb_exp, tdb_act);
        assert_eq!(tcb_exp, tcb_act);

        let tai_act = Tcg.to_tai(tcg_exp);
        let tt_act = Tcg.to_tt(tcg_exp);
        let tdb_act = Tcg.to_tdb(tcg_exp);
        let tcb_act = Tcg.to_tcb(tcg_exp);

        assert_eq!(tai_exp, tai_act);
        assert_eq!(tt_exp, tt_act);
        assert_eq!(tdb_exp, tdb_act);
        assert_eq!(tcb_exp, tcb_act);

        let tai_act = Tdb.to_tai(tdb_exp);
        let tt_act = Tdb.to_tt(tdb_exp);
        let tcg_act = Tdb.to_tcg(tdb_exp);
        let tcb_act = Tdb.to_tcb(tdb_exp);

        assert_eq!(tai_exp, tai_act);
        assert_eq!(tt_exp, tt_act);
        assert_eq!(tcg_exp, tcg_act);
        assert_eq!(tcb_exp, tcb_act);

        let tai_act = Tcb.to_tai(tcb_exp);
        let tt_act = Tcb.to_tt(tcb_exp);
        let tcg_act = Tcb.to_tcg(tcb_exp);
        let tdb_act = Tcb.to_tdb(tcb_exp);

        assert_eq!(tai_exp, tai_act);
        assert_eq!(tt_exp, tt_act);
        assert_eq!(tcg_exp, tcg_act);
        assert_eq!(tdb_exp, tdb_act);
    }

    #[test]
    fn test_transform_tai_tt() {
        let tai = TimeDelta::default();
        let tt = Tai.to_tt(tai);
        let expected = TimeDelta::new(32, Subsecond(0.184));
        assert_eq!(expected, tt);
    }

    #[test]
    fn test_transform_tt_tai() {
        let tt = TimeDelta::new(32, Subsecond(0.184));
        let tai = Tt.to_tai(tt);
        let expected = TimeDelta::new(0, Subsecond::default());
        assert_eq!(expected, tai);
    }

    #[rstest]
    #[case::j0(
        J0,
        TimeDelta::new(-211813488148, Subsecond(0.886_867_966_488_467))
    )]
    #[case::j2000(
        TimeDelta::new(0, Subsecond::default()),
        TimeDelta::new(0, Subsecond(0.505_833_286_021_129))
    )]
    #[should_panic]
    #[case::unrepresentable(PANIC_INDUCING_DELTA, TimeDelta::default())]
    fn test_transform_tt_tcg(#[case] tt: TimeDelta, #[case] expected: TimeDelta) {
        let tcg = Tt.to_tcg(tt);
        assert_eq!(expected, tcg);
    }

    #[rstest]
    #[case::j0(
        J0,
        TimeDelta::new(-211813487853, Subsecond(0.113_131_930_984_139))
    )]
    #[case::j2000(TimeDelta::new(0, Subsecond::default()), TimeDelta::new(-1, Subsecond(0.494_166_714_331_400)))]
    #[should_panic]
    #[case::unrepresentable(PANIC_INDUCING_DELTA, TimeDelta::default())]
    fn test_transform_tcg_tt(#[case] tcg: TimeDelta, #[case] expected: TimeDelta) {
        let tt = Tcg.to_tt(tcg);
        assert_eq!(expected.seconds, tt.seconds);
        assert_float_eq!(expected.subsecond.0, tt.subsecond.0, abs <= 1e-12);
    }

    #[rstest]
    #[case::j0(
        J0,
        TimeDelta::new(-SECONDS_BETWEEN_JD_AND_J2000 + 3272, Subsecond(0.956_215_636_550_950))
    )]
    #[case::j2000(TimeDelta::default(), TimeDelta::new(-12, Subsecond(0.746_212_906_242_706)))]
    fn test_transform_tcb_tdb(#[case] tcb: TimeDelta, #[case] expected: TimeDelta) {
        let tdb = Tcb.to_tdb(tcb);
        assert_eq!(expected.seconds, tdb.seconds);
        // Lox and ERFA agree to the picosecond. However, the paper from which these formulae derive
        // (Fairhead & Bretagnon, 1990) provide coefficients for transformations with only
        // nanosecond accuracy. Chasing greater accuracy may not be practical or useful.
        assert_float_eq!(expected.subsecond.0, tdb.subsecond.0, abs <= 1e-15);
    }

    #[rstest]
    #[case::j0(
        J0,
        TimeDelta::new(-SECONDS_BETWEEN_JD_AND_J2000 - 3273, Subsecond(0.043_733_615_615_110))
    )]
    #[case::j2000(
        TimeDelta::default(),
        TimeDelta::new(11, Subsecond(0.253_787_268_249_489))
    )]
    fn test_transform_tdb_tcb(#[case] tdb: TimeDelta, #[case] expected: TimeDelta) {
        let tcb = Tdb.to_tcb(tdb);
        assert_eq!(expected.seconds, tcb.seconds);
        assert_float_eq!(expected.subsecond.0, tcb.subsecond.0, abs <= 1e-12);
    }

    #[rstest]
    #[case::j0(J0, TimeDelta::new(-SECONDS_BETWEEN_JD_AND_J2000, Subsecond(0.001_600_955_458_249)))]
    #[case::j2000(TimeDelta::default(), TimeDelta::new(-1, Subsecond(0.999_927_263_223_809)))]
    #[should_panic]
    #[case::unrepresentable(PANIC_INDUCING_DELTA, TimeDelta::default())]
    fn test_transform_tt_tdb(#[case] tt: TimeDelta, #[case] expected: TimeDelta) {
        let tdb = Tt.to_tdb(tt);
        assert_eq!(expected, tdb);
    }

    #[rstest]
    #[case::j0(J0, TimeDelta::new(-SECONDS_BETWEEN_JD_AND_J2000 - 1, Subsecond(0.998_399_044_541_884)))]
    #[case::j2000(
        TimeDelta::default(),
        TimeDelta::new(0, Subsecond(0.000_072_736_776_166))
    )]
    #[should_panic]
    #[case::unrepresentable(PANIC_INDUCING_DELTA, TimeDelta::default())]
    fn test_transform_tdb_tt(#[case] tdb: TimeDelta, #[case] expected: TimeDelta) {
        let tt = Tdb.to_tt(tdb);
        assert_eq!(expected, tt);
    }
}
