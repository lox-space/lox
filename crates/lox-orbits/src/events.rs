// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use itertools::Itertools;
use lox_test_utils::ApproxEq;
use lox_time::Time;
use lox_time::time_scales::TimeScale;
use std::collections::VecDeque;
use std::fmt::Display;
use std::iter::zip;
use thiserror::Error;

use lox_math::roots::{FindBracketedRoot, RootFinderError};
use lox_time::deltas::TimeDelta;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZeroCrossing {
    Up,
    Down,
}

impl ZeroCrossing {
    fn new(s0: f64, s1: f64) -> Option<ZeroCrossing> {
        if s0 < 0.0 && s1 > 0.0 {
            Some(ZeroCrossing::Up)
        } else if s0 > 0.0 && s1 < 0.0 {
            Some(ZeroCrossing::Down)
        } else {
            None
        }
    }
}

impl Display for ZeroCrossing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZeroCrossing::Up => write!(f, "up"),
            ZeroCrossing::Down => write!(f, "down"),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum FindEventError {
    #[error("function is always negative")]
    AlwaysNegative,
    #[error("function is always positive")]
    AlwaysPositive,
    #[error(transparent)]
    RootFinder(#[from] RootFinderError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Event<T: TimeScale> {
    crossing: ZeroCrossing,
    time: Time<T>,
}

impl<T: TimeScale> Event<T> {
    pub fn time(&self) -> Time<T>
    where
        T: Copy,
    {
        self.time
    }

    pub fn crossing(&self) -> ZeroCrossing {
        self.crossing
    }
}

pub fn find_events<F, R, T, E>(
    func: F,
    start: Time<T>,
    steps: &[f64],
    root_finder: R,
) -> Result<Vec<Event<T>>, FindEventError>
where
    F: Fn(f64) -> Result<f64, E> + Copy,
    T: TimeScale + Clone,
    E: Display,
    R: FindBracketedRoot<F, E>,
{
    // Determine the sign of `func` at each time step (propagate callback errors)
    let mut signs = Vec::with_capacity(steps.len());
    for &t in steps {
        let v = func(t).map_err(|e| {
            FindEventError::RootFinder(RootFinderError::CallbackError(e.to_string()))
        })?;
        signs.push(v.signum());
    }

    // No events could be detected because the function is always negative
    if signs.iter().all(|&s| s < 0.0) {
        return Err(FindEventError::AlwaysNegative);
    }

    // No events could be detected because the function is always positive
    if signs.iter().all(|&s| s > 0.0) {
        return Err(FindEventError::AlwaysPositive);
    }

    let mut events = vec![];

    // Loop over all time step pairs and determine if the sign of the function changes inbetween
    for ((&t0, s0), (&t1, s1)) in zip(steps, signs).tuple_windows() {
        if let Some(crossing) = ZeroCrossing::new(s0, s1) {
            // If the sign changes, run the root finder to determine the exact point in time when
            // the event occurred
            let t = root_finder
                .find_in_bracket(func, (t0, t1))
                .map_err(FindEventError::RootFinder)?;
            let time = start.clone() + TimeDelta::from_seconds_f64(t);

            events.push(Event { crossing, time });
        }
    }

    Ok(events)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ApproxEq)]
pub struct Window<T: TimeScale> {
    start: Time<T>,
    end: Time<T>,
}

impl<T: TimeScale> Window<T> {
    pub fn new(start: Time<T>, end: Time<T>) -> Self {
        Window { start, end }
    }

    pub fn start(&self) -> Time<T>
    where
        T: Clone,
    {
        self.start.clone()
    }

    pub fn end(&self) -> Time<T>
    where
        T: Clone,
    {
        self.end.clone()
    }

    pub fn duration(&self) -> TimeDelta
    where
        T: Clone,
    {
        self.end() - self.start()
    }

    fn contains(&self, other: &Self) -> bool
    where
        T: Ord,
    {
        self.start <= other.start && self.end >= other.end
    }

    fn intersect(&self, other: &Self) -> Option<Self>
    where
        T: Clone + Ord,
    {
        if self.contains(other) {
            return Some(other.clone());
        }
        if other.contains(self) {
            return Some(self.clone());
        }
        if other.start < self.end && other.end > self.end {
            return Some(Window {
                start: other.start.clone(),
                end: self.end.clone(),
            });
        }
        if self.start < other.end && self.end > other.end {
            return Some(Window {
                start: self.start.clone(),
                end: other.end.clone(),
            });
        }
        None
    }
}

pub fn find_windows<F, T, R, E>(
    func: F,
    start: Time<T>,
    end: Time<T>,
    steps: &[f64],
    root_finder: R,
) -> Result<Vec<Window<T>>, RootFinderError>
where
    F: Fn(f64) -> Result<f64, E> + Copy,
    T: TimeScale + Clone,
    E: Display,
    R: FindBracketedRoot<F, E>,
{
    match find_events(func, start.clone(), steps, root_finder) {
        Err(FindEventError::AlwaysNegative) => Ok(vec![]),
        Err(FindEventError::AlwaysPositive) => Ok(vec![Window { start, end }]),
        Err(FindEventError::RootFinder(err)) => Err(err),
        Ok(events) => {
            let mut events: VecDeque<Event<T>> = events.into();

            if events.is_empty() {
                return Ok(vec![]);
            }

            // If the first event is a downcrossing, insert an upcrossing at the start
            if events.front().unwrap().crossing == ZeroCrossing::Down {
                events.push_front(Event {
                    crossing: ZeroCrossing::Up,
                    time: start,
                });
            }

            // If the last event is an upcrossing, insert a downcrossing at the end
            if events.back().unwrap().crossing == ZeroCrossing::Up {
                events.push_back(Event {
                    crossing: ZeroCrossing::Down,
                    time: end,
                });
            }

            let mut windows = Vec::with_capacity(events.len() / 2);

            for (start, end) in events.into_iter().tuples() {
                debug_assert!(start.crossing == ZeroCrossing::Up);
                debug_assert!(end.crossing == ZeroCrossing::Down);

                windows.push(Window {
                    start: start.time,
                    end: end.time,
                });
            }

            Ok(windows)
        }
    }
}

pub fn intersect_windows<T>(w1: &[Window<T>], w2: &[Window<T>]) -> Vec<Window<T>>
where
    T: TimeScale + Ord + Clone,
{
    let mut output = vec![];
    for w1 in w1 {
        for w2 in w2 {
            if let Some(w) = w1.intersect(w2) {
                output.push(w)
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_math::roots::Brent;
    use lox_test_utils::assert_approx_eq;
    use lox_time::time_scales::Tai;
    use lox_time::{Time, time};
    use std::convert::Infallible;
    use std::f64::consts::{PI, TAU};

    #[test]
    fn test_events() {
        let func = |t: f64| Ok::<f64, Infallible>(t.sin());
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];

        let root_finder = Brent::default();

        let events = find_events(func, start, &steps, root_finder).unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].crossing, ZeroCrossing::Down);
        assert_approx_eq!(
            events[0].time,
            start + TimeDelta::from_seconds_f64(PI),
            rtol <= 1e-6
        );
        assert_eq!(events[1].crossing, ZeroCrossing::Up);
        assert_approx_eq!(
            events[1].time,
            start + TimeDelta::from_seconds_f64(TAU),
            rtol <= 1e-6
        );
    }

    #[test]
    fn test_windows() {
        let func = |t: f64| Ok::<f64, Infallible>(t.sin());
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let end = start + TimeDelta::from_seconds(7);

        let root_finder = Brent::default();

        let windows = find_windows(func, start, end, &steps, root_finder).unwrap();

        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].start, start);
        assert_approx_eq!(
            windows[0].end,
            start + TimeDelta::from_seconds_f64(PI),
            rtol <= 1e-6
        );
    }

    #[test]
    fn test_windows_no_windows() {
        let func = |_: f64| Ok::<f64, Infallible>(-1.0);
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let end = start + TimeDelta::from_seconds(7);

        let root_finder = Brent::default();

        let windows = find_windows(func, start, end, &steps, root_finder).unwrap();

        assert!(windows.is_empty());
    }

    #[test]
    fn test_windows_full_coverage() {
        let func = |_: f64| Ok::<f64, Infallible>(1.0);
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let end = start + TimeDelta::from_seconds(7);

        let root_finder = Brent::default();

        let windows = find_windows(func, start, end, &steps, root_finder).unwrap();

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].start, start);
        assert_eq!(windows[0].end, end);
    }
}
