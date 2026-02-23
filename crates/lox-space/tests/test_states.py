# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox
import numpy as np
import numpy.testing as npt
import pytest


def test_state_to_ground_location():
    time = lox.UTC.from_iso("2024-07-05T09:09:18.173").to_scale("TAI")
    position = (-5530.01774359, -3487.0895338, -1850.03476185)
    velocity = (1.29534407, -5.02456882, 5.6391936)
    state = lox.State(
        time, position, velocity, lox.Origin("Earth"), lox.Frame("ICRF")
    ).to_frame(lox.Frame("IAU_EARTH"))
    npt.assert_allclose(
        state.position(),
        np.array([-5740.259426667957, 3121.1360727954725, -1863.1826563318027]),
    )
    npt.assert_allclose(
        state.velocity(),
        np.array([-3.53237875783652, -3.152377656863808, 5.642296713889555]),
    )
    ground = state.to_ground_location()
    assert ground.longitude() == pytest.approx(2.643578045424445)
    assert ground.latitude() == pytest.approx(-0.27944957125091063)
    assert ground.altitude() == pytest.approx(417.8524151150059)


def test_state_to_origin(ephemeris):
    r = np.array([6068279.27, -1692843.94, -2516619.18]) / 1e3
    v = np.array([-660.415582, 5495.938726, -5303.093233]) / 1e3

    utc = lox.UTC.from_iso("2016-05-30T12:00:00.000")
    tai = utc.to_scale("TAI")

    s_earth = lox.State(tai, tuple(r), tuple(v))
    s_venus = s_earth.to_origin(lox.Origin("Venus"), ephemeris)

    # Verify round-trip: converting back to Earth origin should recover the original state
    s_earth_rt = s_venus.to_origin(lox.Origin("Earth"), ephemeris)

    npt.assert_allclose(s_earth_rt.position(), r, atol=1e-6)
    npt.assert_allclose(s_earth_rt.velocity(), v, atol=1e-6)
