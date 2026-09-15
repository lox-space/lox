# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import math
import pytest
import lox_space as lox


def test_angle_rad():
    a = math.pi * lox.rad
    assert str(a) == "180 deg"
    assert repr(a) == "Angle(3.141592653589793)"
    assert float(a) == math.pi
    assert complex(a) == complex(math.pi)


def test_angle_deg():
    a = 180 * lox.deg
    assert str(a) == "180 deg"
    assert repr(a) == "Angle(3.141592653589793)"
    assert float(a) == math.pi
    assert complex(a) == complex(math.pi)


def test_distance_km():
    d = 1024 * lox.km
    assert str(d) == "1024 km"
    assert repr(d) == "Distance(1024000.0)"
    assert float(d) == 1024000.0
    assert complex(d) == complex(1024000.0)


def test_distance_m():
    d = 2048 * lox.m
    assert str(d) == "2.048 km"
    assert repr(d) == "Distance(2048.0)"
    assert float(d) == 2048.0
    assert complex(d) == complex(2048.0)


def test_frequency_hz():
    f = 1073741824 * lox.Hz
    assert str(f) == "1.073741824 GHz"
    assert repr(f) == "Frequency(1073741824.0)"
    assert float(f) == 1073741824.0
    assert complex(f) == complex(1073741824.0)


def test_frequency_khz():
    f = 2000000 * lox.kHz
    assert str(f) == "2 GHz"
    assert repr(f) == "Frequency(2000000000.0)"
    assert float(f) == 2000000000.0
    assert complex(f) == complex(2000000000.0)


def test_velocity_ms():
    v = 262144 * lox.m_per_s
    assert str(v) == "262.144 km/s"
    assert repr(v) == "Velocity(262144.0)"
    assert float(v) == 262144.0
    assert complex(v) == complex(262144.0)


def test_velocity_kms():
    v = 16 * lox.km_per_s
    assert str(v) == "16 km/s"
    assert repr(v) == "Velocity(16000.0)"
    assert float(v) == 16000.0
    assert complex(v) == complex(16000.0)


# --- Arithmetic operations (covers __add__, __sub__, __neg__, __mul__, __eq__) ---


def test_distance_arithmetic():
    a = 100 * lox.m
    b = 200 * lox.m
    assert a + b == lox.Distance(300.0)
    assert b - a == lox.Distance(100.0)
    assert -a == lox.Distance(-100.0)
    assert a * 3 == lox.Distance(300.0)


def test_angle_arithmetic():
    a = 1.0 * lox.rad
    b = 2.0 * lox.rad
    assert a + b == lox.Angle(3.0)
    assert b - a == lox.Angle(1.0)
    assert -a == lox.Angle(-1.0)
    assert a * 2 == lox.Angle(2.0)


def test_velocity_arithmetic():
    a = 10 * lox.m_per_s
    b = 20 * lox.m_per_s
    assert a + b == lox.Velocity(30.0)
    assert b - a == lox.Velocity(10.0)
    assert -a == lox.Velocity(-10.0)


def test_frequency_eq():
    assert 1000 * lox.Hz == 1 * lox.kHz


# --- Conversion methods ---


def test_angle_conversions():
    a = math.pi * lox.rad
    assert a.to_radians() == pytest.approx(math.pi)
    assert a.to_degrees() == pytest.approx(180.0)
    a2 = lox.Angle(math.pi / 180.0 / 3600.0)
    assert a2.to_arcseconds() == pytest.approx(1.0)


def test_distance_conversions():
    d = 1 * lox.km
    assert d.to_meters() == pytest.approx(1000.0)
    assert d.to_kilometers() == pytest.approx(1.0)
    au = lox.Distance.from_astronomical_units(1.0)
    assert au.to_astronomical_units() == pytest.approx(1.0)


def test_velocity_conversions():
    v = 1 * lox.km_per_s
    assert v.to_meters_per_second() == pytest.approx(1000.0)
    assert v.to_kilometers_per_second() == pytest.approx(1.0)


def test_frequency_conversions():
    f = 1 * lox.GHz
    assert f.to_hertz() == pytest.approx(1e9)
    assert f.to_kilohertz() == pytest.approx(1e6)
    assert f.to_megahertz() == pytest.approx(1e3)
    assert f.to_gigahertz() == pytest.approx(1.0)
    assert f.to_terahertz() == pytest.approx(1e-3)


# --- New unit types ---


def test_angular_rate():
    ar = 1.0 * lox.rad_per_s
    assert str(ar)
    assert repr(ar) == "AngularRate(1.0)"
    assert float(ar) == 1.0
    assert ar.to_radians_per_second() == pytest.approx(1.0)
    assert ar.to_degrees_per_second() == pytest.approx(math.degrees(1.0))


def test_angular_rate_deg_per_s():
    ar = 180.0 * lox.deg_per_s
    assert ar.to_radians_per_second() == pytest.approx(math.pi)
    assert ar.to_degrees_per_second() == pytest.approx(180.0)


def test_angular_rate_arithmetic():
    a = 1.0 * lox.rad_per_s
    b = 2.0 * lox.rad_per_s
    assert a + b == lox.AngularRate(3.0)
    assert b - a == lox.AngularRate(1.0)
    assert -a == lox.AngularRate(-1.0)


def test_power():
    p = 100 * lox.W
    assert repr(p) == "Power(100.0)"
    assert float(p) == 100.0
    assert p.to_watts() == pytest.approx(100.0)
    assert p.to_kilowatts() == pytest.approx(0.1)
    assert p.to_dbw() == pytest.approx(20.0)


def test_power_kw():
    p = 1 * lox.kW
    assert p.to_watts() == pytest.approx(1000.0)


def test_power_arithmetic():
    a = 50 * lox.W
    b = 150 * lox.W
    assert a + b == lox.Power(200.0)
    assert b - a == lox.Power(100.0)
    assert -a == lox.Power(-50.0)


def test_temperature():
    t = 290 * lox.K
    assert repr(t) == "Temperature(290.0)"
    assert float(t) == 290.0
    assert t.to_kelvin() == pytest.approx(290.0)


def test_temperature_arithmetic():
    a = 100 * lox.K
    b = 200 * lox.K
    assert a + b == lox.Temperature(300.0)
    assert b - a == lox.Temperature(100.0)


# --- GravitationalParameter ---


def test_gravitational_parameter():
    gm = lox.GravitationalParameter(3.986004418e14)
    assert repr(gm) == "GravitationalParameter(398600441800000.0)"
    assert float(gm) == 3.986004418e14
    assert gm.to_m3_per_s2() == pytest.approx(3.986004418e14)
    assert gm.to_km3_per_s2() == pytest.approx(398600.4418)


def test_gravitational_parameter_from_km3():
    gm = lox.GravitationalParameter.from_km3_per_s2(398600.4418)
    assert gm.to_m3_per_s2() == pytest.approx(3.986004418e14)
    assert gm.to_km3_per_s2() == pytest.approx(398600.4418)


def test_gravitational_parameter_eq():
    gm1 = lox.GravitationalParameter(1e14)
    gm2 = lox.GravitationalParameter(1e14)
    assert gm1 == gm2


# --- __getnewargs__ (pickle support) ---


def test_distance_getnewargs():
    d = 42.0 * lox.m
    args = d.__getnewargs__()
    assert args == (42.0,)
    assert lox.Distance(*args) == d


def test_gravitational_parameter_getnewargs():
    gm = lox.GravitationalParameter(1e14)
    args = gm.__getnewargs__()
    assert lox.GravitationalParameter(*args) == gm


# --- repr with integer values (covers repr_f64 for non-decimal values) ---


def test_repr_integer_value():
    d = lox.Distance(1000.0)
    assert repr(d) == "Distance(1000.0)"


# --- Module-level dB constant ---


def test_db_constant():
    db = 3 * lox.dB
    assert float(db) == pytest.approx(3.0)


def test_decibel_mul():
    db = lox.Decibel(2.0)
    assert float(db * 3) == pytest.approx(6.0)
    assert float(3 * db) == pytest.approx(6.0)


# --- Ordering ---

ALL_QUANTITIES = [
    lox.Angle,
    lox.AngularRate,
    lox.Decibel,
    lox.Distance,
    lox.Frequency,
    lox.GravitationalParameter,
    lox.Power,
    lox.Pressure,
    lox.Temperature,
    lox.Velocity,
]


@pytest.mark.parametrize("cls", ALL_QUANTITIES)
def test_ordering(cls):
    small, large = cls(1.0), cls(2.0)
    assert small < large
    assert small <= large
    assert large > small
    assert large >= small
    assert small <= cls(1.0)
    assert small >= cls(1.0)
    assert not (small > large)


def test_ordering_sorts_and_reduces():
    assert sorted([500 * lox.km, 100 * lox.km]) == [100 * lox.km, 500 * lox.km]
    assert min(500 * lox.km, 100 * lox.km) == 100 * lox.km
    assert max(26.63 * lox.dB, 3 * lox.dB) == 26.63 * lox.dB
    # The threshold comparison a link budget is written around.
    assert 26.63 * lox.dB > 3 * lox.dB


def test_ordering_nan_is_always_false():
    nan = lox.Distance(float("nan"))
    one = lox.Distance(1.0)
    assert not (nan < one)
    assert not (nan > one)
    assert not (nan == one)
    assert nan != one


def test_ordering_across_types_raises():
    with pytest.raises(TypeError):
        _ = (1 * lox.km) < (1 * lox.rad)
    with pytest.raises(TypeError):
        _ = (1 * lox.km) > (1 * lox.dB)


def test_equality_with_foreign_types_is_false():
    d = 1 * lox.km
    assert d != "foo"
    assert d != 1000.0
    assert d != (1 * lox.rad)


# --- Hashing ---


@pytest.mark.parametrize("cls", ALL_QUANTITIES)
def test_hashable(cls):
    assert hash(cls(1.5)) == hash(cls(1.5))
    assert len({cls(1.0), cls(1.0), cls(2.0)}) == 2


def test_hash_usable_as_dict_key():
    bands = {8.2 * lox.GHz: "X-band", 2.2 * lox.GHz: "S-band"}
    assert bands[8.2 * lox.GHz] == "X-band"


def test_hash_consistent_with_equality_for_negative_zero():
    assert lox.Distance(-0.0) == lox.Distance(0.0)
    assert hash(lox.Distance(-0.0)) == hash(lox.Distance(0.0))


# --- Dimensional safety ---


def test_quantity_times_quantity_raises():
    with pytest.raises(TypeError):
        _ = (1 * lox.km) * (1 * lox.km)
    with pytest.raises(TypeError):
        _ = (10 * lox.dB) * (10 * lox.dB)
    with pytest.raises(TypeError):
        _ = (1 * lox.km) * (1 * lox.rad)


def test_quantity_plus_other_quantity_raises():
    with pytest.raises(TypeError):
        _ = (1 * lox.km) + (1 * lox.rad)


def test_scalar_multiplication_accepts_real_numbers():
    np = pytest.importorskip("numpy")
    for scalar in [2, 2.0, np.float64(2), np.float32(2), np.int64(2)]:
        assert (scalar * lox.Distance(500.0)).to_meters() == pytest.approx(1000.0)
    # bool is a subclass of int, so it scales like the integer it is.
    assert (True * lox.Distance(500.0)).to_meters() == pytest.approx(500.0)


@pytest.mark.parametrize("cls", ALL_QUANTITIES)
def test_constructor_rejects_other_quantities(cls):
    # The constructor takes a number, and PyO3 would otherwise coerce any
    # quantity through __float__.
    other = lox.Angle(1.0) if cls is not lox.Angle else lox.Distance(1.0)
    with pytest.raises(TypeError):
        cls(other)
    with pytest.raises(TypeError):
        cls(1 * lox.hours)


@pytest.mark.parametrize(
    "ctor",
    [
        lox.Distance.from_kilometers,
        lox.Angle.from_degrees,
        lox.Frequency.from_gigahertz,
        lox.Power.from_watts,
        lox.Temperature.from_kelvin,
        lox.Velocity.from_kilometers_per_second,
        lox.Pressure.from_hpa,
        lox.Decibel.from_linear,
        lox.GravitationalParameter.from_km3_per_s2,
    ],
)
def test_from_constructors_reject_other_quantities(ctor):
    with pytest.raises(TypeError):
        ctor(lox.Angle(1.0))


@pytest.mark.parametrize("cls", ALL_QUANTITIES)
def test_constructor_accepts_real_numbers(cls):
    np = pytest.importorskip("numpy")
    for value in [2, 2.0, np.float64(2), np.float32(2), np.int64(2)]:
        assert float(cls(value)) == pytest.approx(2.0)


def test_scalar_multiplication_rejects_non_numbers():
    with pytest.raises(TypeError):
        _ = "2" * lox.Distance(500.0)


# --- Division ---


def test_truediv_by_scalar_keeps_type():
    assert (500 * lox.km) / 2 == 250 * lox.km


def test_truediv_by_same_type_is_a_plain_ratio():
    ratio = (500 * lox.km) / (100 * lox.km)
    assert isinstance(ratio, float)
    assert ratio == pytest.approx(5.0)


def test_truediv_by_other_quantity_raises():
    with pytest.raises(TypeError):
        _ = (500 * lox.km) / (100 * lox.rad)


def test_rtruediv_raises():
    # 1/length has no type here.
    with pytest.raises(TypeError):
        _ = 2.0 / (500 * lox.km)


# --- round() and abs() ---


def test_round_operates_on_the_base_si_value():
    assert round(26.6314 * lox.dB, 2) == lox.Decibel(26.63)
    # Distance displays in km but rounds in metres, matching float().
    d = lox.Distance(909424.94)
    assert round(d, 1).to_meters() == pytest.approx(909424.9)
    assert round(d, -2).to_meters() == pytest.approx(909400.0)
    assert round(d).to_meters() == pytest.approx(909425.0)


def test_abs():
    assert abs(-(26.63 * lox.dB)) == 26.63 * lox.dB
    assert abs(500 * lox.km) == 500 * lox.km


# --- int() is deliberately absent ---


@pytest.mark.parametrize("cls", ALL_QUANTITIES)
def test_no_int_conversion(cls):
    # Silent truncation to whole metres or whole hertz is never what anyone wants.
    with pytest.raises(TypeError):
        int(cls(1.5))


# --- from_* constructors ---


@pytest.mark.parametrize(
    "cls, ctor, accessor, value",
    [
        (lox.Angle, "from_degrees", "to_degrees", 45.0),
        (lox.Angle, "from_radians", "to_radians", 1.5),
        (lox.Angle, "from_arcseconds", "to_arcseconds", 3600.0),
        (lox.AngularRate, "from_degrees_per_second", "to_degrees_per_second", 15.0),
        (lox.Distance, "from_kilometers", "to_kilometers", 500.0),
        (lox.Distance, "from_meters", "to_meters", 500.0),
        (lox.Distance, "from_astronomical_units", "to_astronomical_units", 1.0),
        (lox.Frequency, "from_gigahertz", "to_gigahertz", 8.2),
        (lox.Frequency, "from_megahertz", "to_megahertz", 150.0),
        (lox.Power, "from_watts", "to_watts", 100.0),
        (lox.Power, "from_kilowatts", "to_kilowatts", 1.5),
        (lox.Pressure, "from_hpa", "to_hpa", 1013.25),
        (lox.Temperature, "from_kelvin", "to_kelvin", 290.0),
        (lox.Velocity, "from_kilometers_per_second", "to_kilometers_per_second", 7.8),
        (lox.GravitationalParameter, "from_km3_per_s2", "to_km3_per_s2", 398600.435),
    ],
)
def test_from_constructors_round_trip(cls, ctor, accessor, value):
    quantity = getattr(cls, ctor)(value)
    assert getattr(quantity, accessor)() == pytest.approx(value, rel=1e-12)


def test_from_constructor_matches_unit_constant():
    assert lox.Distance.from_kilometers(500) == 500 * lox.km


# --- __format__ ---


@pytest.mark.parametrize(
    "spec, expected",
    [
        ("", "909.42494 km"),
        (".2f", "909.42 km"),
        (".1f", "909.4 km"),
        (">12.1f", "    909.4 km"),
        ("<12.1f", "909.4 km    "),
        ("*^20.3e", "****9.094e+02 km****"),
        ("+.1f", "+909.4 km"),
        ("08.2f", "00909.42 km"),
        (".4g", "909.4 km"),
    ],
)
def test_format_renders_the_display_unit(spec, expected):
    assert format(lox.Distance(909424.94), spec) == expected


def test_format_matches_str_for_an_empty_spec():
    d = 909.42494 * lox.km
    assert f"{d}" == str(d)


@pytest.mark.parametrize(
    "quantity, expected",
    [
        (lox.Angle.from_degrees(45.0), "45.00 deg"),
        (lox.Frequency.from_gigahertz(8.2), "8.20 GHz"),
        (lox.Velocity.from_kilometers_per_second(7.8), "7.80 km/s"),
        (lox.Power.from_watts(100.0), "100.00 W"),
        (lox.Temperature.from_kelvin(290.0), "290.00 K"),
        (lox.Decibel(26.6314), "26.63 dB"),
        (lox.GravitationalParameter.from_km3_per_s2(398600.0), "398600.00 km³/s²"),
    ],
)
def test_format_uses_each_types_display_unit(quantity, expected):
    assert f"{quantity:.2f}" == expected


def test_format_invalid_spec_raises():
    with pytest.raises(ValueError):
        format(500 * lox.km, "d")


# --- str() suppresses float noise, repr() stays exact ---


@pytest.mark.parametrize(
    "quantity, expected",
    [
        (29 * lox.GHz, "29 GHz"),
        (150 * lox.MHz, "0.15 GHz"),
        (8.2 * lox.GHz, "8.2 GHz"),
        (0.25 * lox.m, "0.00025 km"),
    ],
)
def test_str_suppresses_scaling_noise(quantity, expected):
    assert str(quantity) == expected


@pytest.mark.parametrize("cls", ALL_QUANTITIES)
def test_repr_round_trips(cls):
    quantity = cls(1234.5678)
    assert eval(repr(quantity), {cls.__name__: cls}) == quantity


# --- NumPy interoperability ---


def test_numpy_array_is_float64_in_base_si():
    np = pytest.importorskip("numpy")
    array = np.array([500 * lox.km, 100 * lox.km])
    assert array.dtype == np.float64
    assert array.tolist() == [500000.0, 100000.0]


def test_numpy_asarray_of_a_scalar():
    np = pytest.importorskip("numpy")
    assert np.asarray(500 * lox.km).item() == pytest.approx(500000.0)
    assert np.array(500 * lox.km, dtype=np.float32).dtype == np.float32


def test_numpy_scalar_multiplication_preserves_the_type():
    np = pytest.importorskip("numpy")
    # Without __array_ufunc__ = None numpy would take over and return a bare float.
    assert isinstance(np.float64(2) * (500 * lox.km), lox.Distance)
    assert isinstance(np.float32(2) * (500 * lox.km), lox.Distance)
    assert isinstance((500 * lox.km) * np.float64(2), lox.Distance)


# --- Pickling ---


@pytest.mark.parametrize("cls", ALL_QUANTITIES)
def test_pickle_round_trip(cls):
    import pickle

    quantity = cls(13.5)
    assert pickle.loads(pickle.dumps(quantity)) == quantity


# --- Unit objects ---

UNIT_CONSTANTS = [
    (lox.rad, lox.AngleUnit, lox.Angle, "rad", "rad", 1.0),
    (lox.deg, lox.AngleUnit, lox.Angle, "deg", "deg", math.pi / 180.0),
    (lox.rad_per_s, lox.AngularRateUnit, lox.AngularRate, "rad_per_s", "rad/s", 1.0),
    (lox.deg_per_s, lox.AngularRateUnit, lox.AngularRate, "deg_per_s", "deg/s", math.pi / 180.0),
    (lox.m, lox.DistanceUnit, lox.Distance, "m", "m", 1.0),
    (lox.km, lox.DistanceUnit, lox.Distance, "km", "km", 1e3),
    (lox.Hz, lox.FrequencyUnit, lox.Frequency, "Hz", "Hz", 1.0),
    (lox.GHz, lox.FrequencyUnit, lox.Frequency, "GHz", "GHz", 1e9),
    (lox.W, lox.PowerUnit, lox.Power, "W", "W", 1.0),
    (lox.kW, lox.PowerUnit, lox.Power, "kW", "kW", 1e3),
    (lox.Pa, lox.PressureUnit, lox.Pressure, "Pa", "Pa", 1.0),
    (lox.hPa, lox.PressureUnit, lox.Pressure, "hPa", "hPa", 100.0),
    (lox.K, lox.TemperatureUnit, lox.Temperature, "K", "K", 1.0),
    (lox.m_per_s, lox.VelocityUnit, lox.Velocity, "m_per_s", "m/s", 1.0),
    (lox.km_per_s, lox.VelocityUnit, lox.Velocity, "km_per_s", "km/s", 1e3),
    (lox.dB, lox.DecibelUnit, lox.Decibel, "dB", "dB", 1.0),
]


@pytest.mark.parametrize(
    "unit, unit_cls, quantity_cls, symbol, suffix, scale", UNIT_CONSTANTS
)
def test_unit_constants(unit, unit_cls, quantity_cls, symbol, suffix, scale):
    assert isinstance(unit, unit_cls)
    assert unit.symbol == symbol
    assert unit.suffix == suffix
    assert unit.scale == pytest.approx(scale)
    assert str(unit) == suffix
    assert repr(unit) == f'{unit_cls.__name__}("{symbol}")'
    # A unit is not a quantity.
    assert not isinstance(unit, quantity_cls)


@pytest.mark.parametrize(
    "unit, unit_cls, quantity_cls, symbol, suffix, scale", UNIT_CONSTANTS
)
def test_unit_scales_in_both_directions(
    unit, unit_cls, quantity_cls, symbol, suffix, scale
):
    assert isinstance(2 * unit, quantity_cls)
    assert isinstance(unit * 2, quantity_cls)
    assert float(2 * unit) == pytest.approx(2 * scale)
    assert (2 * unit) == (unit * 2)


def test_unit_lookup_by_symbol_and_suffix():
    assert lox.DistanceUnit("km") == lox.km
    assert lox.VelocityUnit("km_per_s") == lox.km_per_s
    # The display suffix works too, which is what the format spec accepts.
    assert lox.VelocityUnit("km/s") == lox.km_per_s


def test_unit_unknown_name_raises():
    with pytest.raises(ValueError, match="not a DistanceUnit"):
        lox.DistanceUnit("furlong")


def test_unit_equality_and_hashing():
    assert lox.km == lox.DistanceUnit("km")
    assert lox.km != lox.m
    assert hash(lox.km) == hash(lox.DistanceUnit("km"))
    assert {lox.km: "kilometers"}[lox.DistanceUnit("km")] == "kilometers"


def test_unit_pickle_round_trip():
    import pickle

    for unit, *_ in UNIT_CONSTANTS:
        assert pickle.loads(pickle.dumps(unit)) == unit


def test_unit_repr_round_trips():
    namespace = {cls.__name__: cls for _, cls, *_ in UNIT_CONSTANTS}
    for unit, *_ in UNIT_CONSTANTS:
        assert eval(repr(unit), namespace) == unit


def test_unit_is_not_a_scalar():
    with pytest.raises(TypeError):
        float(lox.km)


def test_unit_arithmetic_is_rejected():
    with pytest.raises(TypeError):
        _ = lox.km + lox.km
    with pytest.raises(TypeError):
        _ = lox.km * lox.km
    with pytest.raises(TypeError):
        _ = lox.km * lox.m
    with pytest.raises(TypeError):
        _ = (1 * lox.km) * lox.km


def test_units_are_not_ordered():
    with pytest.raises(TypeError):
        _ = lox.km < lox.m


# --- Converting through a unit ---


def test_divide_by_unit_gives_a_plain_float():
    d = 909.42494 * lox.km
    assert d / lox.m == pytest.approx(909424.94)
    assert d / lox.km == pytest.approx(909.42494)
    assert isinstance(d / lox.m, float)


def test_divide_by_a_foreign_unit_raises():
    with pytest.raises(TypeError):
        _ = (500 * lox.km) / lox.GHz


# --- Selecting a unit in the format spec ---


@pytest.mark.parametrize(
    "spec, expected",
    [
        (".1f", "909.4 km"),
        (".1f m", "909424.9 m"),
        (".1f km", "909.4 km"),
        (">14.1f m", "    909424.9 m"),
        # A leading space is the sign option, and a space fill keeps its meaning.
        (" .1f", " 909.4 km"),
        ("> 12.1f", "    909.4 km"),
    ],
)
def test_format_unit_token(spec, expected):
    assert format(lox.Distance(909424.94), spec) == expected


def test_format_unit_token_accepts_the_display_suffix():
    v = 7.8 * lox.km_per_s
    assert f"{v:.1f m/s}" == "7800.0 m/s"
    assert f"{v:.1f m_per_s}" == "7800.0 m/s"


def test_format_unknown_unit_raises():
    with pytest.raises(ValueError, match="not a DistanceUnit"):
        format(500 * lox.km, ".1f dB")
