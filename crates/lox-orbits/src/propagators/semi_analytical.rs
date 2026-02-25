// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_core::coords::Cartesian;
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::{DynTimeScale, TimeScale};
use thiserror::Error;

use lox_bodies::{DynOrigin, Origin, PointMass, TryPointMass, UndefinedOriginPropertyError};

use crate::orbits::{CartesianOrbit, TrajectorError, Trajectory};
use crate::propagators::{Propagator, stumpff};
use lox_frames::{
    DynFrame, NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame, TryQuasiInertial,
};

#[derive(Debug, Error)]
pub enum ValladoError {
    #[error("did not converge")]
    NotConverged,
    #[error(transparent)]
    TrajectoryError(#[from] TrajectorError),
    #[error(transparent)]
    UndefinedOriginProperty(#[from] UndefinedOriginPropertyError),
    #[error(transparent)]
    NonQuasiInertialFrame(#[from] NonQuasiInertialFrameError),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vallado<T: TimeScale, O: Origin, R: ReferenceFrame> {
    initial_state: CartesianOrbit<T, O, R>,
    max_iter: i32,
    step: Option<TimeDelta>,
}

pub type DynVallado = Vallado<DynTimeScale, DynOrigin, DynFrame>;

// Infallible — static bounds guarantee inertial frame and point mass.
impl<T, O, R> Vallado<T, O, R>
where
    T: TimeScale,
    O: PointMass + Copy,
    R: QuasiInertial,
{
    pub fn new(initial_state: CartesianOrbit<T, O, R>) -> Self {
        Self {
            initial_state,
            max_iter: 300,
            step: None,
        }
    }
}

// Fallible — Try* bounds (covers DynOrigin and DynFrame).
impl<T, O, R> Vallado<T, O, R>
where
    T: TimeScale,
    O: TryPointMass + Copy,
    R: TryQuasiInertial + Copy,
{
    pub fn try_new(initial_state: CartesianOrbit<T, O, R>) -> Result<Self, ValladoError> {
        initial_state.origin().try_gravitational_parameter()?;
        initial_state.reference_frame().try_quasi_inertial()?;
        Ok(Self {
            initial_state,
            max_iter: 300,
            step: None,
        })
    }
}

impl<T, O, R> Vallado<T, O, R>
where
    T: TimeScale,
    O: TryPointMass + Copy,
    R: ReferenceFrame,
{
    fn gravitational_parameter(&self) -> f64 {
        self.initial_state
            .origin()
            .try_gravitational_parameter()
            .expect("gravitational parameter should be available")
            .as_f64()
    }

    pub fn with_max_iter(mut self, max_iter: i32) -> Self {
        self.max_iter = max_iter;
        self
    }

    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = Some(step);
        self
    }

    pub fn origin(&self) -> O {
        self.initial_state.origin()
    }

    pub fn reference_frame(&self) -> R
    where
        R: Copy,
    {
        self.initial_state.reference_frame()
    }

    /// Propagate to a single time.
    pub fn state_at(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, R>, ValladoError>
    where
        T: Copy,
        R: Copy,
    {
        let frame = self.reference_frame();
        let origin = self.origin();
        let mu = self.gravitational_parameter();
        let t0 = self.initial_state.time();
        let dt = (time - t0).to_seconds().to_f64();
        let sqrt_mu = mu.sqrt();
        let p0 = self.initial_state.position();
        let v0 = self.initial_state.velocity();
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

                return Ok(CartesianOrbit::new(
                    Cartesian::from_vecs(p, v),
                    time,
                    origin,
                    frame,
                ));
            } else {
                count += 1
            }
        }
        Err(ValladoError::NotConverged)
    }
}

// Single impl covers both typed and DynVallado
impl<T, O, R> Propagator<T, O> for Vallado<T, O, R>
where
    T: TimeScale + Copy + PartialOrd,
    O: TryPointMass + Copy,
    R: ReferenceFrame + Copy,
{
    type Frame = R;
    type Error = ValladoError;

    fn state_at(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, R>, ValladoError> {
        self.state_at(time)
    }

    fn propagate(&self, interval: TimeInterval<T>) -> Result<Trajectory<T, O, R>, ValladoError> {
        let step = self.step.unwrap_or(TimeDelta::from_seconds(1));
        let states = interval
            .step_by(step)
            .map(|t| self.state_at(t))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Trajectory::try_new(states)?)
    }
}

#[cfg(test)]
mod tests {
    use lox_bodies::Earth;
    use lox_core::elements::Keplerian as CoreKeplerian;
    use lox_core::units::{AngleUnits, DistanceUnits};
    use lox_frames::Icrf;
    use lox_test_utils::assert_approx_eq;
    use lox_time::intervals::Interval;
    use lox_time::time_scales::Tdb;
    use lox_time::utc;
    use lox_time::utc::Utc;

    use super::*;

    #[test]
    fn test_vallado_state_at() {
        let utc = utc!(2023, 3, 25, 21, 8, 0.0).unwrap();
        let time = utc.to_time().to_scale(Tdb);
        let semi_major = 24464.560;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;

        let k0 = CoreKeplerian::builder()
            .with_semi_major_axis(semi_major.km(), eccentricity)
            .with_inclination(inclination.rad())
            .with_longitude_of_ascending_node(ascending_node.rad())
            .with_argument_of_periapsis(periapsis_arg.rad())
            .with_true_anomaly(true_anomaly.rad())
            .build()
            .unwrap();

        let mu = Earth.gravitational_parameter();
        let s0 = CartesianOrbit::new(k0.to_cartesian(mu), time, Earth, Icrf);
        let period = k0.orbital_period(mu).unwrap();
        let t1 = time + period;

        let propagator = Vallado::new(s0);
        let s1 = propagator.state_at(t1).expect("propagator should converge");

        let k1 = s1.to_keplerian();
        assert_approx_eq!(
            k1.semi_major_axis().as_f64(),
            semi_major.km().as_f64(),
            rtol <= 1e-8
        );
        assert_approx_eq!(k1.eccentricity().as_f64(), eccentricity, rtol <= 1e-8);
        assert_approx_eq!(k1.inclination().as_f64(), inclination, rtol <= 1e-8);
        assert_approx_eq!(
            k1.longitude_of_ascending_node().as_f64(),
            ascending_node,
            rtol <= 1e-8
        );
        assert_approx_eq!(
            k1.argument_of_periapsis().as_f64(),
            periapsis_arg,
            rtol <= 1e-8
        );
        assert_approx_eq!(k1.true_anomaly().as_f64(), true_anomaly, rtol <= 1e-8);
        assert_approx_eq!(k1.time(), t1);
    }

    #[test]
    fn test_vallado_propagate() {
        let utc = utc!(2023, 3, 25, 21, 8, 0.0).unwrap();
        let time = utc.to_time().to_scale(Tdb);
        let semi_major = 24464.560;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;

        let k0 = CoreKeplerian::builder()
            .with_semi_major_axis(semi_major.km(), eccentricity)
            .with_inclination(inclination.rad())
            .with_longitude_of_ascending_node(ascending_node.rad())
            .with_argument_of_periapsis(periapsis_arg.rad())
            .with_true_anomaly(true_anomaly.rad())
            .build()
            .unwrap();

        let mu = Earth.gravitational_parameter();
        let s0 = CartesianOrbit::new(k0.to_cartesian(mu), time, Earth, Icrf);
        let period = k0.orbital_period(mu).unwrap();
        let t_end = time + period;
        let interval = Interval::new(time, t_end);
        let trajectory = Vallado::new(s0).propagate(interval).unwrap();
        let s1 = trajectory.interpolate(period);
        let k1 = s1.to_keplerian();

        assert_approx_eq!(
            k1.semi_major_axis().as_f64(),
            semi_major.km().as_f64(),
            rtol <= 1e-8
        );
        assert_approx_eq!(k1.eccentricity().as_f64(), eccentricity, rtol <= 1e-8);
        assert_approx_eq!(k1.inclination().as_f64(), inclination, rtol <= 1e-8);
        assert_approx_eq!(
            k1.longitude_of_ascending_node().as_f64(),
            ascending_node,
            rtol <= 1e-8
        );
        assert_approx_eq!(
            k1.argument_of_periapsis().as_f64(),
            periapsis_arg,
            rtol <= 1e-8
        );
        assert_approx_eq!(k1.true_anomaly().as_f64(), true_anomaly, rtol <= 1e-8);
    }

    #[test]
    fn test_try_new_with_static_types() {
        let utc = utc!(2023, 3, 25, 21, 8, 0.0).unwrap();
        let time = utc.to_time().to_scale(Tdb);
        let pos = glam::DVec3::new(-1076225.32, -6765896.36, -332308.78);
        let vel = glam::DVec3::new(9356.86, -3312.35, -1188.02);
        let s0 = CartesianOrbit::new(
            lox_core::coords::Cartesian::from_vecs(pos, vel),
            time,
            Earth,
            Icrf,
        );
        // try_new should accept static types that implement TryPointMass
        let vallado = Vallado::try_new(s0);
        assert!(vallado.is_ok());
    }

    #[test]
    fn test_try_new_rejects_non_point_mass() {
        use lox_bodies::DynOrigin;

        let utc = utc!(2023, 3, 25, 21, 8, 0.0).unwrap();
        let time = utc.to_dyn_time();
        let pos = glam::DVec3::new(-1076225.32, -6765896.36, -332308.78);
        let vel = glam::DVec3::new(9356.86, -3312.35, -1188.02);
        let s0 = CartesianOrbit::new(
            lox_core::coords::Cartesian::from_vecs(pos, vel),
            time,
            DynOrigin::Callirrhoe,
            DynFrame::Icrf,
        );
        let result = Vallado::try_new(s0);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_new_rejects_non_inertial_frame() {
        use lox_bodies::DynOrigin;

        let utc = utc!(2023, 3, 25, 21, 8, 0.0).unwrap();
        let time = utc.to_dyn_time();
        let pos = glam::DVec3::new(-1076225.32, -6765896.36, -332308.78);
        let vel = glam::DVec3::new(9356.86, -3312.35, -1188.02);
        let s0 = CartesianOrbit::new(
            lox_core::coords::Cartesian::from_vecs(pos, vel),
            time,
            DynOrigin::Earth,
            DynFrame::Iau(DynOrigin::Earth),
        );
        let result = Vallado::try_new(s0);
        assert!(result.is_err());
    }
}
