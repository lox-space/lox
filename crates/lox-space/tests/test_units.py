# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
# SPDX-License-Identifier: MPL-2.0

import math
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
    assert repr(d) == "Distance(1024000)"
    assert float(d) == 1024000.0
    assert int(d) == 1024000
    assert complex(d) == complex(1024000.0)

def test_distance_m():
    d = 2048 * lox.m
    assert str(d) == "2.048 km"
    assert repr(d) == "Distance(2048)"
    assert float(d) == 2048.0
    assert int(d) == 2048
    assert complex(d) == complex(2048.0)

def test_frequency_hz():
    f = 1073741824 * lox.hz
    assert str(f) == "1.073741824 GHz"
    assert repr(f) == "Frequency(1073741824)"
    assert float(f) == 1073741824.0
    assert int(f) == 1073741824
    assert complex(f) == complex(1073741824.0)

def test_frequency_khz():
    f = 2000000 * lox.khz
    assert str(f) == "2 GHz"
    assert repr(f) == "Frequency(2000000000)"
    assert float(f) == 2000000000.0
    assert int(f) == 2000000000
    assert complex(f) == complex(2000000000.0)

def test_velocity_ms():
    v = 262144 * lox.ms
    assert str(v) == "262.144 km/s"
    assert repr(v) == "Velocity(262144)"
    assert float(v) == 262144.0
    assert int(v) == 262144
    assert complex(v) == complex(262144.0)

def test_velocity_kms():
    v = 16 * lox.kms
    assert str(v) == "16 km/s"
    assert repr(v) == "Velocity(16000)"
    assert float(v) == 16000.0
    assert int(v) == 16000
    assert complex(v) == complex(16000.0)
