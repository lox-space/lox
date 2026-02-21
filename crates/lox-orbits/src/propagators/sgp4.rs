// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use glam::DVec3;
use lox_core::coords::Cartesian;
pub use sgp4::Elements;
use sgp4::{Constants, ElementsError, MinutesSinceEpoch};
use thiserror::Error;

use lox_bodies::Earth;
use lox_core::f64::consts::SECONDS_PER_MINUTE;
use lox_frames::Icrf;
use lox_frames::Teme;
use lox_frames::providers::DefaultRotationProvider;
use lox_frames::rotations::RotationError;
use lox_time::Time;
use lox_time::time_scales::Tai;
use lox_time::utc::UtcError;

use crate::orbits::{CartesianOrbit, TrajectorError};
use crate::propagators::Propagator;

#[derive(Debug, Error)]
pub enum Sgp4Error {
    #[error(transparent)]
    ElementsError(#[from] ElementsError),
    #[error(transparent)]
    TrajectoryError(#[from] TrajectorError),
    #[error(transparent)]
    Sgp4(#[from] sgp4::Error),
    #[error(transparent)]
    Utc(#[from] UtcError),
    #[error(transparent)]
    Rotation(#[from] RotationError),
}

pub struct Sgp4 {
    constants: Constants,
    time: Time<Tai>,
}

impl Sgp4 {
    pub fn new(initial_state: Elements) -> Result<Self, Sgp4Error> {
        let time: Time<Tai> = initial_state.datetime.and_utc().into();
        // Use AFSPC compatibility mode because TLE data is fitted using
        // AFSPC constants (WGS72). Using WGS84 with WGS72-fitted data
        // introduces systematic errors.
        let constants = Constants::from_elements_afspc_compatibility_mode(&initial_state)?;
        Ok(Self { constants, time })
    }

    pub fn time(&self) -> Time<Tai> {
        self.time
    }
}

impl Propagator<Tai, Earth, Icrf> for Sgp4 {
    type Error = Sgp4Error;

    fn propagate(&self, time: Time<Tai>) -> Result<CartesianOrbit<Tai, Earth, Icrf>, Self::Error> {
        let dt = (time - self.time).to_seconds().to_f64() / SECONDS_PER_MINUTE;
        let prediction = self.constants.propagate(MinutesSinceEpoch(dt))?;
        // sgp4 crate returns km and km/s, convert to m and m/s
        let position = DVec3::from_array(prediction.position) * 1e3;
        let velocity = DVec3::from_array(prediction.velocity) * 1e3;

        // SGP4 outputs TEME coordinates — convert to ICRF
        let teme = CartesianOrbit::new(Cartesian::from_vecs(position, velocity), time, Earth, Teme);
        Ok(teme.try_to_frame(Icrf, &DefaultRotationProvider)?)
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;
    use lox_time::deltas::TimeDelta;

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
        let t1 = sgp4.time() + TimeDelta::from_minutes(orbital_period);
        let s1 = sgp4.propagate(t1).unwrap();
        let k1 = s1.to_keplerian();
        assert_approx_eq!(
            k1.orbital_period().unwrap().to_seconds().to_f64() / SECONDS_PER_MINUTE,
            orbital_period,
            rtol <= 1e-4
        );
    }
}
