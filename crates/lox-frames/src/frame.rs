// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::str::FromStr;

use lox_bodies::{CoordinateOrigin, Origin, TryRotationalElements};
use thiserror::Error;

use crate::{
    frames::{Cirf, Iau, Icrf, Itrf, J2000, Mod, Pef, Teme, Tirf, Tod, iau_abbreviation, iau_name},
    iers::{Iau2000Model, IersSystem, ReferenceSystem},
    traits::{
        FrameKey, NonBodyFixedFrameError, NonQuasiInertialFrameError, ReferenceFrame, TryBodyFixed,
        TryQuasiInertial, frame_key,
    },
};

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
            Frame::Icrf | Frame::J2000 | Frame::Cirf | Frame::Mod(_) | Frame::Tod(_) => Ok(()),
            _ => Err(NonQuasiInertialFrameError(self.abbreviation())),
        }
    }
}

impl TryBodyFixed for Frame {
    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError> {
        match self {
            Frame::Iau(_) | Frame::Itrf | Frame::Tirf | Frame::Pef(_) => Ok(()),
            _ => Err(NonBodyFixedFrameError(self.abbreviation())),
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

    use crate::rotations::TryRotation;
    use crate::{Iau, providers::DefaultRotationProvider};

    use lox_approx::assert_approx_eq;
    use lox_bodies::{Earth, Origin};
    use lox_core::glam::DVec3;
    use lox_time::utc::Utc;
    use rstest::rstest;

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
        use crate::iers::{Iers1996, Iers2003, Iers2010};

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
