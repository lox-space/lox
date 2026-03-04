# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox
import numpy.testing as npt
import pytest

ISS_TLE = """ISS (ZARYA)
1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996
2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731
"""


ISS_TLE_LINES = [
    "ISS (ZARYA)",
    "1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996",
    "2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731",
]


def test_tle_from_string():
    tle = lox.TLE(ISS_TLE)
    assert tle.object_name() == "ISS (ZARYA)"
    assert tle.international_designator() == "1998-067A"
    assert tle.norad_id() == 25544
    assert tle.classification() == "U"


def test_tle_two_line():
    two_line = "\n".join(ISS_TLE_LINES[1:])
    tle = lox.TLE(two_line)
    assert tle.object_name() is None
    assert tle.norad_id() == 25544


def test_tle_from_list():
    tle = lox.TLE(ISS_TLE_LINES)
    assert tle.object_name() == "ISS (ZARYA)"
    assert tle.norad_id() == 25544


def test_tle_epoch():
    tle = lox.TLE(ISS_TLE)
    epoch = tle.epoch()
    assert isinstance(epoch, lox.Time)
    # 2024, day 170.37528350 → June 18, 2024
    assert "2024-06-18" in str(epoch)


def test_tle_orbital_elements():
    tle = lox.TLE(ISS_TLE)
    assert tle.inclination().to_degrees() == pytest.approx(51.6410)
    assert tle.right_ascension().to_degrees() == pytest.approx(309.3890)
    assert tle.eccentricity() == pytest.approx(0.0010444)
    assert tle.argument_of_perigee().to_degrees() == pytest.approx(339.5369)
    assert tle.mean_anomaly().to_degrees() == pytest.approx(107.8830)
    assert tle.mean_motion() == pytest.approx(15.49495945)


def test_tle_metadata():
    tle = lox.TLE(ISS_TLE)
    assert tle.element_set_number() == 999
    assert tle.revolution_number() == 45873
    assert tle.ephemeris_type() == 0
    assert tle.mean_motion_dot() == pytest.approx(0.00016566)
    assert tle.mean_motion_ddot() == pytest.approx(0.0)
    assert tle.drag_term() == pytest.approx(0.30244e-3)


def test_tle_repr_and_str():
    tle = lox.TLE(ISS_TLE)
    assert "TLE(" in repr(tle)
    assert "25544" in str(tle)


def test_tle_pickle():
    import pickle

    tle = lox.TLE(ISS_TLE)
    roundtripped = pickle.loads(pickle.dumps(tle))
    assert roundtripped.norad_id() == tle.norad_id()
    assert str(roundtripped) == str(tle)


def test_tle_invalid():
    with pytest.raises(ValueError):
        lox.TLE("not a valid TLE")
    with pytest.raises((ValueError, TypeError)):
        lox.TLE(42)


def test_sgp4_from_tle_object():
    tle = lox.TLE(ISS_TLE)
    sgp4 = lox.SGP4(tle)
    t1 = sgp4.time() + lox.TimeDelta.from_minutes(92.821)
    s1 = sgp4.propagate(t1).to_frame(lox.Frame("ICRF"))
    k1 = s1.to_keplerian()
    assert k1.orbital_period().to_decimal_seconds() == pytest.approx(
        92.821 * 60, rel=1e-4
    )


def test_sgp4_from_list():
    sgp4 = lox.SGP4(ISS_TLE_LINES)
    assert isinstance(sgp4.time(), lox.Time)


def test_sgp4_tle_accessor():
    sgp4 = lox.SGP4(ISS_TLE)
    tle = sgp4.tle()
    assert isinstance(tle, lox.TLE)
    assert tle.norad_id() == 25544


def test_sgp4_repr():
    sgp4 = lox.SGP4(ISS_TLE)
    assert "SGP4(" in repr(sgp4)
    assert "25544" in repr(sgp4)


def test_sgp4():
    sgp4 = lox.SGP4(ISS_TLE)
    t1 = sgp4.time() + lox.TimeDelta.from_minutes(92.821)
    # SGP4 now returns TEME states; convert to ICRF for Keplerian conversion
    s1 = sgp4.propagate(t1).to_frame(lox.Frame("ICRF"))
    k1 = s1.to_keplerian()
    assert k1.orbital_period().to_decimal_seconds() == pytest.approx(
        92.821 * 60, rel=1e-4
    )


def iss_state():
    """Create an ISS-like state in Earth orbit."""
    t = lox.Time("TAI", 2024, 1, 1)
    return lox.Cartesian(
        t,
        position=(6678.0 * lox.km, 0.0 * lox.km, 0.0 * lox.km),
        velocity=(0.0 * lox.km_per_s, 7.73 * lox.km_per_s, 0.0 * lox.km_per_s),
    )


def test_j2_single_time():
    state = iss_state()
    j2 = lox.J2(state)
    t1 = state.time() + 90 * lox.minutes
    result = j2.propagate(t1)
    assert isinstance(result, lox.Cartesian)


def test_j2_time_interval():
    state = iss_state()
    j2 = lox.J2(state)
    t0 = state.time()
    t1 = t0 + 1 * lox.hours
    trajectory = j2.propagate(t0, end=t1)
    assert isinstance(trajectory, lox.Trajectory)


def test_j2_multiple_times():
    state = iss_state()
    j2 = lox.J2(state)
    t0 = state.time() + 1 * lox.minutes
    t1 = t0 + 9 * lox.minutes
    times = lox.Interval(t0, t1).step_by(1 * lox.minutes)
    trajectory = j2.propagate(times)
    assert isinstance(trajectory, lox.Trajectory)


def test_j2_custom_tolerances():
    state = iss_state()
    j2 = lox.J2(state, rtol=1e-12, atol=1e-10)
    t1 = state.time() + 90 * lox.minutes
    result = j2.propagate(t1)
    assert isinstance(result, lox.Cartesian)


def test_j2_custom_step_size():
    state = iss_state()
    j2 = lox.J2(state, h_max=10.0, h_min=1e-8, max_steps=200_000)
    t1 = state.time() + 90 * lox.minutes
    result = j2.propagate(t1)
    assert isinstance(result, lox.Cartesian)


def test_j2_repr():
    state = iss_state()
    j2 = lox.J2(state)
    assert "J2(" in repr(j2)


def test_ground(provider):
    tai = lox.UTC.from_iso("2022-01-31T23:00:00").to_scale("TAI")
    loc = lox.GroundLocation(
        lox.Origin("Earth"), -4.3676 * lox.deg, 40.4527 * lox.deg, 0.0 * lox.km
    )
    ground = lox.GroundPropagator(loc)
    # GroundPropagator now returns body-fixed (IAU) states; transform to ICRF
    state = ground.propagate(tai).to_frame(lox.Frame("ICRF"))
    expected_km = [-1765.9535510583582, 4524.585984442561, 4120.189198495323]
    actual_km = state.position() * 1e-3
    npt.assert_allclose(actual_km, expected_km)
