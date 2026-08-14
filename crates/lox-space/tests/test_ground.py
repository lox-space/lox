# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox
import numpy as np
import pytest


def test_observables():
    location = lox.EllipsoidLocation(
        "IAU_EARTH", -4 * lox.deg, 41 * lox.deg, 0 * lox.km
    )
    time = lox.Time("TDB", 2012, 7, 1)
    state = lox.Cartesian(
        time,
        position=[3359927.0, -2398072.0, 5153000.0],
        velocity=[5065.7, 5485.0, -744.0],
        frame=lox.Frame("IAU_EARTH"),
    )
    observables = location.observables(state)
    expected_range = 2707.7
    expected_range_rate = -7.16
    expected_azimuth = np.radians(-53.418)
    expected_elevation = np.radians(-7.077)
    assert observables.range().to_kilometers() == pytest.approx(
        expected_range, rel=1e-2
    )
    assert observables.range_rate().to_kilometers_per_second() == pytest.approx(
        expected_range_rate, rel=1e-2
    )
    assert float(observables.azimuth()) == pytest.approx(expected_azimuth, rel=1e-2)
    assert float(observables.elevation()) == pytest.approx(expected_elevation, rel=1e-2)


def test_ground_location_repr():
    location = lox.EllipsoidLocation(
        "IAU_EARTH", -4 * lox.deg, 41 * lox.deg, 0 * lox.km
    )
    r = repr(location)
    assert r.startswith("EllipsoidLocation(")
    assert "Frame(" in r
    assert "Angle(" in r
    assert "Distance(" in r
    assert "Ellipsoid(" in r


def test_ground_location_origin():
    location = lox.EllipsoidLocation(
        lox.Frame("IAU_EARTH"), 0 * lox.deg, 0 * lox.deg, 0 * lox.km
    )
    assert location.origin().name() == "Earth"


def test_ground_location_string_frame():
    location = lox.EllipsoidLocation("IAU_EARTH", 0 * lox.deg, 0 * lox.deg, 0 * lox.km)
    assert repr(location.frame()) == 'Frame("IAU_EARTH")'
    assert location.origin().name() == "Earth"


def test_ground_location_terrestrial_frame():
    """Any body-fixed frame works, not just IAU ones."""
    location = lox.EllipsoidLocation("ITRF", 0 * lox.deg, 0 * lox.deg, 0 * lox.km)
    assert repr(location.frame()) == 'Frame("ITRF")'
    assert location.origin().name() == "Earth"
    assert location.ellipsoid() == lox.Ellipsoid.GRS80


def test_ground_location_rejects_non_body_fixed_frame():
    with pytest.raises(ValueError, match="body-fixed"):
        lox.EllipsoidLocation("ICRF", 0 * lox.deg, 0 * lox.deg, 0 * lox.km)


def test_ground_location_default_ellipsoid():
    """An IAU frame defaults to its body's own spheroid."""
    location = lox.EllipsoidLocation("IAU_EARTH", 0 * lox.deg, 0 * lox.deg, 0 * lox.km)
    assert location.ellipsoid() != lox.Ellipsoid.WGS84


def test_ground_location_ellipsoid_override():
    """The frame's conventional ellipsoid is a default, not a constraint."""
    default = lox.EllipsoidLocation("IAU_EARTH", 0 * lox.deg, 0 * lox.deg, 0 * lox.km)
    overridden = lox.EllipsoidLocation(
        "IAU_EARTH",
        0 * lox.deg,
        0 * lox.deg,
        0 * lox.km,
        ellipsoid=lox.Ellipsoid.WGS84,
    )
    assert overridden.ellipsoid() == lox.Ellipsoid.WGS84
    assert overridden.ellipsoid() != default.ellipsoid()


def test_ground_location_coordinates():
    location = lox.EllipsoidLocation(
        "IAU_EARTH", -4 * lox.deg, 41 * lox.deg, 0.1 * lox.km
    )
    lon, lat, alt = location.coordinates()
    assert lon == location.longitude()
    assert lat == location.latitude()
    assert alt == location.altitude()


def test_ground_location_body_fixed_position():
    location = lox.EllipsoidLocation("IAU_EARTH", 0 * lox.deg, 0 * lox.deg, 0 * lox.km)
    pos = location.body_fixed_position()
    assert pos.shape == (3,)
    # On the equator at the prime meridian, the position is along +x.
    assert pos[0] == pytest.approx(
        location.ellipsoid().equatorial_radius().to_meters()
    )
    assert pos[1] == pytest.approx(0.0)
    assert pos[2] == pytest.approx(0.0)


def test_ellipsoid_constants():
    assert lox.Ellipsoid.WGS84.equatorial_radius().to_meters() == pytest.approx(
        6378137.0
    )
    assert lox.Ellipsoid.WGS84.flattening() == pytest.approx(1 / 298.257223563)
    assert lox.Ellipsoid.GRS80.equatorial_radius().to_meters() == pytest.approx(
        6378137.0
    )
    assert lox.Ellipsoid.WGS84 != lox.Ellipsoid.GRS80


def test_ellipsoid_construction():
    ellipsoid = lox.Ellipsoid(6378137.0 * lox.m, 1 / 298.257223563)
    assert ellipsoid == lox.Ellipsoid.WGS84
    assert repr(ellipsoid).startswith("Ellipsoid(")


def test_ellipsoid_rejects_invalid_flattening():
    with pytest.raises(ValueError, match="flattening"):
        lox.Ellipsoid(6378137.0 * lox.m, 1.5)


def test_elevation_mask():
    mask = lox.ElevationMask.variable(
        np.array([-np.pi, 0.0, np.pi]), np.array([0.0, 5.0, 0.0])
    )
    assert float(mask.min_elevation(lox.Angle(np.pi / 2))) == pytest.approx(2.5)
