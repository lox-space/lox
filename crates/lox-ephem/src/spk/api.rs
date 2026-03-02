// SPDX-FileCopyrightText: 2023 Andrei Zisu <matzipan@gmail.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use glam::DVec3;
use itertools::Itertools;
use lox_bodies::Origin;
use lox_core::coords::Cartesian;
use lox_core::time::deltas::{Seconds, ToDelta};
use lox_time::Time;
use lox_time::time_scales::Tdb;

use crate::{Ephemeris, path_from_ids};

use super::parser::{DafSpkError, Spk, SpkArray, SpkSegment, SpkType2Coefficients};

type Body = i32;

const MAX_CHEBYSHEV_DEGREE: usize = 32;

/// Common setup for Chebyshev evaluation: finds the segment, validates the epoch,
/// and computes the Chebyshev argument `x`, record index, and polynomial degree.
struct ChebyshevSetup<'a> {
    record: &'a [SpkType2Coefficients],
    n: usize,
    x: f64,
    intlen: f64,
    sign: f64,
}

impl Spk {
    fn find_segment(
        &self,
        origin: Body,
        target: Body,
    ) -> Result<(&SpkSegment, isize), DafSpkError> {
        let mut sign = 1;

        let mut target = target;
        let mut origin = origin;
        if target < origin {
            (origin, target) = (target, origin);
            sign = -1;
        }

        // An SPK file may contain any number of segments. A single file may contain overlapping
        // segments: segments containing data for the same body over a common interval. When this
        // happens, the latest segment in a file supersedes any competing segments earlier in the
        // file.
        let segment = self
            .segments
            .get(&origin)
            .ok_or(DafSpkError::UnableToFindMatchingSegment)?
            .get(&target)
            .ok_or(DafSpkError::UnableToFindMatchingSegment)?
            .last()
            .ok_or(DafSpkError::UnableToFindMatchingSegment)?;

        Ok((segment, sign))
    }

    /// Performs the common setup for Chebyshev evaluation: segment lookup, epoch
    /// bounds check, high-precision time computation, record selection.
    fn chebyshev_setup(
        &self,
        time: Time<Tdb>,
        origin: Body,
        target: Body,
    ) -> Result<ChebyshevSetup<'_>, DafSpkError> {
        let (segment, sign) = self.find_segment(origin, target)?;

        // Coarse epoch bounds check using f64
        let epoch = time.to_delta().to_seconds().to_f64();
        if epoch < segment.initial_epoch || epoch > segment.final_epoch {
            return Err(DafSpkError::UnableToFindMatchingSegment);
        }

        let SpkArray::Type2(array) = &segment.data;

        // High-precision time computation using two-part Seconds
        let time_seconds = time.to_delta().to_seconds();
        let initial_seconds = Seconds::from_f64(segment.initial_epoch);
        let seconds_from_start = time_seconds - initial_seconds;

        let intlen = array.intlen as f64;
        let mut record_number = (seconds_from_start.to_f64() / intlen).floor() as usize;
        let mut fraction = seconds_from_start - Seconds::from_f64(record_number as f64 * intlen);

        // Chebyshev piecewise polynomials overlap at patchpoints. This means that one
        // can safely take the end of the interval from the next record. But this implies
        // special handling of the last record, where there's no next record that we can
        // draw from.
        if record_number == array.n as usize {
            record_number -= 1;
            fraction = Seconds::from_f64(intlen);
        }

        let record = array
            .records
            .get(record_number)
            .ok_or(DafSpkError::UnableToFindMatchingRecord)?;

        // Chebyshev argument in [-1, 1] with compensated arithmetic
        let x = (fraction * (2.0 / intlen) - Seconds::from_f64(1.0)).to_f64();
        let n = array.degree_of_polynomial() as usize;

        Ok(ChebyshevSetup {
            record,
            n,
            x,
            intlen,
            sign: sign as f64,
        })
    }

    pub fn get_segments(&self) -> &HashMap<i32, HashMap<i32, Vec<SpkSegment>>> {
        &self.segments
    }

    /// Compute the state (position and velocity) for a single adjacent body pair.
    /// Returns a `Cartesian` in meters and m/s (converted from SPK km and km/s).
    fn segment_state(
        &self,
        time: Time<Tdb>,
        origin: Body,
        target: Body,
    ) -> Result<Cartesian, DafSpkError> {
        let setup = self.chebyshev_setup(time, origin, target)?;
        let ChebyshevSetup {
            record,
            n,
            x,
            intlen,
            sign,
        } = setup;

        // Chebyshev polynomials T_i(x) on the stack
        let mut t = [0.0; MAX_CHEBYSHEV_DEGREE];
        t[0] = 1.0;
        t[1] = x;
        for i in 2..n {
            t[i] = 2.0 * x * t[i - 1] - t[i - 2];
        }

        // Position
        let mut pos = DVec3::ZERO;
        for i in 0..n {
            pos += t[i] * record[i].to_dvec3();
        }
        pos *= sign;

        // Chebyshev derivative T'_i(x)
        let mut dt = [0.0; MAX_CHEBYSHEV_DEGREE];
        dt[1] = 1.0;
        if n > 2 {
            dt[2] = 4.0 * x;
            for i in 3..n {
                dt[i] = 2.0 * x * dt[i - 1] - dt[i - 2] + 2.0 * t[i - 1];
            }
        }

        let scale = 2.0 / intlen;
        let mut vel = DVec3::ZERO;
        for i in 0..n {
            vel += (scale * dt[i]) * record[i].to_dvec3();
        }
        vel *= sign;

        // Convert km → m, km/s → m/s
        Ok(Cartesian::from_vecs(pos * 1e3, vel * 1e3))
    }

    /// Compute only the position for a single adjacent body pair.
    /// Returns a `DVec3` in meters (converted from SPK km).
    fn segment_position(
        &self,
        time: Time<Tdb>,
        origin: Body,
        target: Body,
    ) -> Result<DVec3, DafSpkError> {
        let setup = self.chebyshev_setup(time, origin, target)?;
        let ChebyshevSetup {
            record, n, x, sign, ..
        } = setup;

        // Chebyshev polynomials T_i(x) on the stack
        let mut t = [0.0; MAX_CHEBYSHEV_DEGREE];
        t[0] = 1.0;
        t[1] = x;
        for i in 2..n {
            t[i] = 2.0 * x * t[i - 1] - t[i - 2];
        }

        // Position only
        let mut pos = DVec3::ZERO;
        for i in 0..n {
            pos += t[i] * record[i].to_dvec3();
        }
        pos *= sign;

        // Convert km → m
        Ok(pos * 1e3)
    }
}

impl Ephemeris for Spk {
    type Error = DafSpkError;

    fn state<O1: Origin, O2: Origin>(
        &self,
        time: Time<Tdb>,
        origin: O1,
        target: O2,
    ) -> Result<Cartesian, DafSpkError> {
        let path = path_from_ids(origin.id().0, target.id().0);
        let mut result = Cartesian::default();
        for (from, to) in path.into_iter().tuple_windows() {
            result += self.segment_state(time, from, to)?;
        }
        Ok(result)
    }

    fn position<O1: Origin, O2: Origin>(
        &self,
        time: Time<Tdb>,
        origin: O1,
        target: O2,
    ) -> Result<DVec3, DafSpkError> {
        let path = path_from_ids(origin.id().0, target.id().0);
        let mut result = DVec3::ZERO;
        for (from, to) in path.into_iter().tuple_windows() {
            result += self.segment_position(time, from, to)?;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use glam::DVec3;
    use lox_bodies::{MercuryBarycenter, SolarSystemBarycenter};
    use lox_time::Time;
    use lox_time::deltas::TimeDelta;
    use lox_time::time_scales::Tdb;

    use crate::spk::parser::test::{FILE_CONTENTS, get_expected_segments};

    use super::*;

    fn test_epoch() -> Time<Tdb> {
        // -14200747200.0 seconds since J2000
        Time::j2000(Tdb) + TimeDelta::from_seconds_f64(-14200747200.0)
    }

    #[test]
    fn test_unable_to_find_segment() {
        let spk = Spk::from_bytes(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        // Use an epoch that doesn't match any segment
        let bad_epoch = Time::from_two_part_julian_date(Tdb, 2457388.5, 0.0);
        let result = spk.state(bad_epoch, SolarSystemBarycenter, MercuryBarycenter);
        assert!(result.is_err());
    }

    #[test]
    fn test_state() {
        let spk = Spk::from_bytes(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        let state = spk
            .state(test_epoch(), SolarSystemBarycenter, MercuryBarycenter)
            .unwrap();

        // Original SPK values were in km, now converted to meters
        let expected_pos = DVec3::new(
            -3.270_325_929_169_953e10,
            31370540.51993667e3,
            20159681.594182793e3,
        );
        let expected_vel = DVec3::new(
            -46.723420416476635e3,
            -28.050723083678367e3,
            -10.055174230490163e3,
        );

        assert_eq!(state.position(), expected_pos);
        assert_eq!(state.velocity(), expected_vel);
    }

    #[test]
    fn test_position() {
        let spk = Spk::from_bytes(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        let pos = spk
            .position(test_epoch(), SolarSystemBarycenter, MercuryBarycenter)
            .unwrap();

        let expected = DVec3::new(
            -3.270_325_929_169_953e10,
            31370540.51993667e3,
            20159681.594182793e3,
        );

        assert_eq!(pos, expected);
    }

    #[test]
    fn test_get_segments() {
        let spk = Spk::from_bytes(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        assert_eq!(&get_expected_segments(), spk.get_segments());
    }
}
