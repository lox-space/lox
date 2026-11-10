// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! SGP4 orbit propagator for TLE-based satellite prediction.

use chrono::DateTime;
use lox_core::coords::Cartesian;
use lox_core::glam::DVec3;
use sgp4::{Constants, ElementsError, KozaiElementsError, MinutesSinceEpoch};
use std::f64::consts::TAU;
use thiserror::Error;

use lox_bodies::Earth;
use lox_core::time::constants::{SECONDS_PER_DAY, SECONDS_PER_MINUTE};
#[cfg(feature = "serde")]
use lox_core::units::{Angle, AngularRate};
use lox_frames::Teme;
use lox_time::Time;
use lox_time::TimeDelta;
use lox_time::intervals::TimeInterval;
use lox_time::time_scales::Tai;

use crate::orbits::{CartesianOrbit, Trajectory, TrajectoryError};
use crate::propagators::Propagator;
use crate::tle::{Classification, Tle};

/// Errors that can occur during SGP4 propagation.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Sgp4Error {
    /// The elements are unphysical and no propagator could be built.
    #[error("invalid SGP4 elements: {0}")]
    InvalidElements(InvalidElements),
    /// Propagation diverged, typically for a decayed or badly fitted object.
    #[error("SGP4 propagation diverged {}: {kind}", format_epoch_offset(.time_since_epoch))]
    #[non_exhaustive]
    Diverged {
        /// How the propagation diverged.
        kind: Divergence,
        /// Time since the element epoch at which propagation failed.
        time_since_epoch: TimeDelta,
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
    /// The epoch cannot be represented by the SGP4 implementation.
    #[error("the epoch is outside the supported range")]
    EpochOutOfRange,
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

fn invalid_elements(err: ElementsError) -> Sgp4Error {
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

fn diverged(err: sgp4::Error, time_since_epoch: TimeDelta) -> Sgp4Error {
    let kind = match err {
        sgp4::Error::OutOfRangeEccentricity { eccentricity, .. } => {
            Divergence::EccentricityOutOfRange(eccentricity)
        }
        sgp4::Error::OutOfRangePerturbedEccentricity { eccentricity, .. } => {
            Divergence::PerturbedEccentricityOutOfRange(eccentricity)
        }
        sgp4::Error::NegativeSemiLatusRectum { .. } => Divergence::NegativeSemiLatusRectum,
    };
    Sgp4Error::Diverged {
        kind,
        time_since_epoch,
    }
}

fn format_epoch_offset(delta: &TimeDelta) -> String {
    let minutes = delta.as_seconds_f64().abs() / SECONDS_PER_MINUTE as f64;
    let side = if delta.is_negative() {
        "before"
    } else {
        "after"
    };
    format!("{minutes:.3} minutes {side} epoch")
}

fn to_sgp4_elements(tle: &Tle) -> Result<sgp4::Elements, Sgp4Error> {
    let epoch: DateTime<chrono::Utc> = tle
        .epoch
        .try_into()
        .map_err(|_| Sgp4Error::InvalidElements(InvalidElements::EpochOutOfRange))?;
    Ok(sgp4::Elements {
        object_name: tle.object_name.clone(),
        international_designator: tle.international_designator.clone(),
        norad_id: tle.norad_id,
        classification: match tle.classification {
            Classification::Unclassified => sgp4::Classification::Unclassified,
            Classification::Classified => sgp4::Classification::Classified,
            Classification::Secret => sgp4::Classification::Secret,
        },
        datetime: epoch.naive_utc(),
        mean_motion_dot: tle.half_mean_motion_dot_rev_per_day2,
        mean_motion_ddot: tle.sixth_mean_motion_ddot_rev_per_day3,
        drag_term: tle.drag_term_per_earth_radius,
        element_set_number: tle.element_set_number,
        inclination: tle.inclination.to_degrees(),
        right_ascension: tle.right_ascension.to_degrees(),
        eccentricity: tle.eccentricity,
        argument_of_perigee: tle.argument_of_perigee.to_degrees(),
        mean_anomaly: tle.mean_anomaly.to_degrees(),
        mean_motion: tle.mean_motion.to_radians_per_second() * SECONDS_PER_DAY as f64 / TAU,
        revolution_number: tle.revolution_number,
        ephemeris_type: tle.ephemeris_type,
    })
}

#[cfg(feature = "serde")]
fn from_sgp4_elements(elements: sgp4::Elements) -> Tle {
    Tle {
        object_name: elements.object_name,
        international_designator: elements.international_designator,
        norad_id: elements.norad_id,
        classification: match elements.classification {
            sgp4::Classification::Unclassified => Classification::Unclassified,
            sgp4::Classification::Classified => Classification::Classified,
            sgp4::Classification::Secret => Classification::Secret,
        },
        epoch: elements.datetime.and_utc().into(),
        half_mean_motion_dot_rev_per_day2: elements.mean_motion_dot,
        sixth_mean_motion_ddot_rev_per_day3: elements.mean_motion_ddot,
        drag_term_per_earth_radius: elements.drag_term,
        ephemeris_type: elements.ephemeris_type,
        element_set_number: elements.element_set_number,
        inclination: Angle::degrees(elements.inclination),
        right_ascension: Angle::degrees(elements.right_ascension),
        eccentricity: elements.eccentricity,
        argument_of_perigee: Angle::degrees(elements.argument_of_perigee),
        mean_anomaly: Angle::degrees(elements.mean_anomaly),
        mean_motion: AngularRate::radians_per_second(
            elements.mean_motion * TAU / SECONDS_PER_DAY as f64,
        ),
        revolution_number: elements.revolution_number,
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Tle {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let elements = to_sgp4_elements(self).map_err(serde::ser::Error::custom)?;
        serde::Serialize::serialize(&elements, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Tle {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        <sgp4::Elements as serde::Deserialize>::deserialize(deserializer).map(from_sgp4_elements)
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
    pub fn new(initial_state: Tle) -> Result<Self, Sgp4Error> {
        let elements = to_sgp4_elements(&initial_state)?;
        let time = initial_state.epoch.to_time();
        // Use AFSPC compatibility mode because TLE data is fitted using
        // AFSPC constants (WGS72). Using WGS84 with WGS72-fitted data
        // introduces systematic errors.
        let constants = Constants::from_elements_afspc_compatibility_mode(&elements)
            .map_err(invalid_elements)?;
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
        let delta = time - self.time;
        let dt = delta.as_seconds_f64() / SECONDS_PER_MINUTE as f64;
        let prediction = self
            .constants
            .propagate(MinutesSinceEpoch(dt))
            .map_err(|err| diverged(err, delta))?;
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
    use lox_core::units::AngularRate;
    use lox_frames::Icrf;
    use lox_frames::providers::DefaultRotationProvider;
    use lox_time::intervals::Interval;
    use lox_time::{TimeDelta, TimeDeltaUnit};

    use super::*;

    #[test]
    fn test_sgp4_state_at() {
        let tle = Tle::from_lines(
            Some("ISS (ZARYA)".to_string()),
            "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
            "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
        )
        .unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let orbital_period = 92.821;
        let t1 = sgp4.time() + TimeDelta::new_in(orbital_period, TimeDeltaUnit::Minutes);
        let s1 = sgp4.state_at(t1).unwrap();
        let s1_icrf = s1.try_to_frame(Icrf, &DefaultRotationProvider).unwrap();
        let k1 = s1_icrf.to_keplerian().unwrap();
        assert_approx_eq!(
            k1.orbital_period().unwrap().as_seconds_f64() / SECONDS_PER_MINUTE as f64,
            orbital_period,
            rtol <= 1e-4
        );
    }

    #[test]
    fn test_sgp4_propagate() {
        let tle = Tle::from_lines(
            Some("ISS (ZARYA)".to_string()),
            "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996".as_bytes(),
            "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731".as_bytes(),
        )
        .unwrap();
        let sgp4 = Sgp4::new(tle).unwrap();
        let t0 = sgp4.time();
        let t1 = t0 + TimeDelta::new_in(92.821, TimeDeltaUnit::Minutes);
        let interval = Interval::new(t0, t1);
        let traj = sgp4.propagate(interval.into_dynamic()).unwrap();
        // With 60s default step over ~93 min, we should have ~94 points
        assert!(traj.states().len() > 90);
    }

    #[test]
    fn test_sgp4_propagate_in_frame() {
        let tle = Tle::from_lines(
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

    fn elements(line_1: &str, line_2: &str) -> Tle {
        Tle::from_lines(None, line_1, line_2).unwrap()
    }

    #[test]
    fn test_sgp4_invalid_elements() {
        let mut tle = elements(DIVERGING_LINE_1, DIVERGING_LINE_2);
        tle.mean_motion = AngularRate::radians_per_second(0.0);
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
                time_since_epoch,
                ..
            } if time_since_epoch == TimeDelta::from_minutes(25)
        ));
        assert!(err.to_string().contains("25.000 minutes after epoch"));
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

    #[test]
    fn test_invalid_elements_brouwer_mean_motion() {
        let err = invalid_elements(ElementsError::KozaiElementsError(
            KozaiElementsError::NegativeBrouwerMeanMotion,
        ));
        assert!(matches!(
            err,
            Sgp4Error::InvalidElements(InvalidElements::NegativeBrouwerMeanMotion)
        ));
    }

    #[test]
    fn test_diverged_eccentricity() {
        let delta = TimeDelta::from_minutes(10);
        let err = diverged(
            sgp4::Error::OutOfRangeEccentricity {
                eccentricity: 1.5,
                t: 10.0,
            },
            delta,
        );
        assert!(matches!(
            err,
            Sgp4Error::Diverged {
                kind: Divergence::EccentricityOutOfRange(e),
                time_since_epoch,
                ..
            } if e == 1.5 && time_since_epoch == delta
        ));
    }

    #[test]
    fn test_diverged_message_before_epoch() {
        let err = diverged(
            sgp4::Error::NegativeSemiLatusRectum { t: -1440.0 },
            TimeDelta::from_minutes(-1440),
        );
        assert!(err.to_string().contains("1440.000 minutes before epoch"));
    }

    #[test]
    fn test_diverged_message_fractional_minutes() {
        let err = diverged(
            sgp4::Error::NegativeSemiLatusRectum { t: 1234.5666 },
            TimeDelta::from_seconds_f64(1234.5666 * 60.0),
        );
        assert!(err.to_string().contains("1234.567 minutes after epoch"));
    }

    #[test]
    fn test_epoch_out_of_chrono_range() {
        let mut tle = elements(DIVERGING_LINE_1, DIVERGING_LINE_2);
        tle.epoch = lox_time::utc::Utc::from_delta(TimeDelta::from_seconds(10_000_000_000_000));
        let err = Sgp4::new(tle).unwrap_err();
        assert!(matches!(
            err,
            Sgp4Error::InvalidElements(InvalidElements::EpochOutOfRange)
        ));
    }

    /// The native parser and unit conversions must agree with the `sgp4` crate.
    #[test]
    fn test_tle_matches_sgp4_crate() {
        let data = lox_test_utils::read_data_file("oneweb_tle.txt");
        let lines: Vec<_> = data.lines().collect();
        assert!(lines.len() >= 3);
        for chunk in lines.as_chunks::<3>().0 {
            let name = Some(chunk[0].trim().to_string());
            let native = Tle::from_lines(name.clone(), chunk[1], chunk[2]).unwrap();
            let actual = to_sgp4_elements(&native).unwrap();
            let expected =
                sgp4::Elements::from_tle(name, chunk[1].as_bytes(), chunk[2].as_bytes()).unwrap();

            assert_eq!(actual.object_name, expected.object_name);
            assert_eq!(
                actual.international_designator,
                expected.international_designator
            );
            assert_eq!(actual.norad_id, expected.norad_id);
            assert_eq!(actual.classification, expected.classification);
            assert_eq!(actual.datetime, expected.datetime);
            assert_eq!(actual.mean_motion_dot, expected.mean_motion_dot);
            assert_eq!(actual.mean_motion_ddot, expected.mean_motion_ddot);
            assert_eq!(actual.drag_term, expected.drag_term);
            assert_eq!(actual.eccentricity, expected.eccentricity);
            assert_eq!(actual.element_set_number, expected.element_set_number);
            assert_eq!(actual.revolution_number, expected.revolution_number);
            assert_eq!(actual.ephemeris_type, expected.ephemeris_type);
            assert_approx_eq!(actual.inclination, expected.inclination, rtol <= 1e-14);
            assert_approx_eq!(
                actual.right_ascension,
                expected.right_ascension,
                rtol <= 1e-14
            );
            assert_approx_eq!(
                actual.argument_of_perigee,
                expected.argument_of_perigee,
                rtol <= 1e-14
            );
            assert_approx_eq!(actual.mean_anomaly, expected.mean_anomaly, rtol <= 1e-14);
            assert_approx_eq!(actual.mean_motion, expected.mean_motion, rtol <= 1e-14);

            let actual = Constants::from_elements_afspc_compatibility_mode(&actual).unwrap();
            let expected = Constants::from_elements_afspc_compatibility_mode(&expected).unwrap();
            for minutes in [0.0, 720.0, 1440.0] {
                let a = actual.propagate(MinutesSinceEpoch(minutes)).unwrap();
                let e = expected.propagate(MinutesSinceEpoch(minutes)).unwrap();
                for i in 0..3 {
                    // km and km/s: agreement to better than a micrometre.
                    assert!((a.position[i] - e.position[i]).abs() < 1e-9);
                    assert!((a.velocity[i] - e.velocity[i]).abs() < 1e-9);
                }
            }
        }
    }
}
