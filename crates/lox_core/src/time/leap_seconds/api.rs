use crate::time::leap_seconds::gen::{LEAP_SECONDS, LS_EPOCHS};

const MJD_EPOCH: f64 = 2400000.5;
const SECONDS_PER_DAY: f64 = 86400.0;

// Constants for calculating the offset between TAI and UTC for
// dates between 1960-01-01 and 1972-01-01
// See ftp://maia.usno.navy.mil/ser7/tai-utc.dat
// Section taken from
// https://github.com/JuliaTime/LeapSeconds.jl/blob/master/src/LeapSeconds.jl#L16

const EPOCHS: [u64; 14] = [
    36934, 37300, 37512, 37665, 38334, 38395, 38486, 38639, 38761, 38820, 38942, 39004, 39126,
    39887,
];

const OFFSETS: [f64; 14] = [
    1.417818, 1.422818, 1.372818, 1.845858, 1.945858, 3.240130, 3.340130, 3.440130, 3.540130,
    3.640130, 3.740130, 3.840130, 4.313170, 4.213170,
];

const DRIFT_EPOCHS: [u64; 14] = [
    37300, 37300, 37300, 37665, 37665, 38761, 38761, 38761, 38761, 38761, 38761, 38761, 39126,
    39126,
];

const DRIFT_RATES: [f64; 14] = [
    0.0012960, 0.0012960, 0.0012960, 0.0011232, 0.0011232, 0.0012960, 0.0012960, 0.0012960,
    0.0012960, 0.0012960, 0.0012960, 0.0012960, 0.0025920, 0.0025920,
];

#[derive(PartialEq, Debug)]
pub enum LeapSecondError {
    // UTC is not defined for dates before 1960-01-01.
    UTCDateBefore1960,
    UTCDateOutOfRange,
}

/// This type is used for increased precision
///
/// The first part is the UTC day number, and the second part is the second offset.
type TwoPartDateTime = (f64, f64);

fn is_sorted(array: &[u64]) -> bool {
    array.windows(2).all(|x| x[0] <= x[1])
}

fn leap_seconds(mjd: f64) -> Result<f64, LeapSecondError> {
    // Invariant: LS_EPOCHS must be sorted for the search below to work
    assert!(is_sorted(&LS_EPOCHS));

    let threshold = mjd.floor() as u64;
    let position = LS_EPOCHS
        .iter()
        .rposition(|item| item <= &threshold)
        .ok_or(LeapSecondError::UTCDateOutOfRange)?;

    Ok(LEAP_SECONDS[position])
}

/// Returns the difference between UTC and TAI for a given date
///
/// Input is a two-part UTC Julian datetime.
pub fn offset_utc_tai(utc_date_time: TwoPartDateTime) -> Result<f64, LeapSecondError> {
    // This function uses the [ERFA convention](https://github.com/liberfa/erfa/blob/master/src/dtf2d.c#L49)
    // for Julian day numbers representing UTC dates during leap seconds.
    let mjd = utc_date_time.0 - MJD_EPOCH + utc_date_time.1;

    // Before 1960-01-01
    if mjd < 36934.0 {
        return Err(LeapSecondError::UTCDateBefore1960);
    }

    // Before 1972-01-01
    if mjd < LS_EPOCHS[1] as f64 {
        // Invariant: EPOCHS must be sorted for the search below to work
        debug_assert!(is_sorted(&EPOCHS));

        let threshold = mjd.floor() as u64;
        let position = EPOCHS
            .iter()
            .rposition(|item| item <= &threshold)
            .ok_or(LeapSecondError::UTCDateOutOfRange)?;

        let offset =
            OFFSETS[position] + (mjd - DRIFT_EPOCHS[position] as f64) * DRIFT_RATES[position];

        return Ok(-offset);
    }

    let mut offset = 0 as f64;
    for _ in 1..=3 {
        offset = leap_seconds(mjd + offset / SECONDS_PER_DAY)?;
    }

    Ok(-offset)
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_offset_utc_tai() {
        // Values validated against LeapSeconds.jl

        // datetime2julian(DateTime(1990, 1, 1))
        assert_eq!(offset_utc_tai((2.4478925e6, 0f64)), Ok(-25.0));
        // datetime2julian(DateTime(2000, 1, 1))
        assert_eq!(offset_utc_tai((2.4515445e6, 0f64)), Ok(-32.0));
        // datetime2julian(DateTime(2020, 1, 1))
        assert_eq!(offset_utc_tai((2.4577545e6, 0f64)), Ok(-37.0));
    }
}
