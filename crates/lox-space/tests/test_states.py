# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import math

import lox_space as lox
import numpy as np
import numpy.testing as npt
import pytest


def test_state_to_ellipsoid_location():
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
    ground = state.to_ellipsoid_location()
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


def test_exceptions_are_exported():
    assert issubclass(lox.UndefinedOriginPropertyError, Exception)
    assert issubclass(lox.FrameTransformationError, Exception)


def test_to_frame_raises_frame_transformation_error(provider):
    time = lox.UTC(2200, 1, 1).to_scale("TAI")
    state = lox.Cartesian(
        time,
        position=[7000e3, 0.0, 0.0],
        velocity=[0.0, 7500.0, 0.0],
        origin=lox.Origin("Earth"),
        frame=lox.Frame("ICRF"),
    )
    with pytest.raises(lox.FrameTransformationError, match="extrapolated"):
        state.to_frame(lox.Frame("ITRF"), provider)


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


def test_keplerian_derived_quantities():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    a = 7178.0
    e = 0.01
    k = lox.Keplerian(
        time,
        a * lox.km,
        e,
        inclination=97.0 * lox.deg,
        longitude_of_ascending_node=10.0 * lox.deg,
        argument_of_periapsis=20.0 * lox.deg,
        true_anomaly=30.0 * lox.deg,
    )
    earth = lox.Origin("Earth")
    mu = earth.gravitational_parameter().to_km3_per_s2()
    r_mean = earth.mean_radius().to_kilometers()

    assert k.semi_parameter().to_kilometers() == pytest.approx(a * (1 - e**2))
    assert k.periapsis_radius().to_kilometers() == pytest.approx(a * (1 - e))
    assert k.apoapsis_radius().to_kilometers() == pytest.approx(a * (1 + e))
    assert k.periapsis_altitude().to_kilometers() == pytest.approx(a * (1 - e) - r_mean)
    assert k.apoapsis_altitude().to_kilometers() == pytest.approx(a * (1 + e) - r_mean)
    assert k.argument_of_latitude().to_degrees() == pytest.approx(50.0)
    assert k.true_longitude().to_degrees() == pytest.approx(60.0)
    assert 0.0 <= float(k.mean_longitude()) < 2 * math.pi
    assert float(k.mean_motion()) == pytest.approx(math.sqrt(mu / a**3))
    assert k.specific_orbital_energy() == pytest.approx(-mu / (2 * a))
    assert k.specific_angular_momentum() == pytest.approx(math.sqrt(mu * a * (1 - e**2)))

    # Consistency with the Cartesian state
    state = k.to_cartesian()
    r = state.position() * 1e-3
    v = state.velocity() * 1e-3
    r_norm = float(np.linalg.norm(r))
    v_norm = float(np.linalg.norm(v))
    assert float(k.flight_path_angle()) == pytest.approx(
        math.asin(float(np.dot(r, v)) / (r_norm * v_norm))
    )
    assert k.specific_orbital_energy() == pytest.approx(v_norm**2 / 2 - mu / r_norm)
    assert k.specific_angular_momentum() == pytest.approx(
        float(np.linalg.norm(np.cross(r, v)))
    )


def test_keplerian_derived_quantities_open_orbit():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(time, -20000.0 * lox.km, 1.5)
    assert k.apoapsis_radius() is None
    assert k.apoapsis_altitude() is None
    assert k.periapsis_radius().to_kilometers() == pytest.approx(10000.0)
    assert k.specific_orbital_energy() > 0.0
    assert k.semi_major_axis().to_kilometers() == pytest.approx(-20000.0)


def test_keplerian_parabolic_has_no_semi_major_axis():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    mu = lox.Origin("Earth").gravitational_parameter().to_km3_per_s2()
    r = 7000.0
    v_escape = math.sqrt(2.0 * mu / r)
    state = lox.Cartesian(
        time,
        x=r * lox.km,
        y=0.0 * lox.km,
        z=0.0 * lox.km,
        vx=0.0 * lox.km_per_s,
        vy=v_escape * lox.km_per_s,
        vz=0.0 * lox.km_per_s,
    )
    k = state.to_keplerian()
    assert k.eccentricity() == pytest.approx(1.0)
    assert k.semi_major_axis() is None
    assert k.periapsis_radius().to_kilometers() == pytest.approx(r)
    assert "Keplerian(" in repr(k)


def test_cartesian_to_keplerian_rejects_zero_angular_momentum():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    radial = lox.Cartesian(
        time,
        x=7000.0 * lox.km,
        y=0.0 * lox.km,
        z=0.0 * lox.km,
        vx=1.0 * lox.km_per_s,
        vy=0.0 * lox.km_per_s,
        vz=0.0 * lox.km_per_s,
    )
    with pytest.raises(ValueError, match="zero angular momentum"):
        radial.to_keplerian()


def test_keplerian_altitude_requires_mean_radius():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(
        time, 7000.0 * lox.km, 0.01, origin=lox.Origin("Earth Barycenter")
    )
    with pytest.raises(lox.UndefinedOriginPropertyError, match="mean radius"):
        k.periapsis_altitude()
    with pytest.raises(lox.UndefinedOriginPropertyError, match="mean radius"):
        k.apoapsis_altitude()


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


def test_keplerian_semi_parameter():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(time, semi_parameter=8000.0 * lox.km, eccentricity=0.1)
    assert k.semi_parameter().to_kilometers() == pytest.approx(8000.0)
    assert k.semi_major_axis().to_kilometers() == pytest.approx(8000.0 / (1.0 - 0.1**2))


def test_keplerian_semi_parameter_parabolic():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    k = lox.Keplerian(
        time,
        semi_parameter=14000.0 * lox.km,
        eccentricity=1.0,
        true_anomaly=90.0 * lox.deg,
    )
    assert k.semi_major_axis() is None
    assert k.periapsis_radius().to_kilometers() == pytest.approx(7000.0)
    assert k.specific_orbital_energy() == pytest.approx(0.0, abs=1e-12)


def test_keplerian_semi_parameter_requires_eccentricity():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    with pytest.raises(ValueError, match="orbital shape"):
        lox.Keplerian(time, semi_parameter=7000.0 * lox.km)


def test_keplerian_semi_parameter_and_semi_major_axis_raises():
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    with pytest.raises(ValueError, match="exactly one"):
        lox.Keplerian(
            time,
            7000.0 * lox.km,
            0.1,
            semi_parameter=7000.0 * lox.km,
        )


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


# --- ModifiedEquinoctial tests ---


def test_modified_equinoctial_roundtrip():
    """Test ModifiedEquinoctial construction and properties."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    # Circular equatorial orbit (~621.86 km altitude)
    mee = lox.ModifiedEquinoctial(
        time,
        p=7000.0 * lox.km,
        f=0.0,
        g=0.0,
        h=0.0,
        k=0.0,
        l=90.0 * lox.deg,
        origin="Earth",
        frame="ICRF",
    )
    
    assert mee.p().to_kilometers() == pytest.approx(7000.0)
    assert mee.f() == pytest.approx(0.0)
    assert mee.g() == pytest.approx(0.0)
    assert mee.h() == pytest.approx(0.0)
    assert mee.k() == pytest.approx(0.0)
    assert mee.l().to_degrees() == pytest.approx(90.0)
    assert mee.eccentricity() == pytest.approx(0.0)
    assert mee.inclination().to_degrees() == pytest.approx(0.0)
    assert mee.origin().name() == "Earth"
    assert repr(mee.frame()) == 'Frame("ICRF")'
    assert repr(mee).startswith("ModifiedEquinoctial(")
    assert str(mee) == repr(mee)

    # Convert to Cartesian
    cart = mee.to_cartesian()
    # For circular equatorial orbit, position should be at y axis, velocity at -x axis
    pos = cart.position()
    vel = cart.velocity()
    npt.assert_allclose(pos, [0.0, 7000e3, 0.0], atol=1e-5)

    # Convert back to MEE
    mee2 = cart.to_modified_equinoctial()
    assert mee2.p().to_kilometers() == pytest.approx(mee.p().to_kilometers())
    assert mee2.f() == pytest.approx(mee.f())
    assert mee2.g() == pytest.approx(mee.g())
    assert mee2.h() == pytest.approx(mee.h())
    assert mee2.k() == pytest.approx(mee.k())
    assert mee2.l().to_degrees() == pytest.approx(mee.l().to_degrees())

def test_modified_equinoctial_to_keplerian():
    """Test conversion to Keplerian elements."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    mee = lox.ModifiedEquinoctial(
        time,
        p=7178.0 * lox.km,
        f=0.001,
        g=0.0,
        h=0.0,
        k=0.0,
        l=0.0 * lox.deg,
    )
    kep = mee.to_keplerian()
    
    # sma = p / (1 - e^2)
    e = mee.eccentricity()
    sma = mee.p().to_meters() / (1.0 - e**2)
    assert kep.semi_major_axis().to_meters() == pytest.approx(sma)
    assert kep.eccentricity() == pytest.approx(0.001)
    
    # And convert back
    mee_back = kep.to_modified_equinoctial()
    assert mee_back.p().to_kilometers() == pytest.approx(mee.p().to_kilometers())
    assert mee_back.f() == pytest.approx(mee.f())
    assert mee_back.g() == pytest.approx(mee.g())

def test_modified_equinoctial_errors():
    """Test error handling in ModifiedEquinoctial conversions."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    
    # Origin without gravitational parameter
    mee_no_mu = lox.ModifiedEquinoctial(
        time,
        p=7000.0 * lox.km,
        f=0.0,
        g=0.0,
        h=0.0,
        k=0.0,
        l=0.0 * lox.deg,
        origin="Itokawa",
    )
    with pytest.raises(
        lox.UndefinedOriginPropertyError, match="undefined property 'gravitational parameter'"
    ):
        mee_no_mu.to_cartesian()
        
    # Cartesian -> MEE with Itokawa origin
    cart = lox.Cartesian(
        time,
        position=[7000e3, 0.0, 0.0],
        velocity=[0.0, 7000.0, 0.0],
        origin="Itokawa"
    )
    with pytest.raises(
        lox.UndefinedOriginPropertyError, match="undefined property 'gravitational parameter'"
    ):
        cart.to_modified_equinoctial()

    # A negative semi-parameter is rejected on construction
    with pytest.raises(ValueError, match="semi-parameter must be finite and positive"):
        lox.ModifiedEquinoctial(
            time,
            p=-7000.0 * lox.km,
            f=0.5,
            g=0.0,
            h=0.0,
            k=0.0,
            l=0.0 * lox.deg,
        )


def test_modified_equinoctial_retrograde():
    """Retrograde orbits use the retrograde set of elements and survive a roundtrip."""
    time = lox.UTC(2024, 1, 1).to_scale("TDB")
    kep = lox.Keplerian(
        time,
        semi_major_axis=7000.0 * lox.km,
        eccentricity=0.01,
        inclination=180.0 * lox.deg,
        longitude_of_ascending_node=0.0 * lox.deg,
        argument_of_periapsis=30.0 * lox.deg,
        true_anomaly=45.0 * lox.deg,
    )
    mee = kep.to_modified_equinoctial()
    assert mee.retrograde()
    assert mee.h() == pytest.approx(0.0, abs=1e-15)
    assert mee.k() == pytest.approx(0.0, abs=1e-15)
    assert mee.inclination().to_degrees() == pytest.approx(180.0)
    assert "retrograde=True" in repr(mee)

    rebuilt = lox.ModifiedEquinoctial(
        time,
        p=mee.p(),
        f=mee.f(),
        g=mee.g(),
        h=mee.h(),
        k=mee.k(),
        l=mee.l(),
        retrograde=True,
    )
    npt.assert_allclose(
        rebuilt.to_cartesian().position(), kep.to_cartesian().position(), atol=1e-6
    )
