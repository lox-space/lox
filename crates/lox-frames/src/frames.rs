// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::{Origin, RotationalElements, TryRotationalElements, UndefinedOriginPropertyError};

use crate::{
    iers::{IersSystem, ReferenceSystem},
    traits::{BodyFixed, FrameKey, QuasiInertial, ReferenceFrame},
};

/// International Celestial Reference Frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
pub struct Icrf;

impl ReferenceFrame for Icrf {
    fn name(&self) -> String {
        "International Celestial Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "ICRF".to_string()
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        Some(FrameKey::Icrf)
    }
}

impl QuasiInertial for Icrf {}

/// J2000 Mean Equator and Equinox frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
pub struct J2000;

impl ReferenceFrame for J2000 {
    fn name(&self) -> String {
        "J2000 Mean Equator and Equinox".to_string()
    }

    fn abbreviation(&self) -> String {
        "J2000".to_string()
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        Some(FrameKey::J2000)
    }
}

impl QuasiInertial for J2000 {}

/// Celestial Intermediate Reference Frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
pub struct Cirf;

impl ReferenceFrame for Cirf {
    fn name(&self) -> String {
        "Celestial Intermediate Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "CIRF".to_string()
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        Some(FrameKey::Cirf)
    }
}

/// Terrestrial Intermediate Reference Frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
pub struct Tirf;

impl ReferenceFrame for Tirf {
    fn name(&self) -> String {
        "Terrestrial Intermediate Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "TIRF".to_string()
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        Some(FrameKey::Tirf)
    }
}

/// International Terrestrial Reference Frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
pub struct Itrf;

impl ReferenceFrame for Itrf {
    fn name(&self) -> String {
        "International Terrestrial Reference Frame".to_string()
    }

    fn abbreviation(&self) -> String {
        "ITRF".to_string()
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        Some(FrameKey::Itrf)
    }
}

/// Mean of Date frame, parameterised by IERS convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mod<T: IersSystem>(pub T);

impl<T> ReferenceFrame for Mod<T>
where
    T: IersSystem + Into<ReferenceSystem> + Copy,
{
    fn name(&self) -> String {
        format!("{} Mean of Date Frame", self.0.name())
    }

    fn abbreviation(&self) -> String {
        format!("MOD({})", self.0.abbreviation())
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        Some(FrameKey::Mod(self.0.into()))
    }
}

/// True of Date frame, parameterised by IERS convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tod<T: IersSystem>(pub T);

impl<T> ReferenceFrame for Tod<T>
where
    T: IersSystem + Into<ReferenceSystem> + Copy,
{
    fn name(&self) -> String {
        format!("{} True of Date Frame", self.0.name())
    }

    fn abbreviation(&self) -> String {
        format!("TOD({})", self.0.abbreviation())
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        Some(FrameKey::Tod(self.0.into()))
    }
}

/// Pseudo-Earth Fixed frame, parameterised by IERS convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pef<T: IersSystem>(pub T);

impl<T> ReferenceFrame for Pef<T>
where
    T: IersSystem + Into<ReferenceSystem> + Copy,
{
    fn name(&self) -> String {
        format!("{} Pseudo-Earth Fixed Frame", self.0.name())
    }

    fn abbreviation(&self) -> String {
        format!("PEF({})", self.0.abbreviation())
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        Some(FrameKey::Pef(self.0.into()))
    }
}

/// True Equator Mean Equinox frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
pub struct Teme;

impl ReferenceFrame for Teme {
    fn name(&self) -> String {
        "True Equator Mean Equinox".to_owned()
    }

    fn abbreviation(&self) -> String {
        "TEME".to_owned()
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        Some(FrameKey::Teme)
    }
}

impl BodyFixed for Itrf {}

// -- serde: serialize frame ZSTs as their abbreviation --

macro_rules! impl_frame_serde {
    ($ty:ident, $abbrev:literal) => {
        #[cfg(feature = "serde")]
        impl From<$ty> for &'static str {
            fn from(_: $ty) -> Self {
                $abbrev
            }
        }

        #[cfg(feature = "serde")]
        impl TryFrom<String> for $ty {
            type Error = String;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                if s == $abbrev {
                    Ok($ty)
                } else {
                    Err(format!("expected \"{}\", got \"{}\"", $abbrev, s))
                }
            }
        }
    };
}

impl_frame_serde!(Icrf, "ICRF");
impl_frame_serde!(J2000, "J2000");
impl_frame_serde!(Cirf, "CIRF");
impl_frame_serde!(Tirf, "TIRF");
impl_frame_serde!(Itrf, "ITRF");
impl_frame_serde!(Teme, "TEME");

/// IAU body-fixed reference frame derived from rotational elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Iau<T: TryRotationalElements>(T);

// Deserialization goes through `try_new` so a body with undefined rotational
// elements cannot produce a frame that later panics on use.
#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for Iau<T>
where
    T: TryRotationalElements + serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let body = T::deserialize(deserializer)?;
        Iau::try_new(body).map_err(serde::de::Error::custom)
    }
}

impl<T> Iau<T>
where
    T: RotationalElements,
{
    /// Creates an IAU frame for a body with known rotational elements.
    pub fn new(body: T) -> Self {
        Self(body)
    }
}

impl<T> Iau<T>
where
    T: TryRotationalElements,
{
    /// Creates an IAU frame, returning an error if rotational elements are undefined.
    pub fn try_new(body: T) -> Result<Self, UndefinedOriginPropertyError> {
        let _ = body.try_right_ascension(0.0)?;
        Ok(Self(body))
    }

    /// Returns the underlying body.
    pub fn body(&self) -> T
    where
        T: Copy,
    {
        self.0
    }

    /// Returns the rotational elements (right ascension, declination, prime meridian) at
    /// the given Julian centuries since J2000.
    pub fn rotational_elements(&self, j2000: f64) -> (f64, f64, f64) {
        // Construction (`new`, `try_new`, and deserialization) guarantees the
        // body has defined rotational elements.
        self.0
            .try_rotational_elements(j2000)
            .expect("Iau frame wraps a body with defined rotational elements")
    }

    /// Returns the time derivatives of the rotational elements.
    pub fn rotational_element_rates(&self, j2000: f64) -> (f64, f64, f64) {
        self.0
            .try_rotational_element_rates(j2000)
            .expect("Iau frame wraps a body with defined rotational elements")
    }
}

impl<T: TryRotationalElements> BodyFixed for Iau<T> {}

/// Full name of the IAU body-fixed frame for a body named `body`.
pub(crate) fn iau_name(body: &str) -> String {
    match body {
        "Sun" | "Moon" => format!("IAU Body-Fixed Reference Frame for the {body}"),
        _ => format!("IAU Body-Fixed Reference Frame for {body}"),
    }
}

/// Abbreviation of the IAU body-fixed frame for a body named `body`.
pub(crate) fn iau_abbreviation(body: &str) -> String {
    format!("IAU_{}", body.replace([' ', '-'], "_").to_uppercase())
}

impl<T> ReferenceFrame for Iau<T>
where
    T: TryRotationalElements + Origin,
{
    fn name(&self) -> String {
        iau_name(self.0.name())
    }

    fn abbreviation(&self) -> String {
        iau_abbreviation(self.0.name())
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        Some(FrameKey::Iau(self.0.id()))
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use lox_bodies::DynOrigin;

    use super::Iau;

    #[test]
    fn deserialize_valid_body() {
        let json = serde_json::to_string(&DynOrigin::Earth).unwrap();
        let frame: Iau<DynOrigin> = serde_json::from_str(&json).unwrap();
        assert_eq!(frame.body(), DynOrigin::Earth);
    }

    #[test]
    fn deserialize_rejects_undefined_elements() {
        // Sycorax has no rotational elements; deserializing it as an IAU frame
        // must fail rather than yield a frame that panics on first use.
        let json = serde_json::to_string(&DynOrigin::Sycorax).unwrap();
        let result: Result<Iau<DynOrigin>, _> = serde_json::from_str(&json);
        assert!(result.is_err());
    }
}
