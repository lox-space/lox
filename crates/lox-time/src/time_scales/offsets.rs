use super::{DynTimeScale, Tai, Tcb, Tcg, Tdb, TimeScale, Tt, Ut1};
use crate::{
    constants::julian_dates::J77,
    deltas::TimeDelta,
    subsecond::Subsecond,
    ut1::{DeltaUt1Tai, ExtrapolatedDeltaUt1Tai},
};
use std::convert::Infallible;
use thiserror::Error;

pub trait Offset<Origin, Target>
where
    Origin: TimeScale,
    Target: TimeScale,
{
    fn offset(&self, origin: Origin, target: Target, delta: TimeDelta) -> TimeDelta;
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultOffsetProvider;

impl<Origin, Target> TryOffset<Origin, Target> for DefaultOffsetProvider
where
    Origin: TimeScale,
    Target: TimeScale,
    DefaultOffsetProvider: Offset<Origin, Target>,
{
    type Error = Infallible;

    fn try_offset(
        &self,
        origin: Origin,
        target: Target,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        Ok(self.offset(origin, target, delta))
    }
}

macro_rules! impl_noop {
    ($($scale:ident),*) => {
        $(
            impl Offset<$scale, $scale> for DefaultOffsetProvider {
                fn offset(&self, _origin: $scale, _target: $scale, _delta: TimeDelta) -> TimeDelta {
                    TimeDelta::default()
                }
            }
        )*
    };
}

impl_noop!(Tai, Tcb, Tcg, Tdb, Tt);

// TAI <-> TT

/// The constant offset between TAI and TT.
pub const D_TAI_TT: TimeDelta = TimeDelta {
    seconds: 32,
    subsecond: Subsecond(0.184),
};

impl Offset<Tai, Tt> for DefaultOffsetProvider {
    fn offset(&self, _origin: Tai, _target: Tt, _delta: TimeDelta) -> TimeDelta {
        D_TAI_TT
    }
}

impl Offset<Tt, Tai> for DefaultOffsetProvider {
    fn offset(&self, _origin: Tt, _target: Tai, _delta: TimeDelta) -> TimeDelta {
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

impl Offset<Tt, Tcg> for DefaultOffsetProvider {
    fn offset(&self, _origin: Tt, _target: Tcg, delta: TimeDelta) -> TimeDelta {
        let tt = delta.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(INV_LG * (tt - J77_TT))
    }
}

impl Offset<Tcg, Tt> for DefaultOffsetProvider {
    fn offset(&self, _origin: Tcg, _target: Tt, delta: TimeDelta) -> TimeDelta {
        let tcg = delta.to_decimal_seconds();
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

impl Offset<Tdb, Tcb> for DefaultOffsetProvider {
    fn offset(&self, _origin: Tdb, _target: Tcb, delta: TimeDelta) -> TimeDelta {
        let tdb = delta.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(-TCB_77 / (1.0 - LB) + INV_LB * tdb)
    }
}

impl Offset<Tcb, Tdb> for DefaultOffsetProvider {
    fn offset(&self, _origin: Tcb, _target: Tdb, delta: TimeDelta) -> TimeDelta {
        let tcb = delta.to_decimal_seconds();
        TimeDelta::from_decimal_seconds(TCB_77 - LB * tcb)
    }
}

// TT <-> TDB

const K: f64 = 1.657e-3;
const EB: f64 = 1.671e-2;
const M_0: f64 = 6.239996;
const M_1: f64 = 1.99096871e-7;

impl Offset<Tt, Tdb> for DefaultOffsetProvider {
    fn offset(&self, _origin: Tt, _target: Tdb, delta: TimeDelta) -> TimeDelta {
        let tt = delta.to_decimal_seconds();
        let g = M_0 + M_1 * tt;
        TimeDelta::from_decimal_seconds(K * (g + EB * g.sin()).sin())
    }
}

impl Offset<Tdb, Tt> for DefaultOffsetProvider {
    fn offset(&self, _origin: Tdb, _target: Tt, delta: TimeDelta) -> TimeDelta {
        let tdb = delta.to_decimal_seconds();
        let mut offset = 0.0;
        for _ in 1..3 {
            let g = M_0 + M_1 * (tdb + offset);
            offset = -K * (g + EB * g.sin()).sin();
        }
        TimeDelta::from_decimal_seconds(offset)
    }
}

// TAI <-> UT1

impl<T: TimeScale> TryOffset<T, Ut1> for DeltaUt1Tai
where
    DefaultOffsetProvider: Offset<T, Tai>,
{
    type Error = ExtrapolatedDeltaUt1Tai;

    fn try_offset(
        &self,
        origin: T,
        _target: Ut1,
        delta: TimeDelta,
    ) -> Result<TimeDelta, ExtrapolatedDeltaUt1Tai> {
        let tai = delta + DefaultOffsetProvider.offset(origin, Tai, delta);
        self.delta_ut1_tai(tai)
    }
}

impl<T: TimeScale> TryOffset<Ut1, T> for DeltaUt1Tai
where
    DefaultOffsetProvider: Offset<Tai, T>,
{
    type Error = ExtrapolatedDeltaUt1Tai;

    fn try_offset(
        &self,
        _origin: Ut1,
        target: T,
        delta: TimeDelta,
    ) -> Result<TimeDelta, ExtrapolatedDeltaUt1Tai> {
        let offset_to_tai = self.delta_tai_ut1(delta)?;
        let tai = delta + offset_to_tai;
        Ok(offset_to_tai + DefaultOffsetProvider.offset(Tai, target, tai))
    }
}

impl<T: TimeScale, S: TimeScale> Offset<T, S> for DeltaUt1Tai
where
    DefaultOffsetProvider: Offset<T, S>,
{
    fn offset(&self, origin: T, target: S, delta: TimeDelta) -> TimeDelta {
        DefaultOffsetProvider.offset(origin, target, delta)
    }
}

// Two-step transformations

fn two_step_offset<T1, T2, T3>(origin: T1, via: T2, target: T3, delta: TimeDelta) -> TimeDelta
where
    T1: TimeScale,
    T2: TimeScale + Copy,
    T3: TimeScale,
    DefaultOffsetProvider: Offset<T1, T2>,
    DefaultOffsetProvider: Offset<T2, T3>,
{
    let mut offset = DefaultOffsetProvider.offset(origin, via, delta);
    offset += DefaultOffsetProvider.offset(via, target, delta + offset);
    offset
}

macro_rules! impl_two_step {
    ($(($origin:ident, $via:ident, $target:ident)),*) => {
        $(
            impl Offset<$origin, $target> for DefaultOffsetProvider {
                fn offset(&self, origin: $origin, target: $target, delta: TimeDelta) -> TimeDelta {
                    two_step_offset(origin, $via, target, delta)
                }
            }

            impl Offset<$target, $origin> for DefaultOffsetProvider {
                fn offset(&self, origin: $target, target: $origin, delta: TimeDelta) -> TimeDelta {
                    two_step_offset(origin, $via, target, delta)
                }
            }
        )*
    }
}

impl_two_step!(
    (Tai, Tt, Tdb),
    (Tdb, Tt, Tcg),
    (Tai, Tt, Tcg),
    (Tai, Tdb, Tcb),
    (Tt, Tdb, Tcb),
    (Tcb, Tdb, Tcg)
);

// DynTimeScale

#[derive(Debug, Error, Clone, Copy, Eq, PartialEq)]
#[error("an EOP provider is required for transformations from/to UT1")]
pub struct MissingEopProviderError;

// Macro to generate TryOffset implementations for static scales with DynTimeScale
macro_rules! impl_dyn {
    ($($scale:ident),*) => {
        $(
            impl TryOffset<DynTimeScale, $scale> for DefaultOffsetProvider {
                type Error = MissingEopProviderError;

                fn try_offset(
                    &self,
                    origin: DynTimeScale,
                    target: $scale,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    match origin {
                        DynTimeScale::Tai => Ok(self.offset(Tai, target, delta)),
                        DynTimeScale::Tcb => Ok(self.offset(Tcb, target, delta)),
                        DynTimeScale::Tcg => Ok(self.offset(Tcg, target, delta)),
                        DynTimeScale::Tdb => Ok(self.offset(Tdb, target, delta)),
                        DynTimeScale::Tt => Ok(self.offset(Tt, target, delta)),
                        DynTimeScale::Ut1 => Err(MissingEopProviderError),
                    }
                }
            }

            impl TryOffset<$scale, DynTimeScale> for DefaultOffsetProvider {
                type Error = MissingEopProviderError;

                fn try_offset(
                    &self,
                    origin: $scale,
                    target: DynTimeScale,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    match target {
                        DynTimeScale::Tai => Ok(self.offset(origin, Tai, delta)),
                        DynTimeScale::Tcb => Ok(self.offset(origin, Tcb, delta)),
                        DynTimeScale::Tcg => Ok(self.offset(origin, Tcg, delta)),
                        DynTimeScale::Tdb => Ok(self.offset(origin, Tdb, delta)),
                        DynTimeScale::Tt => Ok(self.offset(origin, Tt, delta)),
                        DynTimeScale::Ut1 => Err(MissingEopProviderError),
                    }
                }
            }
        )*
    }
}

// Apply the macro for all static scales
impl_dyn!(Tai, Tcb, Tcg, Tdb, Tt);

impl TryOffset<DynTimeScale, DynTimeScale> for DefaultOffsetProvider {
    type Error = MissingEopProviderError;

    fn try_offset(
        &self,
        origin: DynTimeScale,
        target: DynTimeScale,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
        match (origin, target) {
            (DynTimeScale::Tai, DynTimeScale::Tcb) => Ok(self.offset(Tai, Tcb, delta)),
            (DynTimeScale::Tai, DynTimeScale::Tcg) => Ok(self.offset(Tai, Tcg, delta)),
            (DynTimeScale::Tai, DynTimeScale::Tdb) => Ok(self.offset(Tai, Tdb, delta)),
            (DynTimeScale::Tai, DynTimeScale::Tt) => Ok(self.offset(Tai, Tt, delta)),
            (DynTimeScale::Tcb, DynTimeScale::Tai) => Ok(self.offset(Tcb, Tai, delta)),
            (DynTimeScale::Tcb, DynTimeScale::Tcg) => Ok(self.offset(Tcb, Tcg, delta)),
            (DynTimeScale::Tcb, DynTimeScale::Tdb) => Ok(self.offset(Tcb, Tdb, delta)),
            (DynTimeScale::Tcb, DynTimeScale::Tt) => Ok(self.offset(Tcb, Tt, delta)),
            (DynTimeScale::Tcg, DynTimeScale::Tai) => Ok(self.offset(Tcg, Tai, delta)),
            (DynTimeScale::Tcg, DynTimeScale::Tcb) => Ok(self.offset(Tcg, Tcb, delta)),
            (DynTimeScale::Tcg, DynTimeScale::Tdb) => Ok(self.offset(Tcg, Tdb, delta)),
            (DynTimeScale::Tcg, DynTimeScale::Tt) => Ok(self.offset(Tcg, Tt, delta)),
            (DynTimeScale::Tdb, DynTimeScale::Tai) => Ok(self.offset(Tdb, Tai, delta)),
            (DynTimeScale::Tdb, DynTimeScale::Tcb) => Ok(self.offset(Tdb, Tcb, delta)),
            (DynTimeScale::Tdb, DynTimeScale::Tcg) => Ok(self.offset(Tdb, Tcg, delta)),
            (DynTimeScale::Tdb, DynTimeScale::Tt) => Ok(self.offset(Tdb, Tt, delta)),
            (DynTimeScale::Tt, DynTimeScale::Tai) => Ok(self.offset(Tt, Tai, delta)),
            (DynTimeScale::Tt, DynTimeScale::Tcb) => Ok(self.offset(Tt, Tcb, delta)),
            (DynTimeScale::Tt, DynTimeScale::Tcg) => Ok(self.offset(Tt, Tcg, delta)),
            (DynTimeScale::Tt, DynTimeScale::Tdb) => Ok(self.offset(Tt, Tdb, delta)),
            (_, DynTimeScale::Ut1) => Err(MissingEopProviderError),
            (DynTimeScale::Ut1, _) => Err(MissingEopProviderError),
            // `origin` and `target` are the same time scale
            (_, _) => Ok(TimeDelta::default()),
        }
    }
}

// Macro to generate TryOffset implementations for DeltaUt1Tai with static scales
macro_rules! impl_dyn_ut1 {
    ($($scale:ident),*) => {
        $(
            impl TryOffset<DynTimeScale, $scale> for DeltaUt1Tai {
                type Error = ExtrapolatedDeltaUt1Tai;

                fn try_offset(
                    &self,
                    origin: DynTimeScale,
                    target: $scale,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    match origin {
                        DynTimeScale::Ut1 => self.try_offset(Ut1, target, delta),
                        DynTimeScale::Tai => Ok(DefaultOffsetProvider.offset(Tai, target, delta)),
                        DynTimeScale::Tcb => Ok(DefaultOffsetProvider.offset(Tcb, target, delta)),
                        DynTimeScale::Tcg => Ok(DefaultOffsetProvider.offset(Tcg, target, delta)),
                        DynTimeScale::Tdb => Ok(DefaultOffsetProvider.offset(Tdb, target, delta)),
                        DynTimeScale::Tt => Ok(DefaultOffsetProvider.offset(Tt, target, delta)),
                    }
                }
            }

            impl TryOffset<$scale, DynTimeScale> for DeltaUt1Tai {
                type Error = ExtrapolatedDeltaUt1Tai;

                fn try_offset(
                    &self,
                    origin: $scale,
                    target: DynTimeScale,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    match target {
                        DynTimeScale::Ut1 => self.try_offset(origin, Ut1, delta),
                        DynTimeScale::Tai => Ok(DefaultOffsetProvider.offset(origin, Tai, delta)),
                        DynTimeScale::Tcb => Ok(DefaultOffsetProvider.offset(origin, Tcb, delta)),
                        DynTimeScale::Tcg => Ok(DefaultOffsetProvider.offset(origin, Tcg, delta)),
                        DynTimeScale::Tdb => Ok(DefaultOffsetProvider.offset(origin, Tdb, delta)),
                        DynTimeScale::Tt => Ok(DefaultOffsetProvider.offset(origin, Tt, delta)),
                    }
                }
            }
        )*
    }
}

// Apply the macro for all static scales
impl_dyn_ut1!(Tai, Tcb, Tcg, Tdb, Tt);

impl TryOffset<DynTimeScale, DynTimeScale> for DeltaUt1Tai {
    type Error = ExtrapolatedDeltaUt1Tai;

    fn try_offset(
        &self,
        origin: DynTimeScale,
        target: DynTimeScale,
        delta: TimeDelta,
    ) -> Result<TimeDelta, ExtrapolatedDeltaUt1Tai> {
        match (origin, target) {
            // UT1 to UT1 is a no-op
            (DynTimeScale::Ut1, DynTimeScale::Ut1) => Ok(TimeDelta::default()),
            // UT1 to other scales
            (DynTimeScale::Ut1, target) => {
                let offset_to_tai = self.try_offset(Ut1, Tai, delta)?;
                let tai = delta + offset_to_tai;
                // We know target is not UT1 here, so this is safe
                Ok(match target {
                    DynTimeScale::Tai => offset_to_tai,
                    DynTimeScale::Tcb => {
                        offset_to_tai + DefaultOffsetProvider.offset(Tai, Tcb, tai)
                    }
                    DynTimeScale::Tcg => {
                        offset_to_tai + DefaultOffsetProvider.offset(Tai, Tcg, tai)
                    }
                    DynTimeScale::Tdb => {
                        offset_to_tai + DefaultOffsetProvider.offset(Tai, Tdb, tai)
                    }
                    DynTimeScale::Tt => offset_to_tai + DefaultOffsetProvider.offset(Tai, Tt, tai),
                    DynTimeScale::Ut1 => unreachable!(), // Already handled above
                })
            }
            // Other scales to UT1
            (origin, DynTimeScale::Ut1) => {
                let offset_to_tai = match origin {
                    DynTimeScale::Tai => TimeDelta::default(),
                    DynTimeScale::Tcb => DefaultOffsetProvider.offset(Tcb, Tai, delta),
                    DynTimeScale::Tcg => DefaultOffsetProvider.offset(Tcg, Tai, delta),
                    DynTimeScale::Tdb => DefaultOffsetProvider.offset(Tdb, Tai, delta),
                    DynTimeScale::Tt => DefaultOffsetProvider.offset(Tt, Tai, delta),
                    DynTimeScale::Ut1 => unreachable!(), // Already handled above
                };
                let tai = delta + offset_to_tai;
                Ok(offset_to_tai + self.try_offset(Tai, Ut1, tai)?)
            }
            // Neither origin nor target is UT1
            (origin, target) => Ok(DefaultOffsetProvider
                .try_offset(origin, target, delta)
                .unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::{
        DynTime, calendar_dates::Date, deltas::ToDelta, test_helpers::delta_ut1_tai,
        time_of_day::TimeOfDay,
    };
    use lox_math::is_close::IsClose;

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
