#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

import pickle

import lox_space as lox
import pytest


@pytest.mark.parametrize("obj", [
    lox.Sun(),
    lox.Barycenter("Solar System Barycenter"),
    lox.Planet("Earth"),
    lox.Satellite("Moon"),
    lox.MinorBody("Ceres"),
])
def test_pickle(obj):
    pickled = pickle.dumps(obj)
    unpickled = pickle.loads(pickled)
    assert unpickled == obj
