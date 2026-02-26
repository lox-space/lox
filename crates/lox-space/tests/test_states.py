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
