/*
 * Copyright (c) 2025. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/*!
    Module `ut1` exposes [DeltaUt1TaiProvider], which describes an API for providing the delta
    between UT1 and TAI at a time of interest.

    [DeltaUt1Tai] is `lox-time`'s default implementation of [DeltaUt1TaiProvider], which parses
    Earth Orientation Parameters from an IERS CSV file.
*/

use crate::iers::{EopProvider, EopProviderError};
use lox_time::deltas::TimeDelta;
use lox_time::offsets::{DefaultOffsetProvider, Offset, TryOffset};
use lox_time::time_scales::{DynTimeScale, Tai, Tcb, Tcg, Tdb, TimeScale, Tt, Ut1};

// TAI <-> UT1

macro_rules! impl_ut1_via_tai {
    ($($scale:ident),*) => {
        $(
            impl TryOffset<$scale, Ut1> for EopProvider
            {
                type Error = EopProviderError;

                fn try_offset(
                    &self,
                    origin: $scale,
                    _target: Ut1,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    let tai = delta + DefaultOffsetProvider.offset(origin, Tai, delta);
                    self.delta_ut1_tai(tai)
                }
            }

            impl TryOffset<Ut1, $scale> for EopProvider
            {
                type Error = EopProviderError;

                fn try_offset(
                    &self,
                    _origin: Ut1,
                    target: $scale,
                    delta: TimeDelta,
                ) -> Result<TimeDelta, Self::Error> {
                    let offset_to_tai = self.delta_tai_ut1(delta)?;
                    let tai = delta + offset_to_tai;
                    Ok(offset_to_tai + DefaultOffsetProvider.offset(Tai, target, tai))
                }
            }
        )*
    }
}

impl_ut1_via_tai!(Tai, Tcb, Tcg, Tt, Tdb);

impl<T: TimeScale, S: TimeScale> Offset<T, S> for EopProvider
where
    DefaultOffsetProvider: Offset<T, S>,
{
    fn offset(&self, origin: T, target: S, delta: TimeDelta) -> TimeDelta {
        DefaultOffsetProvider.offset(origin, target, delta)
    }
}

// Macro to generate TryOffset implementations for DeltaUt1Tai with static scales
macro_rules! impl_dyn_ut1 {
    ($($scale:ident),*) => {
        $(
            impl TryOffset<DynTimeScale, $scale> for EopProvider {
                type Error = EopProviderError;

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

            impl TryOffset<$scale, DynTimeScale> for EopProvider {
                type Error = EopProviderError;

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

impl TryOffset<DynTimeScale, DynTimeScale> for EopProvider {
    type Error = EopProviderError;

    fn try_offset(
        &self,
        origin: DynTimeScale,
        target: DynTimeScale,
        delta: TimeDelta,
    ) -> Result<TimeDelta, Self::Error> {
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
    use std::sync::OnceLock;

    use crate::iers::EopParser;

    use super::*;
    use float_eq::assert_float_eq;
    use lox_math::assert_close;
    use lox_math::is_close::IsClose;
    use lox_test_utils::data_dir;
    use lox_time::Time;
    use lox_time::deltas::ToDelta;
    use lox_time::subsecond::Subsecond;
    use lox_time::time;
    use lox_time::time_scales::Ut1;
    use lox_time::utc::leap_seconds::BuiltinLeapSeconds;
    use lox_time::utc::transformations::ToUtc;
    use lox_time::{DynTime, calendar_dates::Date, time_of_day::TimeOfDay};
    use rstest::{fixture, rstest};

    #[rstest]
    #[case(536414400, -36.40775963091942)]
    #[case(536415400, -36.40776991477994)]
    #[case(536416400, -36.407780212136835)]
    #[case(536417400, -36.40779052210695)]
    #[case(536418400, -36.407800844763194)]
    #[case(536419400, -36.407811180178484)]
    #[case(536420400, -36.40782152842571)]
    #[case(536421400, -36.40783188957778)]
    #[case(536422400, -36.4078422637076)]
    #[case(536423400, -36.407852650888074)]
    #[case(536424400, -36.4078630511921)]
    #[case(536425400, -36.4078734646926)]
    #[case(536426400, -36.40788389146246)]
    #[case(536427400, -36.407894331574596)]
    #[case(536428400, -36.407904785101906)]
    #[case(536429400, -36.40791525211729)]
    #[case(536430400, -36.40792573269367)]
    #[case(536431400, -36.407936226903935)]
    #[case(536432400, -36.40794673482099)]
    #[case(536433400, -36.40795725651775)]
    #[case(536434400, -36.407967792067105)]
    #[case(536435400, -36.40797834154198)]
    #[case(536436400, -36.40798890501525)]
    #[case(536437400, -36.407999482559845)]
    #[case(536438400, -36.40801007424866)]
    #[case(536439400, -36.4080206801546)]
    #[case(536440400, -36.40803130035057)]
    #[case(536441400, -36.40804193490947)]
    #[case(536442400, -36.408052583904215)]
    #[case(536443400, -36.408063247407696)]
    #[case(536444400, -36.40807392549283)]
    #[case(536445400, -36.408084618232515)]
    #[case(536446400, -36.40809532569965)]
    #[case(536447400, -36.40810604796715)]
    #[case(536448400, -36.408116785107914)]
    #[case(536449400, -36.408127537194844)]
    #[case(536450400, -36.408138304300856)]
    #[case(536451400, -36.40814908649884)]
    #[case(536452400, -36.40815988386171)]
    #[case(536453400, -36.40817069646236)]
    #[case(536454400, -36.40818152437371)]
    #[case(536455400, -36.408192367668654)]
    #[case(536456400, -36.4082032264201)]
    #[case(536457400, -36.408214100700945)]
    #[case(536458400, -36.4082249905841)]
    #[case(536459400, -36.40823589614247)]
    #[case(536460400, -36.40824681744896)]
    #[case(536461400, -36.408257754576475)]
    #[case(536462400, -36.40826870759791)]
    #[case(536463400, -36.40827967658618)]
    #[case(536464400, -36.408290661614195)]
    #[case(536465400, -36.40830166275484)]
    #[case(536466400, -36.40831268008103)]
    #[case(536467400, -36.40832371366567)]
    #[case(536468400, -36.40833476358167)]
    #[case(536469400, -36.408345829901926)]
    #[case(536470400, -36.40835691269934)]
    #[case(536471400, -36.40836801204682)]
    #[case(536472400, -36.40837912801728)]
    #[case(536473400, -36.40839026068361)]
    #[case(536474400, -36.40840141011872)]
    #[case(536475400, -36.40841257639551)]
    #[case(536476400, -36.4084237595869)]
    #[case(536477400, -36.40843495976578)]
    #[case(536478400, -36.408446177005054)]
    #[case(536479400, -36.40845741137764)]
    #[case(536480400, -36.40846866295642)]
    #[case(536481400, -36.40847993181432)]
    #[case(536482400, -36.40849121802424)]
    #[case(536483400, -36.408502521659074)]
    #[case(536484400, -36.40851384279173)]
    #[case(536485400, -36.40852518149512)]
    #[case(536486400, -36.40853653784214)]
    #[case(536487400, -36.4085479119057)]
    #[case(536488400, -36.40855930375871)]
    #[case(536489400, -36.408570713474056)]
    #[case(536490400, -36.40858214112466)]
    #[case(536491400, -36.40859358678342)]
    #[case(536492400, -36.408605050523235)]
    #[case(536493400, -36.408616532417014)]
    #[case(536494400, -36.40862803253767)]
    #[case(536495400, -36.40863955095809)]
    #[case(536496400, -36.408651087751196)]
    #[case(536497400, -36.40866264298989)]
    #[case(536498400, -36.40867421674706)]
    #[case(536499400, -36.40868580909562)]
    #[case(536500400, -36.40869742010849)]
    fn test_delta_ut1_tai_orekit(
        #[case] seconds: i64,
        #[case] expected: f64,
        provider: &EopProvider,
    ) {
        let tai = Time::new(Tai, seconds, Subsecond::default());
        let ut1 = Time::new(Ut1, seconds, Subsecond::default());
        let actual = provider
            .delta_ut1_tai(tai.to_delta())
            .unwrap()
            .to_decimal_seconds();
        assert_float_eq!(actual, expected, rel <= 1e-6);
        let actual = provider
            .delta_tai_ut1(ut1.to_delta())
            .unwrap()
            .to_decimal_seconds();
        assert_float_eq!(actual, -expected, rel <= 1e-6);
    }

    // #[rstest]
    // #[case(time!(Tai, 1973, 1, 1).unwrap(), Err(ExtrapolatedDeltaUt1Tai {
    //     req_date: Date::new(1973, 1, 1).unwrap(),
    //     min_date: Date::new(1973, 1, 2).unwrap(),
    //     max_date: Date::new(2025, 3, 15).unwrap(),
    //     extrapolated_value: TimeDelta::try_from_decimal_seconds(-11.188739245677642).unwrap(),
    // }))]
    // #[case(time!(Tai, 2025, 3, 16).unwrap(), Err(ExtrapolatedDeltaUt1Tai {
    //     req_date: Date::new(2025, 3, 16).unwrap(),
    //     min_date: Date::new(1973, 1, 2).unwrap(),
    //     max_date: Date::new(2025, 3, 15).unwrap(),
    //     extrapolated_value: TimeDelta::try_from_decimal_seconds(-36.98893121380733).unwrap(),
    // }))]
    // fn test_delta_ut1_tai_extrapolation(
    //     #[case] time: Time<Tai>,
    //     #[case] expected: Result<TimeDelta, ExtrapolatedDeltaUt1Tai>,
    // ) {
    //     let provider = eop_provider();
    //     let expected = expected
    //         .unwrap_err()
    //         .extrapolated_value
    //         .to_decimal_seconds();
    //     let actual = provider
    //         .delta_ut1_tai(time.to_delta())
    //         .unwrap_err()
    //         .extrapolated_value
    //         .to_decimal_seconds();
    //     assert_float_eq!(actual, expected, rel <= 1e-8);
    //     let ut1 =
    //         time.with_scale_and_delta(Ut1, TimeDelta::try_from_decimal_seconds(actual).unwrap());
    //     let actual = provider
    //         .delta_tai_ut1(ut1.to_delta())
    //         .unwrap_err()
    //         .extrapolated_value
    //         .to_decimal_seconds();
    //     assert_float_eq!(actual, -expected, rel <= 1e-8);
    // }

    const UT1_TOL: f64 = 1e-2;

    // Reference values from Orekit
    //
    // Since we use a different algorithm for TCB UT1 we need to
    // adjust the tolerance.
    //
    #[rstest]
    #[case::tai_ut1("TAI", "UT1", -36.949521832072996)]
    #[case::tcb_ut1("TCB", "UT1", -92.61803559995818)]
    #[case::tcg_ut1("TCG", "UT1", -70.1891114139689)]
    #[case::tdb_ut1("TDB", "UT1", -69.13340440689674)]
    #[case::tt_ut1("TT", "UT1", -69.13352209269237)]
    #[case::ut1_tai("UT1", "TAI", 36.949521532869305)]
    #[case::ut1_tcb("UT1", "TCB", 92.61803631703046)]
    #[case::ut1_tcg("UT1", "TCG", 70.18911089451464)]
    #[case::ut1_tdb("UT1", "TDB", 69.13340387022173)]
    #[case::ut1_tt("UT1", "TT", 69.13352153286931)]
    #[case::ut1_ut1("UT1", "UT1", 0.0)]
    fn test_dyn_time_scale_ut1(
        #[case] scale1: &str,
        #[case] scale2: &str,
        #[case] exp: f64,
        provider: &EopProvider,
    ) {
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
        assert_close!(act, exp, 1e-7, UT1_TOL);
    }

    #[test]
    fn test_ut1_to_utc() {
        let tai = time!(Tai, 2024, 5, 17, 12, 13, 14.0).unwrap();
        let exp = tai.to_utc().unwrap();
        let ut1 = tai.try_to_scale(Ut1, provider()).unwrap();
        let act = ut1.try_to_scale(Tai, provider()).unwrap().to_utc().unwrap();
        assert_eq!(act, exp);
    }

    #[fixture]
    fn provider() -> &'static EopProvider {
        static PROVIDER: OnceLock<EopProvider> = OnceLock::new();
        PROVIDER.get_or_init(|| {
            EopParser::new()
                .from_paths(
                    data_dir().join("finals.all.csv"),
                    data_dir().join("finals2000A.all.csv"),
                )
                .leap_seconds_provider(Box::new(BuiltinLeapSeconds))
                .parse()
                .unwrap()
        })
    }
}
