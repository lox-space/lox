/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use thiserror::Error;

use crate::base::{BaseCartesian, BaseState, BaseTwoBody};
use crate::frames::ReferenceFrame;
use crate::trajectories::base::BaseCubicSplineTrajectory;
use crate::trajectories::{CubicSplineTrajectory, LoxTrajectoryError, Trajectory};
use crate::two_body::Cartesian;
use crate::CoordinateSystem;
use lox_bodies::{Earth, PointMass};
use lox_time::base_time::BaseTime;
use lox_time::deltas::TimeDelta;
use lox_time::time_scales::TimeScale;

use super::base::BasePropagator;
use super::{stumpff, Propagator};

#[derive(Debug, Error, Eq, PartialEq)]
pub enum LoxValladoError {
    #[error("did not converge")]
    NotConverged,
    #[error(transparent)]
    TrajectoryError(#[from] LoxTrajectoryError),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BaseVallado {
    pub max_iter: i32,
    pub gravitational_parameter: f64,
}

impl Default for BaseVallado {
    fn default() -> Self {
        Self {
            max_iter: 300,
            gravitational_parameter: Earth.gravitational_parameter(),
        }
    }
}

impl BasePropagator for BaseVallado {
    type Error = LoxValladoError;
    type Output = BaseCubicSplineTrajectory;

    fn state_from_delta(
        &self,
        initial_state: (BaseTime, impl BaseTwoBody),
        delta: TimeDelta,
    ) -> Result<BaseState, Self::Error> {
        let (t0, initial_state) = initial_state;
        let mu = self.gravitational_parameter;
        let initial_state = initial_state.to_cartesian_state(mu);
        let dt = delta.to_decimal_seconds();
        let sqrt_mu = mu.sqrt();
        let p0 = initial_state.position();
        let v0 = initial_state.velocity();
        let dot_p0v0 = p0.dot(v0);
        let norm_p0 = p0.length();
        let alpha = -v0.dot(v0) / mu + 2.0 / norm_p0;

        let mut xi_new = if alpha > 0.0 {
            sqrt_mu * dt * alpha
        } else if alpha < 0.0 {
            dt.signum()
                * (-1.0 / alpha).powf(0.5)
                * (-2.0 * mu * alpha * dt
                    / (dot_p0v0 + dt.signum() * (-mu / alpha).sqrt() * (1.0 - norm_p0 * alpha)))
                    .ln()
        } else {
            sqrt_mu * dt / norm_p0
        };

        let mut count = 0;
        while count < self.max_iter {
            let xi = xi_new;
            let psi = xi * xi * alpha;
            let c2_psi = stumpff::c2(psi);
            let c3_psi = stumpff::c3(psi);
            let norm_r = xi.powi(2) * c2_psi
                + dot_p0v0 / sqrt_mu * xi * (1.0 - psi * c3_psi)
                + norm_p0 * (1.0 - psi * c2_psi);
            let delta_xi = (sqrt_mu * dt
                - xi.powi(3) * c3_psi
                - dot_p0v0 / sqrt_mu * xi.powi(2) * c2_psi
                - norm_p0 * xi * (1.0 - psi * c3_psi))
                / norm_r;
            xi_new = xi + delta_xi;
            if (xi_new - xi).abs() < 1e-7 {
                let f = 1.0 - xi.powi(2) / norm_p0 * c2_psi;
                let g = dt - xi.powi(3) / sqrt_mu * c3_psi;

                let gdot = 1.0 - xi.powi(2) / norm_r * c2_psi;
                let fdot = sqrt_mu / (norm_r * norm_p0) * xi * (psi * c3_psi - 1.0);

                debug_assert!((f * gdot - fdot * g - 1.0).abs() < 1e-5);

                let p = f * p0 + g * v0;
                let v = fdot * p0 + gdot * v0;

                let t = t0 + delta;
                let final_state = BaseCartesian::new(p, v);
                return Ok((t, final_state));
            } else {
                count += 1
            }
        }
        Err(LoxValladoError::NotConverged)
    }

    fn state_from_time(
        &self,
        initial_state: (BaseTime, impl BaseTwoBody),
        time: BaseTime,
    ) -> Result<BaseState, Self::Error> {
        let (t0, _) = initial_state;
        let delta = t0 - time;
        self.state_from_delta(initial_state, delta)
    }

    fn trajectory_from_deltas(
        &self,
        initial_state: (BaseTime, impl BaseTwoBody),
        deltas: &[TimeDelta],
    ) -> Result<Self::Output, Self::Error> {
        let (t0, s) = initial_state;
        let mut s = (t0, s.to_cartesian_state(self.gravitational_parameter));
        let mut states: Vec<BaseState> = vec![s];
        for &delta in deltas {
            s = self.state_from_delta(s, delta)?;
            states.push(s);
        }
        let trajectory = BaseCubicSplineTrajectory::new(&states)?;
        Ok(trajectory)
    }

    fn trajectory_from_times(
        &self,
        initial_state: (BaseTime, impl BaseTwoBody),
        times: &[BaseTime],
    ) -> Result<Self::Output, Self::Error> {
        let (t0, s) = initial_state;
        let deltas: Vec<TimeDelta> = times.iter().map(|&t| t - t0).collect();
        let mut s = (t0, s.to_cartesian_state(self.gravitational_parameter));
        let mut states: Vec<BaseState> = vec![s];
        for delta in deltas {
            s = self.state_from_delta(s, delta)?;
            states.push(s);
        }
        let trajectory = BaseCubicSplineTrajectory::new(&states)?;
        Ok(trajectory)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Vallado {
    pub max_iter: i32,
}

impl Vallado {
    fn to_base(self, origin: impl PointMass) -> BaseVallado {
        let gravitational_parameter = origin.gravitational_parameter();
        BaseVallado {
            max_iter: self.max_iter,
            gravitational_parameter,
        }
    }
}

impl Default for Vallado {
    fn default() -> Self {
        Self { max_iter: 300 }
    }
}

impl<T, O, F> Propagator<T, O, F> for Vallado
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    type Error = LoxValladoError;

    fn state_from_delta(
        &self,
        initial_state: Cartesian<T, O, F>,
        delta: TimeDelta,
    ) -> Result<Cartesian<T, O, F>, Self::Error> {
        let base = self.to_base(initial_state.origin());
        let final_state = base.state_from_delta(initial_state.base(), delta)?;
        Ok(Cartesian::from_base(
            initial_state.time().scale(),
            initial_state.origin(),
            initial_state.reference_frame(),
            final_state,
        ))
    }

    fn state_from_time(
        &self,
        initial_state: Cartesian<T, O, F>,
        time: lox_time::Time<T>,
    ) -> Result<Cartesian<T, O, F>, Self::Error> {
        let delta = time - initial_state.time();
        self.state_from_delta(initial_state, delta)
    }

    fn trajectory_from_deltas(
        &self,
        initial_state: Cartesian<T, O, F>,
        deltas: &[TimeDelta],
    ) -> Result<impl Trajectory<T, O, F>, Self::Error> {
        let base = self.to_base(initial_state.origin());
        let trajectory: BaseCubicSplineTrajectory =
            base.trajectory_from_deltas(initial_state.base(), deltas)?;
        Ok(CubicSplineTrajectory::from_base(
            initial_state.time().scale(),
            initial_state.origin(),
            initial_state.reference_frame(),
            trajectory,
        ))
    }

    fn trajectory_from_times(
        &self,
        initial_state: Cartesian<T, O, F>,
        times: &[lox_time::Time<T>],
    ) -> Result<impl Trajectory<T, O, F>, Self::Error> {
        let base = self.to_base(initial_state.origin());
        let times: Vec<BaseTime> = times.iter().map(|t| t.base_time()).collect();
        let trajectory: BaseCubicSplineTrajectory =
            base.trajectory_from_times(initial_state.base(), &times)?;
        Ok(CubicSplineTrajectory::from_base(
            initial_state.time().scale(),
            initial_state.origin(),
            initial_state.reference_frame(),
            trajectory,
        ))
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use crate::frames::Icrf;
    use crate::two_body::{Keplerian, TwoBody};
    use lox_time::calendar_dates::Date;
    use lox_time::subsecond::Subsecond;
    use lox_time::time_scales::Tdb;
    use lox_time::utc::Utc;
    use lox_time::Time;

    use super::*;

    #[test]
    fn test_vallado_propagator() {
        let date = Date::new(2023, 3, 25).expect("Date should be valid");
        let utc = Utc::new(21, 8, 0, Subsecond::new(0.0).unwrap()).expect("Time should be valid");
        let time = Time::from_date_and_utc_timestamp(Tdb, date, utc);
        let semi_major = 24464560.0e-3;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;

        let k0 = Keplerian::new(
            time,
            Earth,
            Icrf,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );
        let s0 = k0.to_cartesian();
        let dt = TimeDelta::from_decimal_seconds(k0.orbital_period()).expect("should be valid");
        let t1 = time + dt;

        let propagator = Vallado::default();
        let s1 = propagator
            .state_from_time(s0, t1)
            .expect("propagator should converge");

        let k1 = s1.to_keplerian();
        assert_float_eq!(k1.semi_major_axis(), semi_major, rel <= 1e-8);
        assert_float_eq!(k1.eccentricity(), eccentricity, rel <= 1e-8);
        assert_float_eq!(k1.inclination(), inclination, rel <= 1e-8);
        assert_float_eq!(k1.ascending_node(), ascending_node, rel <= 1e-8);
        assert_float_eq!(k1.periapsis_argument(), periapsis_arg, rel <= 1e-8);
        assert_float_eq!(k1.true_anomaly(), true_anomaly, rel <= 1e-8);
    }
}
