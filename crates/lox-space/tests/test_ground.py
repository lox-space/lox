import lox_space as lox
import numpy as np
import pytest


def test_observables():
    longitude = np.radians(-4)
    latitude = np.radians(41)
    location = lox.GroundLocation(lox.Origin("Earth"), longitude, latitude, 0)
    position = (3359.927, -2398.072, 5153.0)
    velocity = (5.0657, 5.485, -0.744)
    time = lox.Time("TDB", 2012, 7, 1)
    state = lox.State(time, position, velocity, frame=lox.Frame("IAU_EARTH"))
    observables = location.observables(state)
    expected_range = 2707.7
    expected_range_rate = -7.16
    expected_azimuth = np.radians(-53.418)
    expected_elevation = np.radians(-7.077)
    assert observables.range() == pytest.approx(expected_range, rel=1e-2)
    assert observables.range_rate() == pytest.approx(expected_range_rate, rel=1e-2)
    assert observables.azimuth() == pytest.approx(expected_azimuth, rel=1e-2)
    assert observables.elevation() == pytest.approx(expected_elevation, rel=1e-2)


def test_elevation_mask():
    mask = lox.ElevationMask.variable(
        np.array([-np.pi, 0.0, np.pi]), np.array([0.0, 5.0, 0.0])
    )
    assert mask.min_elevation(np.pi / 2) == 2.5
