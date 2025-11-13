// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::{Origin, RotationalElements, TryRotationalElements, UndefinedOriginPropertyError};

use crate::traits::{BodyFixed, QuasiInertial, ReferenceFrame};

const ICRF_ID: i32 = 0;
const CIRF_ID: i32 = 1;
const TIRF_ID: i32 = 2;
const ITRF_ID: i32 = 3;

const J2000_ID: i32 = 10;
const MOD_ID: i32 = 11;
const TOD_ID: i32 = 12;
const PEF_ID: i32 = 13;

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

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<i32> {
        Some(ICRF_ID)
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

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<i32> {
        Some(CIRF_ID)
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

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<i32> {
        Some(TIRF_ID)
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

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<i32> {
        Some(ITRF_ID)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct J2000;

impl ReferenceFrame for J2000 {
    fn name(&self) -> String {
        "J2000".to_owned()
    }

    fn abbreviation(&self) -> String {
        "J2000".to_owned()
    }

    fn is_rotating(&self) -> bool {
        false
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<i32> {
        Some(J2000_ID)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mod;

impl ReferenceFrame for Mod {
    fn name(&self) -> String {
        "Mean of Date".to_owned()
    }

    fn abbreviation(&self) -> String {
        "MOD".to_owned()
    }

    fn is_rotating(&self) -> bool {
        false
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<i32> {
        Some(MOD_ID)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tod;

impl ReferenceFrame for Tod {
    fn name(&self) -> String {
        "True of Date".to_owned()
    }

    fn abbreviation(&self) -> String {
        "TOD".to_owned()
    }

    fn is_rotating(&self) -> bool {
        false
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<i32> {
        Some(TOD_ID)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pef;

impl ReferenceFrame for Pef {
    fn name(&self) -> String {
        "Pseudo Earth Fixed".to_owned()
    }

    fn abbreviation(&self) -> String {
        "PEF".to_owned()
    }

    fn is_rotating(&self) -> bool {
        true
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<i32> {
        Some(PEF_ID)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Teme;

impl ReferenceFrame for Teme {
    fn name(&self) -> String {
        "True Equator Mean Equinox".to_owned()
    }

    fn abbreviation(&self) -> String {
        "TEME".to_owned()
    }

    fn is_rotating(&self) -> bool {
        false
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<i32> {
        Some(7)
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
    T: TryRotationalElements + Origin,
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

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<i32> {
        Some(1000 + self.0.id().0)
    }
}
