#[derive(Debug, Copy, Clone)]
pub enum Calendar {
    ProlepticJulian,
    Julian,
    Gregorian,
}

#[derive(Debug, Copy, Clone)]
pub struct Date {
    calendar: Calendar,
    year: i64,
    month: i64,
    day: i64,
}

impl Date {
    pub fn calendar(&self) -> Calendar {
        self.calendar
    }

    pub fn year(&self) -> i64 {
        self.year
    }

    pub fn month(&self) -> i64 {
        self.month
    }

    pub fn day(&self) -> i64 {
        self.day
    }

    pub fn new(year: i64, month: i64, day: i64) -> Result<Self, &'static str> {
        if !(1..=12).contains(&month) {
            Err("Invalid month")
        } else {
            let calendar = get_calendar(year, month, day);
            let check = match Date::from_days(j2000(calendar, year, month, day)) {
                Ok(check) => check,
                Err(msg) => return Err(msg),
            };

            if check.year() != year || check.month() != month || check.day() != day {
                Err("Invalid date")
            } else {
                Ok(Date {
                    calendar,
                    year,
                    month,
                    day,
                })
            }
        }
    }

    pub fn from_days(offset: i64) -> Result<Self, &'static str> {
        let calendar = if offset < -152384 {
            if offset > -730122 {
                Calendar::Julian
            } else {
                Calendar::ProlepticJulian
            }
        } else {
            Calendar::Gregorian
        };

        let year = find_year(calendar, offset);
        let leap = is_leap(calendar, year);
        let day_in_year = offset - last_j2000_dayofyear(calendar, year - 1);
        let month = find_month(day_in_year, leap);
        let day = find_day(day_in_year, month, leap);

        match day {
            Ok(day) => Ok(Date {
                calendar,
                year,
                month,
                day,
            }),
            Err(msg) => Err(msg),
        }
    }

    pub fn j2000(&self) -> i64 {
        j2000(self.calendar, self.year, self.month, self.day)
    }
}

// #[derive(Debug, Copy, Clone)]
// pub struct Time {
//     hour: i64,
//     minute: i64,
//     second: i64,
//     attosecond: i64,
// }
//
// #[derive(Debug, Copy, Clone)]
// pub struct DateTime {
//     date: Date,
//     time: Time,
// }

fn find_year(calendar: Calendar, j2000day: i64) -> i64 {
    match calendar {
        Calendar::ProlepticJulian => -((-4 * j2000day - 2920488) / 1461),
        Calendar::Julian => -((-4 * j2000day - 2921948) / 1461),
        Calendar::Gregorian => {
            let mut year = (400 * j2000day + 292194288) / 146097;
            if j2000day <= last_j2000_dayofyear(Calendar::Gregorian, year - 1) {
                year -= 1
            }
            year
        }
    }
}

fn last_j2000_dayofyear(calendar: Calendar, year: i64) -> i64 {
    match calendar {
        Calendar::ProlepticJulian => 365 * year + (year + 1) / 4 - 730123,
        Calendar::Julian => 365 * year + year / 4 - 730122,
        Calendar::Gregorian => 365 * year + year / 4 - year / 100 + year / 400 - 730120,
    }
}

fn is_leap(calendar: Calendar, year: i64) -> bool {
    match calendar {
        Calendar::ProlepticJulian | Calendar::Julian => year % 4 == 0,
        Calendar::Gregorian => year % 4 == 0 && (year % 400 == 0 || year % 100 != 0),
    }
}

const PREVIOUS_MONTH_END_DAY_LEAP: [i64; 12] =
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];

const PREVIOUS_MONTH_END_DAY: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

fn find_month(day_in_year: i64, isleap: bool) -> i64 {
    let offset = if isleap { 313 } else { 323 };
    if day_in_year < 32 {
        1
    } else {
        (10 * day_in_year + offset) / 306
    }
}

fn find_day(dayinyear: i64, month: i64, isleap: bool) -> Result<i64, &'static str> {
    if !isleap && dayinyear > 365 {
        Err("Day of year cannot be 366 for a non-leap year.")
    } else {
        let previous_days = if isleap {
            PREVIOUS_MONTH_END_DAY_LEAP
        } else {
            PREVIOUS_MONTH_END_DAY
        };
        Ok(dayinyear - previous_days[(month - 1) as usize])
    }
}

fn find_day_in_year(month: i64, day: i64, isleap: bool) -> i64 {
    let previous_days = if isleap {
        PREVIOUS_MONTH_END_DAY_LEAP
    } else {
        PREVIOUS_MONTH_END_DAY
    };
    day + previous_days[(month - 1) as usize]
}

fn get_calendar(year: i64, month: i64, day: i64) -> Calendar {
    if year < 1583 {
        if year < 1 {
            Calendar::ProlepticJulian
        } else if year < 1582 || month < 10 || (month < 11 && day < 5) {
            Calendar::Julian
        } else {
            Calendar::Gregorian
        }
    } else {
        Calendar::Gregorian
    }
}

fn j2000(calendar: Calendar, year: i64, month: i64, day: i64) -> i64 {
    let d1 = last_j2000_dayofyear(calendar, year - 1);
    let d2 = find_day_in_year(month, day, is_leap(calendar, year));
    d1 + d2
}
