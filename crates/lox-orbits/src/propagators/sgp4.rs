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
use lox_frames::Teme;
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::Tai;
use lox_time::utc::UtcError;

use crate::orbits::{CartesianOrbit, TrajectorError, Trajectory};
use crate::propagators::Propagator;

/// Errors that can occur during SGP4 propagation.
#[derive(Debug, Error)]
pub enum Sgp4Error {
    /// Invalid TLE elements.
    #[error(transparent)]
    ElementsError(#[from] ElementsError),
    /// Error constructing the output trajectory.
    #[error(transparent)]
    TrajectoryError(#[from] TrajectorError),
    /// SGP4 prediction error.
    #[error(transparent)]
    Sgp4(#[from] sgp4::Error),
    /// UTC conversion error.
    #[error(transparent)]
    Utc(#[from] UtcError),
}

/// SGP4/SDP4 orbit propagator for satellites described by Two-Line Element sets.
#[derive(Debug, Clone)]
pub struct Sgp4 {
    constants: Constants,
    time: Time<Tai>,
    step: Option<TimeDelta>,
}

impl Sgp4 {
    /// Create a new SGP4 propagator from TLE elements.
    pub fn new(initial_state: Elements) -> Result<Self, Sgp4Error> {
        let time: Time<Tai> = initial_state.datetime.and_utc().into();
        // Use AFSPC compatibility mode because TLE data is fitted using
        // AFSPC constants (WGS72). Using WGS84 with WGS72-fitted data
        // introduces systematic errors.
        let constants = Constants::from_elements_afspc_compatibility_mode(&initial_state)?;
        Ok(Self {
            constants,
            time,
            step: None,
        })
    }

    /// Set the fixed time step used during propagation.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = Some(step);
        self
    }

    /// Return the TLE epoch as a TAI time.
    pub fn time(&self) -> Time<Tai> {
        self.time
    }

    /// Propagate to a single time, returning a TEME state.
    pub fn state_at(&self, time: Time<Tai>) -> Result<CartesianOrbit<Tai, Earth, Teme>, Sgp4Error> {
        let dt = (time - self.time).to_seconds().to_f64() / SECONDS_PER_MINUTE;
        let prediction = self.constants.propagate(MinutesSinceEpoch(dt))?;
        // sgp4 crate returns km and km/s, convert to m and m/s
        let position = DVec3::from_array(prediction.position) * 1e3;
        let velocity = DVec3::from_array(prediction.velocity) * 1e3;
        Ok(CartesianOrbit::new(
            Cartesian::from_vecs(position, velocity),
            time,
            Earth,
            Teme,
        ))
    }
}

impl Propagator<Tai, Earth> for Sgp4 {
    type Frame = Teme;
    type Error = Sgp4Error;

    fn state_at(&self, time: Time<Tai>) -> Result<CartesianOrbit<Tai, Earth, Teme>, Sgp4Error> {
        self.state_at(time)
    }

    fn propagate(
        &self,
        interval: TimeInterval<Tai>,
    ) -> Result<Trajectory<Tai, Earth, Teme>, Sgp4Error> {
        let step = self.step.unwrap_or(TimeDelta::from_seconds(60));
        let states = interval
            .step_by(step)
            .map(|t| self.state_at(t))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Trajectory::try_new(states)?)
    }
}

#[cfg(test)]
mod tests {
    use lox_frames::Icrf;
    use lox_frames::providers::DefaultRotationProvider;
    use lox_test_utils::assert_approx_eq;
    use lox_time::deltas::TimeDelta;
    use lox_time::intervals::Interval;

    use super::*;

    #[test]
    fn test_sgp4_state_at() {
        let tle = Elements::from_tle(
            Some("ISS (ZARYA)".to_string()),
            "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
            "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
        )
        .unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let orbital_period = 92.821;
        let t1 = sgp4.time() + TimeDelta::from_minutes_f64(orbital_period);
        let s1 = sgp4.state_at(t1).unwrap();
        let s1_icrf = s1.try_to_frame(Icrf, &DefaultRotationProvider).unwrap();
        let k1 = s1_icrf.to_keplerian();
        assert_approx_eq!(
            k1.orbital_period().unwrap().to_seconds().to_f64() / SECONDS_PER_MINUTE,
            orbital_period,
            rtol <= 1e-4
        );
    }

    #[test]
    fn test_sgp4_propagate() {
        let tle = Elements::from_tle(
            Some("ISS (ZARYA)".to_string()),
            "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
            "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
        )
        .unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let t0 = sgp4.time();
        let t1 = t0 + TimeDelta::from_minutes_f64(92.821);
        let interval = Interval::new(t0, t1);
        let traj = sgp4.propagate(interval).unwrap();
        // With 60s default step over ~93 min, we should have ~94 points
        assert!(traj.states().len() > 90);
    }

    #[test]
    fn test_sgp4_propagate_in_frame() {
        let tle = Elements::from_tle(
            Some("ISS (ZARYA)".to_string()),
            "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
            "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
        )
        .unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let t0 = sgp4.time();
        let t1 = t0 + TimeDelta::from_minutes(10);
        let interval = Interval::new(t0, t1);
        let traj = sgp4
            .propagate(interval)
            .unwrap()
            .into_frame(Icrf, &DefaultRotationProvider)
            .unwrap();
        assert!(traj.states().len() > 5);
        assert_eq!(traj.reference_frame(), Icrf);
    }
}
