#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

import lox_space as lox
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