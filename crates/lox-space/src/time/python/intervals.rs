// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::time::python::deltas::PyTimeDelta;
use crate::time::python::time::PyTime;
use crate::time::time_scales::DynTimeScale;
use lox_time::intervals::{
    TimeInterval, complement_intervals, intersect_intervals, union_intervals,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Represents a time window (interval between two times).
///
/// Intervals are used to represent periods when certain conditions are met,
/// such as visibility intervals between a ground station and spacecraft.
///
/// Args:
///     start: The start time of the interval.
///     end: The end time of the interval.
#[pyclass(name = "Interval", module = "lox_space", frozen, from_py_object)]
#[derive(Clone, Debug)]
pub struct PyInterval(pub TimeInterval<DynTimeScale>);

#[pymethods]
impl PyInterval {
    #[new]
    fn new(start: PyTime, end: PyTime) -> Self {
        PyInterval(TimeInterval::new(start.0, end.0))
    }

    /// Return the developer-facing string representation of this interval.
    pub fn __repr__(&self) -> String {
        format!(
            "Interval({}, {})",
            self.start().__repr__(),
            self.end().__repr__(),
        )
    }

    /// Return the start time of this interval.
    fn start(&self) -> PyTime {
        PyTime(self.0.start())
    }

    /// Return the end time of this interval.
    fn end(&self) -> PyTime {
        PyTime(self.0.end())
    }

    /// Return the duration of this interval.
    fn duration(&self) -> PyTimeDelta {
        PyTimeDelta(self.0.duration())
    }

    /// Return whether this interval is empty (start >= end).
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return whether this interval contains the given time.
    fn contains_time(&self, time: PyTime) -> bool {
        self.0.contains_time(time.0)
    }

    /// Return whether this interval fully contains another interval.
    fn contains(&self, other: &PyInterval) -> bool {
        self.0.contains(&other.0)
    }

    /// Return the intersection of this interval with another.
    fn intersect(&self, other: PyInterval) -> PyInterval {
        PyInterval(self.0.intersect(other.0))
    }

    /// Return whether this interval overlaps with another.
    fn overlaps(&self, other: PyInterval) -> bool {
        self.0.overlaps(other.0)
    }

    /// Return a list of times spaced by the given step within this interval.
    ///
    /// Args:
    ///     step: The step size (must be non-zero).
    ///
    /// Returns:
    ///     List of Time objects.
    ///
    /// Raises:
    ///     ValueError: If step is zero.
    fn step_by(&self, step: PyTimeDelta) -> PyResult<Vec<PyTime>> {
        if step.0.is_zero() {
            return Err(PyValueError::new_err("step must be non-zero"));
        }
        Ok(self.0.step_by(step.0).map(PyTime).collect())
    }

    /// Return a list of n evenly-spaced times within this interval.
    ///
    /// Args:
    ///     n: Number of points (must be >= 2).
    ///
    /// Returns:
    ///     List of Time objects.
    ///
    /// Raises:
    ///     ValueError: If n < 2.
    fn linspace(&self, n: usize) -> PyResult<Vec<PyTime>> {
        if n < 2 {
            return Err(PyValueError::new_err("n must be >= 2"));
        }
        Ok(self.0.linspace(n).into_iter().map(PyTime).collect())
    }
}

/// Intersect two sorted lists of intervals.
///
/// Args:
///     a: First list of intervals.
///     b: Second list of intervals.
///
/// Returns:
///     List of intervals representing the intersection.
#[pyfunction]
#[pyo3(name = "intersect_intervals")]
pub fn py_intersect_intervals(a: Vec<PyInterval>, b: Vec<PyInterval>) -> Vec<PyInterval> {
    let a: Vec<_> = a.into_iter().map(|i| i.0).collect();
    let b: Vec<_> = b.into_iter().map(|i| i.0).collect();
    intersect_intervals(&a, &b)
        .into_iter()
        .map(PyInterval)
        .collect()
}

/// Compute the union of two sorted lists of intervals.
///
/// Args:
///     a: First list of intervals.
///     b: Second list of intervals.
///
/// Returns:
///     List of merged intervals representing the union.
#[pyfunction]
#[pyo3(name = "union_intervals")]
pub fn py_union_intervals(a: Vec<PyInterval>, b: Vec<PyInterval>) -> Vec<PyInterval> {
    let a: Vec<_> = a.into_iter().map(|i| i.0).collect();
    let b: Vec<_> = b.into_iter().map(|i| i.0).collect();
    union_intervals(&a, &b)
        .into_iter()
        .map(PyInterval)
        .collect()
}

/// Compute the complement of intervals within a bounding interval.
///
/// Args:
///     intervals: List of intervals to complement.
///     bound: Bounding interval.
///
/// Returns:
///     List of gap intervals within the bound.
#[pyfunction]
#[pyo3(name = "complement_intervals")]
pub fn py_complement_intervals(intervals: Vec<PyInterval>, bound: PyInterval) -> Vec<PyInterval> {
    let intervals: Vec<_> = intervals.into_iter().map(|i| i.0).collect();
    complement_intervals(&intervals, bound.0)
        .into_iter()
        .map(PyInterval)
        .collect()
}
