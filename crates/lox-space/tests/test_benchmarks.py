#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

import itertools
import pytest

import lox_space as lox


@pytest.mark.benchmark()
def test_visibility_benchmark(provider, oneweb, estrack):
    mask = lox.ElevationMask.fixed(0)
    t0 = next(iter(oneweb.values())).states()[0].time()
    times = [t0 + t for t in lox.TimeDelta.range(0, 86400, 3000)]

    passes = {}

    for (gs_name, gs), (sc_name, sc) in filter(
        lambda pair: isinstance(pair[0][1], lox.GroundLocation)
        and isinstance(pair[1][1], lox.Trajectory),
        itertools.combinations(itertools.chain(estrack.items(), oneweb.items()), 2),
    ):
        assert isinstance(gs, lox.GroundLocation)
        assert isinstance(sc, lox.Trajectory)

        if not sc_name in passes:
            passes[sc_name] = {}

        passes[sc_name][gs_name] = lox.visibility(times, gs, mask, sc, provider)

    assert len(passes) == len(oneweb)
    for sc_passes in passes.values():
        assert len(sc_passes) == len(estrack)
