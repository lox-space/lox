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
    assert int(a) == 3
    assert complex(a) == complex(math.pi)


def test_angle_deg():
    a = 180 * lox.deg
    assert str(a) == "180 deg"
    assert repr(a) == "Angle(3.141592653589793)"
    assert float(a) == math.pi
    assert int(a) == 3
    assert complex(a) == complex(math.pi)


def test_distance_km():
    d = 1024 * lox.km
    assert str(d) == "1024 km"
    assert repr(d) == "Distance(1024000.0)"
    assert float(d) == 1024000.0
    assert int(d) == 1024000
    assert complex(d) == complex(1024000.0)


def test_distance_m():
    d = 2048 * lox.m
    assert str(d) == "2.048 km"
    assert repr(d) == "Distance(2048.0)"
    assert float(d) == 2048.0
    assert int(d) == 2048
    assert complex(d) == complex(2048.0)


def test_frequency_hz():
    f = 1073741824 * lox.Hz
    assert str(f) == "1.073741824 GHz"
    assert repr(f) == "Frequency(1073741824.0)"
    assert float(f) == 1073741824.0
    assert int(f) == 1073741824
    assert complex(f) == complex(1073741824.0)


def test_frequency_khz():
    f = 2000000 * lox.kHz
    assert str(f) == "2 GHz"
    assert repr(f) == "Frequency(2000000000.0)"
    assert float(f) == 2000000000.0
    assert int(f) == 2000000000
    assert complex(f) == complex(2000000000.0)


def test_velocity_ms():
    v = 262144 * lox.m_per_s
    assert str(v) == "262.144 km/s"
    assert repr(v) == "Velocity(262144.0)"
    assert float(v) == 262144.0
    assert int(v) == 262144
    assert complex(v) == complex(262144.0)


def test_velocity_kms():
    v = 16 * lox.km_per_s
    assert str(v) == "16 km/s"
    assert repr(v) == "Velocity(16000.0)"
    assert float(v) == 16000.0
    assert int(v) == 16000
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
    au = lox.Distance(lox.au.to_meters())
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


def test_data_rate():
    dr = 1000 * lox.bps
    assert repr(dr) == "DataRate(1000.0)"
    assert float(dr) == 1000.0
    assert dr.to_bits_per_second() == pytest.approx(1000.0)
    assert dr.to_kilobits_per_second() == pytest.approx(1.0)
    assert dr.to_megabits_per_second() == pytest.approx(0.001)


def test_data_rate_kbps():
    dr = 1 * lox.kbps
    assert dr.to_bits_per_second() == pytest.approx(1000.0)


def test_data_rate_mbps():
    dr = 1 * lox.Mbps
    assert dr.to_bits_per_second() == pytest.approx(1e6)


def test_data_rate_arithmetic():
    a = 100 * lox.bps
    b = 200 * lox.bps
    assert a + b == lox.DataRate(300.0)
    assert b - a == lox.DataRate(100.0)


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
