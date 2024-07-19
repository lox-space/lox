/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use glam::DVec3;
pub use sgp4::Elements;
use sgp4::{Constants, ElementsError, MinutesSinceEpoch};
use thiserror::Error;

use lox_bodies::Earth;
use lox_math::constants::f64::time::SECONDS_PER_MINUTE;
use lox_time::deltas::TimeDelta;
use lox_time::time_scales::Tai;
use lox_time::transformations::ToTai;
use lox_time::utc::Utc;
use lox_time::Time;

use crate::frames::Icrf;
use crate::propagators::Propagator;
use crate::states::State;
use crate::trajectories::TrajectoryError;

#[derive(Debug, Clone, Error)]
pub enum Sgp4Error {
    #[error(transparent)]
    TrajectoryError(#[from] TrajectoryError),
    #[error(transparent)]
    Sgp4(#[from] sgp4::Error),
}

pub struct Sgp4 {
    constants: Constants,
    time: Time<Tai>,
}

impl Sgp4 {
    pub fn new(initial_state: Elements) -> Result<Self, ElementsError> {
        let epoch = initial_state.epoch();
        let time = Utc::from_delta(TimeDelta::from_julian_years(epoch).unwrap()).to_tai();
        let constants = Constants::from_elements(&initial_state)?;
        Ok(Self { constants, time })
    }

    pub fn time(&self) -> Time<Tai> {
        self.time
    }
}

impl Propagator<Time<Tai>, Earth, Icrf> for Sgp4 {
    type Error = Sgp4Error;

    fn propagate(&self, time: Time<Tai>) -> Result<State<Time<Tai>, Earth, Icrf>, Self::Error> {
        let dt = (time - self.time).to_decimal_seconds() / SECONDS_PER_MINUTE;
        let prediction = self.constants.propagate(MinutesSinceEpoch(dt))?;
        let position = DVec3::from_array(prediction.position);
        let velocity = DVec3::from_array(prediction.velocity);
        Ok(State::new(time, position, velocity, Earth, Icrf))
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use crate::elements::ToKeplerian;

    use super::*;

    #[test]
    fn test_sgp4() {
        let tle = Elements::from_tle(
            Some("ISS (ZARYA)".to_string()),
            "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
            "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
        )
        .unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let orbital_period = 92.821;
        let t1 = sgp4.time() + TimeDelta::from_minutes(orbital_period).unwrap();
        let s1 = sgp4.propagate(t1).unwrap();
        let k1 = s1.to_keplerian();
        assert_float_eq!(
            k1.orbital_period().to_decimal_seconds() / SECONDS_PER_MINUTE,
            orbital_period,
            rel <= 1e-4
        );
    }
}
