/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::ops::RangeInclusive;

use crate::deltas::TimeDelta;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeDeltaRange {
    start: TimeDelta,
    end: TimeDelta,
    step: Option<TimeDelta>,
    curr: Option<TimeDelta>,
}

impl TimeDeltaRange {
    pub fn new(start: TimeDelta, end: TimeDelta) -> Self {
        TimeDeltaRange {
            start,
            end,
            step: None,
            curr: None,
        }
    }

    pub fn with_step(mut self, step: TimeDelta) -> Self {
        self.step = Some(step);
        self
    }

    pub fn start(&self) -> TimeDelta {
        self.start
    }

    pub fn end(&self) -> TimeDelta {
        self.end
    }

    pub fn step(&self) -> TimeDelta {
        self.step.unwrap_or(TimeDelta::from_seconds(1))
    }
}

impl From<RangeInclusive<i64>> for TimeDeltaRange {
    fn from(range: RangeInclusive<i64>) -> Self {
        TimeDeltaRange::new(
            TimeDelta::from_seconds(*range.start()),
            TimeDelta::from_seconds(*range.end()),
        )
    }
}

impl From<RangeInclusive<i32>> for TimeDeltaRange {
    fn from(range: RangeInclusive<i32>) -> Self {
        TimeDeltaRange::new(
            TimeDelta::from_seconds(*range.start() as i64),
            TimeDelta::from_seconds(*range.end() as i64),
        )
    }
}

impl Iterator for TimeDeltaRange {
    type Item = TimeDelta;

    fn next(&mut self) -> Option<Self::Item> {
        match self.curr {
            None => {
                self.curr = Some(self.start);
                self.curr
            }
            Some(curr) => {
                let next = curr + self.step();
                if next <= self.end {
                    self.curr = Some(next);
                    self.curr
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_delta_range() {
        let range = TimeDeltaRange::new(TimeDelta::from_seconds(0), TimeDelta::from_seconds(10));
        let values: Vec<TimeDelta> = range.collect();
        assert_eq!(values.len(), 11);
        assert_eq!(values[0], TimeDelta::from_seconds(0));
        assert_eq!(values[1], TimeDelta::from_seconds(1));
        assert_eq!(values[10], TimeDelta::from_seconds(10));
    }
}
