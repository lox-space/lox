# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox
import numpy as np
import numpy.testing as npt
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
    s0 = orbit.to_cartesian()
    prop = lox.Vallado(s0)
    return prop.propagate(orbit.time(), end=orbit.time() + dt)


def test_from_numpy():
    utc = lox.UTC(2023, 3, 25, 21, 8, 0.0)
    time = utc.to_scale("TDB")
    states = np.array(
        [
            [0.0, 1e3, 1e3, 1e3, 1e3, 1e3, 1e3],
            [1.0, 2e3, 2e3, 2e3, 1e3, 1e3, 1e3],
            [2.0, 3e3, 3e3, 3e3, 1e3, 1e3, 1e3],
            [3.0, 4e3, 4e3, 4e3, 1e3, 1e3, 1e3],
        ]
    )
    trajectory = lox.Trajectory.from_numpy(time, states)
    npt.assert_allclose(
        trajectory.at(time + lox.TimeDelta(1.5)).position(),
        [2.5e3, 2.5e3, 2.5e3],
    )
    states1 = trajectory.to_numpy()
    npt.assert_allclose(states, states1)


def test_at_outside_trajectory_raises():
    utc = lox.UTC(2023, 3, 25, 21, 8, 0.0)
    time = utc.to_scale("TDB")
    states = np.array(
        [
            [0.0, 1e3, 1e3, 1e3, 1e3, 1e3, 1e3],
            [1.0, 2e3, 2e3, 2e3, 1e3, 1e3, 1e3],
        ]
    )
    trajectory = lox.Trajectory.from_numpy(time, states)

    trajectory.at(time + lox.TimeDelta(1.0))
    with pytest.raises(ValueError, match="outside the trajectory"):
        trajectory.at(time + lox.TimeDelta(1.5))
    with pytest.raises(ValueError, match="outside the trajectory"):
        trajectory.at(lox.TimeDelta(-0.5))


def test_interval():
    utc = lox.UTC(2023, 3, 25, 21, 8, 0.0)
    time = utc.to_scale("TDB")
    states = np.array(
        [
            [0.0, 1e3, 1e3, 1e3, 1e3, 1e3, 1e3],
            [1.0, 2e3, 2e3, 2e3, 1e3, 1e3, 1e3],
            [3.0, 4e3, 4e3, 4e3, 1e3, 1e3, 1e3],
        ]
    )
    a = lox.Trajectory.from_numpy(time, states)
    b = lox.Trajectory.from_numpy(time + lox.TimeDelta(2.0), states)

    assert a.interval().start() == time
    assert float(a.interval().duration()) == pytest.approx(3.0)
    overlap = a.interval().intersect(b.interval())
    assert overlap.start() == time + lox.TimeDelta(2.0)
    assert float(overlap.duration()) == pytest.approx(1.0)


def test_propagate_covers_interval_end(orbit):
    end = orbit.time() + orbit.orbital_period()
    trajectory = lox.Vallado(orbit.to_cartesian()).propagate(orbit.time(), end=end)

    trajectory.at(end)


def test_interpolation(orbit, trajectory):
    dt = orbit.orbital_period()
    s1 = trajectory.at(dt)
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


def test_keplerian_repr(orbit):
    r = repr(orbit)
    assert r.startswith("Keplerian(")
    assert "Distance(" in r
    assert "Angle(" in r
    assert "Origin(" in r


def test_keplerian_returns_unit_types(orbit):
    assert isinstance(orbit.semi_major_axis(), lox.Distance)
    assert isinstance(orbit.inclination(), lox.Angle)
    assert isinstance(orbit.longitude_of_ascending_node(), lox.Angle)
    assert isinstance(orbit.argument_of_periapsis(), lox.Angle)
    assert isinstance(orbit.true_anomaly(), lox.Angle)


def test_trajectory_repr(trajectory):
    r = repr(trajectory)
    assert r.startswith("Trajectory(")
    assert "states" in r
    assert "Origin(" in r
    assert "Frame(" in r


def test_event_constructor():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    event = lox.Event(time, "up")
    assert event.crossing() == "up"
    r = repr(event)
    assert r.startswith("Event(")


def test_event_down():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    event = lox.Event(time, "down")
    assert event.crossing() == "down"


def test_interval_constructor():
    t1 = lox.UTC(2024, 1, 1).to_scale("TDB")
    t2 = lox.UTC(2024, 1, 2).to_scale("TDB")
    interval = lox.Interval(t1, t2)
    r = repr(interval)
    assert r.startswith("Interval(")


def test_keplerian_with_string_origin():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(
        time,
        24464.560 * lox.km,
        0.7311,
        0.122138 * lox.rad,
        1.00681 * lox.rad,
        3.10686 * lox.rad,
        0.44369564302687126 * lox.rad,
        origin="Earth",
    )
    assert k.origin().name() == "Earth"
