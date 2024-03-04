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

use crate::continuous::{BaseTime, Time, TimeDelta, TimeScale, TAI, TCG, TT};
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

/// The difference between J2000 and TT at 1977 January 1.0 TAI.
const J77_TT: TimeDelta = TimeDelta {
    seconds: -725803167,
    subsecond: Subsecond(0.816),
};

/// The rate of change of TCG with respect to TT.
const LG_RATE: f64 = 6.969290134e-10;

/// The rate of change of TT with respect to TCG.
const REV_LG_RATE: f64 = LG_RATE / (1.0 - LG_RATE);

impl TransformTimeScale<TT, TCG> for &TimeScaleTransformer {
    fn transform(&self, time: Time<TT>) -> Time<TCG> {
        let delta = tt_tcg_delta(time);
        Time::from_base_time(TCG, time.base_time() + delta)
    }
}

fn tt_tcg_delta(time: Time<TT>) -> TimeDelta {
    let time = time.base_time().to_f64();
    let offset = REV_LG_RATE * (time - -7.25803167816e8); // f64 literal approach matches ERFA to 15 dp
                                                          // let offset = REV_LG_RATE * (time - J77_TT).base_time().to_f64(); // Time const approach matches ERFA to only 8 dp
    TimeDelta::from_decimal_seconds(offset).unwrap_or_else(|err| {
        panic!(
            "Calculated TT to TCG offset `{}` could not be converted to `TimeDelta`: {}",
            offset, err
        );
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Subsecond;

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

    #[test]
    fn test_transform_tt_tcg() {
        let transformer = &TimeScaleTransformer {};
        let tt = Time::new(TT, 0, Subsecond::default());
        let tcg = transformer.transform(tt);
        let expected = Time::new(TCG, 0, Subsecond(0.505833286021129));
        assert_eq!(expected, tcg);
    }
}
