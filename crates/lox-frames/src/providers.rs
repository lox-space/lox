// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::convert::Infallible;

use lox_time::{
    Time,
    deltas::TimeDelta,
    offsets::OffsetProvider,
    time_scales::{Tai, TimeScale},
    utc::{
        Utc,
        leap_seconds::{DefaultLeapSecondsProvider, LeapSecondsProvider},
    },
};

use crate::{
    iers::{Corrections, ReferenceSystem, polar_motion::PoleCoords},
    rotations::RotationProvider,
};

#[derive(Copy, Clone, Debug)]
pub struct DefaultRotationProvider;

impl OffsetProvider for DefaultRotationProvider {
    type Error = Infallible;

    fn tai_to_ut1(&self, delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
        let Some(_) = delta.seconds() else {
            return Ok(TimeDelta::ZERO);
        };
        let tai = Time::from_delta(Tai, delta);
        Ok(DefaultLeapSecondsProvider.delta_tai_utc(tai))
    }

    fn ut1_to_tai(&self, delta: TimeDelta) -> Result<TimeDelta, Self::Error> {
        let Ok(utc) = Utc::from_delta(delta) else {
            return Ok(TimeDelta::ZERO);
        };
        Ok(DefaultLeapSecondsProvider.delta_utc_tai(utc))
    }
}

impl<T> RotationProvider<T> for DefaultRotationProvider
where
    T: TimeScale,
{
    type EopError = Infallible;

    fn corrections(
        &self,
        _time: Time<T>,
        _sys: ReferenceSystem,
    ) -> Result<Corrections, Self::EopError> {
        Ok(Corrections::default())
    }

    fn pole_coords(&self, _time: Time<T>) -> Result<PoleCoords, Self::EopError> {
        Ok(PoleCoords::default())
    }
}
