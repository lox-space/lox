#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

import lox_space as lox
import numpy as np
import math
import pytest


@pytest.fixture
def orbit():
    utc = lox.UTC(2023, 3, 25, 21, 8, 0.0)
    time = utc.to_tdb()
    semi_major_axis = 24464.560
    eccentricity = 0.7311
    inclination = 0.122138
    longitude_of_ascending_node = 1.00681
    argument_of_periapsis = 3.10686
    true_anomaly = 0.44369564302687126
    return lox.Keplerian(time, semi_major_axis, eccentricity, inclination, longitude_of_ascending_node,
                         argument_of_periapsis, true_anomaly)


@pytest.fixture
def trajectory(orbit):
    dt = orbit.orbital_period()
    rng = lox.TimeDelta.range(0, math.ceil(dt))
    s0 = orbit.to_cartesian()
    prop = lox.Vallado()
    return prop.propagate(s0, rng)


def test_interpolation(orbit, trajectory):
    dt = orbit.orbital_period()
    s1 = trajectory.interpolate(dt)
    k1 = s1.to_keplerian()
    assert orbit.semi_major_axis() == pytest.approx(k1.semi_major_axis(), rel=1e-8)
    assert orbit.eccentricity() == pytest.approx(k1.eccentricity(), rel=1e-8)
    assert orbit.inclination() == pytest.approx(k1.inclination(), rel=1e-8)
    assert orbit.longitude_of_ascending_node() == pytest.approx(k1.longitude_of_ascending_node(), rel=1e-8)
    assert orbit.argument_of_periapsis() == pytest.approx(k1.argument_of_periapsis(), rel=1e-8)
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
