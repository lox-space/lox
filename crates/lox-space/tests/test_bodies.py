# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import pytest
import lox_space as lox


def test_origin_from_string():
    earth = lox.Origin("Earth")
    assert earth.name() == "Earth"


def test_origin_from_id():
    earth = lox.Origin(399)
    assert earth.name() == "Earth"


def test_origin_repr():
    earth = lox.Origin("Earth")
    assert repr(earth) == 'Origin("Earth")'


def test_gravitational_parameter():
    earth = lox.Origin("Earth")
    gm = earth.gravitational_parameter()
    assert isinstance(gm, lox.GravitationalParameter)
    assert gm.to_km3_per_s2() == pytest.approx(398600.4418, rel=1e-4)


def test_mean_radius():
    earth = lox.Origin("Earth")
    r = earth.mean_radius()
    assert isinstance(r, lox.Distance)
    assert r.to_kilometers() == pytest.approx(6371.0084, rel=1e-3)


def test_radii():
    earth = lox.Origin("Earth")
    a, b, c = earth.radii()
    assert isinstance(a, lox.Distance)
    assert isinstance(b, lox.Distance)
    assert isinstance(c, lox.Distance)
    assert a.to_kilometers() == pytest.approx(6378.1366, rel=1e-3)


def test_equatorial_radius():
    earth = lox.Origin("Earth")
    r = earth.equatorial_radius()
    assert isinstance(r, lox.Distance)
    assert r.to_kilometers() == pytest.approx(6378.1366, rel=1e-3)


def test_polar_radius():
    earth = lox.Origin("Earth")
    r = earth.polar_radius()
    assert isinstance(r, lox.Distance)
    assert r.to_kilometers() == pytest.approx(6356.7519, rel=1e-3)


def test_rotational_elements():
    earth = lox.Origin("Earth")
    ra, dec, rot = earth.rotational_elements(0.0)
    assert isinstance(ra, lox.Angle)
    assert isinstance(dec, lox.Angle)
    assert isinstance(rot, lox.Angle)


def test_rotational_element_rates():
    earth = lox.Origin("Earth")
    ra_rate, dec_rate, rot_rate = earth.rotational_element_rates(0.0)
    assert isinstance(ra_rate, lox.AngularRate)
    assert isinstance(dec_rate, lox.AngularRate)
    assert isinstance(rot_rate, lox.AngularRate)


def test_right_ascension():
    earth = lox.Origin("Earth")
    ra = earth.right_ascension(0.0)
    assert isinstance(ra, lox.Angle)


def test_right_ascension_rate():
    earth = lox.Origin("Earth")
    ra_rate = earth.right_ascension_rate(0.0)
    assert isinstance(ra_rate, lox.AngularRate)


def test_declination():
    earth = lox.Origin("Earth")
    dec = earth.declination(0.0)
    assert isinstance(dec, lox.Angle)


def test_declination_rate():
    earth = lox.Origin("Earth")
    dec_rate = earth.declination_rate(0.0)
    assert isinstance(dec_rate, lox.AngularRate)


def test_rotation_angle():
    earth = lox.Origin("Earth")
    rot = earth.rotation_angle(0.0)
    assert isinstance(rot, lox.Angle)


def test_rotation_rate():
    earth = lox.Origin("Earth")
    rot_rate = earth.rotation_rate(0.0)
    assert isinstance(rot_rate, lox.AngularRate)


def test_origin_string_in_cartesian():
    """Test that string origin works in Cartesian constructor."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    state = lox.Cartesian(
        time,
        position=[7000e3, 0.0, 0.0],
        velocity=[0.0, 7500.0, 0.0],
        origin="Earth",
    )
    assert state.origin().name() == "Earth"


def test_origin_id_in_cartesian():
    """Test that integer origin ID works in Cartesian constructor."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    state = lox.Cartesian(
        time,
        position=[7000e3, 0.0, 0.0],
        velocity=[0.0, 7500.0, 0.0],
        origin=399,
    )
    assert state.origin().name() == "Earth"
