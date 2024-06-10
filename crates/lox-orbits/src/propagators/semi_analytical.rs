/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use thiserror::Error;

use lox_bodies::PointMass;
use lox_time::deltas::TimeDelta;
use lox_time::TimeLike;

use crate::frames::{CoordinateSystem, Icrf};
use crate::origins::{CoordinateOrigin, Origin};
use crate::propagators::{stumpff, Propagator};
use crate::states::State;
use crate::trajectories::{Trajectory, TrajectoryError};

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ValladoError {
    #[error("did not converge")]
    NotConverged,
    #[error(transparent)]
    TrajectoryError(#[from] TrajectoryError),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vallado<O: Origin> {
    pub origin: O,
    pub max_iter: i32,
}

impl<O: Origin + Clone> CoordinateOrigin<O> for Vallado<O> {
    fn origin(&self) -> O {
        self.origin.clone()
    }
}

impl<O: Origin> CoordinateSystem<Icrf> for Vallado<O> {
    fn reference_frame(&self) -> Icrf {
        Icrf
    }
}

impl<O: Origin + PointMass> Vallado<O> {
    pub fn new(origin: O) -> Self {
        Self {
            origin,
            max_iter: 300,
        }
    }

    pub fn with_max_iter(&mut self, max_iter: i32) -> &mut Self {
        self.max_iter = max_iter;
        self
    }
}

impl<T, O> Propagator<T, O, Icrf> for Vallado<O>
where
    T: TimeLike + Clone,
    O: Origin + PointMass + Clone,
{
    type Error = ValladoError;

    fn state_from_delta(
        &self,
        initial_state: State<T, O, Icrf>,
        delta: TimeDelta,
    ) -> Result<State<T, O, Icrf>, Self::Error> {
        let frame = self.reference_frame();
        let origin = self.origin();
        let mu = origin.gravitational_parameter();
        let t0 = initial_state.time();
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
                return Ok(State::new(t, origin, frame, p, v));
            } else {
                count += 1
            }
        }
        Err(ValladoError::NotConverged)
    }

    fn trajectory_from_deltas(
        &self,
        initial_state: State<T, O, Icrf>,
        deltas: &[TimeDelta],
    ) -> Result<Trajectory<T, O, Icrf>, Self::Error> {
        let mut s = initial_state.clone();
        let mut states: Vec<State<T, O, Icrf>> = vec![s.clone()];
        for &delta in deltas {
            s = self.state_from_delta(s, delta)?;
            states.push(s.clone());
        }
        let trajectory = Trajectory::new(&states)?;
        Ok(trajectory)
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use lox_bodies::Earth;

    use crate::elements::{Keplerian, ToKeplerian};
    use lox_time::transformations::ToTdb;
    use lox_time::utc;
    use lox_time::utc::Utc;

    use crate::states::ToCartesian;

    use super::*;

    #[test]
    fn test_vallado_propagator() {
        let utc = utc!(2023, 3, 25, 21, 8, 0.0).unwrap();
        let time = utc.to_tdb();
        let semi_major = 24464.560;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;

        let k0 = Keplerian::new(
            time,
            Earth,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );
        let s0 = k0.to_cartesian();
        let dt = k0.orbital_period();
        let t1 = time + dt;

        let propagator = Vallado::new(Earth);
        let s1 = propagator
            .state_from_time(s0, t1)
            .expect("propagator should converge");

        let k1 = s1.to_keplerian();
        assert_float_eq!(k1.semi_major_axis(), semi_major, rel <= 1e-8);
        assert_float_eq!(k1.eccentricity(), eccentricity, rel <= 1e-8);
        assert_float_eq!(k1.inclination(), inclination, rel <= 1e-8);
        assert_float_eq!(
            k1.longitude_of_ascending_node(),
            ascending_node,
            rel <= 1e-8
        );
        assert_float_eq!(k1.argument_of_periapsis(), periapsis_arg, rel <= 1e-8);
        assert_float_eq!(k1.true_anomaly(), true_anomaly, rel <= 1e-8);
    }
}
