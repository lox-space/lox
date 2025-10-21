# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
# SPDX-License-Identifier: MPL-2.0

import pytest

import lox_space as lox


@pytest.fixture(scope="session")
def times(oneweb):
    t0 = next(iter(oneweb.values())).states()[0].time()
    return [t0 + t for t in lox.TimeDelta.range(0, 86400, 3000)]


@pytest.fixture(scope="session")
def ensemble(oneweb):
    return lox.Ensemble(oneweb)


@pytest.mark.benchmark()
def test_visibility_benchmark(estrack, ensemble, oneweb, times, ephemeris):
    passes = lox.visibility_all(times, estrack, ensemble, ephemeris)
    assert len(passes) == len(oneweb)
    for sc_passes in passes.values():
        assert len(sc_passes) == len(estrack)
