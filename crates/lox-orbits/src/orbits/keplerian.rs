// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{f64::consts::TAU, iter::zip};

use super::{CartesianOrbit, KeplerianOrbit, Orbit, Trajectory};
use lox_bodies::{
    CoordinateOrigin, PointMass, TryMeanRadius, TryPointMass, UndefinedOriginPropertyError,
};
use lox_core::units::{Angle, AngleUnits, AngularRate, Distance};
use lox_core::{
    anomalies::{AnomalyError, EccentricAnomaly, MeanAnomaly, TrueAnomaly},
    coords::{CartesianTrajectory, TimeStampedCartesian},
    elements::{
        ArgumentOfPeriapsis, Eccentricity, Inclination, Keplerian, LongitudeOfAscendingNode,
    },
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

    /// The periapsis radius is below the central body's mean radius.
    #[error("periapsis radius is below the origin mean radius")]
    PeriapsisCrossesBodyRadius,
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
        if let Ok(body_mean_radius) = origin.try_mean_radius()
            && k.periapsis_radius() < body_mean_radius
        {
            return Err(KeplerianOrbitError::PeriapsisCrossesBodyRadius);
        }

        Ok(())
    }

    /// Constructs a Keplerian orbit with validation of the frame and periapsis radius.
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

    /// Returns the semi-major axis, or `None` for parabolic orbits.
    pub fn semi_major_axis(&self) -> Option<Distance> {
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

    /// Returns the semi-parameter (semi-latus rectum).
    pub fn semi_parameter(&self) -> Distance {
        self.state().semi_parameter()
    }

    /// Returns the periapsis radius.
    pub fn periapsis_radius(&self) -> Distance {
        self.state().periapsis_radius()
    }

    /// Returns the apoapsis radius, or `None` for open orbits.
    pub fn apoapsis_radius(&self) -> Option<Distance> {
        self.state().apoapsis_radius()
    }

    /// Returns the periapsis altitude above the origin's mean radius.
    pub fn periapsis_altitude(&self) -> Result<Distance, UndefinedOriginPropertyError>
    where
        O: TryMeanRadius,
    {
        Ok(self
            .state()
            .periapsis_altitude(self.origin.try_mean_radius()?))
    }

    /// Returns the apoapsis altitude above the origin's mean radius, or `None` for open orbits.
    pub fn apoapsis_altitude(&self) -> Result<Option<Distance>, UndefinedOriginPropertyError>
    where
        O: TryMeanRadius,
    {
        Ok(self
            .state()
            .apoapsis_altitude(self.origin.try_mean_radius()?))
    }

    /// Returns the argument of latitude in the interval [0, 2π).
    pub fn argument_of_latitude(&self) -> Angle {
        self.state().argument_of_latitude()
    }

    /// Returns the true longitude in the interval [0, 2π).
    pub fn true_longitude(&self) -> Angle {
        self.state().true_longitude()
    }

    /// Returns the mean longitude, see [`Keplerian::mean_longitude`].
    pub fn mean_longitude(&self) -> Result<Angle, AnomalyError> {
        self.state().mean_longitude()
    }

    /// Returns the flight-path angle.
    pub fn flight_path_angle(&self) -> Angle {
        self.state().flight_path_angle()
    }

    /// Returns the mean motion.
    pub fn mean_motion(&self) -> Result<AngularRate, UndefinedOriginPropertyError>
    where
        O: TryPointMass,
    {
        Ok(self
            .state()
            .mean_motion(self.try_gravitational_parameter()?))
    }

    /// Returns the specific orbital energy in J/kg (m²/s²).
    pub fn specific_orbital_energy(&self) -> Result<f64, UndefinedOriginPropertyError>
    where
        O: TryPointMass,
    {
        Ok(self
            .state()
            .specific_orbital_energy(self.try_gravitational_parameter()?))
    }

    /// Returns the magnitude of the specific angular momentum in m²/s.
    pub fn specific_angular_momentum(&self) -> Result<f64, UndefinedOriginPropertyError>
    where
        O: TryPointMass,
    {
        Ok(self
            .state()
            .specific_angular_momentum(self.try_gravitational_parameter()?))
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
        let grav_param = self.try_gravitational_parameter().ok()?;
        let state = self.state();
        let state_iter = state.iter_trace(grav_param, n)?;
        let mean_motion = state.mean_motion(grav_param).as_f64();
        // The samples span one revolution in [0, 2π], so the epoch must fall in the same range.
        let mean_anomaly_at_epoch = MeanAnomaly::new(
            self.true_anomaly()
                .to_mean(self.eccentricity())
                .ok()?
                .as_angle()
                .mod_two_pi(),
        );

        let time_of_flight = |ecc: f64| {
            let mean_anomaly = EccentricAnomaly::new(ecc.rad()).to_mean(self.eccentricity());
            TimeDelta::from_seconds_f64(
                (mean_anomaly - mean_anomaly_at_epoch).as_f64() / mean_motion,
            )
        };

        let data: CartesianTrajectory = zip(Linspace::new(0.0, TAU, n), state_iter)
            .map(|(ecc, state)| TimeStampedCartesian {
                time: time_of_flight(ecc),
                state,
            })
            .collect();

        // The trajectory data is relative to its first sample, at periapsis on or before the epoch.
        Some(Trajectory::from_parts(
            self.time() + time_of_flight(0.0),
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
    use lox_approx::assert_approx_eq;
    use lox_bodies::{Earth, MeanRadius};
    use lox_core::units::DistanceUnits;
    use lox_frames::Icrf;
    use lox_time::time_scales::TimeScale;
    use lox_time::{Time, utc::Utc};
    use rstest::rstest;

    use super::*;

    const JD1: f64 = 2458849.5;
    const JD2: f64 = 49.78099017 - 1.0;

    #[test]
    fn test_valid_keplerian() {
        let elements = Keplerian::from_semi_parameter(
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
        let elements = Keplerian::from_semi_parameter(
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

    #[rstest]
    #[case::periapsis(0.0)]
    #[case::ascending(90.0)]
    #[case::apoapsis(180.0)]
    #[case::descending(270.0)]
    #[case::before_periapsis(359.0)]
    fn test_trace_matches_epoch_state(#[case] true_anomaly: f64) {
        let time = Time::j2000(TimeScale::Tai);
        let kep = Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.1)
            .with_inclination(51.6.deg())
            .with_true_anomaly(true_anomaly.deg())
            .build()
            .unwrap();
        let orbit = KeplerianOrbit::new(kep, time, Earth, Icrf);
        let trajectory = orbit.trace(100).unwrap();

        assert!(trajectory.start_time() <= time && time <= trajectory.end_time());
        assert_approx_eq!(
            (trajectory.end_time() - trajectory.start_time()).as_seconds_f64(),
            orbit.orbital_period().unwrap().as_seconds_f64(),
            rtol <= 1e-12
        );

        let expected = orbit.to_cartesian();
        let actual = trajectory.at(time);
        assert_approx_eq!(actual.position(), expected.position(), atol <= 1.0);
        assert_approx_eq!(actual.velocity(), expected.velocity(), atol <= 0.1);
    }
}
