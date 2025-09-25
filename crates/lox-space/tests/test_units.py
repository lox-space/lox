import math
import lox_space as lox

def test_angle():
    a = math.pi * lox.rad
    assert str(a) == "180 deg"
    assert repr(a) == "Angle(3.141592653589793)"
    
