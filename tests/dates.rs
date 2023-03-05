use lox::astrotime::dates::Date;
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
    let date = Date::new(year, month, day).expect("Invalid date");
    assert_eq!(exp, date.j2000())
}

#[test]
fn test_illegal_dates() {
    assert!(Date::new(2018, 2, 29).is_err());
    assert!(Date::new(2018, 0, 1).is_err());
    assert!(Date::new(2018, 13, 1).is_err());
}
