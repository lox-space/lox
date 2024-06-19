#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

import lox_space as lox
import numpy as np
import numpy.testing as npt
import pytest

ISS_TLE = """ISS (ZARYA)
1 25544U 98067A   24170.37528350  .00016566  00000+0  30244-3 0  9996
2 25544  51.6410 309.3890 0010444 339.5369 107.8830 15.49495945458731
"""


def test_sgp4():
    sgp4 = lox.SGP4(ISS_TLE)
    t1 = sgp4.time() + lox.TimeDelta.from_minutes(92.821)
    s1 = sgp4.propagate(t1)
    k1 = s1.to_keplerian()
    assert k1.orbital_period().to_decimal_seconds() == pytest.approx(92.821 * 60, rel=1e-4)


def test_ground(provider):
    lat = np.radians(40.4527)
    lon = np.radians(-4.3676)
    tai = lox.UTC.from_iso("2022-01-31T23:00:00").to_tai()
    loc = lox.GroundLocation(lox.Planet("Earth"), lon, lat, 0.0)
    ground = lox.GroundPropagator(loc, provider)
    expected = np.array([-1765.9535510583582, 4524.585984442561, 4120.189198495323])
    actual = ground.propagate(tai).position()
    npt.assert_allclose(actual, expected, rtol=1e-4)
