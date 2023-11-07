/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use glam::DVec3;

pub trait Frame {
    fn is_inertial() -> bool;

    fn is_rotating() -> bool;
}

pub trait Transform {
    fn transform_position(position: DVec3) -> DVec3;

    fn transform_velocity(velocity: DVec3) -> DVec3;

    fn transform_state(position: DVec3, velocity: DVec3) -> (DVec3, DVec3);
}

struct ICRF;

impl Frame for ICRF {
    fn is_inertial() -> bool {
        true
    }

    fn is_rotating() -> bool {
        false
    }
}

pub fn take_frame(_foo: impl Frame) {
    println!("Huzzah")
}

#[cfg(test)]
mod tests {
    use crate::frames::{take_frame, ICRF};

    #[test]
    fn test_icrf() {
        take_frame(ICRF)
    }
}
