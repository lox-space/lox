// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::Origin;
use lox_frames::ReferenceFrame;
use lox_time::Time;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::{DynTimeScale, TimeScale};

use crate::orbits::{CartesianOrbit, DynTrajectory, TrajectorError, Trajectory};

use self::numerical::{DynJ2Propagator, J2Error};
use self::semi_analytical::{DynVallado, ValladoError};
use self::sgp4::{Sgp4, Sgp4Error};

pub mod numerical;
pub mod semi_analytical;
pub mod sgp4;
mod stumpff;

pub trait Propagator<T, O>
where
    T: TimeScale + Copy,
    O: Origin + Copy,
{
    /// The propagator's native reference frame.
    type Frame: ReferenceFrame + Copy;
    type Error: std::error::Error + 'static;

    /// Evaluate the state at a single time.
    fn state_at(&self, time: Time<T>) -> Result<CartesianOrbit<T, O, Self::Frame>, Self::Error>;

    /// Propagate over the given interval in the native frame.
    /// The propagator chooses the time steps.
    fn propagate(
        &self,
        interval: TimeInterval<T>,
    ) -> Result<Trajectory<T, O, Self::Frame>, Self::Error>;

    /// Propagate to an iterable of caller-chosen times.
    fn propagate_to(
        &self,
        times: impl IntoIterator<Item = Time<T>>,
    ) -> Result<Trajectory<T, O, Self::Frame>, Self::Error>
    where
        Self::Error: From<TrajectorError>,
    {
        let states: Result<Vec<_>, _> = times.into_iter().map(|t| self.state_at(t)).collect();
        Ok(Trajectory::try_new(states?)?)
    }
}

/// An orbit source that can be propagated over a time interval to produce
/// a [`DynTrajectory`].
///
/// Wraps the concrete propagator types (SGP4, Vallado, J2) or a pre-computed
/// trajectory.
#[derive(Debug, Clone)]
pub enum OrbitSource {
    Sgp4(Sgp4),
    Vallado(DynVallado),
    J2(DynJ2Propagator),
    Trajectory(DynTrajectory),
}

#[derive(Debug, thiserror::Error)]
pub enum PropagateError {
    #[error(transparent)]
    Sgp4(#[from] Sgp4Error),
    #[error(transparent)]
    Vallado(#[from] ValladoError),
    #[error(transparent)]
    J2(#[from] J2Error),
}

impl OrbitSource {
    /// Propagate the orbit source over the given interval, returning a
    /// [`DynTrajectory`] in the source's native reference frame.
    pub fn propagate(
        &self,
        interval: TimeInterval<DynTimeScale>,
    ) -> Result<DynTrajectory, PropagateError> {
        match self {
            Self::Sgp4(sgp4) => {
                let tai_interval = TimeInterval::new(
                    interval.start().to_scale(lox_time::time_scales::Tai),
                    interval.end().to_scale(lox_time::time_scales::Tai),
                );
                let traj = Propagator::propagate(sgp4, tai_interval)?;
                Ok(traj.into_dyn())
            }
            Self::Vallado(v) => Ok(Propagator::propagate(v, interval)?),
            Self::J2(j2) => Ok(Propagator::propagate(j2, interval)?),
            Self::Trajectory(t) => Ok(t.clone()),
        }
    }
}
