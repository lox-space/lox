// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::{Origin, RotationalElements, TryRotationalElements, UndefinedOriginPropertyError};

use crate::{
    iers::IersSystem,
    traits::{BodyFixed, QuasiInertial, ReferenceFrame},
};

const ICRF_ID: usize = 0;
const CIRF_ID: usize = 1;
const TIRF_ID: usize = 2;
const ITRF_ID: usize = 3;

const MOD_ID: usize = 11;
const TOD_ID: usize = 12;
const PEF_ID: usize = 13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Icrf;

impl ReferenceFrame for Icrf {
    fn name(&self) -> String {
        "International Celestial Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "ICRF".to_string()
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<usize> {
        Some(ICRF_ID)
    }
}

impl QuasiInertial for Icrf {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cirf;

impl ReferenceFrame for Cirf {
    fn name(&self) -> String {
        "Celestial Intermediate Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "CIRF".to_string()
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<usize> {
        Some(CIRF_ID)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tirf;

impl ReferenceFrame for Tirf {
    fn name(&self) -> String {
        "Terrestrial Intermediate Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "TIRF".to_string()
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<usize> {
        Some(TIRF_ID)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Itrf;

impl ReferenceFrame for Itrf {
    fn name(&self) -> String {
        "International Terrestrial Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "ITRF".to_string()
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<usize> {
        Some(ITRF_ID)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mod<T: IersSystem>(pub T);

impl<T> ReferenceFrame for Mod<T>
where
    T: IersSystem,
{
    fn name(&self) -> String {
        format!("{} Mean of Date Frame", self.0.name())
    }

    fn abbreviation(&self) -> String {
        format!("MOD({})", self.0.name())
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<usize> {
        Some(MOD_ID * 10 + self.0.id())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tod<T: IersSystem>(pub T);

impl<T> ReferenceFrame for Tod<T>
where
    T: IersSystem,
{
    fn name(&self) -> String {
        format!("{} True of Date Frame", self.0.name())
    }

    fn abbreviation(&self) -> String {
        format!("TOD({})", self.0.name())
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<usize> {
        Some(TOD_ID * 10 + self.0.id())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pef<T: IersSystem>(pub T);

impl<T> ReferenceFrame for Pef<T>
where
    T: IersSystem,
{
    fn name(&self) -> String {
        format!("{} Pseudo-Earth Fixed Frame", self.0.name())
    }

    fn abbreviation(&self) -> String {
        format!("PEF({})", self.0.name())
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<usize> {
        Some(PEF_ID * 10 + self.0.id())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Teme;

impl ReferenceFrame for Teme {
    fn name(&self) -> String {
        "True Equator Mean Equinox".to_owned()
    }

    fn abbreviation(&self) -> String {
        "TEME".to_owned()
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<usize> {
        Some(7)
    }
}

impl BodyFixed for Itrf {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<usize> {
        Some(1000 + self.0.id().0 as usize)
    }
}
