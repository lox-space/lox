/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::RotationalElements;

pub trait ReferenceFrame {
    fn name(&self) -> &str;
    fn abbreviation(&self) -> &str;
}

pub trait TryToFrame<T: ReferenceFrame> {
    type Error;

    fn try_to_frame(&self, frame: T) -> Result<T, Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Icrf;

impl ReferenceFrame for Icrf {
    fn name(&self) -> &str {
        "International Celestial Reference Frame"
    }

    fn abbreviation(&self) -> &str {
        "ICRF"
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Bodyfixed<T: RotationalElements>(T);

impl<T: RotationalElements> ReferenceFrame for Bodyfixed<T> {
    fn name(&self) -> &str {
        todo!()
    }

    fn abbreviation(&self) -> &str {
        todo!()
    }
}
