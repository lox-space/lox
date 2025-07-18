/*
 * Copyright (c) 2025. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::RotationalElements;

use crate::traits::{ReferenceFrame, QuasiInertial, BodyFixed};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Icrf;

impl ReferenceFrame for Icrf {
    fn name(&self) -> String {
        "International Celestial Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "ICRF".to_string()
    }

    fn is_rotating(&self) -> bool {
        false
    }
}

impl QuasiInertial for Icrf {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Cirf;

impl ReferenceFrame for Cirf {
    fn name(&self) -> String {
        "Celestial Intermediate Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "CIRF".to_string()
    }

    fn is_rotating(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Tirf;

impl ReferenceFrame for Tirf {
    fn name(&self) -> String {
        "Terrestrial Intermediate Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "TIRF".to_string()
    }

    fn is_rotating(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Itrf;

impl ReferenceFrame for Itrf {
    fn name(&self) -> String {
        "International Terrestrial Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "ITRF".to_string()
    }

    fn is_rotating(&self) -> bool {
        true
    }
}

impl BodyFixed for Itrf {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Iau<T: RotationalElements>(pub T);

impl<T: RotationalElements> BodyFixed for Iau<T> {}

impl<T> ReferenceFrame for Iau<T>
where
    T: RotationalElements,
{
    fn name(&self) -> String {
        let body = self.0.name();
        match body {
            "Sun" | "Moon" => format!("IAU Body-Fixed Reference Frame for the {body}"),
            _ => format!("IAU Body-Fixed Reference Frame for {body}"),
        }
    }

    fn abbreviation(&self) -> String {
        let body = self.0.name().replace([' ', '-'], "_").to_uppercase();
        format!("IAU_{body}")
    }

    fn is_rotating(&self) -> bool {
        true
    }
}