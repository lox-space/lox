use lox::astrotime::dates::{Date, SubSecond, Time};
use rstest::rstest;

#[rstest]
#[case(-4713, 12, 31, -2451546)]
#[case(-4712, 1, 1, -2451545)]
#[case(0, 12, 31, -730122)]
#[case(1, 1, 1, -730121)]
#[case(1500, 2, 28, -182554)]
#[case(1500, 2, 29, -182553)]
#[case(1500, 3, 1, -182552)]
#[case(1582, 10, 4, -152385)]
#[case(1582, 10, 15, -152384)]
#[case(1600, 2, 28, -146039)]
#[case(1600, 2, 29, -146038)]
#[case(1600, 3, 1, -146037)]
#[case(1700, 2, 28, -109514)]
#[case(1700, 3, 1, -109513)]
#[case(1800, 2, 28, -72990)]
#[case(1800, 3, 1, -72989)]
#[case(1858, 11, 15, -51546)]
#[case(1858, 11, 16, -51545)]
#[case(1999, 12, 31, -1)]
#[case(2000, 1, 1, 0)]
#[case(2000, 2, 28, 58)]
#[case(2000, 2, 29, 59)]
#[case(2000, 3, 1, 60)]
fn test_dates(#[case] year: i64, #[case] month: i64, #[case] day: i64, #[case] exp: i64) {
    let date = Date::new(year, month, day).expect("date should be valid");
    assert_eq!(exp, date.j2000())
}

#[test]
fn test_illegal_dates() {
    assert!(Date::new(2018, 2, 29).is_err());
    assert!(Date::new(2018, 0, 1).is_err());
    assert!(Date::new(2018, 13, 1).is_err());
}

#[test]
fn test_sub_second() {
    let s1 = SubSecond::from_seconds(0.123).expect("seconds should be valid");
    assert_eq!(123, s1.milli());
    assert_eq!(0, s1.micro());
    assert_eq!(0, s1.nano());
    assert_eq!(0, s1.pico());
    assert_eq!(0, s1.femto());
    assert_eq!(0, s1.atto());
    let s2 = SubSecond::from_seconds(0.123_456).expect("seconds should be valid");
    assert_eq!(123, s2.milli());
    assert_eq!(456, s2.micro());
    assert_eq!(0, s2.nano());
    assert_eq!(0, s2.pico());
    assert_eq!(0, s2.femto());
    assert_eq!(0, s2.atto());
    let s3 = SubSecond::from_seconds(0.123_456_789).expect("seconds should be valid");
    assert_eq!(123, s3.milli());
    assert_eq!(456, s3.micro());
    assert_eq!(789, s3.nano());
    assert_eq!(0, s3.pico());
    assert_eq!(0, s3.femto());
    assert_eq!(0, s3.atto());
    let s4 = SubSecond::from_seconds(0.123_456_789_123).expect("seconds should be valid");
    assert_eq!(123, s4.milli());
    assert_eq!(456, s4.micro());
    assert_eq!(789, s4.nano());
    assert_eq!(123, s4.pico());
    assert_eq!(0, s4.femto());
    assert_eq!(0, s4.atto());
    let s5 = SubSecond::from_seconds(0.123_456_789_123_456).expect("seconds should be valid");
    assert_eq!(123, s5.milli());
    assert_eq!(456, s5.micro());
    assert_eq!(789, s5.nano());
    assert_eq!(123, s5.pico());
    assert_eq!(456, s5.femto());
    assert_eq!(0, s5.atto());
    let s6 = SubSecond::from_seconds(0.123_456_789_123_456_78).expect("seconds should be valid");
    assert_eq!(123, s6.milli());
    assert_eq!(456, s6.micro());
    assert_eq!(789, s6.nano());
    assert_eq!(123, s6.pico());
    assert_eq!(456, s6.femto());
    assert_eq!(780, s6.atto());
    let s7 = SubSecond::from_seconds(0.000_000_000_000_000_01).expect("seconds should be valid");
    assert_eq!(0, s7.milli());
    assert_eq!(0, s7.micro());
    assert_eq!(0, s7.nano());
    assert_eq!(0, s7.pico());
    assert_eq!(0, s7.femto());
    assert_eq!(10, s7.atto());
}

#[test]
fn test_illegal_sub_second() {
    assert!(SubSecond::from_seconds(2.0).is_err());
    assert!(SubSecond::from_seconds(-0.2).is_err());
}

#[test]
fn test_illegal_times() {
    assert!(Time::from_seconds(24, 59, 59.0).is_err());
    assert!(Time::from_seconds(23, 60, 59.0).is_err());
    assert!(Time::from_seconds(23, 59, 61.0).is_err());
}
