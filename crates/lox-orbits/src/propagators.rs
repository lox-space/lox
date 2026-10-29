// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::CoordinateOrigin;
use lox_frames::ReferenceFrame;
use lox_time::Time;
use lox_time::intervals::TimeInterval;

use crate::orbits::{CartesianOrbit, Trajectory, TrajectoryError};

use self::j2::{J2Error, J2Propagator};
use self::j4::{J4Error, J4Propagator};
use self::numerical::{NumericalError, NumericalPropagator};
use self::semi_analytical::{Vallado, ValladoError};
use self::sgp4::{Sgp4, Sgp4Error};

/// Analytical J2 orbit propagators (Kozai secular ± Kwok short-period).
pub mod j2;
/// Analytical J4 orbit propagators (Kozai secular ± Kwok short-period).
pub mod j4;
/// Shared math for Kozai-based analytical propagators.
pub mod kozai;
/// Numerical orbit propagators (e.g. J2 perturbation via ODE integration).
pub mod numerical;
/// Semi-analytical orbit propagators (e.g. Vallado universal variable method).
pub mod semi_analytical;
/// SGP4 orbit propagator for TLE-based satellite prediction.
pub mod sgp4;
mod stumpff;

/// Common interface for orbit propagators.
pub trait Propagator<O>
where
    O: CoordinateOrigin + Copy,
{
    /// The propagator's native reference frame.
    type Frame: ReferenceFrame + Copy;
    /// The error type returned by propagation methods.
    type Error: std::error::Error + 'static;

    /// Evaluate the state at a single time.
    fn state_at(&self, time: Time) -> Result<CartesianOrbit<O, Self::Frame>, Self::Error>;

    /// Propagate over the given interval in the native frame.
    /// The propagator chooses the time steps.
    fn propagate(&self, interval: TimeInterval) -> Result<Trajectory<O, Self::Frame>, Self::Error>;

    /// Propagate to an iterable of caller-chosen times.
    fn propagate_to(
        &self,
        times: impl IntoIterator<Item = Time>,
    ) -> Result<Trajectory<O, Self::Frame>, Self::Error>
    where
        Self::Error: From<TrajectoryError>,
    {
        let states: Result<Vec<_>, _> = times.into_iter().map(|t| self.state_at(t)).collect();
        Ok(Trajectory::try_new(states?)?)
    }
}

/// An orbit source that can be propagated over a time interval to produce
/// a [`Trajectory`].
///
/// Wraps the concrete propagator types (SGP4, Vallado, Numerical) or a
/// pre-computed trajectory.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::large_enum_variant)]
pub enum OrbitSource {
    /// SGP4 propagator initialized from a TLE.
    Sgp4(Sgp4),
    /// Vallado universal-variable Keplerian propagator.
    Vallado(Vallado),
    /// Numerical orbit propagator.
    Numerical(NumericalPropagator),
    /// Kozai J2 propagator (secular, optionally osculating).
    J2(J2Propagator),
    /// Kozai J4 propagator (secular, optionally osculating).
    J4(J4Propagator),
    /// Pre-computed trajectory used as-is.
    Trajectory(Trajectory),
}

/// Errors that can occur when propagating an [`OrbitSource`].
#[derive(Debug, thiserror::Error)]
pub enum PropagateError {
    /// SGP4 propagation error.
    #[error(transparent)]
    Sgp4(#[from] Sgp4Error),
    /// Vallado propagation error.
    #[error(transparent)]
    Vallado(#[from] ValladoError),
    /// Numerical propagation error.
    #[error(transparent)]
    Numerical(#[from] NumericalError),
    /// J2 propagation error.
    #[error(transparent)]
    J2(#[from] J2Error),
    /// J4 propagation error.
    #[error(transparent)]
    J4(#[from] J4Error),
    /// A pre-computed trajectory does not cover the requested interval.
    #[error("trajectory from {start} to {end} does not cover the interval {interval}")]
    NotCovered {
        /// The start time of the trajectory.
        start: Time,
        /// The end time of the trajectory.
        end: Time,
        /// The requested interval.
        interval: TimeInterval,
    },
}

impl OrbitSource {
    /// Propagate the orbit source over the given interval, returning a
    /// [`Trajectory`] in the source's native reference frame.
    ///
    /// # Errors
    ///
    /// Returns [`PropagateError::NotCovered`] if a pre-computed trajectory
    /// does not cover `interval`, allowing for its
    /// [extrapolation limit](Trajectory::extrapolation_limit).
    pub fn propagate(&self, interval: TimeInterval) -> Result<Trajectory, PropagateError> {
        match self {
            Self::Sgp4(sgp4) => Ok(Propagator::propagate(sgp4, interval)?.into_dynamic()),
            Self::Vallado(v) => Ok(Propagator::propagate(v, interval)?),
            Self::Numerical(n) => Ok(Propagator::propagate(n, interval)?),
            Self::J2(p) => Ok(Propagator::propagate(p, interval)?),
            Self::J4(p) => Ok(Propagator::propagate(p, interval)?),
            Self::Trajectory(t) => {
                let scale = t.start_time().scale();
                let covers = |time: Time| t.at(time.to_scale(scale)).is_some();
                if !covers(interval.start()) || !covers(interval.end()) {
                    return Err(PropagateError::NotCovered {
                        start: t.start_time(),
                        end: t.end_time(),
                        interval,
                    });
                }
                Ok(t.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_bodies::Origin;
    use lox_frames::Frame;
    use lox_time::deltas::TimeDelta;
    use lox_time::time_scales::TimeScale;

    fn make_trajectory() -> Trajectory {
        Trajectory::from_csv_dynamic(
            &lox_test_utils::read_data_file("trajectory_lunar.csv"),
            Origin::Earth,
            Frame::Icrf,
        )
        .unwrap()
    }

    #[test]
    fn test_orbit_source_trajectory_propagate() {
        let traj = make_trajectory();
        let interval = TimeInterval::new(
            traj.start_time().to_scale(TimeScale::Tai),
            traj.end_time().to_scale(TimeScale::Tai),
        );
        let source = OrbitSource::Trajectory(traj.clone());
        let result = source.propagate(interval).unwrap();
        assert_eq!(result.states().len(), traj.states().len());
    }

    #[test]
    fn test_orbit_source_trajectory_propagate_not_covered() {
        let traj = make_trajectory();
        let start = traj.start_time().to_scale(TimeScale::Tai);
        let end = traj.end_time().to_scale(TimeScale::Tai);
        let source = OrbitSource::Trajectory(traj.clone());

        // Within the extrapolation limit.
        let slack = TimeDelta::from_microseconds(500);
        assert!(
            source
                .propagate(TimeInterval::new(start - slack, end + slack))
                .is_ok()
        );

        let interval = TimeInterval::new(start, end + TimeDelta::from_seconds(1));
        let err = source.propagate(interval).unwrap_err();
        assert!(matches!(
            err,
            PropagateError::NotCovered { interval: i, .. } if i == interval
        ));
        assert!(err.to_string().contains("does not cover"));
    }
}
