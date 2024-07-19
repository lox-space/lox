/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
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

use std::iter::zip;
use thiserror::Error;

use crate::calendar_dates::{CalendarDate, Date};
use crate::constants::i64::SECONDS_PER_DAY;
use crate::constants::julian_dates::SECONDS_BETWEEN_MJD_AND_J2000;
use crate::deltas::TimeDelta;
use crate::julian_dates::JulianDate;
use crate::subsecond::Subsecond;
use crate::time_scales::{Tai, Ut1};
use crate::transformations::{LeapSecondsProvider, OffsetProvider};
use crate::utc::Utc;
use crate::Time;
use lox_io::iers::{EarthOrientationParams, ParseFinalsCsvError};
use lox_math::series::{Series, SeriesError};
use num::ToPrimitive;
use std::path::Path;

/// Implementers of `DeltaUt1TaiProvider` provide the difference between UT1 and TAI at an instant
/// in either time scale.
///
/// This crate provides a standard implementation over IERS Earth Orientation Parameters in
/// [DeltaUt1Tai].
pub trait DeltaUt1TaiProvider: OffsetProvider {
    /// Returns the difference between UT1 and TAI at the given TAI instant.
    fn delta_ut1_tai(&self, tai: &Time<Tai>) -> Result<TimeDelta, Self::Error>;

    /// Returns the difference between TAI and UT1 at the given UT1 instant.
    fn delta_tai_ut1(&self, ut1: &Time<Ut1>) -> Result<TimeDelta, Self::Error>;
}

/// Error type returned when [DeltaUt1Tai] instantiation fails.
#[derive(Clone, Debug, Error)]
pub enum DeltaUt1TaiError {
    #[error(transparent)]
    Csv(#[from] ParseFinalsCsvError),
    #[error(transparent)]
    Series(#[from] SeriesError),
}

/// Error type indicating that an input date to [DeltaUt1Tai] was outside the range of available
/// Earth Orientation Parameters.
///
/// It includes an extrapolated value for the input date which is unlikely to be accurate and should
/// be used with great caution.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("UT1-TAI is only available between {min_date} and {max_date}; value for {req_date} was extrapolated")]
pub struct ExtrapolatedDeltaUt1Tai {
    req_date: Date,
    min_date: Date,
    max_date: Date,
    pub extrapolated_value: TimeDelta,
}

impl ExtrapolatedDeltaUt1Tai {
    fn new(t0: f64, tn: f64, t: f64, val: f64) -> Self {
        let min_date = Time::new(Tai, t0.to_i64().unwrap(), Subsecond::default());
        let max_date = Time::new(Tai, tn.to_i64().unwrap(), Subsecond::default());
        let req_date = Time::new(Tai, t.to_i64().unwrap(), Subsecond::default());
        Self {
            req_date: req_date.date(),
            min_date: min_date.date(),
            max_date: max_date.date(),
            extrapolated_value: TimeDelta::from_decimal_seconds(val).unwrap(),
        }
    }
}

/// Provides a standard implementation of [DeltaUt1TaiProvider] based on cubic spline interpolation
/// of the target time over IERS Earth Orientation Parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct DeltaUt1Tai(Series<Vec<f64>, Vec<f64>>);

impl DeltaUt1Tai {
    /// Instantiates a new [DeltaUt1Tai] provider from a path to an IERS Earth Orientation
    /// Parameters finals CSV and a [LeapSecondsProvider].
    ///
    /// `ls` should provide leap second data for the full range of the EOP data.
    ///
    /// # Errors
    ///
    /// - [DeltaUt1TaiError::Csv] if the CSV file could not be parsed.
    /// - [DeltaUt1TaiError::Series] if construction of a cubic spline from the input series fails.
    pub fn new<P: AsRef<Path>>(
        path: P,
        ls: &impl LeapSecondsProvider,
    ) -> Result<Self, DeltaUt1TaiError> {
        let eop = EarthOrientationParams::parse_finals_csv(path)?;
        let deltas: Vec<TimeDelta> = eop
            .mjd()
            .iter()
            .map(|&mjd| {
                TimeDelta::from_seconds(
                    mjd as i64 * SECONDS_PER_DAY - SECONDS_BETWEEN_MJD_AND_J2000,
                )
            })
            .collect();
        let delta_ut1_tai: Vec<f64> = zip(deltas.clone(), eop.delta_ut1_utc().iter())
            .map(|(delta, &delta_ut1_utc)| {
                let utc = Utc::from_delta(delta);
                let delta_utc_tai = ls.delta_utc_tai(utc).unwrap();
                let delta_ut1_tai =
                    TimeDelta::from_decimal_seconds(delta_ut1_utc).unwrap() + delta_utc_tai;
                delta_ut1_tai.to_decimal_seconds()
            })
            .collect();
        let seconds: Vec<f64> = deltas.iter().map(|dt| dt.to_decimal_seconds()).collect();
        let series = Series::with_cubic_spline(seconds, delta_ut1_tai)?;
        Ok(Self(series))
    }
}

impl OffsetProvider for DeltaUt1Tai {
    type Error = ExtrapolatedDeltaUt1Tai;
}

impl DeltaUt1TaiProvider for DeltaUt1Tai {
    fn delta_ut1_tai(&self, tai: &Time<Tai>) -> Result<TimeDelta, Self::Error> {
        let seconds = tai.seconds_since_j2000();
        let (t0, _) = self.0.first();
        let (tn, _) = self.0.last();
        let val = self.0.interpolate(seconds);
        if seconds < t0 || seconds > tn {
            return Err(ExtrapolatedDeltaUt1Tai::new(t0, tn, seconds, val));
        }
        Ok(TimeDelta::from_decimal_seconds(val).unwrap())
    }

    fn delta_tai_ut1(&self, ut1: &Time<Ut1>) -> Result<TimeDelta, Self::Error> {
        let seconds = ut1.seconds_since_j2000();
        let (t0, _) = self.0.first();
        let (tn, _) = self.0.last();
        // Use the UT1 offset as an initial guess even though the table is based on TAI
        let mut val = self.0.interpolate(seconds);
        // Interpolate again with the adjusted offsets
        for _ in 0..2 {
            val = self.0.interpolate(seconds - val);
        }
        if seconds < t0 || seconds > tn {
            return Err(ExtrapolatedDeltaUt1Tai::new(t0, tn, seconds, -val));
        }
        Ok(-TimeDelta::from_decimal_seconds(val).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use super::*;
    use crate::subsecond::Subsecond;
    use crate::time;
    use crate::utc::leap_seconds::BuiltinLeapSeconds;
    use float_eq::assert_float_eq;
    use rstest::rstest;

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
    fn test_delta_ut1_tai_orekit(#[case] seconds: i64, #[case] expected: f64) {
        let tai = Time::new(Tai, seconds, Subsecond::default());
        let ut1 = Time::new(Ut1, seconds, Subsecond::default());
        let provider = delta_ut1_tai();
        let actual = provider.delta_ut1_tai(&tai).unwrap().to_decimal_seconds();
        assert_float_eq!(actual, expected, rel <= 1e-6);
        let actual = provider.delta_tai_ut1(&ut1).unwrap().to_decimal_seconds();
        assert_float_eq!(actual, -expected, rel <= 1e-6);
    }

    #[rstest]
    #[case(time!(Tai, 1973, 1, 1).unwrap(), Err(ExtrapolatedDeltaUt1Tai {
        req_date: Date::new(1973, 1, 1).unwrap(),
        min_date: Date::new(1973, 1, 2).unwrap(),
        max_date: Date::new(2025, 3, 15).unwrap(),
        extrapolated_value: TimeDelta::from_decimal_seconds(-11.188739245677642).unwrap(),
    }))]
    #[case(time!(Tai, 2025, 3, 16).unwrap(), Err(ExtrapolatedDeltaUt1Tai {
        req_date: Date::new(2025, 3, 16).unwrap(),
        min_date: Date::new(1973, 1, 2).unwrap(),
        max_date: Date::new(2025, 3, 15).unwrap(),
        extrapolated_value: TimeDelta::from_decimal_seconds(-36.98893121380733).unwrap(),
    }))]
    fn test_delta_ut1_tai_extrapolation(
        #[case] time: Time<Tai>,
        #[case] expected: Result<TimeDelta, ExtrapolatedDeltaUt1Tai>,
    ) {
        let provider = delta_ut1_tai();
        let expected = expected
            .unwrap_err()
            .extrapolated_value
            .to_decimal_seconds();
        let actual = provider
            .delta_ut1_tai(&time)
            .unwrap_err()
            .extrapolated_value
            .to_decimal_seconds();
        assert_float_eq!(actual, expected, rel <= 1e-8);
        let ut1 = time.with_scale_and_delta(Ut1, TimeDelta::from_decimal_seconds(actual).unwrap());
        let actual = provider
            .delta_tai_ut1(&ut1)
            .unwrap_err()
            .extrapolated_value
            .to_decimal_seconds();
        assert_float_eq!(actual, -expected, rel <= 1e-8);
    }

    fn delta_ut1_tai() -> &'static DeltaUt1Tai {
        static PROVIDER: OnceLock<DeltaUt1Tai> = OnceLock::new();
        PROVIDER.get_or_init(|| {
            DeltaUt1Tai::new(
                format!(
                    "{}/../../data/finals2000A.all.csv",
                    env!("CARGO_MANIFEST_DIR")
                ),
                &BuiltinLeapSeconds,
            )
            .unwrap()
        })
    }
}
