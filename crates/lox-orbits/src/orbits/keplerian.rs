// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    f64::consts::{PI, TAU},
    iter::zip,
};

use lox_bodies::{Origin, PointMass, TryPointMass, UndefinedOriginPropertyError};
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

use super::{CartesianOrbit, KeplerianOrbit, Orbit, Trajectory};

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
        Orbit {
            state: keplerian,
            time,
            origin,
            frame,
        }
    }

    pub fn try_from_keplerian(
        keplerian: Keplerian,
        time: lox_time::Time<T>,
        origin: O,
        frame: R,
    ) -> Result<Self, NonQuasiInertialFrameError>
    where
        R: TryQuasiInertial,
    {
        frame.try_quasi_inertial()?;
        Ok(Orbit {
            state: keplerian,
            time,
            origin,
            frame,
        })
    }

    pub fn semi_major_axis(&self) -> Distance {
        self.state.semi_major_axis()
    }

    pub fn eccentricity(&self) -> Eccentricity {
        self.state.eccentricity()
    }

    pub fn inclination(&self) -> Inclination {
        self.state.inclination()
    }

    pub fn longitude_of_ascending_node(&self) -> LongitudeOfAscendingNode {
        self.state.longitude_of_ascending_node()
    }

    pub fn argument_of_periapsis(&self) -> ArgumentOfPeriapsis {
        self.state.argument_of_periapsis()
    }

    pub fn true_anomaly(&self) -> TrueAnomaly {
        self.state.true_anomaly()
    }

    pub fn to_cartesian(&self) -> CartesianOrbit<T, O, R>
    where
        T: Copy,
        O: Copy + PointMass,
        R: Copy,
    {
        Orbit {
            state: self.state.to_cartesian(self.gravitational_parameter()),
            time: self.time,
            origin: self.origin,
            frame: self.frame,
        }
    }

    pub fn try_to_cartesian(&self) -> Result<CartesianOrbit<T, O, R>, UndefinedOriginPropertyError>
    where
        T: Copy,
        O: Copy + TryPointMass,
        R: Copy,
    {
        Ok(Orbit {
            state: self.state.to_cartesian(self.try_gravitational_parameter()?),
            time: self.time,
            origin: self.origin,
            frame: self.frame,
        })
    }

    pub fn orbital_period(&self) -> Option<TimeDelta>
    where
        O: TryPointMass,
    {
        self.state
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

        let state_iter = self
            .state
            .iter_trace(self.try_gravitational_parameter().ok()?, n);

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

        Some(Trajectory {
            epoch: self.time,
            origin: self.origin,
            frame: self.frame,
            data,
        })
    }
}
