/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_core::bodies::PointMass;
use lox_core::coords::two_body::Cartesian;
use lox_core::coords::CoordinateSystem;
use lox_core::frames::ReferenceFrame;

use crate::{stumpff, Propagator};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Vallado {
    pub max_iter: i32,
}

impl Default for Vallado {
    fn default() -> Self {
        Self { max_iter: 300 }
    }
}

impl<T: PointMass + Copy, S: ReferenceFrame + Copy> Propagator<T, S> for Vallado {
    type Error = &'static str;

    fn state_from_delta(
        &self,
        initial_state: Cartesian<T, S>,
        delta: f64,
    ) -> Result<Cartesian<T, S>, Self::Error> {
        let dt = delta;
        let mu = initial_state.origin().gravitational_parameter();
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
            let delta = (sqrt_mu * dt
                - xi.powi(3) * c3_psi
                - dot_p0v0 / sqrt_mu * xi.powi(2) * c2_psi
                - norm_p0 * xi * (1.0 - psi * c3_psi))
                / norm_r;
            xi_new = xi + delta;
            if (xi_new - xi).abs() < 1e-7 {
                let f = 1.0 - xi.powi(2) / norm_p0 * c2_psi;
                let g = dt - xi.powi(3) / sqrt_mu * c3_psi;

                let gdot = 1.0 - xi.powi(2) / norm_r * c2_psi;
                let fdot = sqrt_mu / (norm_r * norm_p0) * xi * (psi * c3_psi - 1.0);

                debug_assert!((f * gdot - fdot * g - 1.0).abs() < 1e-5);

                let p = f * p0 + g * v0;
                let v = fdot * p0 + gdot * v0;
                // FIXME: This is wrong
                let t = initial_state.time();
                let final_state = Cartesian::new(
                    t,
                    initial_state.origin(),
                    initial_state.reference_frame(),
                    p,
                    v,
                );
                return Ok(final_state);
            } else {
                count += 1
            }
        }
        Err("did not converge")
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use lox_core::bodies::Earth;
    use lox_core::coords::two_body::{Keplerian, TwoBody};
    use lox_core::frames::Icrf;
    use lox_core::time::continuous::{Time, TimeScale};
    use lox_core::time::dates::Date;
    use lox_core::time::utc::UTC;

    use super::*;

    #[test]
    fn test_vallado_propagator() {
        let date = Date::new(2023, 3, 25).expect("Date should be valid");
        let utc = UTC::new(21, 8, 0).expect("Time should be valid");
        let time = Time::from_date_and_utc_timestamp(TimeScale::TDB, date, utc);
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
        let dt = k0.orbital_period();

        let propagator = Vallado::default();
        let s1 = propagator
            .state_from_delta(s0, dt)
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
