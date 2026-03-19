// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Time-indexed interpolation series.

use std::sync::Arc;

use lox_core::math::series::{InterpolationType, Series, SeriesError};

use crate::deltas::TimeDelta;
use crate::time::Time;
use crate::time_scales::TimeScale;

/// An interpolated 1-D data series indexed by [`Time`].
///
/// `TimeSeries` wraps a [`Series`] together with a start epoch, allowing interpolation
/// by absolute [`Time`] values rather than raw `f64` offsets.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeSeries<T: TimeScale> {
    epoch: Time<T>,
    series: Series,
}

impl<T: TimeScale + Copy> TimeSeries<T> {
    /// Creates a new `TimeSeries` from an epoch and a pre-built [`Series`].
    pub fn new(epoch: Time<T>, series: Series) -> Self {
        Self { epoch, series }
    }

    /// Creates a new `TimeSeries`, returning an error if the data is invalid.
    pub fn try_new(
        epoch: Time<T>,
        x: impl Into<Arc<[f64]>>,
        y: impl Into<Arc<[f64]>>,
        interpolation: InterpolationType,
    ) -> Result<Self, SeriesError> {
        let series = Series::try_new(x, y, interpolation)?;
        Ok(Self { epoch, series })
    }

    /// Interpolates the series at the given `time`.
    ///
    /// Converts `time - epoch` to seconds and delegates to the underlying [`Series`].
    pub fn interpolate(&self, time: Time<T>) -> f64 {
        let dt = time - self.epoch;
        self.series.interpolate(delta_to_f64(dt))
    }

    /// Returns the start epoch.
    pub fn epoch(&self) -> Time<T> {
        self.epoch
    }

    /// Returns a reference to the underlying [`Series`].
    pub fn series(&self) -> &Series {
        &self.series
    }

    /// Returns absolute timestamps for each data point.
    pub fn times(&self) -> Vec<Time<T>> {
        self.series
            .x()
            .iter()
            .map(|&x| self.epoch + TimeDelta::from_seconds_f64(x))
            .collect()
    }

    /// Returns the y values of the underlying series.
    pub fn values(&self) -> &[f64] {
        self.series.y()
    }

    /// Returns an iterator over `(Time<T>, f64)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (Time<T>, f64)> + '_ {
        self.series
            .x()
            .iter()
            .zip(self.series.y().iter())
            .map(|(&x, &y)| (self.epoch + TimeDelta::from_seconds_f64(x), y))
    }

    /// Returns the first data point as `(time, value)`.
    pub fn first(&self) -> (Time<T>, f64) {
        let (x, y) = self.series.first();
        (self.epoch + TimeDelta::from_seconds_f64(x), y)
    }

    /// Returns the last data point as `(time, value)`.
    pub fn last(&self) -> (Time<T>, f64) {
        let (x, y) = self.series.last();
        (self.epoch + TimeDelta::from_seconds_f64(x), y)
    }
}

fn delta_to_f64(delta: TimeDelta) -> f64 {
    delta.to_seconds().to_f64()
}

#[cfg(test)]
mod tests {
    use lox_core::math::series::{InterpolationType, SeriesError};

    use crate::time::Time;
    use crate::time_scales::Tai;

    use super::*;

    fn epoch() -> Time<Tai> {
        Time::builder_with_scale(Tai)
            .with_ymd(2024, 1, 1)
            .with_hms(0, 0, 0.0)
            .build()
            .unwrap()
    }

    #[test]
    fn test_new_and_interpolate() {
        let ep = epoch();
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0, 1.0, 4.0, 9.0, 16.0];
        let ts = TimeSeries::try_new(ep, x, y, InterpolationType::Linear).unwrap();

        let t = ep + TimeDelta::from_seconds_f64(1.5);
        let val = ts.interpolate(t);
        assert!((val - 2.5).abs() < 1e-12);
    }

    #[test]
    fn test_epoch_and_series() {
        let ep = epoch();
        let x = vec![0.0, 1.0, 2.0];
        let y = vec![10.0, 20.0, 30.0];
        let ts = TimeSeries::try_new(ep, x, y, InterpolationType::Linear).unwrap();

        assert_eq!(ts.epoch(), ep);
        assert_eq!(ts.series().x(), &[0.0, 1.0, 2.0]);
        assert_eq!(ts.values(), &[10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_times() {
        let ep = epoch();
        let x = vec![0.0, 60.0, 120.0];
        let y = vec![1.0, 2.0, 3.0];
        let ts = TimeSeries::try_new(ep, x, y, InterpolationType::Linear).unwrap();

        let times = ts.times();
        assert_eq!(times.len(), 3);
        assert_eq!(times[0], ep);
        assert_eq!(times[1], ep + TimeDelta::from_seconds_f64(60.0));
        assert_eq!(times[2], ep + TimeDelta::from_seconds_f64(120.0));
    }

    #[test]
    fn test_iter() {
        let ep = epoch();
        let x = vec![0.0, 1.0];
        let y = vec![5.0, 10.0];
        let ts = TimeSeries::try_new(ep, x, y, InterpolationType::Linear).unwrap();

        let pairs: Vec<_> = ts.iter().collect();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], (ep, 5.0));
        assert_eq!(pairs[1], (ep + TimeDelta::from_seconds_f64(1.0), 10.0));
    }

    #[test]
    fn test_first_last() {
        let ep = epoch();
        let x = vec![0.0, 100.0, 200.0];
        let y = vec![1.0, 2.0, 3.0];
        let ts = TimeSeries::try_new(ep, x, y, InterpolationType::Linear).unwrap();

        let (ft, fv) = ts.first();
        assert_eq!(ft, ep);
        assert_eq!(fv, 1.0);

        let (lt, lv) = ts.last();
        assert_eq!(lt, ep + TimeDelta::from_seconds_f64(200.0));
        assert_eq!(lv, 3.0);
    }

    #[test]
    fn test_try_new_insufficient_points() {
        let ep = epoch();
        let result = TimeSeries::try_new(ep, vec![1.0], vec![1.0], InterpolationType::Linear);
        assert_eq!(result.unwrap_err(), SeriesError::InsufficientPoints(1));
    }

    #[test]
    fn test_try_new_dimension_mismatch() {
        let ep = epoch();
        let result = TimeSeries::try_new(ep, vec![0.0, 1.0], vec![1.0], InterpolationType::Linear);
        assert_eq!(result.unwrap_err(), SeriesError::DimensionMismatch(2, 1));
    }

    #[test]
    fn test_try_new_non_monotonic() {
        let ep = epoch();
        let result = TimeSeries::try_new(
            ep,
            vec![0.0, 2.0, 1.0],
            vec![1.0, 2.0, 3.0],
            InterpolationType::Linear,
        );
        assert_eq!(result.unwrap_err(), SeriesError::NonMonotonic);
    }
}
