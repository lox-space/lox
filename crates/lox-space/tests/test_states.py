# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox
import numpy.testing as npt
import pytest


def test_state_to_ground_location():
    time = lox.UTC.from_iso("2024-07-05T09:09:18.173").to_scale("TAI")
    state = lox.Cartesian(
        time,
        position=[-5530017.74359, -3487089.5338, -1850034.76185],
        velocity=[1295.34407, -5024.56882, 5639.1936],
        origin=lox.Origin("Earth"),
        frame=lox.Frame("ICRF"),
    ).to_frame(lox.Frame("IAU_EARTH"))
    npt.assert_allclose(
        state.position() * 1e-3,
        [-5740.259426667957, 3121.1360727954725, -1863.1826563318027],
    )
    npt.assert_allclose(
        state.velocity() * 1e-3,
        [-3.53237875783652, -3.152377656863808, 5.642296713889555],
    )
    ground = state.to_ground_location()
    assert float(ground.longitude()) == pytest.approx(2.643578045424445)
    assert float(ground.latitude()) == pytest.approx(-0.27944957125091063)
    assert ground.altitude().to_kilometers() == pytest.approx(417.8524151150059)


def test_state_to_origin(ephemeris):
    r_m = [6068279.27, -1692843.94, -2516619.18]
    v_ms = [-660.415582, 5495.938726, -5303.093233]

    utc = lox.UTC.from_iso("2016-05-30T12:00:00.000")
    tai = utc.to_scale("TAI")

    s_earth = lox.Cartesian(
        tai,
        position=r_m,
        velocity=v_ms,
    )
    s_venus = s_earth.to_origin(lox.Origin("Venus"), ephemeris)

    # Verify round-trip: converting back to Earth origin should recover the original state
    s_earth_rt = s_venus.to_origin(lox.Origin("Earth"), ephemeris)

    npt.assert_allclose(s_earth_rt.position(), r_m, atol=1e-3)
    npt.assert_allclose(s_earth_rt.velocity(), v_ms, atol=1e-3)


def test_cartesian_component_kwargs():
    """Test Cartesian construction with x, y, z, vx, vy, vz keyword arguments."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    state = lox.Cartesian(
        time,
        x=7000 * lox.km,
        y=0 * lox.km,
        z=0 * lox.km,
        vx=0 * lox.m_per_s,
        vy=7500 * lox.m_per_s,
        vz=0 * lox.m_per_s,
    )
    assert state.x.to_kilometers() == pytest.approx(7000.0)
    assert state.y.to_kilometers() == pytest.approx(0.0)
    assert state.z.to_kilometers() == pytest.approx(0.0)
    assert state.vx.to_meters_per_second() == pytest.approx(0.0)
    assert state.vy.to_meters_per_second() == pytest.approx(7500.0)
    assert state.vz.to_meters_per_second() == pytest.approx(0.0)


def test_cartesian_component_getters():
    """Test that x, y, z, vx, vy, vz return unit types."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    state = lox.Cartesian(
        time,
        position=[7000e3, 1000e3, 500e3],
        velocity=[100.0, 7500.0, -200.0],
    )
    assert isinstance(state.x, lox.Distance)
    assert isinstance(state.y, lox.Distance)
    assert isinstance(state.z, lox.Distance)
    assert isinstance(state.vx, lox.Velocity)
    assert isinstance(state.vy, lox.Velocity)
    assert isinstance(state.vz, lox.Velocity)
    assert state.x.to_meters() == pytest.approx(7000e3)
    assert state.vy.to_meters_per_second() == pytest.approx(7500.0)


def test_cartesian_repr():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    state = lox.Cartesian(
        time,
        position=[7000e3, 0.0, 0.0],
        velocity=[0.0, 7500.0, 0.0],
    )
    r = repr(state)
    assert r.startswith("Cartesian(")
    assert "7000000.0" in r


def test_cartesian_string_origin_and_frame():
    """Test that string origin and frame work in Cartesian constructor."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    state = lox.Cartesian(
        time,
        position=[7000e3, 0.0, 0.0],
        velocity=[0.0, 7500.0, 0.0],
        origin="Earth",
        frame="ICRF",
    )
    assert state.origin().name() == "Earth"
    assert repr(state.reference_frame()) == 'Frame("ICRF")'


def test_cartesian_to_frame_string():
    """Test that to_frame accepts a string frame name."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    state = lox.Cartesian(
        time,
        position=[7000e3, 0.0, 0.0],
        velocity=[0.0, 7500.0, 0.0],
    )
    s2 = state.to_frame("ICRF")
    assert repr(s2.reference_frame()) == 'Frame("ICRF")'


def test_cartesian_to_origin_string(ephemeris):
    """Test that to_origin accepts a string origin name."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    state = lox.Cartesian(
        time,
        position=[7000e3, 0.0, 0.0],
        velocity=[0.0, 7500.0, 0.0],
    )
    s2 = state.to_origin("Moon", ephemeris)
    assert s2.origin().name() == "Moon"
