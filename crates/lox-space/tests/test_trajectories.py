# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox
import numpy as np
import numpy.testing as npt
import math
import pytest


@pytest.fixture
def orbit():
    utc = lox.UTC(2023, 3, 25, 21, 8, 0.0)
    time = utc.to_scale("TDB")
    semi_major_axis = 24464.560
    eccentricity = 0.7311
    inclination = 0.122138
    longitude_of_ascending_node = 1.00681
    argument_of_periapsis = 3.10686
    true_anomaly = 0.44369564302687126
    return lox.Keplerian(
        time,
        semi_major_axis,
        eccentricity,
        inclination,
        longitude_of_ascending_node,
        argument_of_periapsis,
        true_anomaly,
    )


@pytest.fixture
def trajectory(orbit):
    dt = orbit.orbital_period()
    rng = [orbit.time() + dt for dt in lox.TimeDelta.range(0, math.ceil(dt))]
    s0 = orbit.to_cartesian()
    prop = lox.Vallado(s0)
    return prop.propagate(rng)


def test_from_numpy():
    utc = lox.UTC(2023, 3, 25, 21, 8, 0.0)
    time = utc.to_scale("TDB")
    states = np.array(
        [
            [0.0, 1e3, 1e3, 1e3, 1.0, 1.0, 1.0],
            [1.0, 2e3, 2e3, 2e3, 2.0, 2.0, 2.0],
            [2.0, 3e3, 3e3, 3e3, 3.0, 3.0, 3.0],
            [3.0, 4e3, 4e3, 4e3, 4.0, 4.0, 4.0],
        ]
    )
    trajectory = lox.Trajectory.from_numpy(time, states)
    npt.assert_allclose(
        trajectory.interpolate(time + lox.TimeDelta(1.5)).position(),
        np.array([2.5e3, 2.5e3, 2.5e3]),
    )
    states1 = trajectory.to_numpy()
    npt.assert_allclose(states, states1)


def test_interpolation(orbit, trajectory):
    dt = orbit.orbital_period()
    s1 = trajectory.interpolate(dt)
    k1 = s1.to_keplerian()
    assert orbit.semi_major_axis() == pytest.approx(k1.semi_major_axis(), rel=1e-8)
    assert orbit.eccentricity() == pytest.approx(k1.eccentricity(), rel=1e-8)
    assert orbit.inclination() == pytest.approx(k1.inclination(), rel=1e-8)
    assert orbit.longitude_of_ascending_node() == pytest.approx(
        k1.longitude_of_ascending_node(), rel=1e-8
    )
    assert orbit.argument_of_periapsis() == pytest.approx(
        k1.argument_of_periapsis(), rel=1e-8
    )
    assert orbit.true_anomaly() == pytest.approx(k1.true_anomaly(), rel=1e-8)


def test_events(trajectory):
    def apsis_pass(s):
        return s.position() @ s.velocity()

    events = trajectory.find_events(apsis_pass)
    assert len(events) == 2
    k1 = trajectory.interpolate(events[0].time()).to_keplerian()
    assert k1.true_anomaly() == pytest.approx(np.pi, rel=1e-8)
    k2 = trajectory.interpolate(events[1].time()).to_keplerian()
    assert k2.true_anomaly() == pytest.approx(0.0, abs=1e-8)


def test_windows(trajectory):
    def above_equator(s):
        return s.position()[2]

    windows = trajectory.find_windows(above_equator)
    assert len(windows) == 1


import re

def expect_callback_error(call, expected_exc, match=None):
    try:
        result = call()
    except Exception as e:
        if not isinstance(e, expected_exc):
            pytest.fail(
                f"Expected {expected_exc.__name__} from callback, "
                f"but got {type(e).__name__}: {e!r}"
            )
        if match is not None and re.search(match, str(e)) is None:
            pytest.fail(
                f"Exception message mismatch.\n"
                f"  expected pattern: {match}\n"
                f"  actual message:   {e}"
            )
        return
    pytest.fail(
        f"Expected {expected_exc.__name__} from callback, "
        f"but no exception was raised. Function returned: {result!r}"
    )

# these can be rewritten as
# with pytest.raises(...) but this way we can also check the exception message
def test_events_callback_error(trajectory):
    def bad_func(_s):
        raise ValueError("boom in events")
    expect_callback_error(lambda: trajectory.find_events(bad_func), ValueError, r"boom in events")

def test_windows_callback_error(trajectory):
    def bad_func(_s):
        raise ValueError("boom in windows")
    expect_callback_error(
        lambda: trajectory.find_windows(bad_func),
        ValueError,
        r"boom in windows"
    )
