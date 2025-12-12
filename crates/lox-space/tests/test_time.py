# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox
import pytest


def test_time(provider):
    tai_exp = lox.Time("TAI", 2000, 1, 1)
    tai_act = lox.Time.from_iso("2000-01-01T00:00:00.000 TAI")
    assert tai_exp == tai_act
    tai_act = tai_exp.to_scale("TAI")
    assert tai_exp == tai_act
    tai_act = tai_exp.to_scale("TCB").to_scale("TAI")
    assert tai_exp.isclose(tai_act)
    tai_act = tai_exp.to_scale("TCG").to_scale("TAI")
    assert tai_exp.isclose(tai_act)
    tai_act = tai_exp.to_scale("TDB").to_scale("TAI")
    assert tai_exp.isclose(tai_act)
    tai_act = tai_exp.to_scale("TT").to_scale("TAI")
    assert tai_exp.isclose(tai_act)
    tai_act = tai_exp.to_scale("UT1", provider).to_scale("TAI", provider)
    assert tai_exp.isclose(tai_act)
    tai1 = lox.Time("TAI", 2000, 1, 1, 0, 0, 0.5)
    assert tai1 > tai_exp
    assert tai1 >= tai_exp
    assert tai_exp < tai1
    assert tai_exp <= tai1
    assert tai_exp != tai1
    dt = lox.TimeDelta(0.5)
    assert (tai_exp + dt).isclose(tai1)
    assert (tai1 - dt).isclose(tai_exp)
    assert float(tai1 - tai_exp) == pytest.approx(float(dt))


def test_utc(provider):
    utc_exp = lox.UTC(2000, 1, 1)
    utc_act = lox.UTC.from_iso("2000-01-01T00:00:00.000")
    assert utc_exp == utc_act
    utc_act = lox.UTC.from_iso("2000-01-01T00:00:00.000Z")
    assert utc_exp == utc_act
    utc_act = lox.UTC.from_iso("2000-01-01T00:00:00.000 UTC")
    assert utc_exp == utc_act
    utc_act = utc_exp.to_scale("TAI").to_utc()
    assert utc_exp == utc_act
    utc_act = utc_exp.to_scale("TCB").to_utc()
    assert utc_exp.isclose(utc_act)
    utc_act = utc_exp.to_scale("TCG").to_utc()
    assert utc_exp.isclose(utc_act)
    utc_act = utc_exp.to_scale("TDB").to_utc()
    assert utc_exp.isclose(utc_act)
    utc_act = utc_exp.to_scale("TT").to_utc()
    assert utc_exp == utc_act
    utc_act = utc_exp.to_scale("UT1", provider).to_utc(provider)
    assert utc_exp.isclose(utc_act)


def test_time_delta():
    delta = lox.TimeDelta(1.5)
    assert str(delta) == "1.5 seconds"
    assert repr(delta) == "TimeDelta(1.5)"
    assert delta.seconds() == 1
    assert delta.subsecond() == 0.5
    assert str(delta + delta) == "3 seconds"
    assert str(delta - delta) == "0 seconds"
    assert str(-delta) == "-1.5 seconds"
    with pytest.raises(lox.NonFiniteTimeDeltaError):
        lox.TimeDelta(float("nan")).seconds()


def test_time_delta_constructors():
    td = lox.TimeDelta.from_seconds(123)
    assert td.to_decimal_seconds() == 123.0
    td = lox.TimeDelta.from_minutes(2.0)
    assert td.to_decimal_seconds() == 120.0
    td = lox.TimeDelta.from_hours(2.0)
    assert td.to_decimal_seconds() == 7200.0
    td = lox.TimeDelta.from_days(2.0)
    assert td.to_decimal_seconds() == 172800.0
    td = lox.TimeDelta.from_julian_years(2.0)
    assert td.to_decimal_seconds() == 63115200.0
    td = lox.TimeDelta.from_julian_centuries(2.0)
    assert td.to_decimal_seconds() == 6311520000.0


def test_time_repr():
    time = lox.Time("TAI", 2000, 1, 1, 0, 0, 12.123456789123)
    assert repr(time) == 'Time("TAI", 2000, 1, 1, 0, 0, 12.123456789123)'
    assert str(time) == "2000-01-01T00:00:12.123 TAI"


def test_time_accessors():
    time = lox.Time("TAI", 2000, 1, 1, 0, 0, 12.123456789123)
    assert time.scale().abbreviation() == "TAI"
    assert time.year() == 2000
    assert time.month() == 1
    assert time.day() == 1
    assert time.hour() == 0
    assert time.minute() == 0
    assert time.second() == 12
    assert time.millisecond() == 123
    assert time.microsecond() == 456
    assert time.nanosecond() == 789
    assert time.picosecond() == 123
    assert time.femtosecond() == 0
    assert time.decimal_seconds() == pytest.approx(12.123456789123, rel=1e-15)


def test_time_invalid_date():
    with pytest.raises(ValueError, match="invalid date"):
        lox.Time("TAI", 2000, 13, 1)


def test_time_invalid_hour():
    with pytest.raises(ValueError, match="hour must be in the range"):
        lox.Time("TAI", 2000, 12, 1, 24, 0, 0.0)


def test_time_sub_different_scales():
    t1 = lox.Time("TAI", 2000, 1, 1, 0, 0, 1.0)
    t0 = lox.Time("TT", 2000, 1, 1, 0, 0, 1.0)
    with pytest.raises(ValueError, match="cannot subtract.*different time scales"):
        t1 - t0


def test_time_isclose_different_scales():
    t0 = lox.Time("TAI", 2000, 1, 1)
    t1 = lox.Time("TT", 2000, 1, 1)
    with pytest.raises(ValueError, match="cannot compare.*different time scales"):
        t0.isclose(t1)


def test_time_from_iso_invalid():
    with pytest.raises(ValueError, match="invalid ISO"):
        lox.Time.from_iso("2000-01-01X00:00:00 TAI")


def test_time_from_iso_invalid_scale():
    with pytest.raises(ValueError, match="invalid ISO"):
        lox.Time.from_iso("2000-01-01T00:00:00 UTC")


def test_time_from_iso_invalid_scale_arg():
    with pytest.raises(ValueError, match="unknown time scale: UTC"):
        lox.Time.from_iso("2000-01-01T00:00:00 TAI", scale="UTC")


def test_time_julian_date():
    time = lox.Time.from_julian_date("TAI", 0.0, "j2000")
    assert time.julian_date("j2000", "seconds") == 0.0
    assert time.julian_date("j2000", "days") == 0.0
    assert time.julian_date("j2000", "centuries") == 0.0
    assert time.julian_date("jd", "days") == 2451545.0
    assert time.julian_date("mjd", "days") == 51544.5
    assert time.julian_date("j1950", "days") == 18262.5

    time = lox.Time.from_julian_date("TAI", 0.0, "j1950")
    assert time.julian_date("j1950", "days") == 0.0

    time = lox.Time.from_julian_date("TAI", 0.0, "mjd")
    assert time.julian_date("mjd", "days") == 0.0

    time = lox.Time.from_julian_date("TAI", 0.0, "jd")
    assert time.julian_date("jd", "days") == 0.0


def test_time_invalid_epoch():
    time = lox.Time("TAI", 2000, 1, 1)
    with pytest.raises(ValueError, match="unknown epoch: unknown"):
        time.julian_date("unknown", "days")


def test_time_invalid_unit():
    time = lox.Time("TAI", 2000, 1, 1)
    with pytest.raises(ValueError, match="unknown unit: unknown"):
        time.julian_date("jd", "unknown")


def test_time_from_two_part_julian_date():
    expected = lox.Time("TAI", 2024, 7, 11, 8, 2, 14.0)
    jd1, jd2 = expected.two_part_julian_date()
    actual = lox.Time.from_two_part_julian_date("TAI", jd1, jd2)
    assert expected.isclose(actual)


def test_time_from_day_of_year():
    expected = lox.Time("TAI", 2024, 12, 31)
    actual = lox.Time.from_day_of_year("TAI", 2024, 366)
    assert actual == expected


def test_utc_accessors():
    utc = lox.UTC(2000, 1, 1, 12, 13, 14.123456789123)
    assert utc.year() == 2000
    assert utc.month() == 1
    assert utc.day() == 1
    assert utc.hour() == 12
    assert utc.minute() == 13
    assert utc.second() == 14
    assert utc.millisecond() == 123
    assert utc.microsecond() == 456
    assert utc.nanosecond() == 789
    assert utc.picosecond() == 123
    assert utc.decimal_seconds() == 14.123456789123
    assert str(utc) == "2000-01-01T12:13:14.123 UTC"
    assert repr(utc) == "UTC(2000, 1, 1, 12, 13, 14.123456789123)"


def test_utc_invalid_date():
    with pytest.raises(ValueError, match="invalid date"):
        lox.UTC(2000, 0, 1)


def test_utc_from_iso_invalid():
    with pytest.raises(ValueError, match="invalid ISO"):
        lox.UTC.from_iso("2000-01-01X00:00:00 UTC")


def test_eop_provider_invalid_path():
    with pytest.raises(lox.EopParserError):
        lox.EOPProvider("invalid_path")


def test_eop_provider_extrapolated(provider):
    tai = lox.Time("TAI", 2100, 1, 1)
    with pytest.raises(lox.EopProviderError, match="extrapolated"):
        tai.to_scale("UT1", provider)
