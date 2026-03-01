# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import pytest

import lox_space as lox


@pytest.fixture(scope="session")
def space_assets(oneweb):
    return [lox.Spacecraft(name, traj) for name, traj in oneweb.items()]


@pytest.fixture(scope="session")
def t0(oneweb):
    return next(iter(oneweb.values())).states()[0].time()


@pytest.fixture(scope="session")
def t1(t0):
    return t0 + lox.TimeDelta(86400)


@pytest.mark.benchmark()
def test_visibility_benchmark(estrack, space_assets, oneweb, t0, t1, ephemeris):
    analysis = lox.VisibilityAnalysis(
        estrack, space_assets, min_pass_duration=lox.TimeDelta(600)
    )
    results = analysis.compute(t0, t1, ephemeris)
    assert results.num_pairs() == len(oneweb) * len(estrack)
