/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub fn hyperbolic_to_true(hyperbolic_anomaly: f64, eccentricity: f64) -> f64 {
    2.0 * (((1.0 + eccentricity) / (eccentricity - 1.0)).sqrt() * (hyperbolic_anomaly / 2.0).tanh())
        .atan()
}

pub fn eccentric_to_true(eccentric_anomaly: f64, eccentricity: f64) -> f64 {
    2.0 * (((1.0 + eccentricity) / (1.0 - eccentricity)).sqrt() * (eccentric_anomaly / 2.0).tan())
        .atan()
}
