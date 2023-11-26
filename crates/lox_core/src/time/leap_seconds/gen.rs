/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

// From here: https://github.com/JuliaTime/LeapSeconds.jl/blob/master/gen/leap_seconds.jl

pub const LS_EPOCHS: [u64; 28] = [
    41317, 41499, 41683, 42048, 42413, 42778, 43144, 43509, 43874, 44239, 44786, 45151, 45516,
    46247, 47161, 47892, 48257, 48804, 49169, 49534, 50083, 50630, 51179, 53736, 54832, 56109,
    57204, 57754,
];

pub const LEAP_SECONDS: [f64; 28] = [
    10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
    26.0, 27.0, 28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0,
];
