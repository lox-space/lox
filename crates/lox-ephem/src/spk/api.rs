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
use lox_core::types::julian_dates::Epoch;
use lox_time::Time;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;

use crate::{Ephemeris, path_from_ids};

use super::parser::{DafSpkError, Spk, SpkSegment, SpkType2Array, SpkType2Coefficients};

type Body = i32;

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

        // An SPK file may contain any number of segments. A single file may contain overlapping segments:
        // segments containing data for the same body over a common interval. When this happens, the
        // latest segment in a file supersedes any competing segments earlier in the file.
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

    fn find_record<'a>(
        &'a self,
        array: &'a SpkType2Array,
        initial_epoch: Epoch,
        epoch: Epoch,
    ) -> Result<(&'a Vec<SpkType2Coefficients>, f64), DafSpkError> {
        let seconds_from_record_start = epoch - initial_epoch;

        let intlen = array.intlen as f64;
        let mut record_number = (seconds_from_record_start / intlen).floor() as usize;
        let mut fraction = seconds_from_record_start % intlen;

        // Chebyshev piecewise polynomials overlap at patchpoints. This means that one
        // can safely take the end of the interval from the next record. But this implies
        // special handling of the last record, where there's no next record that we can
        // draw from.
        if record_number == array.n as usize {
            record_number -= 1;
            fraction = array.intlen as f64;
        }

        let record = array
            .records
            .get(record_number)
            .ok_or(DafSpkError::UnableToFindMatchingRecord)?;

        Ok((record, fraction))
    }

    pub fn get_segments(&self) -> &HashMap<i32, HashMap<i32, Vec<SpkSegment>>> {
        &self.segments
    }

    fn get_chebyshev_polynomial<'a>(
        &'a self,
        epoch: Epoch,
        segment: &'a SpkSegment,
    ) -> Result<(Vec<f64>, &'a Vec<SpkType2Coefficients>), DafSpkError> {
        let (coefficients, record) = match &segment.data {
            super::parser::SpkArray::Type2(array) => {
                let (record, fraction) = self.find_record(array, segment.initial_epoch, epoch)?;

                let degree_of_polynomial = array.degree_of_polynomial() as usize;
                let mut coefficients = Vec::<f64>::with_capacity(degree_of_polynomial);

                coefficients.push(1f64);
                coefficients.push(2f64 * fraction / array.intlen as f64 - 1f64);

                for i in 2..degree_of_polynomial {
                    coefficients
                        .push(2f64 * coefficients[1] * coefficients[i - 1] - coefficients[i - 2]);
                }

                (coefficients, record)
            }
        };

        Ok((coefficients, record))
    }

    /// Compute the state (position and velocity) for a single adjacent body pair.
    /// Returns a `Cartesian` in meters and m/s (converted from SPK km and km/s).
    fn segment_state(
        &self,
        epoch: Epoch,
        origin: Body,
        target: Body,
    ) -> Result<Cartesian, DafSpkError> {
        let (segment, sign) = self.find_segment(origin, target)?;

        if epoch < segment.initial_epoch || epoch > segment.final_epoch {
            return Err(DafSpkError::UnableToFindMatchingSegment);
        }

        let mut px = 0f64;
        let mut py = 0f64;
        let mut pz = 0f64;
        let mut vx = 0f64;
        let mut vy = 0f64;
        let mut vz = 0f64;

        match &segment.data {
            super::parser::SpkArray::Type2(array) => {
                let (polynomial, record) = self.get_chebyshev_polynomial(epoch, segment)?;
                let sign = sign as f64;
                let degree_of_polynomial = array.degree_of_polynomial() as usize;

                // Compute position
                #[allow(clippy::needless_range_loop)]
                for i in 0..degree_of_polynomial {
                    px += sign * record[i].x * polynomial[i];
                    py += sign * record[i].y * polynomial[i];
                    pz += sign * record[i].z * polynomial[i];
                }

                // Compute velocity derivative
                let mut derivative = Vec::<f64>::with_capacity(degree_of_polynomial);
                derivative.push(0f64);
                derivative.push(1f64);

                if degree_of_polynomial > 2 {
                    derivative.push(4f64 * polynomial[1]);
                    for i in 3..degree_of_polynomial {
                        let d = 2f64 * polynomial[1] * derivative[i - 1] - derivative[i - 2]
                            + polynomial[i - 1]
                            + polynomial[i - 1];
                        derivative.push(d);
                    }
                }

                let derivative: Vec<f64> = derivative
                    .iter()
                    .map(|d| 2.0 * d / array.intlen as f64)
                    .collect();

                #[allow(clippy::needless_range_loop)]
                for i in 0..degree_of_polynomial {
                    vx += sign * record[i].x * derivative[i];
                    vy += sign * record[i].y * derivative[i];
                    vz += sign * record[i].z * derivative[i];
                }
            }
        }

        // Convert km → m, km/s → m/s
        let pos = DVec3::new(px, py, pz) * 1e3;
        let vel = DVec3::new(vx, vy, vz) * 1e3;
        Ok(Cartesian::from_vecs(pos, vel))
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
        let epoch = time.seconds_since_j2000();
        let path = path_from_ids(origin.id().0, target.id().0);
        let mut result = Cartesian::default();
        for (from, to) in path.into_iter().tuple_windows() {
            result += self.segment_state(epoch, from, to)?;
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

    use crate::spk::parser::parse_daf_spk;
    use crate::spk::parser::test::{FILE_CONTENTS, get_expected_segments};

    use super::*;

    fn test_epoch() -> Time<Tdb> {
        // -14200747200.0 seconds since J2000
        Time::j2000(Tdb) + TimeDelta::from_seconds_f64(-14200747200.0)
    }

    #[test]
    fn test_unable_to_find_segment() {
        let spk = parse_daf_spk(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        // Use an epoch that doesn't match any segment
        let bad_epoch = Time::from_two_part_julian_date(Tdb, 2457388.5, 0.0);
        let result = spk.state(bad_epoch, SolarSystemBarycenter, MercuryBarycenter);
        assert!(result.is_err());
    }

    #[test]
    fn test_state() {
        let spk = parse_daf_spk(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        let state = spk
            .state(test_epoch(), SolarSystemBarycenter, MercuryBarycenter)
            .unwrap();

        // Original SPK values were in km, now converted to meters
        let expected_pos = DVec3::new(
            -32703259.291699532e3,
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
        let spk = parse_daf_spk(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        let pos = spk
            .position(test_epoch(), SolarSystemBarycenter, MercuryBarycenter)
            .unwrap();

        let expected = DVec3::new(
            -32703259.291699532e3,
            31370540.51993667e3,
            20159681.594182793e3,
        );

        assert_eq!(pos, expected);
    }

    #[test]
    fn test_get_segments() {
        let spk = parse_daf_spk(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        assert_eq!(&get_expected_segments(), spk.get_segments());
    }
}
