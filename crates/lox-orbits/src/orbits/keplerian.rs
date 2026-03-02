// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    f64::consts::{PI, TAU},
    iter::zip,
};

use super::{CartesianOrbit, KeplerianOrbit, Orbit, Trajectory};
use lox_bodies::{Origin, PointMass, TryMeanRadius, TryPointMass, UndefinedOriginPropertyError};
use lox_core::units::{AngleUnits, Distance};
use lox_core::{
    anomalies::{EccentricAnomaly, TrueAnomaly},
    coords::{CartesianTrajectory, TimeStampedCartesian},
    elements::{
        ArgumentOfPeriapsis, Eccentricity, Inclination, Keplerian, LongitudeOfAscendingNode,
    },
    utils::Linspace,
};
use lox_frames::{NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame, TryQuasiInertial};
use lox_time::{deltas::TimeDelta, time_scales::TimeScale};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum KeplerianOrbitError {
    #[error(transparent)]
    NonQuasiInertial(#[from] NonQuasiInertialFrameError),

    #[error("perigee radius is below the origin mean radius")]
    PerigeeCrossesBodyRadius,
}

impl<T, O, R> KeplerianOrbit<T, O, R>
where
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    pub const fn new(keplerian: Keplerian, time: lox_time::Time<T>, origin: O, frame: R) -> Self
    where
        R: QuasiInertial,
    {
        Orbit::from_state(keplerian, time, origin, frame)
    }

    fn validate_keplerian(k: &Keplerian, origin: &O) -> Result<(), KeplerianOrbitError>
    where
        O: TryMeanRadius,
    {
        let a_km = k.semi_major_axis().to_kilometers();
        let e = k.eccentricity().as_f64();
        let perigee_km = a_km * (1.0 - e);

        if let Ok(body_mean_radius) = origin.try_mean_radius()
            && perigee_km < body_mean_radius.to_kilometers()
        {
            return Err(KeplerianOrbitError::PerigeeCrossesBodyRadius);
        }

        Ok(())
    }

    pub fn try_from_keplerian(
        keplerian: Keplerian,
        time: lox_time::Time<T>,
        origin: O,
        frame: R,
    ) -> Result<Self, KeplerianOrbitError>
    where
        R: TryQuasiInertial,
        O: TryMeanRadius,
    {
        frame.try_quasi_inertial()?;
        Self::validate_keplerian(&keplerian, &origin)?;
        Ok(Orbit::from_state(keplerian, time, origin, frame))
    }

    pub fn semi_major_axis(&self) -> Distance {
        self.state().semi_major_axis()
    }

    pub fn eccentricity(&self) -> Eccentricity {
        self.state().eccentricity()
    }

    pub fn inclination(&self) -> Inclination {
        self.state().inclination()
    }

    pub fn longitude_of_ascending_node(&self) -> LongitudeOfAscendingNode {
        self.state().longitude_of_ascending_node()
    }

    pub fn argument_of_periapsis(&self) -> ArgumentOfPeriapsis {
        self.state().argument_of_periapsis()
    }

    pub fn true_anomaly(&self) -> TrueAnomaly {
        self.state().true_anomaly()
    }

    pub fn to_cartesian(&self) -> CartesianOrbit<T, O, R>
    where
        T: Copy,
        O: Copy + PointMass,
        R: Copy,
    {
        Orbit::from_state(
            self.state().to_cartesian(self.gravitational_parameter()),
            self.time(),
            self.origin(),
            self.reference_frame(),
        )
    }

    pub fn try_to_cartesian(&self) -> Result<CartesianOrbit<T, O, R>, UndefinedOriginPropertyError>
    where
        T: Copy,
        O: Copy + TryPointMass,
        R: Copy,
    {
        Ok(Orbit::from_state(
            self.state()
                .to_cartesian(self.try_gravitational_parameter()?),
            self.time(),
            self.origin(),
            self.reference_frame(),
        ))
    }

    pub fn orbital_period(&self) -> Option<TimeDelta>
    where
        O: TryPointMass,
    {
        self.state()
            .orbital_period(self.try_gravitational_parameter().ok()?)
    }

    pub fn trace(&self, n: usize) -> Option<Trajectory<T, O, R>>
    where
        T: Copy,
        O: TryPointMass + Copy,
        R: Copy,
    {
        let period = self.orbital_period()?;
        let mean_motion = TAU / period.to_seconds().to_f64();
        let mean_anomaly_at_epoch = self.true_anomaly().to_mean(self.eccentricity()).ok()?;

        let state = self.state();
        let state_iter = state.iter_trace(self.try_gravitational_parameter().ok()?, n);

        let data: CartesianTrajectory = zip(Linspace::new(-PI, PI, n), state_iter)
            .map(|(ecc, state)| {
                let mean_anomaly = EccentricAnomaly::new(ecc.rad()).to_mean(self.eccentricity());
                let time_of_flight = (mean_anomaly - mean_anomaly_at_epoch).as_f64() / mean_motion;
                TimeStampedCartesian {
                    time: TimeDelta::from_seconds_f64(time_of_flight),
                    state,
                }
            })
            .collect();

        Some(Trajectory::from_parts(
            self.time(),
            self.origin(),
            self.reference_frame(),
            data,
        ))
    }
}

#[cfg(test)]
mod tests {
    use lox_bodies::{Earth, MeanRadius};
    use lox_frames::Icrf;
    use lox_time::{Time, time_scales::Tai, utc::Utc};
    use lox_units::DistanceUnits;

    use super::*;

    const JD1: f64 = 2458849.5;
    const JD2: f64 = 49.78099017 - 1.0;

    #[test]
    fn test_valid_keplerian() {
        let elements = Keplerian::new(
            MeanRadius::mean_radius(&Earth) + 500.0.km(),
            Eccentricity::try_new(0.0).unwrap(),
            Inclination::try_new(97.0.deg()).unwrap(),
            LongitudeOfAscendingNode::try_new(0.0.deg()).unwrap(),
            ArgumentOfPeriapsis::try_new(0.0.deg()).unwrap(),
            TrueAnomaly::new(0.0.deg()),
        );

        let epoch: Time<Tai> = Utc::from_delta(TimeDelta::from_two_part_julian_date(JD1, JD2))
            .unwrap()
            .to_time();

        let result = KeplerianOrbit::try_from_keplerian(elements, epoch, Earth, Icrf);
        assert!(result.is_ok());
    }

    #[test]
    #[should_panic]
    fn test_invalid_sma() {
        let elements = Keplerian::new(
            // negative altitude
            MeanRadius::mean_radius(&Earth) - 500.0.km(),
            Eccentricity::try_new(0.0).unwrap(),
            Inclination::try_new(97.0.deg()).unwrap(),
            LongitudeOfAscendingNode::try_new(0.0.deg()).unwrap(),
            ArgumentOfPeriapsis::try_new(0.0.deg()).unwrap(),
            TrueAnomaly::new(0.0.deg()),
        );

        let epoch: Time<Tai> = Utc::from_delta(TimeDelta::from_two_part_julian_date(JD1, JD2))
            .unwrap()
            .to_time();

        KeplerianOrbit::try_from_keplerian(elements, epoch, Earth, Icrf).unwrap();
    }
}
