// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::{RotationalElements, TryRotationalElements, UndefinedOriginPropertyError};

use crate::traits::{BodyFixed, QuasiInertial, ReferenceFrame};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Iau<T: TryRotationalElements>(T);

impl<T> Iau<T>
where
    T: RotationalElements,
{
    pub fn new(body: T) -> Self {
        Self(body)
    }
}

impl<T> Iau<T>
where
    T: TryRotationalElements,
{
    pub fn try_new(body: T) -> Result<Self, UndefinedOriginPropertyError> {
        let _ = body.try_right_ascension(0.0)?;
        Ok(Self(body))
    }

    pub fn body(&self) -> T
    where
        T: Copy,
    {
        self.0
    }

    pub fn rotational_elements(&self, j2000: f64) -> (f64, f64, f64) {
        self.0.try_rotational_elements(j2000).unwrap()
    }

    pub fn rotational_element_rates(&self, j2000: f64) -> (f64, f64, f64) {
        self.0.try_rotational_element_rates(j2000).unwrap()
    }
}

impl<T: TryRotationalElements> BodyFixed for Iau<T> {}

impl<T> ReferenceFrame for Iau<T>
where
    T: TryRotationalElements,
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
