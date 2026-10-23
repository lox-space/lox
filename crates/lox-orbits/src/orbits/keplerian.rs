// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    f64::consts::{PI, TAU},
    iter::zip,
};

use super::{CartesianOrbit, KeplerianOrbit, Orbit, Trajectory};
use lox_bodies::{
    CoordinateOrigin, PointMass, TryMeanRadius, TryPointMass, UndefinedOriginPropertyError,
};
use lox_core::units::{AngleUnits, Distance};
use lox_core::{
    anomalies::{EccentricAnomaly, TrueAnomaly},
    elements::{
        ArgumentOfPeriapsis, Eccentricity, Inclination, Keplerian, LongitudeOfAscendingNode,
    },
    trajectory::HermiteTrajectory,
    utils::Linspace,
};
use lox_frames::{NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame, TryQuasiInertial};
use lox_time::deltas::TimeDelta;
use thiserror::Error;

/// Errors that can occur when constructing a Keplerian orbit.
#[derive(Debug, Clone, Error)]
pub enum KeplerianOrbitError {
    /// The reference frame is not quasi-inertial.
    #[error(transparent)]
    NonQuasiInertial(#[from] NonQuasiInertialFrameError),

    /// The perigee radius is below the central body's mean radius.
    #[error("perigee radius is below the origin mean radius")]
    PerigeeCrossesBodyRadius,
}

impl<O, R> KeplerianOrbit<O, R>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Constructs a new Keplerian orbit in a quasi-inertial frame.
    pub const fn new(keplerian: Keplerian, time: lox_time::Time, origin: O, frame: R) -> Self
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

    /// Constructs a Keplerian orbit with validation of the frame and perigee radius.
    pub fn try_from_keplerian(
        keplerian: Keplerian,
        time: lox_time::Time,
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

    /// Returns the semi-major axis.
    pub fn semi_major_axis(&self) -> Distance {
        self.state().semi_major_axis()
    }

    /// Returns the eccentricity.
    pub fn eccentricity(&self) -> Eccentricity {
        self.state().eccentricity()
    }

    /// Returns the inclination.
    pub fn inclination(&self) -> Inclination {
        self.state().inclination()
    }

    /// Returns the longitude of the ascending node.
    pub fn longitude_of_ascending_node(&self) -> LongitudeOfAscendingNode {
        self.state().longitude_of_ascending_node()
    }

    /// Returns the argument of periapsis.
    pub fn argument_of_periapsis(&self) -> ArgumentOfPeriapsis {
        self.state().argument_of_periapsis()
    }

    /// Returns the true anomaly.
    pub fn true_anomaly(&self) -> TrueAnomaly {
        self.state().true_anomaly()
    }

    /// Converts this Keplerian orbit to Cartesian position and velocity.
    pub fn to_cartesian(&self) -> CartesianOrbit<O, R>
    where
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

    /// Converts this Keplerian orbit to Cartesian, returning an error if the gravitational parameter is undefined.
    pub fn try_to_cartesian(&self) -> Result<CartesianOrbit<O, R>, UndefinedOriginPropertyError>
    where
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

    /// Returns the orbital period, or `None` for non-elliptical orbits or undefined gravitational parameters.
    pub fn orbital_period(&self) -> Option<TimeDelta>
    where
        O: TryPointMass,
    {
        self.state()
            .orbital_period(self.try_gravitational_parameter().ok()?)
    }

    /// Generates a trajectory by tracing the full orbit with `n` evenly-spaced samples.
    pub fn trace(&self, n: usize) -> Option<Trajectory<O, R>>
    where
        O: TryPointMass + Copy,
        R: Copy,
    {
        let period = self.orbital_period()?;
        let mean_motion = TAU / period.to_seconds().to_f64();
        let mean_anomaly_at_epoch = self.true_anomaly().to_mean(self.eccentricity()).ok()?;

        let state = self.state();
        let state_iter = state.iter_trace(self.try_gravitational_parameter().ok()?, n);

        let samples: Vec<_> = zip(Linspace::new(-PI, PI, n), state_iter)
            .map(|(ecc, state)| {
                let mean_anomaly = EccentricAnomaly::new(ecc.rad()).to_mean(self.eccentricity());
                let time_of_flight = (mean_anomaly - mean_anomaly_at_epoch).as_f64() / mean_motion;
                (TimeDelta::from_seconds_f64(time_of_flight), state)
            })
            .collect();
        let epoch = self.time() + samples.first()?.0;
        let data = HermiteTrajectory::try_new(samples).ok()?;

        Some(Trajectory::from_parts(
            epoch,
            self.origin(),
            self.reference_frame(),
            data,
        ))
    }
}

impl<O, R> TryFrom<KeplerianOrbit<O, R>> for CartesianOrbit<O, R>
where
    O: CoordinateOrigin + TryPointMass + Copy,
    R: ReferenceFrame + Copy,
{
    type Error = UndefinedOriginPropertyError;

    fn try_from(orbit: KeplerianOrbit<O, R>) -> Result<Self, Self::Error> {
        orbit.try_to_cartesian()
    }
}

#[cfg(test)]
mod tests {
    use lox_bodies::{Earth, MeanRadius};
    use lox_core::units::DistanceUnits;
    use lox_frames::Icrf;
    use lox_time::time_scales::TimeScale;
    use lox_time::{Time, utc::Utc};

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

        let epoch =
            Utc::from_delta(TimeDelta::from_two_part_julian_date(JD1, JD2)).to_dynamic_time();

        let result =
            KeplerianOrbit::try_from_keplerian(elements, epoch.into_dynamic(), Earth, Icrf);
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

        let epoch =
            Utc::from_delta(TimeDelta::from_two_part_julian_date(JD1, JD2)).to_dynamic_time();

        KeplerianOrbit::try_from_keplerian(elements, epoch.into_dynamic(), Earth, Icrf).unwrap();
    }

    #[test]
    fn test_try_from_keplerian_to_cartesian() {
        use lox_core::elements::Keplerian;
        use lox_core::units::DistanceUnits;
        use lox_core::units::{AngleUnits, Distance};

        let time = Time::j2000(TimeScale::Tai);
        let kep = Keplerian::builder()
            .with_semi_major_axis(Distance::new(7_000_000.0), 0.001)
            .with_inclination(51.6_f64.to_radians().rad())
            .with_longitude_of_ascending_node(0.0.rad())
            .with_argument_of_periapsis(0.0.rad())
            .with_true_anomaly(0.0.rad())
            .build()
            .unwrap();
        let kep_orbit = KeplerianOrbit::new(kep, time, Earth, Icrf);
        let cart: CartesianOrbit<_, _> = kep_orbit.try_into().unwrap();
        assert!(cart.position().length() > 6_000.0.km().as_f64());
    }

    #[test]
    fn test_trace_interpolates_at_sample_times() {
        let time = Time::j2000(TimeScale::Tai);
        let kep = Keplerian::builder()
            .with_semi_major_axis(Distance::new(7_000_000.0), 0.001)
            .with_inclination(51.6.deg())
            .with_longitude_of_ascending_node(0.0.rad())
            .with_argument_of_periapsis(0.0.rad())
            .with_true_anomaly(30.0.deg())
            .build()
            .unwrap();
        let orbit = KeplerianOrbit::new(kep, time, Earth, Icrf);
        let traj = orbit.trace(100).unwrap();
        let states = traj.states();

        let seconds_between = |a: Time, b: Time| (a - b).to_seconds().to_f64().abs();
        assert!(seconds_between(traj.start_time(), states[0].time()) < 1e-6);
        assert!(seconds_between(traj.end_time(), states[states.len() - 1].time()) < 1e-6);

        for state in &states {
            let interpolated = traj.at(state.time()).unwrap();
            let error = (interpolated.position() - state.position()).length();
            assert!(error < 1e-3, "position error of {error} m at a knot");
        }
    }
}
