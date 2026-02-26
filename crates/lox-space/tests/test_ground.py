# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import lox_space as lox
import numpy as np
import pytest


def test_observables():
    location = lox.GroundLocation(
        lox.Origin("Earth"), -4 * lox.deg, 41 * lox.deg, 0 * lox.km
    )
    time = lox.Time("TDB", 2012, 7, 1)
    state = lox.Cartesian(
        time,
        position=[3359927.0, -2398072.0, 5153000.0],
        velocity=[5065.7, 5485.0, -744.0],
        frame=lox.Frame("IAU_EARTH"),
    )
    observables = location.observables(state)
    expected_range = 2707.7
    expected_range_rate = -7.16
    expected_azimuth = np.radians(-53.418)
    expected_elevation = np.radians(-7.077)
    assert observables.range().to_kilometers() == pytest.approx(
        expected_range, rel=1e-2
    )
    assert observables.range_rate().to_kilometers_per_second() == pytest.approx(
        expected_range_rate, rel=1e-2
    )
    assert float(observables.azimuth()) == pytest.approx(expected_azimuth, rel=1e-2)
    assert float(observables.elevation()) == pytest.approx(expected_elevation, rel=1e-2)


def test_elevation_mask():
    mask = lox.ElevationMask.variable(
        np.array([-np.pi, 0.0, np.pi]), np.array([0.0, 5.0, 0.0])
    )
    assert float(mask.min_elevation(lox.Angle(np.pi / 2))) == pytest.approx(2.5)
