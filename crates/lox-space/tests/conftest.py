#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

import pathlib

import pytest
import lox_space as lox

DATA_DIR = pathlib.Path(__file__).parents[3].joinpath("data")


@pytest.fixture
def provider():
    return lox.UT1Provider(str(DATA_DIR.joinpath("finals2000A.all.csv")))
