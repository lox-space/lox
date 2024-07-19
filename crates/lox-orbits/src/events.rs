use itertools::Itertools;
use std::collections::VecDeque;
use std::fmt::Display;
use std::iter::zip;
use thiserror::Error;

use lox_math::roots::FindBracketedRoot;
use lox_time::deltas::TimeDelta;
use lox_time::TimeLike;

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

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum FindEventError {
    #[error("function is always negative")]
    AlwaysNegative,
    #[error("function is always positive")]
    AlwaysPositive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Event<T: TimeLike> {
    crossing: ZeroCrossing,
    time: T,
}

impl<T: TimeLike> Event<T> {
    pub fn time(&self) -> &T {
        &self.time
    }

    pub fn crossing(&self) -> ZeroCrossing {
        self.crossing
    }
}

pub fn find_events<F: Fn(f64) -> f64 + Copy, T: TimeLike + Clone, R: FindBracketedRoot<F>>(
    func: F,
    start: T,
    steps: &[f64],
    root_finder: R,
) -> Result<Vec<Event<T>>, FindEventError> {
    // Determine the sign of `func` at each time step
    let signs: Vec<f64> = steps.iter().map(|&t| func(t).signum()).collect();

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
                .expect("sign changed but root finder failed");
            let time = start.clone() + TimeDelta::from_decimal_seconds(t).unwrap();

            events.push(Event { crossing, time });
        }
    }

    Ok(events)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Window<T: TimeLike> {
    start: T,
    end: T,
}

impl<T: TimeLike> Window<T> {
    pub fn new(start: T, end: T) -> Self {
        Window { start, end }
    }

    pub fn start(&self) -> &T {
        &self.start
    }

    pub fn end(&self) -> &T {
        &self.end
    }

    pub fn duration(&self) -> TimeDelta
    where
        T: Clone,
    {
        self.end.clone() - self.start.clone()
    }
}

pub fn find_windows<F: Fn(f64) -> f64 + Copy, T: TimeLike + Clone, R: FindBracketedRoot<F>>(
    func: F,
    start: T,
    end: T,
    steps: &[f64],
    root_finder: R,
) -> Vec<Window<T>> {
    let events = find_events(func, start.clone(), steps, root_finder);

    match events {
        Err(error) => match error {
            FindEventError::AlwaysNegative => vec![],
            FindEventError::AlwaysPositive => vec![Window { start, end }],
        },
        Ok(events) => {
            let mut events: VecDeque<Event<T>> = events.into();

            if events.is_empty() {
                return vec![];
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

            windows
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_math::assert_close;
    use lox_math::is_close::IsClose;
    use lox_math::roots::Brent;
    use lox_time::time_scales::Tai;
    use lox_time::{time, Time};
    use std::f64::consts::{PI, TAU};

    #[test]
    fn test_events() {
        let func = |t: f64| t.sin();
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];

        let root_finder = Brent::default();

        let events = find_events(func, start, &steps, root_finder).unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].crossing, ZeroCrossing::Down);
        assert_close!(
            events[0].time,
            start + TimeDelta::from_decimal_seconds(PI).unwrap(),
            1e-6
        );
        assert_eq!(events[1].crossing, ZeroCrossing::Up);
        assert_close!(
            events[1].time,
            start + TimeDelta::from_decimal_seconds(TAU).unwrap(),
            1e-6
        );
    }

    #[test]
    fn test_windows() {
        let func = |t: f64| t.sin();
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let end = start + TimeDelta::from_seconds(7);

        let root_finder = Brent::default();

        let windows = find_windows(func, start, end, &steps, root_finder);

        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].start, start);
        assert_close!(
            windows[0].end,
            start + TimeDelta::from_decimal_seconds(PI).unwrap(),
            1e-6
        );
    }

    #[test]
    fn test_windows_no_windows() {
        let func = |_: f64| -1.0;
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let end = start + TimeDelta::from_seconds(7);

        let root_finder = Brent::default();

        let windows = find_windows(func, start, end, &steps, root_finder);

        assert!(windows.is_empty());
    }

    #[test]
    fn test_windows_full_coverage() {
        let func = |_: f64| 1.0;
        let start = time!(Tai, 2000, 1, 1, 12).unwrap();
        let steps = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let end = start + TimeDelta::from_seconds(7);

        let root_finder = Brent::default();

        let windows = find_windows(func, start, end, &steps, root_finder);

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].start, start);
        assert_eq!(windows[0].end, end);
    }
}
