use crate::time::constants;
use crate::time::continuous::Time;

/// Although strictly TDB, TT is sufficient for most applications.
pub type TDBJulianCenturiesSinceJ2000 = f64;

pub fn tdb_julian_centuries_since_j2000(time: Time) -> TDBJulianCenturiesSinceJ2000 {
    match time {
        Time::TT(_) | Time::TDB(_) => {
            time.days_since_j2000() / constants::f64::DAYS_PER_JULIAN_CENTURY
        }
        _ => todo!("perform the simpler of the conversions to TT or TDB first"),
    }
}

pub type TTJulianCenturiesSinceJ2000 = f64;

pub type UT1DaysSinceJ2000 = f64;

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use crate::time::continuous::{Time, TimeScale};
    use crate::time::dates::Calendar::Gregorian;
    use crate::time::dates::Date;
    use crate::time::utc::UTC;

    use super::tdb_julian_centuries_since_j2000;

    /// A somewhat arbitrary tolerance for floating point comparisons.
    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_tdb_julian_centuries_since_j2000_tt() {
        let jd0 = Time::jd0(TimeScale::TT);
        assert_float_eq!(
            -67.11964407939767,
            tdb_julian_centuries_since_j2000(jd0),
            rel <= TOLERANCE
        );

        let j2000 = Time::j2000(TimeScale::TT);
        assert_float_eq!(
            0.0,
            tdb_julian_centuries_since_j2000(j2000),
            rel <= TOLERANCE
        );

        let j2100 = Time::from_date_and_utc_timestamp(
            TimeScale::TT,
            Date::new_unchecked(Gregorian, 2100, 1, 1),
            UTC::new(12, 0, 0).expect("midday should be a valid time"),
        );
        assert_float_eq!(
            1.0,
            tdb_julian_centuries_since_j2000(j2100),
            rel <= TOLERANCE
        );
    }
}
