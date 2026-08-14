// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::str::FromStr;

use lox_bodies::{
    CoordinateOrigin, Earth, Origin, RotationalElements, Spheroid, TryRotationalElements,
    TrySpheroid, UndefinedOriginPropertyError,
};
use lox_core::coords::Ellipsoid;
use thiserror::Error;

use crate::{
    iers::{Iau2000Model, IersSystem, ReferenceSystem},
    traits::{
        BodyFixed, FrameKey, NonBodyFixedFrameError, NonQuasiInertialFrameError, QuasiInertial,
        ReferenceEllipsoid, ReferenceFrame, TryBodyFixed, TryQuasiInertial, TryReferenceEllipsoid,
        UndefinedReferenceEllipsoidError, frame_key,
    },
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

impl QuasiInertial for Cirf {}

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

impl BodyFixed for Tirf {
    type Origin = Earth;

    fn origin(&self) -> Self::Origin {
        Earth
    }
}

// TIRF differs from ITRF only by polar motion, so it shares ITRF's datum.
impl ReferenceEllipsoid for Tirf {
    fn reference_ellipsoid(&self) -> Ellipsoid {
        Ellipsoid::GRS80
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

impl BodyFixed for Itrf {
    type Origin = Earth;

    fn origin(&self) -> Self::Origin {
        Earth
    }
}

impl ReferenceEllipsoid for Itrf {
    fn reference_ellipsoid(&self) -> Ellipsoid {
        Ellipsoid::GRS80
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

impl<T> QuasiInertial for Mod<T> where T: IersSystem + Into<ReferenceSystem> + Copy {}

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

impl<T> QuasiInertial for Tod<T> where T: IersSystem + Into<ReferenceSystem> + Copy {}

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

impl<T> BodyFixed for Pef<T>
where
    T: IersSystem + Into<ReferenceSystem> + Copy,
{
    type Origin = Earth;

    fn origin(&self) -> Self::Origin {
        Earth
    }
}

// PEF differs from ITRF only by polar motion, so it shares ITRF's datum.
impl<T> ReferenceEllipsoid for Pef<T>
where
    T: IersSystem + Into<ReferenceSystem> + Copy,
{
    fn reference_ellipsoid(&self) -> Ellipsoid {
        Ellipsoid::GRS80
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

impl QuasiInertial for Teme {}

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

impl<T: TryRotationalElements + Copy> BodyFixed for Iau<T> {
    type Origin = T;

    fn origin(&self) -> Self::Origin {
        self.0
    }
}

impl<T: RotationalElements + Spheroid + Copy> ReferenceEllipsoid for Iau<T> {
    fn reference_ellipsoid(&self) -> Ellipsoid {
        self.0.ellipsoid()
    }
}

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
    T: TryRotationalElements + CoordinateOrigin,
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
    use lox_bodies::Origin;

    use super::Iau;

    #[test]
    fn deserialize_valid_body() {
        let json = serde_json::to_string(&Origin::Earth).unwrap();
        let frame: Iau<Origin> = serde_json::from_str(&json).unwrap();
        assert_eq!(frame.body(), Origin::Earth);
    }

    #[test]
    fn deserialize_rejects_undefined_elements() {
        // Sycorax has no rotational elements; deserializing it as an IAU frame
        // must fail rather than yield a frame that panics on first use.
        let json = serde_json::to_string(&Origin::Sycorax).unwrap();
        let result: Result<Iau<Origin>, _> = serde_json::from_str(&json);
        assert!(result.is_err());
    }
}

/// A reference frame determined at runtime.
///
/// Covers the same set of frames as the zero-sized frame types, as a single
/// closed enum. Because the frame is not known statically, frame properties
/// are reached through the fallible checks ([`TryQuasiInertial`],
/// [`TryBodyFixed`]) rather than the marker traits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Frame {
    /// International Celestial Reference Frame.
    #[default]
    Icrf,
    /// J2000 Mean Equator and Equinox.
    J2000,
    /// Celestial Intermediate Reference Frame.
    Cirf,
    /// Terrestrial Intermediate Reference Frame.
    Tirf,
    /// International Terrestrial Reference Frame.
    Itrf,
    /// IAU body-fixed frame for the given origin.
    Iau(Origin),
    /// Mean of Date frame for the given IERS convention.
    Mod(ReferenceSystem),
    /// True of Date frame for the given IERS convention.
    Tod(ReferenceSystem),
    /// Pseudo-Earth Fixed frame for the given IERS convention.
    Pef(ReferenceSystem),
    /// True Equator Mean Equinox.
    Teme,
}

impl ReferenceFrame for Frame {
    fn name(&self) -> String {
        match self {
            Frame::Icrf => Icrf.name(),
            Frame::J2000 => J2000.name(),
            Frame::Cirf => Cirf.name(),
            Frame::Tirf => Tirf.name(),
            Frame::Itrf => Itrf.name(),
            Frame::Iau(dynamic_origin) => iau_name(dynamic_origin.name()),
            Frame::Mod(sys) => Mod(*sys).name(),
            Frame::Tod(sys) => Tod(*sys).name(),
            Frame::Pef(sys) => Pef(*sys).name(),
            Frame::Teme => Teme.name(),
        }
    }

    fn abbreviation(&self) -> String {
        match self {
            Frame::Icrf => Icrf.abbreviation(),
            Frame::J2000 => J2000.abbreviation(),
            Frame::Cirf => Cirf.abbreviation(),
            Frame::Tirf => Tirf.abbreviation(),
            Frame::Itrf => Itrf.abbreviation(),
            Frame::Iau(dynamic_origin) => iau_abbreviation(dynamic_origin.name()),
            Frame::Mod(sys) => Mod(*sys).abbreviation(),
            Frame::Tod(sys) => Tod(*sys).abbreviation(),
            Frame::Pef(sys) => Pef(*sys).abbreviation(),
            Frame::Teme => Teme.abbreviation(),
        }
    }

    fn frame_key(&self, _: crate::traits::private::Internal) -> Option<FrameKey> {
        match self {
            Frame::Icrf => frame_key(&Icrf),
            Frame::J2000 => frame_key(&J2000),
            Frame::Cirf => frame_key(&Cirf),
            Frame::Tirf => frame_key(&Tirf),
            Frame::Itrf => frame_key(&Itrf),
            Frame::Iau(dynamic_origin) => Some(FrameKey::Iau(dynamic_origin.id())),
            Frame::Mod(sys) => frame_key(&Mod(*sys)),
            Frame::Tod(sys) => frame_key(&Tod(*sys)),

            Frame::Pef(sys) => frame_key(&Pef(*sys)),
            Frame::Teme => frame_key(&Teme),
        }
    }
}

impl TryQuasiInertial for Frame {
    fn try_quasi_inertial(&self) -> Result<(), NonQuasiInertialFrameError> {
        match self {
            Frame::Icrf
            | Frame::J2000
            | Frame::Cirf
            | Frame::Mod(_)
            | Frame::Tod(_)
            | Frame::Teme => Ok(()),
            _ => Err(NonQuasiInertialFrameError(self.abbreviation())),
        }
    }
}

impl TryBodyFixed for Frame {
    type Origin = Origin;

    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError> {
        match self {
            Frame::Iau(_) | Frame::Itrf | Frame::Tirf | Frame::Pef(_) => Ok(()),
            _ => Err(NonBodyFixedFrameError(self.abbreviation())),
        }
    }

    fn try_origin(&self) -> Result<Self::Origin, NonBodyFixedFrameError> {
        match self {
            Frame::Iau(origin) => Ok(*origin),
            Frame::Itrf => Ok(Itrf.origin().into()),
            Frame::Tirf => Ok(Tirf.origin().into()),
            Frame::Pef(sys) => Ok(Pef(*sys).origin().into()),
            _ => Err(NonBodyFixedFrameError(self.abbreviation())),
        }
    }
}

impl TryReferenceEllipsoid for Frame {
    fn try_reference_ellipsoid(&self) -> Result<Ellipsoid, UndefinedReferenceEllipsoidError> {
        match self {
            Frame::Iau(origin) => Ok(origin.try_ellipsoid()?),
            Frame::Itrf => Ok(Itrf.reference_ellipsoid()),
            Frame::Tirf => Ok(Tirf.reference_ellipsoid()),
            Frame::Pef(sys) => Ok(Pef(*sys).reference_ellipsoid()),
            _ => Err(UndefinedReferenceEllipsoidError::NotBodyFixed(
                NonBodyFixedFrameError(self.abbreviation()),
            )),
        }
    }
}

// Simple frame conversions.

impl From<Icrf> for Frame {
    fn from(_: Icrf) -> Self {
        Frame::Icrf
    }
}

impl From<J2000> for Frame {
    fn from(_: J2000) -> Self {
        Frame::J2000
    }
}

impl From<Cirf> for Frame {
    fn from(_: Cirf) -> Self {
        Frame::Cirf
    }
}

impl From<Tirf> for Frame {
    fn from(_: Tirf) -> Self {
        Frame::Tirf
    }
}

impl From<Itrf> for Frame {
    fn from(_: Itrf) -> Self {
        Frame::Itrf
    }
}

impl From<Teme> for Frame {
    fn from(_: Teme) -> Self {
        Frame::Teme
    }
}

// Parameterized equinox-based frames.

impl<T: IersSystem + Into<ReferenceSystem>> From<Mod<T>> for Frame {
    fn from(frame: Mod<T>) -> Self {
        Frame::Mod(frame.0.into())
    }
}

impl<T: IersSystem + Into<ReferenceSystem>> From<Tod<T>> for Frame {
    fn from(frame: Tod<T>) -> Self {
        Frame::Tod(frame.0.into())
    }
}

impl<T: IersSystem + Into<ReferenceSystem>> From<Pef<T>> for Frame {
    fn from(frame: Pef<T>) -> Self {
        Frame::Pef(frame.0.into())
    }
}

// IAU body-fixed frames.

impl<T: TryRotationalElements + Copy + Into<Origin>> From<Iau<T>> for Frame {
    fn from(frame: Iau<T>) -> Self {
        Frame::Iau(frame.body().into())
    }
}

fn parse_iau_frame(s: &str) -> Option<Frame> {
    let (prefix, origin) = s.split_once("_")?;
    if prefix.to_lowercase() != "iau" {
        return None;
    }
    let origin: Origin = origin.to_lowercase().parse().ok()?;
    let _ = origin.try_rotational_elements(0.0).ok()?;
    Some(Frame::Iau(origin))
}

fn parse_reference_system(s: &str) -> Option<ReferenceSystem> {
    match s.to_uppercase().as_str() {
        "IERS1996" => Some(ReferenceSystem::Iers1996),
        "IERS2003" | "IERS2003A" => Some(ReferenceSystem::Iers2003(Iau2000Model::A)),
        "IERS2003B" => Some(ReferenceSystem::Iers2003(Iau2000Model::B)),
        "IERS2010" => Some(ReferenceSystem::Iers2010),
        _ => None,
    }
}

/// Parse frames in `FRAME(SYSTEM)` format, e.g. `MOD(IERS2003)`.
fn parse_equinox_frame(s: &str) -> Option<Frame> {
    let s_stripped = s.strip_suffix(')')?;
    let (frame, system) = s_stripped.split_once('(')?;
    let sys = parse_reference_system(system)?;
    match frame.to_uppercase().as_str() {
        "MOD" => Some(Frame::Mod(sys)),
        "TOD" => Some(Frame::Tod(sys)),
        "PEF" => Some(Frame::Pef(sys)),
        _ => None,
    }
}

/// No frame matching the given name is known.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("no frame with name '{0}' is known")]
pub struct UnknownFrameError(String);

impl FromStr for Frame {
    type Err = UnknownFrameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ICRF" => Ok(Frame::Icrf),
            "J2000" | "EME2000" => Ok(Frame::J2000),
            "CIRF" => Ok(Frame::Cirf),
            "TIRF" => Ok(Frame::Tirf),
            "ITRF" => Ok(Frame::Itrf),
            "TEME" => Ok(Frame::Teme),
            "MOD" => Ok(Frame::Mod(ReferenceSystem::Iers1996)),
            "TOD" => Ok(Frame::Tod(ReferenceSystem::Iers1996)),
            "PEF" => Ok(Frame::Pef(ReferenceSystem::Iers1996)),
            _ => {
                if let Some(frame) = parse_equinox_frame(s) {
                    Ok(frame)
                } else if let Some(frame) = parse_iau_frame(s) {
                    Ok(frame)
                } else {
                    Err(UnknownFrameError(s.to_owned()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::iers::{Iers1996, Iers2003, Iers2010};
    use crate::rotations::TryRotation;
    use crate::traits::frame_key;
    use crate::{Iau, providers::DefaultRotationProvider};

    use lox_approx::assert_approx_eq;
    use lox_bodies::{Earth, Origin};
    use lox_core::glam::DVec3;
    use lox_time::utc::Utc;
    use rstest::rstest;

    #[test]
    fn zst_frame_names_and_abbreviations() {
        assert_eq!(Icrf.abbreviation(), "ICRF");
        assert_eq!(J2000.abbreviation(), "J2000");
        assert_eq!(Cirf.abbreviation(), "CIRF");
        assert_eq!(Tirf.abbreviation(), "TIRF");
        assert_eq!(Itrf.abbreviation(), "ITRF");
        assert_eq!(Teme.abbreviation(), "TEME");
        assert!(Icrf.name().contains("Celestial"));
        assert!(J2000.name().contains("J2000"));
        assert!(Cirf.name().contains("Celestial Intermediate"));
        assert!(Tirf.name().contains("Terrestrial Intermediate"));
        assert!(Itrf.name().contains("Terrestrial Reference"));
        assert!(Teme.name().contains("True Equator"));
    }

    #[test]
    fn iau_frame_naming() {
        // Sun/Moon take the "the" article; other bodies do not.
        let sun = Iau::try_new(Origin::Sun).unwrap();
        let earth = Iau::try_new(Origin::Earth).unwrap();
        assert!(sun.name().contains("for the Sun"));
        assert!(earth.name().contains("for Earth"));
        assert_eq!(earth.abbreviation(), "IAU_EARTH");
    }

    #[test]
    fn custom_frame_has_no_key() {
        struct Custom;
        impl ReferenceFrame for Custom {
            fn name(&self) -> String {
                "Custom".to_owned()
            }
            fn abbreviation(&self) -> String {
                "CUS".to_owned()
            }
        }
        assert_eq!(frame_key(&Custom), None);
    }
    #[rstest]
    #[case::valid("IAU_EARTH", Some(Frame::Iau(Origin::Earth)))]
    #[case::invalid_prefix("FOO_EARTH", None)]
    #[case::unkown_body("IAU_RUPERT", None)]
    #[case::undefined_rotation("IAU_SYCORAX", None)]
    fn test_parse_iau_frame(#[case] name: &str, #[case] exp: Option<Frame>) {
        let act = parse_iau_frame(name);
        assert_eq!(act, exp)
    }

    #[rstest]
    #[case(
        Frame::Iau(Origin::Earth),
        DVec3::new(
            -5.740_259_426_667_957e3,
            3.121_136_072_795_472_5e3,
            -1.863_182_656_331_802_7e3,
        ),
        DVec3::new(
            -3.532_378_757_836_52,
            -3.152_377_656_863_808,
            5.642_296_713_889_555,
        ),
    )]
    #[case(
        Frame::Iau(Origin::Moon),
        DVec3::new(
            3.777_805_761_337_502e3,
            -5.633_812_666_439_680_5e3,
            -3.896_880_165_980_424e2,
        ),
        DVec3::new(
            2.576_901_711_027_508_3,
            1.250_106_874_006_032_4,
            7.100_615_382_464_156,
        ),
    )]
    fn test_icrf_to_bodyfixed(#[case] frame: Frame, #[case] r_exp: DVec3, #[case] v_exp: DVec3) {
        let time = Utc::from_iso("2024-07-05T09:09:18.173")
            .unwrap()
            .to_dynamic_time();
        let r = DVec3::new(-5530.01774359, -3487.0895338, -1850.03476185);
        let v = DVec3::new(1.29534407, -5.02456882, 5.6391936);
        let rot = DefaultRotationProvider
            .try_rotation(Frame::Icrf, frame, time)
            .unwrap();
        let (r_act, v_act) = rot.rotate_state(r, v);
        assert_approx_eq!(r_act, r_exp, rtol <= 1e-8);
        assert_approx_eq!(v_act, v_exp, rtol <= 1e-5);
    }

    #[rstest]
    #[case("MOD", Frame::Mod(ReferenceSystem::Iers1996))]
    #[case("mod", Frame::Mod(ReferenceSystem::Iers1996))]
    #[case("TOD", Frame::Tod(ReferenceSystem::Iers1996))]
    #[case("tod", Frame::Tod(ReferenceSystem::Iers1996))]
    #[case("PEF", Frame::Pef(ReferenceSystem::Iers1996))]
    #[case("pef", Frame::Pef(ReferenceSystem::Iers1996))]
    #[case("MOD(IERS1996)", Frame::Mod(ReferenceSystem::Iers1996))]
    #[case(
        "MOD(IERS2003)",
        Frame::Mod(ReferenceSystem::Iers2003(Iau2000Model::A))
    )]
    #[case(
        "mod(iers2003)",
        Frame::Mod(ReferenceSystem::Iers2003(Iau2000Model::A))
    )]
    #[case(
        "TOD(IERS2003)",
        Frame::Tod(ReferenceSystem::Iers2003(Iau2000Model::A))
    )]
    #[case(
        "PEF(IERS2003)",
        Frame::Pef(ReferenceSystem::Iers2003(Iau2000Model::A))
    )]
    #[case("MOD(IERS2010)", Frame::Mod(ReferenceSystem::Iers2010))]
    #[case("TOD(IERS2010)", Frame::Tod(ReferenceSystem::Iers2010))]
    #[case("PEF(IERS2010)", Frame::Pef(ReferenceSystem::Iers2010))]
    fn test_parse_equinox_frames(#[case] name: &str, #[case] exp: Frame) {
        let act: Frame = name.parse().unwrap();
        assert_eq!(act, exp);
    }

    #[test]
    fn test_frame_key() {
        assert_eq!(frame_key(&Icrf), frame_key(&Frame::Icrf));
        assert_eq!(frame_key(&J2000), frame_key(&Frame::J2000));
        assert_eq!(frame_key(&Cirf), frame_key(&Frame::Cirf));
        assert_eq!(frame_key(&Tirf), frame_key(&Frame::Tirf));
        assert_eq!(frame_key(&Itrf), frame_key(&Frame::Itrf));
        assert_eq!(
            frame_key(&Iau::new(Earth)),
            frame_key(&Frame::Iau(Origin::Earth))
        );
        // Parameterized frames agree too, and the nutation model is part of the key.
        let mod_b = ReferenceSystem::Iers2003(Iau2000Model::B);
        assert_eq!(frame_key(&Mod(mod_b)), frame_key(&Frame::Mod(mod_b)));
        assert_eq!(frame_key(&Teme), frame_key(&Frame::Teme));
        assert_ne!(
            frame_key(&Frame::Mod(ReferenceSystem::Iers2003(Iau2000Model::A))),
            frame_key(&Frame::Mod(mod_b))
        );
    }

    #[rstest]
    #[case("J2000", Frame::J2000)]
    #[case("j2000", Frame::J2000)]
    #[case("EME2000", Frame::J2000)]
    fn test_parse_j2000(#[case] name: &str, #[case] exp: Frame) {
        let act: Frame = name.parse().unwrap();
        assert_eq!(act, exp);
    }

    #[test]
    fn test_j2000_quasi_inertial() {
        assert!(Frame::J2000.try_quasi_inertial().is_ok());
    }

    /// Quasi-inertial frames do not rotate with a central body; body-fixed
    /// frames do. Every variant is listed so adding one forces a decision.
    #[rstest]
    #[case(Frame::Icrf, true)]
    #[case(Frame::J2000, true)]
    #[case(Frame::Cirf, true)]
    #[case(Frame::Teme, true)]
    #[case(Frame::Mod(ReferenceSystem::Iers1996), true)]
    #[case(Frame::Mod(ReferenceSystem::Iers2010), true)]
    #[case(Frame::Tod(ReferenceSystem::Iers1996), true)]
    #[case(Frame::Tod(ReferenceSystem::Iers2010), true)]
    #[case(Frame::Tirf, false)]
    #[case(Frame::Itrf, false)]
    #[case(Frame::Pef(ReferenceSystem::Iers1996), false)]
    #[case(Frame::Pef(ReferenceSystem::Iers2010), false)]
    #[case(Frame::Iau(Origin::Earth), false)]
    fn test_quasi_inertial_classification(#[case] frame: Frame, #[case] exp: bool) {
        assert_eq!(frame.try_quasi_inertial().is_ok(), exp);
        // The two classifications are mutually exclusive.
        assert!(!(frame.try_quasi_inertial().is_ok() && frame.try_body_fixed().is_ok()));
    }

    #[rstest]
    #[case(Frame::Tirf, true)]
    #[case(Frame::Itrf, true)]
    #[case(Frame::Pef(ReferenceSystem::Iers1996), true)]
    #[case(Frame::Pef(ReferenceSystem::Iers2010), true)]
    #[case(Frame::Iau(Origin::Earth), true)]
    #[case(Frame::Iau(Origin::Moon), true)]
    #[case(Frame::Icrf, false)]
    #[case(Frame::J2000, false)]
    #[case(Frame::Cirf, false)]
    #[case(Frame::Teme, false)]
    #[case(Frame::Mod(ReferenceSystem::Iers1996), false)]
    #[case(Frame::Tod(ReferenceSystem::Iers1996), false)]
    fn test_body_fixed_classification(#[case] frame: Frame, #[case] exp: bool) {
        assert_eq!(frame.try_body_fixed().is_ok(), exp);
        assert_eq!(frame.try_origin().is_ok(), exp);
    }

    /// The terrestrial frames are all realizations of the same rotating Earth.
    #[rstest]
    #[case(Frame::Itrf)]
    #[case(Frame::Tirf)]
    #[case(Frame::Pef(ReferenceSystem::Iers2010))]
    fn test_terrestrial_frames_share_origin_and_datum(#[case] frame: Frame) {
        assert_eq!(frame.try_origin().unwrap(), Origin::Earth);
        assert_eq!(frame.try_reference_ellipsoid().unwrap(), Ellipsoid::GRS80);
    }

    // The zero-sized types carry their capabilities as compile-time markers and
    // `Frame` as runtime checks. The two must not drift apart: passing a ZST to
    // these helpers requires the marker, and the assertion covers the enum.

    fn assert_quasi_inertial_agrees(frame: impl QuasiInertial + Copy + Into<Frame>) {
        let dynamic: Frame = frame.into();
        assert!(
            dynamic.try_quasi_inertial().is_ok(),
            "{} is quasi-inertial as a ZST but not as a Frame",
            dynamic.abbreviation()
        );
    }

    fn assert_body_fixed_agrees<F>(frame: F)
    where
        F: BodyFixed + Copy + Into<Frame>,
        F::Origin: Into<Origin>,
    {
        let dynamic: Frame = frame.into();
        assert!(
            dynamic.try_body_fixed().is_ok(),
            "{} is body-fixed as a ZST but not as a Frame",
            dynamic.abbreviation()
        );
        assert_eq!(dynamic.try_origin().unwrap(), frame.origin().into());
    }

    fn assert_reference_ellipsoid_agrees(frame: impl ReferenceEllipsoid + Copy + Into<Frame>) {
        let dynamic: Frame = frame.into();
        assert_eq!(
            dynamic.try_reference_ellipsoid().unwrap(),
            frame.reference_ellipsoid()
        );
    }

    #[test]
    fn test_quasi_inertial_zsts_agree_with_frame() {
        assert_quasi_inertial_agrees(Icrf);
        assert_quasi_inertial_agrees(J2000);
        assert_quasi_inertial_agrees(Cirf);
        assert_quasi_inertial_agrees(Teme);
        assert_quasi_inertial_agrees(Mod(Iers1996));
        assert_quasi_inertial_agrees(Mod(Iers2003::default()));
        assert_quasi_inertial_agrees(Mod(Iers2010));
        assert_quasi_inertial_agrees(Tod(Iers1996));
        assert_quasi_inertial_agrees(Tod(Iers2003::default()));
        assert_quasi_inertial_agrees(Tod(Iers2010));
    }

    #[test]
    fn test_body_fixed_zsts_agree_with_frame() {
        assert_body_fixed_agrees(Itrf);
        assert_body_fixed_agrees(Tirf);
        assert_body_fixed_agrees(Pef(Iers1996));
        assert_body_fixed_agrees(Pef(Iers2003::default()));
        assert_body_fixed_agrees(Pef(Iers2010));
        assert_body_fixed_agrees(Iau::new(Earth));
    }

    #[test]
    fn test_reference_ellipsoid_zsts_agree_with_frame() {
        assert_reference_ellipsoid_agrees(Itrf);
        assert_reference_ellipsoid_agrees(Tirf);
        assert_reference_ellipsoid_agrees(Pef(Iers1996));
        assert_reference_ellipsoid_agrees(Pef(Iers2010));
        assert_reference_ellipsoid_agrees(Iau::new(Earth));
    }

    #[test]
    fn test_from_simple_frames() {
        assert_eq!(Frame::from(Icrf), Frame::Icrf);
        assert_eq!(Frame::from(J2000), Frame::J2000);
        assert_eq!(Frame::from(Cirf), Frame::Cirf);
        assert_eq!(Frame::from(Tirf), Frame::Tirf);
        assert_eq!(Frame::from(Itrf), Frame::Itrf);
        assert_eq!(Frame::from(Teme), Frame::Teme);
    }

    #[test]
    fn test_from_parameterized_frames() {
        assert_eq!(
            Frame::from(Mod(Iers1996)),
            Frame::Mod(ReferenceSystem::Iers1996)
        );
        assert_eq!(
            Frame::from(Tod(Iers2003::default())),
            Frame::Tod(ReferenceSystem::Iers2003(Iau2000Model::A))
        );
        assert_eq!(
            Frame::from(Pef(Iers2010)),
            Frame::Pef(ReferenceSystem::Iers2010)
        );
    }

    #[test]
    fn test_from_iau_frame() {
        assert_eq!(Frame::from(Iau::new(Earth)), Frame::Iau(Origin::Earth));
    }

    #[rstest]
    #[case(Frame::Icrf)]
    #[case(Frame::J2000)]
    #[case(Frame::Cirf)]
    #[case(Frame::Tirf)]
    #[case(Frame::Itrf)]
    #[case(Frame::Teme)]
    #[case(Frame::Mod(ReferenceSystem::Iers1996))]
    #[case(Frame::Mod(ReferenceSystem::Iers2003(Iau2000Model::A)))]
    #[case(Frame::Mod(ReferenceSystem::Iers2003(Iau2000Model::B)))]
    #[case(Frame::Mod(ReferenceSystem::Iers2010))]
    #[case(Frame::Tod(ReferenceSystem::Iers1996))]
    #[case(Frame::Tod(ReferenceSystem::Iers2003(Iau2000Model::A)))]
    #[case(Frame::Tod(ReferenceSystem::Iers2003(Iau2000Model::B)))]
    #[case(Frame::Tod(ReferenceSystem::Iers2010))]
    #[case(Frame::Pef(ReferenceSystem::Iers1996))]
    #[case(Frame::Pef(ReferenceSystem::Iers2003(Iau2000Model::A)))]
    #[case(Frame::Pef(ReferenceSystem::Iers2003(Iau2000Model::B)))]
    #[case(Frame::Pef(ReferenceSystem::Iers2010))]
    #[case(Frame::Iau(Origin::Earth))]
    fn test_abbreviation_round_trip(#[case] frame: Frame) {
        let abbr = frame.abbreviation();
        let parsed: Frame = abbr
            .parse()
            .unwrap_or_else(|e| panic!("failed to parse abbreviation '{}': {}", abbr, e));
        assert_eq!(parsed, frame);
    }
}
