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


# --- Keplerian constructor tests ---


def test_keplerian_positional_backward_compat():
    """Test that existing positional usage still works."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(
        time,
        7178.0 * lox.km,
        0.001,
        97.0 * lox.deg,
        0.0 * lox.deg,
        0.0 * lox.deg,
        0.0 * lox.deg,
    )
    assert k.semi_major_axis().to_kilometers() == pytest.approx(7178.0, rel=1e-10)
    assert k.eccentricity() == pytest.approx(0.001, abs=1e-15)


def test_keplerian_keyword_backward_compat():
    """Test that existing keyword usage still works."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(
        time,
        semi_major_axis=7178.0 * lox.km,
        eccentricity=0.001,
        inclination=97.0 * lox.deg,
        longitude_of_ascending_node=0.0 * lox.deg,
        argument_of_periapsis=0.0 * lox.deg,
        true_anomaly=0.0 * lox.deg,
    )
    assert k.semi_major_axis().to_kilometers() == pytest.approx(7178.0, rel=1e-10)
    assert k.eccentricity() == pytest.approx(0.001, abs=1e-15)
    assert k.inclination().to_degrees() == pytest.approx(97.0, rel=1e-10)


def test_keplerian_radii():
    """Test Keplerian construction from periapsis/apoapsis radii."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(
        time,
        periapsis_radius=7000.0 * lox.km,
        apoapsis_radius=7400.0 * lox.km,
    )
    assert k.semi_major_axis().to_kilometers() == pytest.approx(7200.0, rel=1e-10)
    exp_ecc = (7400.0 - 7000.0) / (7400.0 + 7000.0)
    assert k.eccentricity() == pytest.approx(exp_ecc, rel=1e-10)


def test_keplerian_altitudes():
    """Test Keplerian construction from periapsis/apoapsis altitudes."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(
        time,
        periapsis_altitude=600.0 * lox.km,
        apoapsis_altitude=1000.0 * lox.km,
    )
    # Mean radius of Earth ~6371 km
    sma_km = k.semi_major_axis().to_kilometers()
    assert sma_km > 6971.0
    assert sma_km < 7371.0
    assert k.eccentricity() > 0.0


def test_keplerian_defaults():
    """Test that angular elements default to 0."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(
        time,
        periapsis_radius=7000.0 * lox.km,
        apoapsis_radius=7000.0 * lox.km,
    )
    assert float(k.inclination()) == pytest.approx(0.0, abs=1e-15)
    assert float(k.longitude_of_ascending_node()) == pytest.approx(0.0, abs=1e-15)
    assert float(k.argument_of_periapsis()) == pytest.approx(0.0, abs=1e-15)
    assert float(k.true_anomaly()) == pytest.approx(0.0, abs=1e-15)


def test_keplerian_mean_anomaly():
    """Test Keplerian construction with mean_anomaly keyword."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(
        time,
        7178.0 * lox.km,
        0.001,
        mean_anomaly=90.0 * lox.deg,
    )
    # mean anomaly of 90 deg should produce a non-zero true anomaly
    assert float(k.true_anomaly()) != pytest.approx(0.0, abs=1e-3)


def test_keplerian_both_anomalies_raises():
    """Test that providing both true_anomaly and mean_anomaly raises."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    with pytest.raises(ValueError, match="true anomaly.*mean anomaly"):
        lox.Keplerian(
            time,
            7178.0 * lox.km,
            0.001,
            true_anomaly=0.0 * lox.deg,
            mean_anomaly=0.0 * lox.deg,
        )


def test_keplerian_no_shape_raises():
    """Test that omitting shape params raises ValueError."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    with pytest.raises(ValueError, match="orbital shape"):
        lox.Keplerian(time)


def test_keplerian_mixed_shape_raises():
    """Test that mixing shape params raises ValueError."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    with pytest.raises(ValueError, match="exactly one"):
        lox.Keplerian(
            time,
            7000.0 * lox.km,
            0.0,
            periapsis_radius=7000.0 * lox.km,
            apoapsis_radius=7000.0 * lox.km,
        )


def test_keplerian_partial_radii_raises():
    """Test that providing only one radius raises ValueError."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    with pytest.raises(ValueError, match="periapsis_radius"):
        lox.Keplerian(time, apoapsis_radius=7000.0 * lox.km)


# --- Keplerian.circular() tests ---


def test_circular_from_sma():
    """Test circular orbit from semi-major axis."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian.circular(time, semi_major_axis=7178.0 * lox.km)
    assert k.semi_major_axis().to_kilometers() == pytest.approx(7178.0, rel=1e-10)
    assert k.eccentricity() == pytest.approx(0.0, abs=1e-15)


def test_circular_from_altitude():
    """Test circular orbit from altitude."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian.circular(time, altitude=800.0 * lox.km)
    # sma should be altitude + mean radius
    sma_km = k.semi_major_axis().to_kilometers()
    assert sma_km > 7100.0
    assert sma_km < 7200.0
    assert k.eccentricity() == pytest.approx(0.0, abs=1e-15)


def test_circular_with_inclination():
    """Test circular orbit with inclination."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian.circular(
        time,
        semi_major_axis=7178.0 * lox.km,
        inclination=97.0 * lox.deg,
    )
    assert k.inclination().to_degrees() == pytest.approx(97.0, rel=1e-10)


def test_circular_no_size_raises():
    """Test that omitting size raises ValueError."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    with pytest.raises(ValueError, match="exactly one"):
        lox.Keplerian.circular(time)


def test_circular_both_size_raises():
    """Test that providing both sma and altitude raises ValueError."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    with pytest.raises(ValueError, match="exactly one"):
        lox.Keplerian.circular(
            time,
            semi_major_axis=7178.0 * lox.km,
            altitude=800.0 * lox.km,
        )
