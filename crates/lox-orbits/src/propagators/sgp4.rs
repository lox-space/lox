// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_core::coords::Cartesian;
use lox_core::glam::DVec3;
pub use sgp4::Elements;
use sgp4::{Constants, ElementsError, KozaiElementsError, MinutesSinceEpoch};
use thiserror::Error;

use lox_bodies::Earth;
use lox_core::f64::consts::SECONDS_PER_MINUTE;
use lox_frames::Teme;
use lox_time::Time;
use lox_time::deltas::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::Tai;

use crate::orbits::{CartesianOrbit, Trajectory, TrajectoryError};
use crate::propagators::Propagator;

/// Errors that can occur during SGP4 propagation.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Sgp4Error {
    /// The elements are unphysical and no propagator could be built.
    #[error("invalid SGP4 elements: {0}")]
    InvalidElements(InvalidElements),
    /// Propagation diverged, typically for a decayed or badly fitted object.
    #[error("SGP4 propagation diverged {minutes_since_epoch} minutes after epoch: {kind}")]
    Diverged {
        /// How the propagation diverged.
        kind: Divergence,
        /// Minutes since the element epoch at which propagation failed.
        minutes_since_epoch: f64,
    },
    /// Error constructing the output trajectory.
    #[error(transparent)]
    TrajectoryError(#[from] TrajectoryError),
}

/// Reasons SGP4 rejects a set of elements when the propagator is constructed.
#[derive(Debug, Clone, Copy, PartialEq, Error)]
#[non_exhaustive]
pub enum InvalidElements {
    /// The Kozai mean motion derived from the elements is not positive.
    #[error("the Kozai mean motion at epoch is not positive")]
    NegativeKozaiMeanMotion,
    /// The Brouwer mean motion derived from the elements is not positive.
    #[error("the Brouwer mean motion at epoch is not positive")]
    NegativeBrouwerMeanMotion,
    /// The eccentricity at epoch lies outside `[0, 1)`.
    #[error("the epoch eccentricity ({0}) is outside the range [0, 1)")]
    EpochEccentricityOutOfRange(f64),
}

/// Ways in which SGP4 propagation can diverge.
#[derive(Debug, Clone, Copy, PartialEq, Error)]
#[non_exhaustive]
pub enum Divergence {
    /// The propagated eccentricity left `[0, 1)` before third-body perturbations.
    #[error("the eccentricity ({0}) left the range [0, 1) before third-body perturbations")]
    EccentricityOutOfRange(f64),
    /// The propagated eccentricity left `[0, 1)` after third-body perturbations.
    #[error("the eccentricity ({0}) left the range [0, 1) after third-body perturbations")]
    PerturbedEccentricityOutOfRange(f64),
    /// The propagated semi-latus rectum became negative.
    #[error("the semi-latus rectum became negative")]
    NegativeSemiLatusRectum,
}

impl From<ElementsError> for Sgp4Error {
    fn from(err: ElementsError) -> Self {
        Sgp4Error::InvalidElements(match err {
            ElementsError::KozaiElementsError(KozaiElementsError::NegativeKozaiMeanMotion) => {
                InvalidElements::NegativeKozaiMeanMotion
            }
            ElementsError::KozaiElementsError(KozaiElementsError::NegativeBrouwerMeanMotion) => {
                InvalidElements::NegativeBrouwerMeanMotion
            }
            ElementsError::OutOfRangeEpochEccentricity(e) => {
                InvalidElements::EpochEccentricityOutOfRange(e.0)
            }
        })
    }
}

impl From<sgp4::Error> for Sgp4Error {
    fn from(err: sgp4::Error) -> Self {
        let (kind, minutes_since_epoch) = match err {
            sgp4::Error::OutOfRangeEccentricity { eccentricity, t } => {
                (Divergence::EccentricityOutOfRange(eccentricity), t)
            }
            sgp4::Error::OutOfRangePerturbedEccentricity { eccentricity, t } => {
                (Divergence::PerturbedEccentricityOutOfRange(eccentricity), t)
            }
            sgp4::Error::NegativeSemiLatusRectum { t } => (Divergence::NegativeSemiLatusRectum, t),
        };
        Sgp4Error::Diverged {
            kind,
            minutes_since_epoch,
        }
    }
}

/// SGP4/SDP4 orbit propagator for satellites described by Two-Line Element sets.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sgp4 {
    constants: Constants,
    time: Time<Tai>,
    step: Option<TimeDelta>,
}

impl Sgp4 {
    /// Create a new SGP4 propagator from TLE elements.
    pub fn new(initial_state: Elements) -> Result<Self, Sgp4Error> {
        let time: Time<Tai> = initial_state.datetime.and_utc().into();
        // Use AFSPC compatibility mode because TLE data is fitted using
        // AFSPC constants (WGS72). Using WGS84 with WGS72-fitted data
        // introduces systematic errors.
        let constants = Constants::from_elements_afspc_compatibility_mode(&initial_state)?;
        Ok(Self {
            constants,
            time,
            step: None,
        })
    }

    /// Set the fixed time step used during propagation.
    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = Some(step);
        self
    }

    /// Return the TLE epoch as a TAI time.
    pub fn time(&self) -> Time<Tai> {
        self.time
    }

    /// Propagate to a single time, returning a TEME state.
    pub fn state_at(&self, time: Time<Tai>) -> Result<CartesianOrbit<Earth, Teme>, Sgp4Error> {
        let dt = (time - self.time).as_seconds_f64() / SECONDS_PER_MINUTE;
        let prediction = self.constants.propagate(MinutesSinceEpoch(dt))?;
        // sgp4 crate returns km and km/s, convert to m and m/s
        let position = DVec3::from_array(prediction.position) * 1e3;
        let velocity = DVec3::from_array(prediction.velocity) * 1e3;
        Ok(CartesianOrbit::new(
            Cartesian::from_vecs(position, velocity),
            time.into_dynamic(),
            Earth,
            Teme,
        ))
    }
}

impl Propagator<Earth> for Sgp4 {
    type Frame = Teme;
    type Error = Sgp4Error;

    fn state_at(&self, time: Time) -> Result<CartesianOrbit<Earth, Teme>, Sgp4Error> {
        self.state_at(time.to_scale(Tai))
    }

    fn propagate(&self, interval: TimeInterval) -> Result<Trajectory<Earth, Teme>, Sgp4Error> {
        // SGP4 epoch arithmetic is TAI-based; convert the interval once rather
        // than per step.
        let interval =
            TimeInterval::new(interval.start().to_scale(Tai), interval.end().to_scale(Tai));
        let step = self.step.unwrap_or(TimeDelta::from_seconds(60));
        let states = interval
            .step_by_inclusive(step)
            .map(|t| self.state_at(t))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Trajectory::try_new(states)?)
    }
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use lox_frames::Icrf;
    use lox_frames::providers::DefaultRotationProvider;
    use lox_time::deltas::TimeDelta;
    use lox_time::intervals::Interval;

    use super::*;

    #[test]
    fn test_sgp4_state_at() {
        let tle = Elements::from_tle(
            Some("ISS (ZARYA)".to_string()),
            "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
            "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
        )
        .unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let orbital_period = 92.821;
        let t1 = sgp4.time() + TimeDelta::from_minutes_f64(orbital_period);
        let s1 = sgp4.state_at(t1).unwrap();
        let s1_icrf = s1.try_to_frame(Icrf, &DefaultRotationProvider).unwrap();
        let k1 = s1_icrf.to_keplerian().unwrap();
        assert_approx_eq!(
            k1.orbital_period().unwrap().as_seconds_f64() / SECONDS_PER_MINUTE,
            orbital_period,
            rtol <= 1e-4
        );
    }

    #[test]
    fn test_sgp4_propagate() {
        let tle = Elements::from_tle(
            Some("ISS (ZARYA)".to_string()),
            "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
            "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
        )
        .unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let t0 = sgp4.time();
        let t1 = t0 + TimeDelta::from_minutes_f64(92.821);
        let interval = Interval::new(t0, t1);
        let traj = sgp4.propagate(interval.into_dynamic()).unwrap();
        // With 60s default step over ~93 min, we should have ~94 points
        assert!(traj.states().len() > 90);
    }

    #[test]
    fn test_sgp4_propagate_in_frame() {
        let tle = Elements::from_tle(
            Some("ISS (ZARYA)".to_string()),
            "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
            "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
        )
        .unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let t0 = sgp4.time();
        let t1 = t0 + TimeDelta::from_minutes(10);
        let interval = Interval::new(t0, t1);
        let traj = sgp4
            .propagate(interval.into_dynamic())
            .unwrap()
            .into_frame(Icrf, &DefaultRotationProvider)
            .unwrap();
        assert!(traj.states().len() > 5);
        assert_eq!(traj.reference_frame(), Icrf);
    }

    /// Propagation diverges 25 minutes after epoch (negative semi-latus rectum).
    const DIVERGING_LINE_1: &str =
        "1 33333U 05037B   05333.02012661  .25992681  00000-0  24476-3 0  1532";
    const DIVERGING_LINE_2: &str =
        "2 33333  96.4736 157.9986 9950000 244.0492 110.6523  4.00004038 10700";

    /// Propagation diverges at epoch (perturbed eccentricity out of range).
    const DIVERGING_AT_EPOCH_LINE_1: &str =
        "1 33334U 78066F   06174.85818871  .00000620  00000-0  10000-3 0  6806";
    const DIVERGING_AT_EPOCH_LINE_2: &str =
        "2 33334  68.4714 236.1303 5602877 123.7484 302.5767  0.00001000 67521";

    fn elements(line_1: &str, line_2: &str) -> Elements {
        Elements::from_tle(None, line_1.as_bytes(), line_2.as_bytes()).unwrap()
    }

    #[test]
    fn test_sgp4_invalid_elements() {
        let mut tle = elements(DIVERGING_LINE_1, DIVERGING_LINE_2);
        tle.mean_motion = 0.0;
        let err = Sgp4::new(tle).unwrap_err();
        assert!(matches!(
            err,
            Sgp4Error::InvalidElements(InvalidElements::NegativeKozaiMeanMotion)
        ));
        assert!(err.to_string().contains("Kozai mean motion"));
    }

    #[test]
    fn test_sgp4_invalid_epoch_eccentricity() {
        let mut tle = elements(DIVERGING_LINE_1, DIVERGING_LINE_2);
        tle.eccentricity = 1.5;
        let err = Sgp4::new(tle).unwrap_err();
        assert!(matches!(
            err,
            Sgp4Error::InvalidElements(InvalidElements::EpochEccentricityOutOfRange(e))
                if e == 1.5
        ));
    }

    #[test]
    fn test_sgp4_diverged() {
        let sgp4 = Sgp4::new(elements(DIVERGING_LINE_1, DIVERGING_LINE_2)).unwrap();
        let t = sgp4.time() + TimeDelta::from_minutes(25);
        let err = sgp4.state_at(t).unwrap_err();
        assert!(matches!(
            err,
            Sgp4Error::Diverged {
                kind: Divergence::NegativeSemiLatusRectum,
                minutes_since_epoch,
            } if (minutes_since_epoch - 25.0).abs() < 1e-9
        ));
        assert!(err.to_string().contains("25 minutes after epoch"));
    }

    #[test]
    fn test_sgp4_diverged_perturbed_eccentricity() {
        let sgp4 = Sgp4::new(elements(
            DIVERGING_AT_EPOCH_LINE_1,
            DIVERGING_AT_EPOCH_LINE_2,
        ))
        .unwrap();
        let err = sgp4.state_at(sgp4.time()).unwrap_err();
        assert!(matches!(
            err,
            Sgp4Error::Diverged {
                kind: Divergence::PerturbedEccentricityOutOfRange(_),
                ..
            }
        ));
    }
}
