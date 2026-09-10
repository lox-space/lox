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
    with pytest.raises(ValueError):
        lox.TimeDelta(float("nan"))
    with pytest.raises(ValueError):
        delta * float("nan")
    with pytest.raises(ValueError):
        float("inf") * delta


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
    with pytest.raises(ValueError):
        lox.Time.from_two_part_julian_date("TAI", jd1, float("nan"))


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


def test_time_delta_units():
    assert (60 * lox.seconds).to_decimal_seconds() == 60.0
    assert (2 * lox.minutes).to_decimal_seconds() == 120.0
    assert (0.5 * lox.hours).to_decimal_seconds() == 1800.0
    assert (1 * lox.days).to_decimal_seconds() == 86400.0


def test_time_delta_subsecond_constructors():
    td = lox.TimeDelta.from_milliseconds(1500)
    assert td.seconds() == 1
    assert td.subsecond() == 0.5
    td = lox.TimeDelta.from_microseconds(1_000_000)
    assert td.to_decimal_seconds() == 1.0
    td = lox.TimeDelta.from_nanoseconds(500_000_000)
    assert td.to_decimal_seconds() == 0.5


def test_eop_provider_invalid_path():
    with pytest.raises(lox.EopParserError):
        lox.EOPProvider("invalid_path")


def test_eop_provider_extrapolated(provider):
    tai = lox.Time("TAI", 2100, 1, 1)
    with pytest.raises(lox.EopProviderError, match="extrapolated"):
        tai.to_scale("UT1", provider)


def test_time_comparison_same_scale():
    t1 = lox.Time("TAI", 2000, 1, 1)
    t2 = lox.Time("TAI", 2000, 1, 2)
    assert t1 < t2
    assert t2 > t1
    assert t1 == t1


def test_time_comparison_different_scale_raises():
    t_tai = lox.Time("TAI", 2000, 1, 1)
    t_tt = lox.Time("TT", 2000, 1, 1)
    with pytest.raises(ValueError, match="different time scales"):
        _ = t_tai < t_tt
    with pytest.raises(ValueError, match="different time scales"):
        _ = t_tt > t_tai


# --- TimeDelta: ordering, hashing and dimensional safety ---


def test_time_delta_ordering():
    short, long = 90 * lox.minutes, 2 * lox.hours
    assert short < long
    assert short <= long
    assert long > short
    assert long >= short
    assert short <= 90 * lox.minutes
    assert sorted([long, short]) == [short, long]
    assert min(long, short) == short


def test_time_delta_ordering_across_types_raises():
    with pytest.raises(TypeError):
        _ = (1 * lox.hours) < (1 * lox.km)


def test_time_delta_hashable():
    assert hash(1 * lox.hours) == hash(60 * lox.minutes)
    assert len({1 * lox.hours, 60 * lox.minutes, 2 * lox.hours}) == 2
    assert {1 * lox.days: "a day"}[86400 * lox.seconds] == "a day"


def test_time_delta_hash_keeps_attoseconds_distinct():
    # Hashing the decimal seconds would collide these.
    assert lox.TimeDelta.from_attoseconds(1) != lox.TimeDelta.from_attoseconds(2)
    assert len(
        {lox.TimeDelta.from_attoseconds(1), lox.TimeDelta.from_attoseconds(2)}
    ) == 2


def test_time_delta_times_time_delta_raises():
    with pytest.raises(TypeError):
        _ = (1 * lox.hours) * (1 * lox.hours)
    with pytest.raises(TypeError):
        _ = lox.seconds * lox.seconds
    with pytest.raises(TypeError):
        _ = lox.seconds + lox.seconds
    with pytest.raises(TypeError):
        float(lox.minutes)


def test_time_delta_division():
    two_hours = 2 * lox.hours
    assert two_hours / 2 == 1 * lox.hours
    assert two_hours / (30 * lox.minutes) == pytest.approx(4.0)
    assert two_hours / lox.minutes == pytest.approx(120.0)
    assert isinstance(two_hours / lox.minutes, float)


def test_time_delta_round_and_abs():
    assert round(lox.TimeDelta(1.567), 2) == lox.TimeDelta(1.57)
    assert abs(-(5 * lox.minutes)) == 5 * lox.minutes
    assert abs(5 * lox.minutes) == 5 * lox.minutes


def test_time_delta_no_int_conversion():
    with pytest.raises(TypeError):
        int(1 * lox.hours)


@pytest.mark.parametrize(
    "spec, expected",
    [
        ("", "7200 seconds"),
        (".1f", "7200.0 seconds"),
        (".2f minutes", "120.00 minutes"),
        (".3f days", "0.083 days"),
        (">16.1f", "  7200.0 seconds"),
    ],
)
def test_time_delta_format(spec, expected):
    assert format(2 * lox.hours, spec) == expected


def test_time_delta_format_unknown_unit_raises():
    with pytest.raises(ValueError, match="not a TimeDeltaUnit"):
        format(2 * lox.hours, ".1f fortnights")


def test_time_delta_numpy():
    np = pytest.importorskip("numpy")
    array = np.array([1 * lox.hours, 2 * lox.hours])
    assert array.dtype == np.float64
    assert array.tolist() == [3600.0, 7200.0]
    assert isinstance(np.float64(2) * (1 * lox.hours), lox.TimeDelta)


# --- TimeDeltaUnit ---


TIME_UNITS = [
    (lox.seconds, "seconds", 1.0),
    (lox.minutes, "minutes", 60.0),
    (lox.hours, "hours", 3600.0),
    (lox.days, "days", 86400.0),
]


@pytest.mark.parametrize("unit, symbol, scale", TIME_UNITS)
def test_time_delta_unit_constants(unit, symbol, scale):
    assert isinstance(unit, lox.TimeDeltaUnit)
    assert not isinstance(unit, lox.TimeDelta)
    assert unit.symbol == symbol
    assert unit.scale == scale
    assert str(unit) == symbol
    assert repr(unit) == f'TimeDeltaUnit("{symbol}")'
    assert float(2 * unit) == pytest.approx(2 * scale)
    assert (2 * unit) == (unit * 2)


def test_time_delta_unit_lookup_and_pickle():
    import pickle

    assert lox.TimeDeltaUnit("minutes") == lox.minutes
    assert lox.minutes != lox.hours
    assert hash(lox.minutes) == hash(lox.TimeDeltaUnit("minutes"))
    for unit, *_ in TIME_UNITS:
        assert pickle.loads(pickle.dumps(unit)) == unit
        assert eval(repr(unit), {"TimeDeltaUnit": lox.TimeDeltaUnit}) == unit


def test_time_delta_unit_unknown_name_raises():
    with pytest.raises(ValueError, match="not a TimeDeltaUnit"):
        lox.TimeDeltaUnit("fortnights")


def test_time_delta_constructors_reject_quantities():
    with pytest.raises(TypeError):
        lox.TimeDelta(lox.Distance(1.0))
    with pytest.raises(TypeError):
        lox.TimeDelta.from_minutes(lox.Angle(1.0))
    with pytest.raises(TypeError):
        lox.TimeDelta.from_days(1 * lox.hours)


def test_time_delta_constructors_accept_real_numbers():
    np = pytest.importorskip("numpy")
    for value in [90, 90.0, np.float64(90), np.float32(90), np.int64(90)]:
        assert lox.TimeDelta.from_minutes(value) == 90 * lox.minutes
