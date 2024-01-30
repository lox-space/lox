/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module tt provides a type representing instants in the TT timescale and functions for
//! converting between TT and other timescales.

use crate::time::continuous::{RawTime, TimeDelta, UT1};
use crate::time::WallClock;
use crate::wall_clock;

/// Terrestrial Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TT(RawTime);

wall_clock!(TT, tt_wall_clock_tests);

impl TT {
    pub fn new(t: RawTime) -> Self {
        Self(t)
    }

    pub fn to_ut1(&self, dt: TimeDelta) -> UT1 {
        UT1::new(self.0 - dt)
    }

    pub fn to_raw(&self) -> RawTime {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn test_TT_new() {
        let expected = TT(RawTime {
            seconds: 1,
            attoseconds: 0,
        });
        let actual = TT::new(RawTime {
            seconds: 1,
            attoseconds: 0,
        });
        assert_eq!(expected, actual);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_TT_to_raw() {
        let tt = TT(RawTime {
            seconds: 1,
            attoseconds: 0,
        });
        let expected = RawTime {
            seconds: 1,
            attoseconds: 0,
        };
        let actual = tt.to_raw();
        assert_eq!(expected, actual);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_TT_to_UT1() {
        let tt = TT(RawTime {
            seconds: 0,
            attoseconds: 0,
        });
        let dt = TimeDelta {
            seconds: 1,
            attoseconds: 0,
        };
        let expected = UT1::new(RawTime {
            seconds: -1,
            attoseconds: 0,
        });
        let actual = tt.to_ut1(dt);
        assert_eq!(expected, actual);
    }
}
