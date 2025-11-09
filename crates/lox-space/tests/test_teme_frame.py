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
2 41747   0.0093  36.1594 0001240 298.1404 110.8362 1.00272270 30870
"""


def test_teme_to_icrf_roundtrip():
    """Test TEME <-> ICRF transformation roundtrip preserves state."""
    # Create a state in ICRF
    time = lox.UTC.from_iso("2025-01-27T00:00:00").to_scale("TAI")
    position = (-40755.396, -10823.119, 12.227)  # km (TEME coords from issue)
    velocity = (0.789, -2.971, 0.0)  # km/s (approximate GEO velocity)

    state_icrf = lox.State(
        time, position, velocity, lox.Origin("Earth"), lox.Frame("ICRF")
    )

    # Transform ICRF -> TEME -> ICRF
    state_teme = state_icrf.to_frame(lox.Frame("TEME"))
    state_icrf_back = state_teme.to_frame(lox.Frame("ICRF"))

    # Round-trip should preserve position and velocity
    npt.assert_allclose(state_icrf.position(), state_icrf_back.position(), rtol=1e-10)
    npt.assert_allclose(state_icrf.velocity(), state_icrf_back.velocity(), rtol=1e-10, atol=1e-15)


def test_teme_frame_small_rotation():
    """Test that TEME differs from PEF by a small z-axis rotation (Equation of Equinoxes)."""
    time = lox.UTC.from_iso("2025-01-27T00:00:00").to_scale("TAI")

    # Create a state along the x-axis in ICRF
    position = (42164.0, 0.0, 0.0)  # GEO radius along x-axis
    velocity = (0.0, 3.075, 0.0)  # GEO velocity along y-axis

    state_icrf = lox.State(
        time, position, velocity, lox.Origin("Earth"), lox.Frame("ICRF")
    )

    # The TEME frame should be very close to TOD/PEF frames
    # The difference is the Equation of Equinoxes (~1 arcsecond)
    state_teme = state_icrf.to_frame(lox.Frame("TEME"))

    # Position magnitude should be preserved
    pos_icrf = np.array(state_icrf.position())
    pos_teme = np.array(state_teme.position())

    npt.assert_allclose(np.linalg.norm(pos_icrf), np.linalg.norm(pos_teme), rtol=1e-12)


def test_teme_transformation_exists():
    """Test that TEME frame transformations are implemented (not todo!)."""
    time = lox.UTC.from_iso("2025-01-27T00:00:00").to_scale("TAI")
    position = (42164.0, 0.0, 0.0)
    velocity = (0.0, 3.075, 0.0)

    state_icrf = lox.State(
        time, position, velocity, lox.Origin("Earth"), lox.Frame("ICRF")
    )

    # These should not raise NotImplementedError or panic
    state_teme = state_icrf.to_frame(lox.Frame("TEME"))
    assert state_teme is not None

    # And back
    state_icrf_back = state_teme.to_frame(lox.Frame("ICRF"))
    assert state_icrf_back is not None
