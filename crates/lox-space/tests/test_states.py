#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

import lox_space as lox
import numpy as np
import numpy.testing as npt
import pytest
from pathlib import Path


@pytest.fixture
def ephemeris():
    spk = (
        Path(__file__).parent.joinpath("..", "..", "..", "data", "de440s.bsp").resolve()
    )
    return lox.SPK(str(spk))


def test_state_to_ground_location():
    time = lox.Time.from_iso("2024-07-05T09:09:18.173 TAI")
    position = (-5530.01774359, -3487.0895338, -1850.03476185)
    velocity = (1.29534407, -5.02456882, 5.6391936)
    state = lox.State(time, position, velocity, lox.Planet("Earth"), lox.Frame("ICRF"))
    ground = state.to_ground_location()
    assert ground.longitude() == pytest.approx(2.646276127963636)
    assert ground.latitude() == pytest.approx(-0.2794495715104036)
    assert ground.altitude() == pytest.approx(417.8524158044338)


def test_state_to_origin(ephemeris):
    r_venus = np.array(
        [
            1.001977553295792e8,
            2.200234656010247e8,
            9.391473630346918e7,
        ]
    )
    v_venus = np.array([-59.08617935009049, 22.682387107225292, 12.05029567478702])
    r = np.array([6068279.27, -1692843.94, -2516619.18]) / 1e3

    v = np.array([-660.415582, 5495.938726, -5303.093233]) / 1e3

    r_exp = r - r_venus
    v_exp = v - v_venus
    utc = lox.UTC.from_iso("2016-05-30T12:00:00.000")
    tai = utc.to_tai()

    s_earth = lox.State(tai, tuple(r), tuple(v))
    s_venus = s_earth.to_origin(lox.Planet("Venus"), ephemeris)

    r_act = s_venus.position()
    v_act = s_venus.velocity()

    npt.assert_allclose(r_act, r_exp)
    npt.assert_allclose(v_act, v_exp)
