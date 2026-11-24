# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import numpy.testing as npt
import pytest

import lox_space as lox

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
    assert tle.mean_motion().to(lox.deg_per_s) == pytest.approx(
        15.49495945 * 360 / 86400
    )


def test_tle_metadata():
    tle = lox.TLE(ISS_TLE)
    assert tle.element_set_number() == 999
    assert tle.revolution_number() == 45873
    assert tle.ephemeris_type() == 0
    assert tle.half_mean_motion_dot_rev_per_day2() == pytest.approx(0.00016566)
    assert tle.sixth_mean_motion_ddot_rev_per_day3() == pytest.approx(0.0)
    assert tle.drag_term_per_earth_radius() == pytest.approx(0.30244e-3)


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


def celestrak_record(replacements=()):
    """The ISS as a single CelesTrak GP JSON object."""
    record = """{
        "OBJECT_NAME": "ISS (ZARYA)",
        "OBJECT_ID": "1998-067A",
        "EPOCH": "2024-06-18T09:00:24.494400",
        "MEAN_MOTION": 15.49495945,
        "ECCENTRICITY": 0.0010444,
        "INCLINATION": 51.641,
        "RA_OF_ASC_NODE": 309.389,
        "ARG_OF_PERICENTER": 339.5369,
        "MEAN_ANOMALY": 107.883,
        "EPHEMERIS_TYPE": 0,
        "CLASSIFICATION_TYPE": "U",
        "NORAD_CAT_ID": 25544,
        "ELEMENT_SET_NO": 999,
        "REV_AT_EPOCH": 45873,
        "BSTAR": 0.00030244,
        "MEAN_MOTION_DOT": 0.00016566,
        "MEAN_MOTION_DDOT": 0
    }"""
    for old, new in replacements:
        record = record.replace(old, new)
    return record


def assert_same_elements(actual, expected):
    assert actual.object_name() == expected.object_name()
    assert actual.international_designator() == expected.international_designator()
    assert actual.norad_id() == expected.norad_id()
    assert actual.classification() == expected.classification()
    assert actual.epoch() == expected.epoch()
    assert actual.eccentricity() == expected.eccentricity()
    assert actual.inclination() == expected.inclination()
    assert actual.right_ascension() == expected.right_ascension()
    assert actual.argument_of_perigee() == expected.argument_of_perigee()
    assert actual.mean_anomaly() == expected.mean_anomaly()
    assert actual.mean_motion() == expected.mean_motion()
    assert actual.drag_term_per_earth_radius() == expected.drag_term_per_earth_radius()
    assert actual.element_set_number() == expected.element_set_number()
    assert actual.revolution_number() == expected.revolution_number()


def spacetrack_record(replacements=()):
    """The ISS as a complete OMM, with the header that CelesTrak's JSON leaves out."""
    header = """{
        "CCSDS_OMM_VERS": "2.0",
        "CREATION_DATE": "2024-06-18T00:00:00",
        "ORIGINATOR": "18 SPCS",
        "CENTER_NAME": "EARTH",
        "REF_FRAME": "TEME",
        "TIME_SYSTEM": "UTC",
        "MEAN_ELEMENT_THEORY": "SGP4","""
    return celestrak_record(replacements).replace("{", header, 1)


def test_tle_from_json():
    tle = lox.TLE.from_json(celestrak_record())
    assert tle.object_name() == "ISS (ZARYA)"
    assert tle.international_designator() == "1998-067A"
    assert tle.norad_id() == 25544
    assert tle.classification() == "U"
    assert tle.element_set_number() == 999
    assert tle.revolution_number() == 45873
    npt.assert_allclose(tle.eccentricity(), 0.0010444)
    npt.assert_allclose(tle.inclination().to(lox.deg), 51.641)
    npt.assert_allclose(tle.drag_term_per_earth_radius(), 0.00030244)
    npt.assert_allclose(tle.half_mean_motion_dot_rev_per_day2(), 0.00016566)


@pytest.mark.parametrize(
    "tle",
    [
        pytest.param(lambda: lox.TLE.from_json(celestrak_record()), id="json"),
        pytest.param(lambda: lox.TLE.from_omm(spacetrack_record()), id="omm"),
    ],
)
def test_tle_formats_match_two_line_elements(tle):
    tle = tle()
    from_lines = lox.TLE(ISS_TLE)
    assert tle.norad_id() == from_lines.norad_id()
    npt.assert_allclose(tle.eccentricity(), from_lines.eccentricity())
    npt.assert_allclose(
        tle.mean_motion().to(lox.rad_per_s),
        from_lines.mean_motion().to(lox.rad_per_s),
    )
    assert abs((tle.epoch() - from_lines.epoch()).to_seconds()) < 1e-3


def test_tle_from_omm_kvn():
    kvn = """CCSDS_OMM_VERS = 3.0
CREATION_DATE = 2024-06-18T00:00:00
ORIGINATOR = TEST

OBJECT_NAME = ISS (ZARYA)
OBJECT_ID = 1998-067A
CENTER_NAME = EARTH
REF_FRAME = TEME
TIME_SYSTEM = UTC
MEAN_ELEMENT_THEORY = SGP/SGP4

EPOCH = 2024-06-18T09:00:24.214400
MEAN_MOTION = 15.49495945 [rev/day]
ECCENTRICITY = 0.0010444
INCLINATION = 51.641 [deg]
RA_OF_ASC_NODE = 309.389 [deg]
ARG_OF_PERICENTER = 339.5369 [deg]
MEAN_ANOMALY = 107.883 [deg]
EPHEMERIS_TYPE = 0
CLASSIFICATION_TYPE = U
NORAD_CAT_ID = 25544
ELEMENT_SET_NO = 999
REV_AT_EPOCH = 45873
BSTAR = 0.00030244 [1/ER]
MEAN_MOTION_DOT = 0.00016566 [rev/day**2]
MEAN_MOTION_DDOT = 0 [rev/day**3]
"""
    tle = lox.TLE.from_omm(kvn)
    assert tle.norad_id() == 25544
    assert tle.object_name() == "ISS (ZARYA)"


def test_tle_from_json_six_digit_catalog_number():
    # CelesTrak stopped publishing TLEs for new objects once 5-digit numbers ran out.
    tle = lox.TLE.from_json(
        celestrak_record([('"NORAD_CAT_ID": 25544', '"NORAD_CAT_ID": 100123')])
    )
    assert tle.norad_id() == 100123
    sgp4 = lox.SGP4(tle)
    assert sgp4.tle().norad_id() == 100123
    sgp4.propagate(tle.epoch() + lox.TimeDelta.from_minutes(10.0))


NEGATIVE_ECCENTRICITY = [("0.0010444", "-0.0010444")]


@pytest.mark.parametrize(
    ("constructor", "text", "match"),
    [
        ("from_json", "not json", None),
        ("from_json", "{}", None),
        ("from_json", celestrak_record(NEGATIVE_ECCENTRICITY), "eccentricity"),
        ("from_omm", "not an OMM", None),
        ("from_omm", spacetrack_record(NEGATIVE_ECCENTRICITY), "eccentricity"),
        ("from_json_list", "not json", None),
        ("from_json_list", "[{}]", None),
        ("from_json_list", celestrak_record(), None),
    ],
)
def test_tle_alternative_constructors_reject_invalid_input(constructor, text, match):
    with pytest.raises(ValueError, match=match):
        getattr(lox.TLE, constructor)(text)


def test_tle_from_json_rejects_a_download():
    with pytest.raises(ValueError):
        lox.TLE.from_json(f"[{celestrak_record()}]")


def test_tle_from_json_list_download(data_dir):
    tles = lox.TLE.from_json_list((data_dir / "oneweb.json").read_text())
    assert len(tles) == 651
    assert tles[0].object_name() == "ONEWEB-0012"
    assert tles[0].international_designator() == "2019-010A"
    assert tles[0].norad_id() == 44057
    npt.assert_allclose(tles[0].eccentricity(), 0.00015119)
    npt.assert_allclose(tles[0].drag_term_per_earth_radius(), -5.6171168e-5)
    for tle in tles:
        lox.SGP4(tle)


def test_tle_from_json_list_empty():
    assert lox.TLE.from_json_list("[]") == []


def test_tle_from_json_list_pickle(data_dir):
    import pickle

    tles = lox.TLE.from_json_list((data_dir / "oneweb.json").read_text())
    for tle in tles:
        assert_same_elements(pickle.loads(pickle.dumps(tle)), tle)


@pytest.mark.parametrize(
    ("tle", "prefix"),
    [
        pytest.param(lambda: lox.TLE(ISS_TLE), "TLE(", id="lines"),
        pytest.param(
            lambda: lox.TLE.from_json(celestrak_record()), "TLE.from_json(", id="json"
        ),
        pytest.param(
            lambda: lox.TLE.from_omm(spacetrack_record()), "TLE.from_omm(", id="omm"
        ),
    ],
)
def test_tle_rebuilds_from_pickle_and_repr(tle, prefix):
    import pickle

    tle = tle()
    roundtripped = pickle.loads(pickle.dumps(tle))
    assert_same_elements(roundtripped, tle)
    assert str(roundtripped) == str(tle)
    assert repr(tle).startswith(prefix)
    assert_same_elements(eval(repr(tle), {"TLE": lox.TLE}), tle)


def test_sgp4_from_tle_object():
    tle = lox.TLE(ISS_TLE)
    sgp4 = lox.SGP4(tle)
    t1 = sgp4.time() + lox.TimeDelta.from_minutes(92.821)
    s1 = sgp4.propagate(t1).to_frame(lox.Frame("ICRF"))
    k1 = s1.to_keplerian()
    assert k1.orbital_period().to_seconds() == pytest.approx(92.821 * 60, rel=1e-4)


@pytest.mark.parametrize("offsets", [[0, 0], [60, 0], [0]])
def test_propagate_invalid_time_list(offsets):
    sgp4 = lox.SGP4(ISS_TLE)
    steps = [sgp4.time() + lox.TimeDelta.from_seconds(dt) for dt in offsets]
    with pytest.raises(ValueError):
        sgp4.propagate(steps)


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
    assert k1.orbital_period().to_seconds() == pytest.approx(92.821 * 60, rel=1e-4)


def iss_state():
    """Create an ISS-like state in Earth orbit."""
    t = lox.Time("TAI", 2024, 1, 1)
    return lox.Cartesian(
        t,
        x=6678.0 * lox.km,
        y=0.0 * lox.km,
        z=0.0 * lox.km,
        vx=0.0 * lox.km_per_s,
        vy=7.73 * lox.km_per_s,
        vz=0.0 * lox.km_per_s,
    )


def test_numerical_single_time():
    state = iss_state()
    n = lox.Numerical(state)
    t1 = state.time() + 90 * lox.minutes
    result = n.propagate(t1)
    assert isinstance(result, lox.Cartesian)


def test_numerical_time_interval():
    state = iss_state()
    n = lox.Numerical(state)
    t0 = state.time()
    t1 = t0 + 1 * lox.hours
    trajectory = n.propagate(t0, end=t1)
    assert isinstance(trajectory, lox.Trajectory)


def test_numerical_multiple_times():
    state = iss_state()
    n = lox.Numerical(state)
    t0 = state.time() + 1 * lox.minutes
    t1 = t0 + 9 * lox.minutes
    times = lox.Interval(t0, t1).step_by(1 * lox.minutes)
    trajectory = n.propagate(times)
    assert isinstance(trajectory, lox.Trajectory)


def test_numerical_custom_tolerances():
    state = iss_state()
    n = lox.Numerical(state, rtol=1e-12, atol=1e-10)
    t1 = state.time() + 90 * lox.minutes
    result = n.propagate(t1)
    assert isinstance(result, lox.Cartesian)


def test_numerical_custom_step_size():
    state = iss_state()
    n = lox.Numerical(
        state, h_max=10 * lox.seconds, h_min=10 * lox.nanoseconds, max_steps=200_000
    )
    t1 = state.time() + 90 * lox.minutes
    result = n.propagate(t1)
    assert isinstance(result, lox.Cartesian)


def test_numerical_repr():
    state = iss_state()
    n = lox.Numerical(state)
    assert "Numerical(" in repr(n)


def test_j2_single_time():
    state = iss_state()
    j2 = lox.J2(state.to_keplerian())
    t1 = state.time() + 90 * lox.minutes
    result = j2.propagate(t1)
    assert isinstance(result, lox.Cartesian)


def test_j2_time_interval():
    state = iss_state()
    j2 = lox.J2(state.to_keplerian())
    t0 = state.time()
    t1 = t0 + 1 * lox.hours
    trajectory = j2.propagate(t0, end=t1)
    assert isinstance(trajectory, lox.Trajectory)


def test_j2_multiple_times():
    state = iss_state()
    j2 = lox.J2(state.to_keplerian())
    t0 = state.time() + 1 * lox.minutes
    t1 = t0 + 9 * lox.minutes
    times = lox.Interval(t0, t1).step_by(1 * lox.minutes)
    trajectory = j2.propagate(times)
    assert isinstance(trajectory, lox.Trajectory)


def test_j2_custom_step():
    state = iss_state()
    j2 = lox.J2(state.to_keplerian(), step=30 * lox.seconds)
    t0 = state.time()
    t1 = t0 + 10 * lox.minutes
    trajectory = j2.propagate(t0, end=t1)
    assert isinstance(trajectory, lox.Trajectory)


def test_j2_repr():
    state = iss_state()
    j2 = lox.J2(state.to_keplerian())
    assert "J2(" in repr(j2)


def test_ground(provider):
    tai = lox.UTC.from_iso("2022-01-31T23:00:00").to_scale("TAI")
    loc = lox.EllipsoidLocation(
        "IAU_EARTH", -4.3676 * lox.deg, 40.4527 * lox.deg, 0.0 * lox.km
    )
    ground = lox.GroundPropagator(loc)
    # GroundPropagator now returns body-fixed (IAU) states; transform to ICRF
    state = ground.propagate(tai).to_frame(lox.Frame("ICRF"))
    expected_km = [-1765.9535510583582, 4524.585984442561, 4120.189198495323]
    actual_km = state.position() * 1e-3
    npt.assert_allclose(actual_km, expected_km)


@pytest.mark.parametrize("propagator", [lox.J2, lox.J4])
def test_kozai_accepts_mean_keplerian_elements(propagator):
    time = lox.UTC(2024, 1, 1).to_scale("TAI")
    kep = lox.Keplerian(
        time,
        semi_major_axis=7000.0 * lox.km,
        eccentricity=0.001,
        inclination=51.6 * lox.deg,
        longitude_of_ascending_node=30.0 * lox.deg,
        argument_of_periapsis=60.0 * lox.deg,
        true_anomaly=90.0 * lox.deg,
    )
    from_keplerian = propagator(kep).propagate(time)
    npt.assert_allclose(
        from_keplerian.position(), kep.to_cartesian().position(), rtol=1e-10
    )


@pytest.mark.parametrize("propagator", [lox.J2, lox.J4])
def test_kozai_rejects_cartesian_states(propagator):
    time = lox.UTC(2024, 1, 1).to_scale("TAI")
    cart = lox.Cartesian(time, position=[7000e3, 0.0, 0.0], velocity=[0.0, 7500.0, 0.0])
    with pytest.raises(TypeError):
        propagator(cart)
