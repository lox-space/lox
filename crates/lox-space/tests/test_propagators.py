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


def test_sgp4():
    sgp4 = lox.SGP4(ISS_TLE)
    t1 = sgp4.time() + lox.TimeDelta.from_minutes(92.821)
    # SGP4 now returns TEME states; convert to ICRF for Keplerian conversion
    s1 = sgp4.propagate(t1).to_frame(lox.Frame("ICRF"))
    k1 = s1.to_keplerian()
    assert k1.orbital_period().to_decimal_seconds() == pytest.approx(
        92.821 * 60, rel=1e-4
    )


def test_ground(provider):
    tai = lox.UTC.from_iso("2022-01-31T23:00:00").to_scale("TAI")
    loc = lox.GroundLocation(lox.Origin("Earth"), -4.3676 * lox.deg, 40.4527 * lox.deg, 0.0 * lox.km)
    ground = lox.GroundPropagator(loc)
    # GroundPropagator now returns body-fixed (IAU) states; transform to ICRF
    state = ground.propagate(tai).to_frame(lox.Frame("ICRF"))
    expected_km = [-1765.9535510583582, 4524.585984442561, 4120.189198495323]
    actual_km = state.position() * 1e-3
    npt.assert_allclose(actual_km, expected_km)
