/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module transform provides a trait for transforming between pairs of timescales, together
//! with a default implementation for the most commonly used time scale pairs.

use crate::constants::u64::FEMTOSECONDS_PER_MILLISECOND;
use crate::continuous::{Time, TimeDelta, TimeScale, TAI, TT};
use mockall::automock;

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
    femtoseconds: 184 * FEMTOSECONDS_PER_MILLISECOND,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_tai_tt() {
        let transformer = &TimeScaleTransformer {};
        let tai = Time::new(TAI, 0, 0);
        let tt = transformer.transform(tai);
        let expected = Time::new(TT, 32, 184 * FEMTOSECONDS_PER_MILLISECOND);
        assert_eq!(expected, tt);
    }

    #[test]
    fn test_transform_tt_tai() {
        let transformer = &TimeScaleTransformer {};
        let tt = Time::new(TT, 32, 184 * FEMTOSECONDS_PER_MILLISECOND);
        let tai = transformer.transform(tt);
        let expected = Time::new(TAI, 0, 0);
        assert_eq!(expected, tai);
    }
}
