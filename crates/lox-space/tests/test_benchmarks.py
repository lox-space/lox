# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import pytest

import lox_space as lox


@pytest.fixture(scope="session")
def space_assets(oneweb):
    return [lox.Spacecraft(name, sgp4) for name, sgp4 in oneweb.items()]


@pytest.fixture(scope="session")
def t0(oneweb):
    return next(iter(oneweb.values())).time()


@pytest.fixture(scope="session")
def t1(t0):
    return t0 + lox.TimeDelta(86400)


@pytest.fixture(scope="session")
def scenario(t0, t1, space_assets, estrack):
    return lox.Scenario(t0, t1, spacecraft=space_assets, ground_stations=estrack)


@pytest.fixture(scope="session")
def ensemble(scenario):
    return scenario.propagate()


@pytest.mark.benchmark()
def test_visibility_benchmark(scenario, ensemble, oneweb, estrack, ephemeris):
    analysis = lox.VisibilityAnalysis(
        scenario, ensemble=ensemble, min_pass_duration=lox.TimeDelta(600)
    )
    results = analysis.compute(ephemeris)
    assert results.num_pairs() == len(oneweb) * len(estrack)
