#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

import pathlib
import numpy as np

import pytest
import lox_space as lox

DATA_DIR = pathlib.Path(__file__).parents[3].joinpath("data")


@pytest.fixture(scope="session")
def provider():
    return lox.UT1Provider(str(DATA_DIR.joinpath("finals2000A.all.csv")))


@pytest.fixture(scope="session")
def oneweb():
    with open(DATA_DIR.joinpath("oneweb_tle.txt")) as f:
        lines = f.readlines()

    t0 = lox.SGP4("".join(lines[0:3])).time()
    times = [t0 + t for t in lox.TimeDelta.range(0, 86400, 60)]

    trajectories = []
    for i in range(0, len(lines), 3):
        tle = lines[i : i + 3]
        name = tle[0].strip()
        trajectory = lox.SGP4("".join(tle)).propagate(times)
        trajectories.append((name, trajectory))

    return dict(trajectories)


@pytest.fixture(scope="session")
def estrack():
    stations = [
        ("Kiruna", 67.858428, 20.966880),
        ("Esrange Space Center", 67.8833, 21.1833),
        ("Kourou", 5.2360, -52.7686),
        ("Redu", 50.00205516, 5.14518047),
        ("Cebreros", 40.3726, -4.4739),
        ("New Norcia", -30.9855, 116.2041),
    ]
    return {
        name: lox.GroundLocation(
            lox.Origin("Earth"), np.radians(lon), np.radians(lat), 0
        )
        for name, lat, lon in stations
    }
