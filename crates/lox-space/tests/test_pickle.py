# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import pickle

import lox_space as lox
import pytest


@pytest.mark.parametrize(
    "obj",
    [
        lox.Origin("Earth"),
        lox.Frame("ICRF"),
        lox.ElevationMask.fixed(0.0 * lox.rad),
        lox.TimeScale("TAI"),
        lox.Time("TAI", 2000, 1, 1),
        lox.TimeDelta(1.5),
        lox.TimeDelta(1, 1),
        lox.minutes,
        1024 * lox.km,
        26.63 * lox.dB,
        lox.km,
        lox.GravitationalParameter(3.986e14),
    ],
)
def test_pickle(obj):
    pickled = pickle.dumps(obj)
    unpickled = pickle.loads(pickled)
    assert unpickled == obj
