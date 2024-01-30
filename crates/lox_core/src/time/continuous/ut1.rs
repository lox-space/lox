/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module ut1 provides a type representing instants in the UT1 timescale and functions for
//! converting between UT1 and other timescales.

use crate::time::continuous::{RawTime, TimeDelta, TT};
use crate::time::WallClock;
use crate::wall_clock;

/// Universal Time. Defaults to the J2000 epoch.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct UT1(RawTime);

wall_clock!(UT1, ut1_wall_clock_tests);

impl UT1 {
    pub fn new(t: RawTime) -> Self {
        Self(t)
    }

    pub fn to_raw(&self) -> RawTime {
        self.0
    }

    /// Convert from UT1 to TT given a user-supplied value for ΔT.
    pub fn to_tt(&self, dt: TimeDelta) -> TT {
        TT::new(self.0 + dt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn test_UT1_new() {
        let expected = UT1(RawTime {
            seconds: 1,
            attoseconds: 0,
        });
        let actual = UT1::new(RawTime {
            seconds: 1,
            attoseconds: 0,
        });
        assert_eq!(expected, actual);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_UT1_to_raw() {
        let ut1 = UT1(RawTime {
            seconds: 1,
            attoseconds: 0,
        });
        let expected = RawTime {
            seconds: 1,
            attoseconds: 0,
        };
        let actual = ut1.to_raw();
        assert_eq!(expected, actual);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_UT1_to_TT() {
        let ut1 = UT1(RawTime {
            seconds: 0,
            attoseconds: 0,
        });
        let dt = TimeDelta {
            seconds: 1,
            attoseconds: 0,
        };
        let expected = TT::new(RawTime {
            seconds: 1,
            attoseconds: 0,
        });
        let actual = ut1.to_tt(dt);
        assert_eq!(expected, actual);
    }
}
