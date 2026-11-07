# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import math
import pickle

import numpy as np
import pytest

import lox_space as lox

QUANTITIES = [
    (lox.Angle, lox.AngleUnit, "rad"),
    (lox.AngularRate, lox.AngularRateUnit, "rad/s"),
    (lox.Area, lox.AreaUnit, "m²"),
    (lox.AreaToMass, lox.AreaToMassUnit, "m²/kg"),
    (lox.DataRate, lox.DataRateUnit, "bit/s"),
    (lox.DataVolume, lox.DataVolumeUnit, "bit"),
    (lox.Decibel, lox.DecibelUnit, "dB"),
    (lox.DecibelWatts, lox.DecibelWattsUnit, "dBW"),
    (lox.Distance, lox.DistanceUnit, "m"),
    (lox.Frequency, lox.FrequencyUnit, "Hz"),
    (lox.GravitationalParameter, lox.GravitationalParameterUnit, "m³/s²"),
    (lox.Mass, lox.MassUnit, "kg"),
    (lox.Power, lox.PowerUnit, "W"),
    (lox.Pressure, lox.PressureUnit, "Pa"),
    (lox.SpecificAngularMomentum, lox.SpecificAngularMomentumUnit, "m²/s"),
    (lox.SpecificEnergy, lox.SpecificEnergyUnit, "m²/s²"),
    (lox.Temperature, lox.TemperatureUnit, "K"),
    (lox.Velocity, lox.VelocityUnit, "m/s"),
]


@pytest.mark.parametrize(("quantity", "unit", "symbol"), QUANTITIES)
def test_si_unit_round_trip(quantity, unit, symbol):
    si = unit(symbol)
    q = 2.5 * si
    assert isinstance(q, quantity)
    assert q.to(si) == 2.5
    assert str(q) == f"2.5 {symbol}"
    assert eval(repr(q), {quantity.__name__: quantity}) == q
    assert pickle.loads(pickle.dumps(q)) == q
    assert pickle.loads(pickle.dumps(si)) == si


@pytest.mark.parametrize(("quantity", "unit", "symbol"), QUANTITIES)
def test_no_implicit_float_conversion(quantity, unit, symbol):
    q = 1.0 * unit(symbol)
    with pytest.raises(TypeError):
        float(q)
    with pytest.raises(TypeError):
        int(q)
    with pytest.raises(TypeError):
        complex(q)
    with pytest.raises(TypeError):
        quantity(1.0)


def test_units_are_not_quantities():
    assert isinstance(lox.km, lox.DistanceUnit)
    assert lox.km.symbol == "km"
    assert str(lox.km) == "km"
    assert repr(lox.km) == 'DistanceUnit("km")'
    assert lox.DistanceUnit("km") == lox.km
    assert hash(lox.DistanceUnit("km")) == hash(lox.km)
    with pytest.raises(TypeError):
        lox.km + 1 * lox.km
    with pytest.raises(TypeError):
        lox.km * lox.km
    with pytest.raises(ValueError, match="unknown unit"):
        lox.DistanceUnit("parsec")


def test_unit_aliases():
    assert lox.AngleUnit("uas") == lox.uas
    assert lox.AngleUnit("°") == lox.deg
    assert lox.DistanceUnit("AU") == lox.au


def test_angle():
    a = 180 * lox.deg
    assert str(a) == "3.141592653589793 rad"
    assert repr(a) == "Angle.radians(3.141592653589793)"
    assert a.to(lox.rad) == math.pi
    assert a.to_degrees() == pytest.approx(180.0)
    assert lox.Angle.degrees(180) == a
    assert (1 * lox.arcsec).to(lox.mas) == pytest.approx(1000.0)


def test_distance():
    d = 1024 * lox.km
    assert str(d) == "1024000 m"
    assert repr(d) == "Distance.meters(1024000.0)"
    assert d.to(lox.km) == 1024.0
    assert d.to_meters() == 1024000.0
    assert lox.Distance.kilometers(1024) == d
    assert (1 * lox.au).to_astronomical_units() == pytest.approx(1.0)


def test_repr_integer_value():
    assert repr(1 * lox.km) == "Distance.meters(1000.0)"


def test_frequency():
    f = 2.4 * lox.GHz
    assert f.to(lox.MHz) == pytest.approx(2400.0)
    assert f.to_gigahertz() == pytest.approx(2.4)
    assert (2000000 * lox.kHz).to_hertz() == 2e9


def test_velocity():
    v = 7.8 * lox.km_per_s
    assert v.to(lox.m_per_s) == pytest.approx(7800.0)
    assert v.to_kilometers_per_second() == pytest.approx(7.8)
    assert (1 * lox.c).to(lox.m_per_s) == 299792458.0
    assert (1 * lox.au_per_d).to(lox.au_per_d) == pytest.approx(1.0)
    assert (
        lox.Velocity.fraction_of_speed_of_light(1).to_meters_per_second() == 299792458.0
    )


def test_formatting():
    d = 1500 * lox.m
    assert f"{d}" == "1500 m"
    assert f"{d:.1f}" == "1500.0 m"
    assert f"{d:.2e}" == "1.50e+03 m"
    assert d.display_in(lox.km) == "1.5 km"
    assert d.display_in(lox.km, ".3f") == "1.500 km"


def test_arithmetic():
    d = 100 * lox.km + 500 * lox.m
    assert d.to(lox.m) == 100500.0
    assert (d - 500 * lox.m).to(lox.km) == 100.0
    assert (-d).to(lox.m) == -100500.0
    assert abs(-d) == d
    assert +d == d
    assert (2 * d).to(lox.m) == 201000.0
    assert (d * 2).to(lox.m) == 201000.0
    assert (d / 2).to(lox.m) == 50250.0
    assert (2 * lox.km) / (500 * lox.m) == 4.0
    with pytest.raises(TypeError):
        1 * lox.km + 1 * lox.deg
    with pytest.raises(TypeError):
        1 * lox.km + 1.0


def test_comparisons():
    assert 1 * lox.km > 500 * lox.m
    assert 1 * lox.km >= 1000 * lox.m
    assert 1 * lox.m < 1 * lox.km
    assert 1 * lox.m <= 1 * lox.m
    assert 1 * lox.km == 1000 * lox.m
    assert 1 * lox.km != 1 * lox.m
    assert (1 * lox.km == 1 * lox.deg) is False
    with pytest.raises(TypeError):
        _ = 1 * lox.km < 1 * lox.deg


def test_cross_dimension_operators():
    t = 10 * lox.seconds
    v = (70 * lox.km) / t
    assert isinstance(v, lox.Velocity)
    assert v.to(lox.km_per_s) == 7.0
    assert ((7 * lox.km_per_s) * t).to(lox.km) == 70.0
    assert (t * (7 * lox.km_per_s)).to(lox.km) == 70.0
    assert ((90 * lox.deg) / t).to(lox.deg_per_s) == pytest.approx(9.0)
    assert ((9 * lox.deg_per_s) * t).to(lox.deg) == pytest.approx(90.0)
    area = (2 * lox.m) * (3 * lox.m)
    assert isinstance(area, lox.Area)
    assert area.to(lox.m2) == 6.0
    assert (area / (3 * lox.kg)).to(lox.m2_per_kg) == 2.0
    assert ((8 * lox.Mbps) * t).to(lox.MB) == 10.0
    assert ((10 * lox.MB) / t).to(lox.Mbps) == 8.0


def test_decibels():
    p = 20 * lox.W
    level = p.to_decibel_watts()
    assert isinstance(level, lox.DecibelWatts)
    assert level.to(lox.dBW) == pytest.approx(13.0103, abs=1e-4)
    assert level.to_power().to_watts() == pytest.approx(20.0)
    eirp = 10 * lox.dBW + 30 * lox.dB
    assert eirp.to(lox.dBW) == 40.0
    assert (30 * lox.dB + 10 * lox.dBW).to(lox.dBW) == 40.0
    assert (eirp - 3 * lox.dB).to(lox.dBW) == 37.0
    margin = eirp - 10 * lox.dBW
    assert isinstance(margin, lox.Decibel)
    assert margin.to(lox.dB) == 30.0
    assert (0 * lox.dBm).to(lox.dBW) == -30.0
    assert lox.Decibel.from_linear(100).to_decibels() == pytest.approx(20.0)
    assert (3 * lox.dB).to_linear() == pytest.approx(1.9953, abs=1e-4)
    assert (2 * (3 * lox.dB)).to(lox.dB) == 6.0


def test_pressure():
    p = 1013.25 * lox.hPa
    assert p.to_hectopascals() == pytest.approx(1013.25)
    assert p.to(lox.Pa) == pytest.approx(101325.0)


def test_temperature():
    t = 290 * lox.K
    assert t.to_kelvin() == 290.0
    assert (t + 10 * lox.K).to(lox.K) == 300.0


def test_mass_and_data_volume():
    assert (1.5 * lox.kg).to_grams() == 1500.0
    assert lox.Mass.metric_tons(2).to(lox.kg) == 2000.0
    assert (1 * lox.KiB).to(lox.bit) == 8192.0
    assert (1 * lox.B).to_bits() == 8.0
    assert (1500 * lox.g).to(lox.kg) == 1.5
    assert (2 * lox.t).to(lox.kg) == 2000.0


def test_gravitational_parameter():
    gm = 398600.4418 * lox.km3_per_s2
    assert gm.to_m3_per_s2() == pytest.approx(3.986004418e14)
    assert gm.to(lox.km3_per_s2) == pytest.approx(398600.4418)
    assert lox.GravitationalParameter.km3_per_s2(398600.4418) == gm
    assert str(lox.GravitationalParameter.m3_per_s2(2.0)) == "2 m³/s²"
    with pytest.raises(TypeError):
        gm + gm


def test_specific_quantities():
    e = -29.6 * lox.km2_per_s2
    assert e.to_square_meters_per_second_squared() == pytest.approx(-29.6e6)
    h = 52000 * lox.km2_per_s
    assert h.to(lox.m2_per_s) == pytest.approx(5.2e10)


def test_time_delta_units():
    td = 90 * lox.minutes
    assert isinstance(td, lox.TimeDelta)
    assert td.to(lox.hours) == 1.5
    assert td.to(lox.seconds) == 5400.0
    assert (1.5 * lox.milliseconds).to(lox.microseconds) == 1500.0
    assert (1 * lox.julian_years).to(lox.days) == 365.25
    assert td.display_in(lox.hours) == "1.5 h"
    assert f"{td:.1f}" == "5400.0 s"
    assert lox.TimeDeltaUnit("min") == lox.minutes
    assert repr(lox.minutes) == 'TimeDeltaUnit("min")'
    assert pickle.loads(pickle.dumps(td)) == td
    assert eval(repr(td), {"TimeDelta": lox.TimeDelta}) == td
    with pytest.raises(TypeError):
        float(td)
    with pytest.raises(TypeError):
        lox.TimeDelta(1.0)
    with pytest.raises(ValueError):
        math.inf * lox.seconds


def test_time_delta_repr_is_exact():
    td = lox.TimeDelta.from_seconds(2**60) + lox.TimeDelta.from_attoseconds(1)
    assert eval(repr(td), {"TimeDelta": lox.TimeDelta}) == td


def test_time_delta_range():
    deltas = lox.TimeDelta.range(0 * lox.seconds, 1 * lox.minutes, 20 * lox.seconds)
    assert [d.to(lox.seconds) for d in deltas] == [0.0, 20.0, 40.0, 60.0]
    assert len(lox.TimeDelta.range(0 * lox.seconds, 2 * lox.seconds)) == 3
    with pytest.raises(ValueError, match="non-zero"):
        lox.TimeDelta.range(0 * lox.seconds, 2 * lox.seconds, 0 * lox.seconds)
    with pytest.raises(ValueError, match="more than"):
        lox.TimeDelta.range(0 * lox.seconds, 1 * lox.days, 1 * lox.nanoseconds)
    huge = lox.TimeDelta.from_seconds(2**62)
    with pytest.raises(ValueError, match="more than"):
        lox.TimeDelta.range(-huge, huge)


def test_time_delta_units_are_exact_for_whole_values():
    ns = 123456789012
    assert ns * lox.nanoseconds == lox.TimeDelta.from_nanoseconds(ns)
    unix_ns = 1_700_000_000_123_456_789
    assert unix_ns * lox.nanoseconds == lox.TimeDelta.from_nanoseconds(unix_ns)
    assert lox.nanoseconds * np.int64(unix_ns) == lox.TimeDelta.from_nanoseconds(
        unix_ns
    )
    assert (10**19 + 1) * lox.attoseconds == lox.TimeDelta.from_seconds(
        10
    ) + lox.TimeDelta.from_attoseconds(1)
    with pytest.raises(ValueError, match="out-of-range"):
        2**63 * lox.julian_centuries
    with pytest.raises(ValueError, match="out-of-range"):
        2**200 * lox.seconds


def test_time_delta_units_with_numpy():
    assert lox.seconds * np.array(1.5) == lox.TimeDelta.from_seconds(1.5)
    assert lox.seconds * np.float64(1.5) == lox.TimeDelta.from_seconds(1.5)
    deltas = lox.seconds * np.arange(3)
    assert list(deltas) == [lox.TimeDelta.from_seconds(n) for n in range(3)]
    single = lox.seconds * np.array([5])
    assert isinstance(single, np.ndarray)
    assert list(single) == [5 * lox.seconds]
    assert lox.seconds * np.int64(2**60) == lox.TimeDelta.from_seconds(2**60)


def test_time_delta_unit_mul_unsupported():
    with pytest.raises(TypeError, match="unsupported operand"):
        lox.seconds * object()
    with pytest.raises(TypeError, match="unsupported operand"):
        (1 * lox.Mbps) * lox.seconds


def test_time_delta_constructors_are_exact_for_integers():
    big = 2**60 + 7
    assert lox.TimeDelta.from_days(big // 86400) == (big // 86400) * lox.days
    assert lox.TimeDelta.from_days(10**14).seconds() == 86400 * 10**14
    assert lox.TimeDelta.from_minutes(1.5) == 90 * lox.seconds
    with pytest.raises(TypeError, match="expected a number"):
        lox.TimeDelta.from_days("1")


def test_time_delta_repr_round_trips_floats():
    for value in [0.7, 1e-18, 123456.789, -0.1]:
        td = lox.TimeDelta.from_seconds(value)
        assert eval(repr(td), {"TimeDelta": lox.TimeDelta}) == td
    td = lox.TimeDelta.from_attoseconds(700_000_000_000_000_001)
    assert eval(repr(td), {"TimeDelta": lox.TimeDelta}) == td


def test_time_delta_unpickle_validates_parts():
    with pytest.raises(ValueError, match="attoseconds"):
        lox.TimeDelta._from_parts(0, 10**18)
    with pytest.raises(ValueError, match="attoseconds"):
        lox.TimeDelta._from_parts(0, -1)


def test_time_delta_from_seconds_is_exact_for_integers():
    big = 2**60 + 1
    assert lox.TimeDelta.from_seconds(big).seconds() == big
    assert lox.TimeDelta.from_seconds(np.int64(big)).seconds() == big
    with pytest.raises(ValueError, match="out-of-range"):
        lox.TimeDelta.from_seconds(2**70)
    with pytest.raises(ValueError, match="out-of-range"):
        lox.TimeDelta.from_seconds(2.0**70)


NON_LINEAR = (lox.DecibelWatts, lox.GravitationalParameter)
LINEAR_QUANTITIES = [q for q in QUANTITIES if q[0] not in NON_LINEAR]
UNITS = {quantity: unit for quantity, unit, _ in QUANTITIES}


def named_units(quantity):
    """Returns the names `x` of the quantity's `x`/`to_x` constructor-accessor pairs."""
    return [
        name
        for name in dir(quantity)
        if not name.startswith("_") and hasattr(quantity, f"to_{name}")
    ]


@pytest.mark.parametrize(("quantity", "unit", "symbol"), QUANTITIES)
def test_named_units_round_trip(quantity, unit, symbol):
    names = named_units(quantity)
    assert names
    for name in names:
        q = getattr(quantity, name)(2.5)
        assert isinstance(q, quantity)
        assert getattr(q, f"to_{name}")() == pytest.approx(2.5, rel=1e-12)


@pytest.mark.parametrize(("quantity", "unit", "symbol"), QUANTITIES)
def test_comparisons_all_quantities(quantity, unit, symbol):
    si = unit(symbol)
    a, b = 1 * si, 2 * si
    assert a < b and b > a
    assert a <= a and a >= a
    assert a == 1 * si
    assert a != b
    assert (a == 1.0) is False
    assert a.__eq__(1.0) is NotImplemented
    with pytest.raises(TypeError):
        _ = a < 1.0
    with pytest.raises(TypeError):
        hash(a)


@pytest.mark.parametrize(("quantity", "unit", "symbol"), LINEAR_QUANTITIES)
def test_linear_arithmetic(quantity, unit, symbol):
    si = unit(symbol)
    q = 3 * si
    assert -q == -3 * si
    assert abs(-q) == q
    assert +q == q
    assert q + q == 6 * si
    assert q - q == 0 * si
    assert 2 * q == 6 * si
    assert q * 2 == 6 * si
    assert q / 2 == 1.5 * si
    assert q / q == 1.0
    with pytest.raises(TypeError):
        q + 1.0
    with pytest.raises(TypeError):
        q * "2"


@pytest.mark.parametrize("quantity", NON_LINEAR)
def test_non_linear_arithmetic_is_unsupported(quantity):
    q = getattr(quantity, named_units(quantity)[0])(3.0)
    for op in (
        lambda: q + q,
        lambda: 2 * q,
        lambda: q * 2,
        lambda: q / 2,
        lambda: -q,
    ):
        with pytest.raises(TypeError):
            op()


@pytest.mark.parametrize(("quantity", "unit", "symbol"), QUANTITIES)
def test_formatting_all_quantities(quantity, unit, symbol):
    si = unit(symbol)
    q = 2.5 * si
    assert f"{q:.1f}" == f"2.5 {symbol}"
    assert q.display_in(si, ".1f") == f"2.5 {symbol}"


@pytest.mark.parametrize(("quantity", "unit", "symbol"), QUANTITIES)
def test_unit_classes(quantity, unit, symbol):
    u = unit(symbol)
    assert u.symbol == symbol
    assert str(u) == symbol
    assert repr(u) == f'{unit.__name__}("{symbol}")'
    assert eval(repr(u), {unit.__name__: unit}) == u
    assert hash(unit(symbol)) == hash(u)
    with pytest.raises(ValueError, match="unknown unit"):
        unit("not-a-unit")


SECONDS = 3 * lox.seconds


@pytest.mark.parametrize(
    ("product", "result", "symbol", "expected"),
    [
        (lambda: (6 * lox.AngularRateUnit("rad/s")) * SECONDS, lox.Angle, "rad", 18.0),
        (lambda: SECONDS * (6 * lox.AngularRateUnit("rad/s")), lox.Angle, "rad", 18.0),
        (lambda: (6 * lox.rad) / SECONDS, lox.AngularRate, "rad/s", 2.0),
        (lambda: (6 * lox.m2) / (3 * lox.kg), lox.AreaToMass, "m²/kg", 2.0),
        (lambda: (6 * lox.bps) * SECONDS, lox.DataVolume, "bit", 18.0),
        (lambda: SECONDS * (6 * lox.bps), lox.DataVolume, "bit", 18.0),
        (lambda: (6 * lox.bit) / SECONDS, lox.DataRate, "bit/s", 2.0),
        (lambda: (6 * lox.m) * (3 * lox.m), lox.Area, "m²", 18.0),
        (lambda: (6 * lox.m) / SECONDS, lox.Velocity, "m/s", 2.0),
        (lambda: (6 * lox.m_per_s) * SECONDS, lox.Distance, "m", 18.0),
        (lambda: SECONDS * (6 * lox.m_per_s), lox.Distance, "m", 18.0),
    ],
)
def test_cross_dimension_operators_all(product, result, symbol, expected):
    q = product()
    assert isinstance(q, result)
    assert q == expected * UNITS[result](symbol)
