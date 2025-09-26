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
