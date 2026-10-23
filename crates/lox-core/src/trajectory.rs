// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Interpolated trajectories on a relative time axis.
//!
//! Time in this module is always relative: knots are offsets from the first
//! sample, which defines `t = 0`. Pairing a trajectory with an absolute epoch
//! and time scale is the responsibility of the caller.

use alloc::sync::Arc;
use alloc::vec::Vec;

use glam::DVec3;
use thiserror::Error;

use crate::coords::Cartesian;
use crate::time::deltas::TimeDelta;

/// Error returned when constructing a [`TimeAxis`] from invalid knots.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum TimeAxisError {
    /// Fewer than two knots were provided.
    #[error("a time axis needs at least 2 knots but got {0}")]
    TooFewKnots(usize),
    /// A knot is not strictly greater than its predecessor.
    #[error("knot {0} is not strictly greater than its predecessor")]
    NotStrictlyIncreasing(usize),
}

/// Strictly increasing sample times, relative to the first sample.
///
/// The first knot is always zero. Cloning is cheap and shares the underlying
/// storage, so several quantities sampled at the same times can share one axis.
#[derive(Clone, Debug, PartialEq)]
pub struct TimeAxis {
    knots: Arc<[TimeDelta]>,
    seconds: Arc<[f64]>,
    inv_step: Option<f64>,
}

impl TimeAxis {
    /// Creates a time axis from strictly increasing knots.
    ///
    /// The first knot defines `t = 0` and all knots are stored relative to it.
    pub fn try_new(knots: impl IntoIterator<Item = TimeDelta>) -> Result<Self, TimeAxisError> {
        let mut iter = knots.into_iter();
        let (lower, _) = iter.size_hint();
        let Some(first) = iter.next() else {
            return Err(TimeAxisError::TooFewKnots(0));
        };
        let mut knots = Vec::with_capacity(lower);
        knots.push(TimeDelta::ZERO);
        for (i, knot) in iter.enumerate() {
            let knot = knot - first;
            if knot <= knots[i] {
                return Err(TimeAxisError::NotStrictlyIncreasing(i + 1));
            }
            knots.push(knot);
        }
        if knots.len() < 2 {
            return Err(TimeAxisError::TooFewKnots(knots.len()));
        }

        let seconds: Vec<f64> = knots.iter().map(|&k| to_seconds(k)).collect();
        let inv_step = uniform_step(&knots).map(|step| 1.0 / to_seconds(step));

        Ok(Self {
            knots: knots.into(),
            seconds: seconds.into(),
            inv_step,
        })
    }

    /// Returns the number of knots.
    pub fn len(&self) -> usize {
        self.knots.len()
    }

    /// Returns `false`; a time axis always has at least two knots.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Returns the knots relative to the first one.
    pub fn knots(&self) -> &[TimeDelta] {
        &self.knots
    }

    /// Returns the knots relative to the first one, in seconds.
    pub fn seconds(&self) -> &[f64] {
        &self.seconds
    }

    /// Returns the time between the first and the last knot.
    pub fn span(&self) -> TimeDelta {
        self.knots[self.knots.len() - 1]
    }

    /// Returns `true` if all knots except possibly the last are equally spaced.
    pub fn is_uniform(&self) -> bool {
        self.inv_step.is_some()
    }

    /// Returns `true` if `t` seconds lies within the closed range of the axis.
    #[inline]
    pub fn contains(&self, t: f64) -> bool {
        t >= 0.0 && t <= self.seconds[self.seconds.len() - 1]
    }

    /// Returns the index of the segment containing `t` seconds, or `None` if
    /// `t` lies outside the axis.
    ///
    /// Segment `i` spans knots `i` and `i + 1`. A `t` on an inner knot may be
    /// assigned to either adjacent segment.
    #[inline]
    pub fn segment(&self, t: f64) -> Option<usize> {
        if !self.contains(t) {
            return None;
        }
        let last = self.seconds.len() - 2;
        let idx = match self.inv_step {
            Some(inv_step) => (t * inv_step) as usize,
            None => self.seconds.partition_point(|&s| s <= t).saturating_sub(1),
        };
        Some(idx.min(last))
    }

    /// Like [`segment`](Self::segment), but searches outwards from the segment
    /// last returned for this `cursor`.
    ///
    /// This is amortised O(1) for monotonically increasing `t` on any grid,
    /// and O(log k) when `t` skips ahead by `k` segments.
    #[inline]
    pub fn segment_from(&self, cursor: &mut usize, t: f64) -> Option<usize> {
        if self.inv_step.is_some() {
            let idx = self.segment(t)?;
            *cursor = idx;
            return Some(idx);
        }
        if !self.contains(t) {
            return None;
        }
        let s = &self.seconds;
        let last = s.len() - 2;
        let current = (*cursor).min(last);
        let idx = if t < s[current] {
            s[..current].partition_point(|&x| x <= t) - 1
        } else {
            // Gallop forwards until the bracket [lo, hi) contains `t`.
            let mut lo = current;
            let mut hi = current + 1;
            let mut step = 1;
            while hi <= last && s[hi] <= t {
                lo = hi;
                step *= 2;
                hi = (lo + step).min(last + 1);
            }
            lo + s[lo..hi].partition_point(|&x| x <= t) - 1
        };
        *cursor = idx;
        Some(idx)
    }
}

#[inline]
fn to_seconds(delta: TimeDelta) -> f64 {
    delta.to_seconds().to_f64()
}

/// Returns the common step if all knots except possibly the last are equally
/// spaced and the last step is not longer than the others.
fn uniform_step(knots: &[TimeDelta]) -> Option<TimeDelta> {
    let step = knots[1] - knots[0];
    let n = knots.len();
    let inner_uniform = knots[..n - 1].windows(2).all(|w| w[1] - w[0] == step);
    let last_fits = knots[n - 1] - knots[n - 2] <= step;
    (inner_uniform && last_fits).then_some(step)
}

/// One Hermite segment: the knot state and the cubic coefficients up to the next knot.
///
/// Position is `p + v·dt + c2·dt² + c3·dt³` with `dt = t - t0`. The record for
/// the last knot has zero `c2` and `c3`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
struct Segment {
    t0: f64,
    p: DVec3,
    v: DVec3,
    c2: DVec3,
    c3: DVec3,
}

impl Segment {
    #[inline]
    fn position(&self, t: f64) -> DVec3 {
        let dt = t - self.t0;
        self.p + dt * (self.v + dt * (self.c2 + dt * self.c3))
    }

    #[inline]
    fn velocity(&self, t: f64) -> DVec3 {
        let dt = t - self.t0;
        self.v + dt * (2.0 * self.c2 + 3.0 * dt * self.c3)
    }
}

/// A Cartesian trajectory interpolated with piecewise cubic Hermite polynomials.
///
/// Each segment matches position and velocity at both of its knots. Velocity
/// is the derivative of the interpolated position, so the two are always
/// consistent. Times are seconds relative to the first knot of the
/// [`TimeAxis`].
#[derive(Clone, Debug, PartialEq)]
pub struct HermiteTrajectory {
    axis: TimeAxis,
    segments: Arc<[Segment]>,
}

impl HermiteTrajectory {
    /// Creates a trajectory by evaluating `f` at each knot of `axis`.
    ///
    /// `f` receives the knot relative to the first one.
    pub fn from_axis(axis: TimeAxis, mut f: impl FnMut(TimeDelta) -> Cartesian) -> Self {
        let states = axis.knots().iter().map(|&k| f(k)).collect();
        Self::from_states(axis, states)
    }

    /// Like [`from_axis`](Self::from_axis), but `f` may fail.
    pub fn try_from_axis<E>(
        axis: TimeAxis,
        mut f: impl FnMut(TimeDelta) -> Result<Cartesian, E>,
    ) -> Result<Self, E> {
        let states = axis
            .knots()
            .iter()
            .map(|&k| f(k))
            .collect::<Result<_, _>>()?;
        Ok(Self::from_states(axis, states))
    }

    /// Creates a trajectory from `(time, state)` samples.
    ///
    /// The first sample defines `t = 0`.
    pub fn try_new(
        samples: impl IntoIterator<Item = (TimeDelta, Cartesian)>,
    ) -> Result<Self, TimeAxisError> {
        let (knots, states): (Vec<_>, Vec<_>) = samples.into_iter().unzip();
        let axis = TimeAxis::try_new(knots)?;
        Ok(Self::from_states(axis, states))
    }

    /// Creates a trajectory on the same time axis with each state transformed by `f`.
    ///
    /// `f` receives the knot relative to the first one and the state at that knot.
    pub fn map_states(&self, mut f: impl FnMut(TimeDelta, Cartesian) -> Cartesian) -> Self {
        let states = self.iter().map(|(k, s)| f(k, s)).collect();
        Self::from_states(self.axis.clone(), states)
    }

    /// Like [`map_states`](Self::map_states), but `f` may fail.
    pub fn try_map_states<E>(
        &self,
        mut f: impl FnMut(TimeDelta, Cartesian) -> Result<Cartesian, E>,
    ) -> Result<Self, E> {
        let states = self
            .iter()
            .map(|(k, s)| f(k, s))
            .collect::<Result<_, _>>()?;
        Ok(Self::from_states(self.axis.clone(), states))
    }

    fn from_states(axis: TimeAxis, states: Vec<Cartesian>) -> Self {
        debug_assert_eq!(axis.len(), states.len());
        let t = axis.seconds();
        let n = states.len();
        let segments = (0..n)
            .map(|i| {
                let p = states[i].position();
                let v = states[i].velocity();
                let (c2, c3) = if i + 1 < n {
                    let h = t[i + 1] - t[i];
                    let dp = states[i + 1].position() - p;
                    let v1 = states[i + 1].velocity();
                    (
                        3.0 * dp / (h * h) - (2.0 * v + v1) / h,
                        -2.0 * dp / (h * h * h) + (v + v1) / (h * h),
                    )
                } else {
                    (DVec3::ZERO, DVec3::ZERO)
                };
                Segment {
                    t0: t[i],
                    p,
                    v,
                    c2,
                    c3,
                }
            })
            .collect();
        Self { axis, segments }
    }

    /// Returns the time axis.
    pub fn axis(&self) -> &TimeAxis {
        &self.axis
    }

    /// Returns the number of knots.
    pub fn len(&self) -> usize {
        self.axis.len()
    }

    /// Returns `false`; a trajectory always has at least two knots.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Returns the states at the knots.
    pub fn states(&self) -> impl ExactSizeIterator<Item = Cartesian> + '_ {
        self.segments.iter().map(|s| Cartesian::from_vecs(s.p, s.v))
    }

    /// Returns the knots and the states at the knots.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (TimeDelta, Cartesian)> + '_ {
        self.axis.knots().iter().copied().zip(self.states())
    }

    /// Interpolates the state at `t` seconds, or returns `None` if `t` lies
    /// outside the time axis.
    #[inline]
    pub fn at(&self, t: f64) -> Option<Cartesian> {
        let s = &self.segments[self.axis.segment(t)?];
        Some(Cartesian::from_vecs(s.position(t), s.velocity(t)))
    }

    /// Interpolates the position at `t` seconds, or returns `None` if `t` lies
    /// outside the time axis.
    #[inline]
    pub fn position(&self, t: f64) -> Option<DVec3> {
        Some(self.segments[self.axis.segment(t)?].position(t))
    }

    /// Interpolates the velocity at `t` seconds, or returns `None` if `t` lies
    /// outside the time axis.
    #[inline]
    pub fn velocity(&self, t: f64) -> Option<DVec3> {
        Some(self.segments[self.axis.segment(t)?].velocity(t))
    }

    /// Returns a cursor for repeated evaluation in increasing time order.
    pub fn cursor(&self) -> Cursor<'_> {
        Cursor {
            trajectory: self,
            segment: 0,
        }
    }
}

/// Evaluates a [`HermiteTrajectory`] while remembering the last segment.
///
/// Evaluation in increasing time order is amortised O(1) on any grid. Results
/// are identical to [`HermiteTrajectory::at`] for any order of evaluation.
#[derive(Clone, Debug)]
pub struct Cursor<'a> {
    trajectory: &'a HermiteTrajectory,
    segment: usize,
}

impl Cursor<'_> {
    /// Interpolates the state at `t` seconds, or returns `None` if `t` lies
    /// outside the time axis.
    #[inline]
    pub fn at(&mut self, t: f64) -> Option<Cartesian> {
        let s = self.find(t)?;
        Some(Cartesian::from_vecs(s.position(t), s.velocity(t)))
    }

    /// Interpolates the position at `t` seconds, or returns `None` if `t` lies
    /// outside the time axis.
    #[inline]
    pub fn position(&mut self, t: f64) -> Option<DVec3> {
        Some(self.find(t)?.position(t))
    }

    #[inline]
    fn find(&mut self, t: f64) -> Option<&Segment> {
        let idx = self.trajectory.axis.segment_from(&mut self.segment, t)?;
        Some(&self.trajectory.segments[idx])
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use alloc::vec::Vec;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{HermiteTrajectory, TimeAxis, TimeDelta};
    use crate::coords::Cartesian;

    #[derive(Serialize, Deserialize)]
    struct Repr {
        knots: Vec<TimeDelta>,
        states: Vec<Cartesian>,
    }

    impl Serialize for TimeAxis {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.knots().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for TimeAxis {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let knots = Vec::<TimeDelta>::deserialize(deserializer)?;
            TimeAxis::try_new(knots).map_err(serde::de::Error::custom)
        }
    }

    impl Serialize for HermiteTrajectory {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            Repr {
                knots: self.axis.knots().to_vec(),
                states: self.states().collect(),
            }
            .serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for HermiteTrajectory {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let Repr { knots, states } = Repr::deserialize(deserializer)?;
            if knots.len() != states.len() {
                return Err(serde::de::Error::custom(alloc::format!(
                    "expected one state per knot but got {} knots and {} states",
                    knots.len(),
                    states.len()
                )));
            }
            HermiteTrajectory::try_new(knots.into_iter().zip(states))
                .map_err(serde::de::Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use rstest::rstest;

    use super::*;

    fn seconds(s: &[i64]) -> Vec<TimeDelta> {
        s.iter().map(|&s| TimeDelta::from_seconds(s)).collect()
    }

    /// A cubic in each component; Hermite interpolation reproduces it exactly.
    fn cubic(t: f64) -> Cartesian {
        let a = DVec3::new(7.0e6, -1.0e6, 2.0e5);
        let b = DVec3::new(10.0, 7.5e3, -300.0);
        let c = DVec3::new(-0.5, 0.25, 0.1);
        let d = DVec3::new(1e-3, -2e-4, 5e-5);
        Cartesian::from_vecs(
            a + t * (b + t * (c + t * d)),
            b + t * (2.0 * c + 3.0 * t * d),
        )
    }

    fn cubic_trajectory(knots: &[i64]) -> HermiteTrajectory {
        HermiteTrajectory::try_new(
            seconds(knots)
                .into_iter()
                .map(|k| (k, cubic(to_seconds(k)))),
        )
        .unwrap()
    }

    #[rstest]
    #[case(&[], TimeAxisError::TooFewKnots(0))]
    #[case(&[5], TimeAxisError::TooFewKnots(1))]
    #[case(&[0, 10, 10], TimeAxisError::NotStrictlyIncreasing(2))]
    #[case(&[0, 10, 5], TimeAxisError::NotStrictlyIncreasing(2))]
    #[case(&[0, -1], TimeAxisError::NotStrictlyIncreasing(1))]
    fn test_time_axis_invalid(#[case] knots: &[i64], #[case] expected: TimeAxisError) {
        assert_eq!(TimeAxis::try_new(seconds(knots)), Err(expected));
    }

    #[test]
    fn test_time_axis_is_relative_to_first_knot() {
        let axis = TimeAxis::try_new(seconds(&[100, 110, 135])).unwrap();

        assert_eq!(axis.knots(), seconds(&[0, 10, 35]));
        assert_eq!(axis.seconds(), &[0.0, 10.0, 35.0]);
        assert_eq!(axis.span(), TimeDelta::from_seconds(35));
        assert_eq!(axis.len(), 3);
    }

    #[rstest]
    #[case(&[0, 10, 20, 30], true)]
    #[case(&[0, 10, 20, 25], true)]
    #[case(&[0, 10], true)]
    #[case(&[0, 10, 20, 35], false)]
    #[case(&[0, 10, 25, 30], false)]
    fn test_time_axis_uniform(#[case] knots: &[i64], #[case] uniform: bool) {
        assert_eq!(
            TimeAxis::try_new(seconds(knots)).unwrap().is_uniform(),
            uniform
        );
    }

    #[rstest]
    #[case(&[0, 10, 20, 25])]
    #[case(&[0, 3, 20, 21, 50])]
    fn test_time_axis_segment(#[case] knots: &[i64]) {
        let axis = TimeAxis::try_new(seconds(knots)).unwrap();
        let s = axis.seconds();
        let last = s.len() - 2;

        assert_eq!(axis.segment(-1e-9), None);
        assert_eq!(axis.segment(s[last + 1] + 1e-9), None);
        assert_eq!(axis.segment(f64::NAN), None);
        assert_eq!(axis.segment(0.0), Some(0));
        assert_eq!(axis.segment(s[last + 1]), Some(last));
        for i in 0..=last {
            let mid = 0.5 * (s[i] + s[i + 1]);
            assert_eq!(axis.segment(mid), Some(i), "midpoint of segment {i}");
        }
    }

    #[rstest]
    #[case(&[0, 10, 20, 30, 40])]
    #[case(&[0, 3, 20, 21, 50, 51, 52])]
    fn test_time_axis_segment_from_matches_segment(#[case] knots: &[i64]) {
        let axis = TimeAxis::try_new(seconds(knots)).unwrap();
        let span = to_seconds(axis.span());
        let mut cursor = 0;
        // Forward sweep, then a backwards jump, then out of range.
        let times =
            (0..=200)
                .map(|i| span * i as f64 / 200.0)
                .chain([0.5, span * 0.9, span + 1.0, 0.25]);
        for t in times {
            let expected = axis.segment(t);
            let actual = axis.segment_from(&mut cursor, t);
            match (expected, actual) {
                (Some(e), Some(a)) => {
                    let s = axis.seconds();
                    assert!(s[a] <= t && t <= s[a + 1], "t = {t} not in segment {a}");
                    assert!(e.abs_diff(a) <= 1, "t = {t}: {e} vs {a}");
                }
                (e, a) => assert_eq!(e, a, "t = {t}"),
            }
        }
    }

    #[rstest]
    #[case(&[0, 60, 120, 180, 240])]
    #[case(&[0, 7, 60, 61, 200, 330])]
    fn test_hermite_reproduces_cubic(#[case] knots: &[i64]) {
        let traj = cubic_trajectory(knots);
        let span = to_seconds(traj.axis().span());

        for i in 0..=100 {
            let t = span * i as f64 / 100.0;
            let expected = cubic(t);
            let state = traj.at(t).unwrap();
            assert_approx_eq!(state.position(), expected.position(), rtol <= 1e-12);
            assert_approx_eq!(state.velocity(), expected.velocity(), rtol <= 1e-10);
            assert_eq!(traj.position(t), Some(state.position()));
            assert_eq!(traj.velocity(t), Some(state.velocity()));
        }
    }

    #[test]
    fn test_hermite_out_of_range() {
        let traj = cubic_trajectory(&[0, 10, 20]);

        assert!(traj.at(-1.0).is_none());
        assert!(traj.at(20.5).is_none());
        assert!(traj.at(f64::NAN).is_none());
        assert!(traj.position(-1.0).is_none());
        assert!(traj.velocity(21.0).is_none());
        assert!(traj.at(0.0).is_some());
        assert!(traj.at(20.0).is_some());
    }

    #[test]
    fn test_hermite_iter_returns_samples() {
        let knots = seconds(&[50, 60, 75]);
        let samples: Vec<_> = knots
            .iter()
            .enumerate()
            .map(|(i, &k)| (k, cubic(i as f64)))
            .collect();

        let traj = HermiteTrajectory::try_new(samples.clone()).unwrap();

        let expected: Vec<_> = samples.iter().map(|&(k, s)| (k - knots[0], s)).collect();
        assert_eq!(traj.iter().collect::<Vec<_>>(), expected);
        assert_eq!(traj.iter().len(), 3);
        assert_eq!(traj.len(), 3);
    }

    #[test]
    fn test_hermite_try_new_invalid() {
        let state = cubic(0.0);
        assert_eq!(
            HermiteTrajectory::try_new([(TimeDelta::ZERO, state)]),
            Err(TimeAxisError::TooFewKnots(1))
        );
    }

    #[test]
    fn test_hermite_from_axis() {
        let axis = TimeAxis::try_new(seconds(&[0, 10, 30])).unwrap();

        let traj = HermiteTrajectory::from_axis(axis.clone(), |k| cubic(to_seconds(k)));

        assert_eq!(traj, cubic_trajectory(&[0, 10, 30]));
        assert_eq!(traj.axis(), &axis);
    }

    #[test]
    fn test_hermite_try_from_axis_propagates_error() {
        let axis = TimeAxis::try_new(seconds(&[0, 10, 30])).unwrap();

        let result = HermiteTrajectory::try_from_axis(axis, |k| {
            if k > TimeDelta::from_seconds(5) {
                Err("boom")
            } else {
                Ok(cubic(0.0))
            }
        });

        assert_eq!(result, Err("boom"));
    }

    #[test]
    fn test_hermite_map_states_keeps_axis() {
        let traj = cubic_trajectory(&[0, 10, 30]);

        let doubled = traj.map_states(|_, s| s * 2.0);

        assert!(Arc::ptr_eq(&traj.axis().knots, &doubled.axis().knots));
        let expected: Vec<_> = traj.states().map(|s| s * 2.0).collect();
        assert_eq!(doubled.states().collect::<Vec<_>>(), expected);
        assert_eq!(traj.try_map_states(|_, _| Err::<Cartesian, _>(())), Err(()));
    }

    #[rstest]
    #[case(&[0, 60, 120, 180, 240])]
    #[case(&[0, 7, 60, 61, 200, 330])]
    fn test_cursor_matches_at(#[case] knots: &[i64]) {
        let traj = cubic_trajectory(knots);
        let span = to_seconds(traj.axis().span());
        let mut cursor = traj.cursor();
        let times = (0..=500).map(|i| span * i as f64 / 500.0).chain([
            span * 0.3,
            span + 1.0,
            -1.0,
            span * 0.7,
            0.0,
        ]);

        for t in times {
            match (traj.at(t), cursor.at(t)) {
                (Some(expected), Some(actual)) => {
                    assert_approx_eq!(actual.position(), expected.position(), rtol <= 1e-14);
                    assert_approx_eq!(actual.velocity(), expected.velocity(), rtol <= 1e-12);
                }
                (expected, actual) => assert_eq!(expected, actual, "t = {t}"),
            }
            assert_eq!(cursor.position(t).is_some(), traj.position(t).is_some());
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_hermite_serde_roundtrip() {
        let traj = cubic_trajectory(&[0, 7, 60, 61]);

        let json = serde_json::to_string(&traj).unwrap();
        let back: HermiteTrajectory = serde_json::from_str(&json).unwrap();

        // serde_json does not round-trip every f64 exactly
        assert_eq!(back.axis(), traj.axis());
        for (b, t) in back.states().zip(traj.states()) {
            assert_approx_eq!(b, t, rtol <= 1e-15);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_hermite_serde_rejects_invalid() {
        let state = serde_json::to_string(&cubic(0.0)).unwrap();
        let knot = serde_json::to_string(&TimeDelta::ZERO).unwrap();
        let json = alloc::format!(r#"{{"knots":[{knot},{knot}],"states":[{state},{state}]}}"#);

        assert!(serde_json::from_str::<HermiteTrajectory>(&json).is_err());

        let json = alloc::format!(r#"{{"knots":[{knot}],"states":[{state},{state}]}}"#);
        assert!(serde_json::from_str::<HermiteTrajectory>(&json).is_err());
    }

    #[test]
    fn test_segment_layout() {
        assert_eq!(core::mem::size_of::<Segment>(), 104);
    }
}
