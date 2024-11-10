/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;

use lox_math::types::julian_dates::Epoch;

use crate::{Body, Ephemeris, Position, Velocity};

use super::parser::{DafSpkError, Spk, SpkSegment, SpkType2Array, SpkType2Coefficients};

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
    ) -> Result<(&Vec<SpkType2Coefficients>, f64), DafSpkError> {
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
    ) -> Result<(Vec<f64>, &Vec<SpkType2Coefficients>), DafSpkError> {
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
}

impl Ephemeris for Spk {
    type Error = DafSpkError;

    fn position(&self, epoch: Epoch, origin: Body, target: Body) -> Result<Position, DafSpkError> {
        let (segment, sign) = self.find_segment(origin, target)?;

        if epoch < segment.initial_epoch || epoch > segment.final_epoch {
            return Err(DafSpkError::UnableToFindMatchingSegment);
        }

        let mut x = 0f64;
        let mut y = 0f64;
        let mut z = 0f64;

        match &segment.data {
            super::parser::SpkArray::Type2(array) => {
                let (polynomial, record) = self.get_chebyshev_polynomial(epoch, segment)?;
                let sign = sign as f64;

                let degree_of_polynomial = array.degree_of_polynomial() as usize;

                #[allow(clippy::needless_range_loop)]
                for i in 0..degree_of_polynomial {
                    x += sign * record[i].x * polynomial[i];
                    y += sign * record[i].y * polynomial[i];
                    z += sign * record[i].z * polynomial[i];
                }
            }
        }

        Ok((x, y, z))
    }

    fn velocity(&self, epoch: Epoch, origin: Body, target: Body) -> Result<Velocity, DafSpkError> {
        let (segment, sign) = self.find_segment(origin, target)?;

        if epoch < segment.initial_epoch || epoch > segment.final_epoch {
            return Err(DafSpkError::UnableToFindMatchingSegment);
        }

        let mut x = 0f64;
        let mut y = 0f64;
        let mut z = 0f64;

        match &segment.data {
            super::parser::SpkArray::Type2(array) => {
                let (polynomial, record) = self.get_chebyshev_polynomial(epoch, segment)?;
                let sign = sign as f64;

                let degree_of_polynomial = array.degree_of_polynomial() as usize;

                let mut derivative = Vec::<f64>::with_capacity(degree_of_polynomial);

                derivative.push(0f64);
                derivative.push(1f64);

                if degree_of_polynomial > 2 {
                    derivative.push(4f64 * polynomial[1]);
                    for i in 3..degree_of_polynomial {
                        let x = 2f64 * polynomial[1] * derivative[i - 1] - derivative[i - 2]
                            + polynomial[i - 1]
                            + polynomial[i - 1];

                        derivative.push(x);
                    }
                }

                let derivative: Vec<f64> = derivative
                    .iter()
                    .map(|d| 2.0 * d / array.intlen as f64)
                    .collect();

                #[allow(clippy::needless_range_loop)]
                for i in 0..degree_of_polynomial {
                    x += sign * record[i].x * derivative[i];
                    y += sign * record[i].y * derivative[i];
                    z += sign * record[i].z * derivative[i];
                }
            }
        }

        Ok((x, y, z))
    }

    fn state(
        &self,
        epoch: Epoch,
        origin: Body,
        target: Body,
    ) -> Result<(Position, Velocity), DafSpkError> {
        let position = self.position(epoch, origin, target)?;
        let velocity = self.velocity(epoch, origin, target)?;

        Ok((position, velocity))
    }
}

#[cfg(test)]
mod test {
    use crate::spk::parser::parse_daf_spk;
    use crate::spk::parser::test::{get_expected_segments, FILE_CONTENTS};

    use super::*;

    #[test]
    fn test_unable_to_find_segment() {
        let spk = parse_daf_spk(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        assert_eq!(
            Err(DafSpkError::UnableToFindMatchingSegment),
            spk.position(2457388.5000000 as Epoch, 1, 2)
        );
    }

    #[test]
    fn test_position() {
        let spk = parse_daf_spk(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        assert_eq!(
            Ok((-32703259.291699532, 31370540.51993667, 20159681.594182793)),
            spk.position(-14200747200.0 as Epoch, 0, 1)
        );
    }

    #[test]
    fn test_velocity() {
        let spk = parse_daf_spk(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        assert_eq!(
            Ok((
                -46.723420416476635,
                -28.050723083678367,
                -10.055174230490163,
            )),
            spk.velocity(-14200747200.0 as Epoch, 0, 1)
        );
    }

    #[test]
    fn test_state() {
        let spk = parse_daf_spk(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        assert_eq!(
            Ok((
                (-32703259.291699532, 31370540.51993667, 20159681.594182793),
                (
                    -46.723420416476635,
                    -28.050723083678367,
                    -10.055174230490163,
                ),
            )),
            spk.state(-14200747200.0 as Epoch, 0, 1)
        );
    }

    #[test]
    fn test_get_segments() {
        let spk = parse_daf_spk(&FILE_CONTENTS).expect("Unable to parse DAF/SPK");

        assert_eq!(&get_expected_segments(), spk.get_segments());
    }
}
