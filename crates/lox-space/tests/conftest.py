# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import pathlib
from pathlib import Path

import pytest
import lox_space as lox

DATA_DIR = pathlib.Path(__file__).parents[3].joinpath("data")


@pytest.fixture(scope="session")
def data_dir():
    return DATA_DIR


@pytest.fixture(scope="session")
def provider():
    return lox.EOPProvider(
        DATA_DIR.joinpath("iers/finals.all.csv"),
        DATA_DIR.joinpath("iers/finals2000A.all.csv"),
    )


@pytest.fixture(scope="session")
def oneweb():
    with open(DATA_DIR.joinpath("oneweb_tle.txt")) as f:
        lines = f.readlines()

    sgp4s = []
    for i in range(0, len(lines), 3):
        tle = lines[i : i + 3]
        name = tle[0].strip()
        sgp4s.append((name, lox.SGP4("".join(tle))))

    return dict(sgp4s)


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
    return [
        lox.GroundStation(
            name,
            lox.GroundLocation(
                lox.Origin("Earth"), lon * lox.deg, lat * lox.deg, 0 * lox.km
            ),
            lox.ElevationMask.fixed(0 * lox.rad),
        )
        for name, lat, lon in stations
    ]


@pytest.fixture(scope="session")
def ephemeris():
    return lox.SPK(DATA_DIR.joinpath("spice/de440s.bsp"))
