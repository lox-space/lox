use crate::time::constants;
use crate::time::epochs::Epoch;

/// Although strictly TDB, TT is sufficient for most applications.
pub type TDBJulianCenturiesSinceJ2000 = f64;

pub fn tdb_julian_centuries_since_j2000(epoch: Epoch) -> TDBJulianCenturiesSinceJ2000 {
    match epoch {
        Epoch::TT(_) | Epoch::TDB(_) => {
            epoch.days_since_j2000() / constants::f64::DAYS_PER_JULIAN_CENTURY
        }
        _ => todo!("perform the simpler of the conversions to TT or TDB first"),
    }
}

pub type TTJulianCenturiesSinceJ2000 = f64;

pub type UT1DaysSinceJ2000 = f64;

#[cfg(test)]
mod epoch_tests {
    use float_eq::assert_float_eq;

    use crate::time::dates::Calendar::Gregorian;
    use crate::time::dates::{Date, Time};
    use crate::time::epochs::{Epoch, TimeScale};

    use super::tdb_julian_centuries_since_j2000;

    /// A somewhat arbitrary tolerance for floating point comparisons.
    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_tdb_julian_centuries_since_j2000_tt() {
        let jd0 = Epoch::jd0(TimeScale::TT);
        assert_float_eq!(
            -67.11964407939767,
            tdb_julian_centuries_since_j2000(jd0),
            rel <= TOLERANCE
        );

        let j2000 = Epoch::j2000(TimeScale::TT);
        assert_float_eq!(
            0.0,
            tdb_julian_centuries_since_j2000(j2000),
            rel <= TOLERANCE
        );

        let j2100 = Epoch::from_date_and_time(
            TimeScale::TT,
            Date::new_unchecked(Gregorian, 2100, 1, 1),
            Time::new(12, 0, 0).expect("midday should be a valid time"),
        );
        assert_float_eq!(
            1.0,
            tdb_julian_centuries_since_j2000(j2100),
            rel <= TOLERANCE
        );
    }
}
