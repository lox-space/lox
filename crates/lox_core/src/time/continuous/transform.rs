/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module transform provides a trait for transforming between pairs of timescales, together
//! with a default implementation for the most commonly used time scale pairs.

use crate::time::continuous::{Time, TimeScale, TCG, TT};

/// TransformTimeScale transforms a [Time] in [TimeScale] `T` to the corresponding [Time] in
/// [TimeScale] `U`.
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
/// algorithms should implement `TransformTimeScale` for their specific use case.
pub struct TimeScaleTransformer {}

impl TransformTimeScale<TT, TCG> for TimeScaleTransformer {
    const T77T: Time = Time {
        scale: TT,
        timestamp: UnscaledTime {
            seconds:
    }
}
    fn transform(&self, time: Time<TT>) -> Time<TCG> {
        Time::new(time.value())
    }
}
