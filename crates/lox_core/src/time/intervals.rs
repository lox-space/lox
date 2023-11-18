use crate::time::constants;
use crate::time::epochs::Epoch;

/// Strictly TDB, TT is sufficient for most applications.
pub type TDBJulianCenturiesSinceJ2000 = f64;

pub fn tdb_julian_centuries_since_j2000(epoch: Epoch) -> TDBJulianCenturiesSinceJ2000 {
    match epoch {
        Epoch::TT(_) | Epoch::TDB(_) => epoch.j2000() / constants::f64::DAYS_PER_JULIAN_CENTURY,
        _ => todo!("perform the simpler of the conversions to TT or TDB first"),
    }
}

#[cfg(test)]
mod epoch_tests {
    use super::tdb_julian_centuries_since_j2000;
    use crate::time::dates::Calendar::{Gregorian, ProlepticJulian};
    use crate::time::dates::{Date, DateTime, Time};
    use crate::time::epochs::{Epoch, RawEpoch, TimeScale};
    use float_eq::assert_float_eq;
    use lazy_static::lazy_static;

    const TOLERANCE: f64 = 1e-12;

    lazy_static! {
        static ref MIDDAY: Time = Time::new(12, 0, 0).expect("midday should be a valid time");
        static ref JD0: DateTime = DateTime::new(
            Date {
                calendar: ProlepticJulian,
                year: -4713,
                month: 1,
                day: 1,
            },
            *MIDDAY,
        );
        static ref J2100: DateTime = DateTime::new(
            Date {
                calendar: Gregorian,
                year: 2100,
                month: 1,
                day: 1,
            },
            *MIDDAY,
        );
    }

    #[test]
    fn test_tdb_julian_centuries_since_j2000_tt() {
        let jd0 = Epoch::from_datetime(TimeScale::TT, *JD0);
        let j2000 = Epoch::TT(RawEpoch::default());
        let j2100 = Epoch::from_datetime(TimeScale::TT, *J2100);
        assert_float_eq!(
            -67.1196440794,
            tdb_julian_centuries_since_j2000(jd0),
            rel <= TOLERANCE
        );
        assert_float_eq!(
            0.0,
            tdb_julian_centuries_since_j2000(j2000),
            rel <= TOLERANCE
        );
        assert_float_eq!(
            1.0,
            tdb_julian_centuries_since_j2000(j2100),
            rel <= TOLERANCE
        );
    }
}
