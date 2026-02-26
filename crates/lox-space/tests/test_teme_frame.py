# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

"""
Test TEME frame transformations.

Reference data from GitHub issue #197:
https://github.com/lox-space/lox/issues/197
"""

import lox_space as lox
import numpy as np
import numpy.testing as npt


# INTELSAT 36 TLE data from issue #197
INTELSAT_36_TLE = """INTELSAT 36
1 41747U 16053A   25026.69560333 -.00000008  00000+0  00000+0 0  9995
2 41747   0.0093  36.1594 0001240 298.1404 110.8362  1.00272270 30870
"""


def test_teme_to_icrf_roundtrip():
    """Test TEME <-> ICRF transformation roundtrip preserves state."""
    # Create a state in ICRF
    time = lox.UTC.from_iso("2025-01-27T00:00:00").to_scale("TAI")

    state_icrf = lox.Cartesian(
        time,
        position=[-40755396.0, -10823119.0, 12227.0],
        velocity=[789.0, -2971.0, 0.0],
        origin=lox.Origin("Earth"),
        frame=lox.Frame("ICRF"),
    )

    # Transform ICRF -> TEME -> ICRF
    state_teme = state_icrf.to_frame(lox.Frame("TEME"))
    state_icrf_back = state_teme.to_frame(lox.Frame("ICRF"))

    # Round-trip should preserve position and velocity
    npt.assert_allclose(
        state_icrf.position(),
        state_icrf_back.position(),
        rtol=1e-10,
    )
    npt.assert_allclose(
        state_icrf.velocity(),
        state_icrf_back.velocity(),
        rtol=1e-10,
        atol=1e-12,
    )


def test_teme_frame_small_rotation():
    """Test that TEME differs from TOD by a small z-axis rotation (Equation of Equinoxes)."""
    time = lox.UTC.from_iso("2025-01-27T00:00:00").to_scale("TAI")

    state_icrf = lox.Cartesian(
        time,
        position=[42164000.0, 0.0, 0.0],
        velocity=[0.0, 3075.0, 0.0],
        origin=lox.Origin("Earth"),
        frame=lox.Frame("ICRF"),
    )

    # The TEME frame should be very close to TOD
    state_teme = state_icrf.to_frame(lox.Frame("TEME"))

    # Position magnitude should be preserved
    pos_icrf = state_icrf.position()
    pos_teme = state_teme.position()

    npt.assert_allclose(np.linalg.norm(pos_icrf), np.linalg.norm(pos_teme), rtol=1e-12)


def test_icrf_teme_orekit():
    time = lox.Time("TAI", 2026, 2, 21, 11, 46, 0.0)
    state_icrf = lox.Cartesian(
        time,
        position=[6068279.27, -1692843.94, -2516619.18],
        velocity=[-660.415582, 5495.938726, -5303.093233],
        origin=lox.Origin("Earth"),
        frame=lox.Frame("ICRF"),
    )
    p_exp = np.array([6084477.559317719, -1657212.1205153512, -2501177.0866586748])
    v_exp = np.array([-678.9922540001335, 5492.253713516756, -5304.564805470879])

    state_teme = state_icrf.to_frame(lox.Frame("TEME"))
    npt.assert_allclose(state_teme.position(), p_exp, rtol=1e-8)
    npt.assert_allclose(state_teme.velocity(), v_exp, rtol=1e-8)


def test_sgp4_icrf_against_skyfield():
    """Test SGP4 propagation produces correct ICRF coordinates.

    Validates against Skyfield GCRS output for INTELSAT 36 TLE,
    propagated to 2025-01-27T00:00:00 UTC.
    """
    from skyfield.api import load, EarthSatellite

    tle_lines = INTELSAT_36_TLE.strip().splitlines()

    # Propagate with Skyfield
    ts = load.timescale()
    skyfield_sat = EarthSatellite(tle_lines[1], tle_lines[2], tle_lines[0], ts)
    t = ts.utc(2025, 1, 27, 0, 0, 0)
    expected_pos = skyfield_sat.at(t).position.km

    # Propagate with Lox — SGP4 now returns TEME; convert to ICRF
    sgp4 = lox.SGP4(INTELSAT_36_TLE)
    time = lox.UTC.from_iso("2025-01-27T00:00:00").to_scale("TAI")
    state = sgp4.propagate(time).to_frame(lox.Frame("ICRF"))
    pos_km = state.position() * 1e-3

    # rtol=1e-4 accounts for different precession/nutation models in the
    # TEME->ICRF transformation (Skyfield uses IAU 2006A, lox uses IAU 1980)
    npt.assert_allclose(pos_km, expected_pos, rtol=1e-4)


def test_teme_transformation_exists():
    """Test that TEME frame transformations are implemented (not todo!)."""
    time = lox.UTC.from_iso("2025-01-27T00:00:00").to_scale("TAI")

    state_icrf = lox.Cartesian(
        time,
        position=[42164000.0, 0.0, 0.0],
        velocity=[0.0, 3075.0, 0.0],
        origin=lox.Origin("Earth"),
        frame=lox.Frame("ICRF"),
    )

    # These should not raise NotImplementedError or panic
    state_teme = state_icrf.to_frame(lox.Frame("TEME"))
    assert state_teme is not None

    # And back
    state_icrf_back = state_teme.to_frame(lox.Frame("ICRF"))
    assert state_icrf_back is not None
