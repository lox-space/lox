#  Copyright (c) 2024. Helge Eichhorn and the LOX contributors
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, you can obtain one at https://mozilla.org/MPL/2.0/.

import lox_space as lox
import pytest


def test_state_to_ground_location():
    time = lox.Time.from_iso("2024-07-05T09:09:18.173 TAI")
    position = (-5530.01774359, -3487.0895338, -1850.03476185)
    velocity = (1.29534407, -5.02456882, 5.6391936)
    state = lox.State(time, position, velocity, lox.Planet("Earth"), lox.Frame("ICRF"))
    ground = state.to_ground_location()
    assert ground.longitude() == pytest.approx(2.646276127963636)
    assert ground.latitude() == pytest.approx(-0.2794495715104036)
    assert ground.altitude() == pytest.approx(417.8524158044338)
