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
    return lox.Keplerian(
        time,
        24464.560 * lox.km,
        0.7311,
        0.122138 * lox.rad,
        1.00681 * lox.rad,
        3.10686 * lox.rad,
        0.44369564302687126 * lox.rad,
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
        [2.5e3, 2.5e3, 2.5e3],
    )
    states1 = trajectory.to_numpy()
    npt.assert_allclose(states, states1)


def test_interpolation(orbit, trajectory):
    dt = orbit.orbital_period()
    s1 = trajectory.interpolate(dt)
    k1 = s1.to_keplerian()
    assert float(orbit.semi_major_axis()) == pytest.approx(
        float(k1.semi_major_axis()), rel=1e-8
    )
    assert orbit.eccentricity() == pytest.approx(k1.eccentricity(), rel=1e-8)
    assert float(orbit.inclination()) == pytest.approx(
        float(k1.inclination()), rel=1e-8
    )
    assert float(orbit.longitude_of_ascending_node()) == pytest.approx(
        float(k1.longitude_of_ascending_node()), rel=1e-8
    )
    assert float(orbit.argument_of_periapsis()) == pytest.approx(
        float(k1.argument_of_periapsis()), rel=1e-8
    )
    assert float(orbit.true_anomaly()) == pytest.approx(
        float(k1.true_anomaly()), rel=1e-8
    )
