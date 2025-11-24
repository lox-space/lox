// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::convert::Infallible;

use lox_time::{
    Time,
    deltas::TimeDelta,
    offsets::OffsetProvider,
    time_scales::Tai,
    utc::{
        Utc,
        leap_seconds::{DefaultLeapSecondsProvider, LeapSecondsProvider},
    },
};

#[macro_export]
macro_rules! transform_provider {
    ($provider:ident) => {
        impl $crate::transformations::TransformProvider for $provider {}
    };
}

#[derive(Copy, Clone, Debug)]
pub struct DefaultTransformProvider;

impl OffsetProvider for DefaultTransformProvider {
    type Error = Infallible;

    fn tai_to_ut1(&self, delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
        let tai = Time::from_delta(Tai, delta);
        Ok(DefaultLeapSecondsProvider
            .delta_tai_utc(tai)
            .unwrap_or_default())
    }

    fn ut1_to_tai(&self, delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
        Ok(Utc::from_delta(delta)
            .ok()
            .map(|utc| {
                DefaultLeapSecondsProvider
                    .delta_utc_tai(utc)
                    .unwrap_or_default()
            })
            .unwrap_or_default())
    }
}

transform_provider!(DefaultTransformProvider);
