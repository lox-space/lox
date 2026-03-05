// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::str::FromStr;

use lox_bodies::{DynOrigin, Origin, TryRotationalElements};
use thiserror::Error;

use crate::{
    frames::{Cirf, Iau, Icrf, Itrf, J2000, Mod, Pef, Teme, Tirf, Tod},
    iers::{Iau2000Model, IersSystem, ReferenceSystem},
    traits::{
        NonBodyFixedFrameError, NonQuasiInertialFrameError, ReferenceFrame, TryBodyFixed,
        TryQuasiInertial, frame_id,
    },
};

/// Enum representation of all known reference frames, for dynamic dispatch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DynFrame {
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
    Iau(DynOrigin),
    /// Mean of Date frame for the given IERS convention.
    Mod(ReferenceSystem),
    /// True of Date frame for the given IERS convention.
    Tod(ReferenceSystem),
    /// Pseudo-Earth Fixed frame for the given IERS convention.
    Pef(ReferenceSystem),
    /// True Equator Mean Equinox.
    Teme,
}

impl ReferenceFrame for DynFrame {
    fn name(&self) -> String {
        match self {
            DynFrame::Icrf => Icrf.name(),
            DynFrame::J2000 => J2000.name(),
            DynFrame::Cirf => Cirf.name(),
            DynFrame::Tirf => Tirf.name(),
            DynFrame::Itrf => Itrf.name(),
            DynFrame::Iau(dyn_origin) => {
                let body = dyn_origin.name();
                match body {
                    "Sun" | "Moon" => format!("IAU Body-Fixed Reference Frame for the {body}"),
                    _ => format!("IAU Body-Fixed Reference Frame for {body}"),
                }
            }
            DynFrame::Mod(sys) => Mod(*sys).name(),
            DynFrame::Tod(sys) => Tod(*sys).name(),
            DynFrame::Pef(sys) => Pef(*sys).name(),
            DynFrame::Teme => Teme.name(),
        }
    }

    fn abbreviation(&self) -> String {
        match self {
            DynFrame::Icrf => Icrf.abbreviation(),
            DynFrame::J2000 => J2000.abbreviation(),
            DynFrame::Cirf => Cirf.abbreviation(),
            DynFrame::Tirf => Tirf.abbreviation(),
            DynFrame::Itrf => Itrf.abbreviation(),
            DynFrame::Iau(dyn_origin) => {
                let body = dyn_origin.name().replace([' ', '-'], "_").to_uppercase();
                format!("IAU_{body}")
            }
            DynFrame::Mod(sys) => Mod(*sys).abbreviation(),
            DynFrame::Tod(sys) => Tod(*sys).abbreviation(),
            DynFrame::Pef(sys) => Pef(*sys).abbreviation(),
            DynFrame::Teme => Teme.abbreviation(),
        }
    }

    fn frame_id(&self, _: crate::traits::private::Internal) -> Option<usize> {
        match self {
            DynFrame::Icrf => frame_id(&Icrf),
            DynFrame::J2000 => frame_id(&J2000),
            DynFrame::Cirf => frame_id(&Cirf),
            DynFrame::Tirf => frame_id(&Tirf),
            DynFrame::Itrf => frame_id(&Itrf),
            DynFrame::Iau(dyn_origin) => Some(1000 + dyn_origin.id().0 as usize),
            DynFrame::Mod(sys) => frame_id(&Mod(*sys)),
            DynFrame::Tod(sys) => frame_id(&Tod(*sys)),

            DynFrame::Pef(sys) => frame_id(&Pef(*sys)),
            DynFrame::Teme => frame_id(&Teme),
        }
    }
}

impl TryQuasiInertial for DynFrame {
    fn try_quasi_inertial(&self) -> Result<(), NonQuasiInertialFrameError> {
        match self {
            DynFrame::Icrf
            | DynFrame::J2000
            | DynFrame::Cirf
            | DynFrame::Mod(_)
            | DynFrame::Tod(_) => Ok(()),
            _ => Err(NonQuasiInertialFrameError(self.abbreviation())),
        }
    }
}

impl TryBodyFixed for DynFrame {
    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError> {
        match self {
            DynFrame::Iau(_) | DynFrame::Itrf | DynFrame::Tirf | DynFrame::Pef(_) => Ok(()),
            _ => Err(NonBodyFixedFrameError(self.abbreviation())),
        }
    }
}

// Simple frame conversions.

impl From<Icrf> for DynFrame {
    fn from(_: Icrf) -> Self {
        DynFrame::Icrf
    }
}

impl From<J2000> for DynFrame {
    fn from(_: J2000) -> Self {
        DynFrame::J2000
    }
}

impl From<Cirf> for DynFrame {
    fn from(_: Cirf) -> Self {
        DynFrame::Cirf
    }
}

impl From<Tirf> for DynFrame {
    fn from(_: Tirf) -> Self {
        DynFrame::Tirf
    }
}

impl From<Itrf> for DynFrame {
    fn from(_: Itrf) -> Self {
        DynFrame::Itrf
    }
}

impl From<Teme> for DynFrame {
    fn from(_: Teme) -> Self {
        DynFrame::Teme
    }
}

// Parameterized equinox-based frames.

impl<T: IersSystem + Into<ReferenceSystem>> From<Mod<T>> for DynFrame {
    fn from(frame: Mod<T>) -> Self {
        DynFrame::Mod(frame.0.into())
    }
}

impl<T: IersSystem + Into<ReferenceSystem>> From<Tod<T>> for DynFrame {
    fn from(frame: Tod<T>) -> Self {
        DynFrame::Tod(frame.0.into())
    }
}

impl<T: IersSystem + Into<ReferenceSystem>> From<Pef<T>> for DynFrame {
    fn from(frame: Pef<T>) -> Self {
        DynFrame::Pef(frame.0.into())
    }
}

// IAU body-fixed frames.

impl<T: TryRotationalElements + Copy + Into<DynOrigin>> From<Iau<T>> for DynFrame {
    fn from(frame: Iau<T>) -> Self {
        DynFrame::Iau(frame.body().into())
    }
}

fn parse_iau_frame(s: &str) -> Option<DynFrame> {
    let (prefix, origin) = s.split_once("_")?;
    if prefix.to_lowercase() != "iau" {
        return None;
    }
    let origin: DynOrigin = origin.to_lowercase().parse().ok()?;
    let _ = origin.try_rotational_elements(0.0).ok()?;
    Some(DynFrame::Iau(origin))
}

fn parse_reference_system(s: &str) -> Option<ReferenceSystem> {
    match s.to_uppercase().as_str() {
        "IERS1996" => Some(ReferenceSystem::Iers1996),
        "IERS2003" => Some(ReferenceSystem::Iers2003(Iau2000Model::A)),
        "IERS2010" => Some(ReferenceSystem::Iers2010),
        _ => None,
    }
}

/// Parse frames in `FRAME(SYSTEM)` format, e.g. `MOD(IERS2003)`.
fn parse_equinox_frame(s: &str) -> Option<DynFrame> {
    let s_stripped = s.strip_suffix(')')?;
    let (frame, system) = s_stripped.split_once('(')?;
    let sys = parse_reference_system(system)?;
    match frame.to_uppercase().as_str() {
        "MOD" => Some(DynFrame::Mod(sys)),
        "TOD" => Some(DynFrame::Tod(sys)),
        "PEF" => Some(DynFrame::Pef(sys)),
        _ => None,
    }
}

/// No frame matching the given name is known.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("no frame with name '{0}' is known")]
pub struct UnknownFrameError(String);

impl FromStr for DynFrame {
    type Err = UnknownFrameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ICRF" => Ok(DynFrame::Icrf),
            "J2000" | "EME2000" => Ok(DynFrame::J2000),
            "CIRF" => Ok(DynFrame::Cirf),
            "TIRF" => Ok(DynFrame::Tirf),
            "ITRF" => Ok(DynFrame::Itrf),
            "TEME" => Ok(DynFrame::Teme),
            "MOD" => Ok(DynFrame::Mod(ReferenceSystem::Iers1996)),
            "TOD" => Ok(DynFrame::Tod(ReferenceSystem::Iers1996)),
            "PEF" => Ok(DynFrame::Pef(ReferenceSystem::Iers1996)),
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

    use glam::DVec3;
    use lox_bodies::{DynOrigin, Earth};
    use lox_test_utils::assert_approx_eq;
    use lox_time::utc::Utc;
    use rstest::rstest;

    #[rstest]
    #[case::valid("IAU_EARTH", Some(DynFrame::Iau(DynOrigin::Earth)))]
    #[case::invalid_prefix("FOO_EARTH", None)]
    #[case::unkown_body("IAU_RUPERT", None)]
    #[case::undefined_rotation("IAU_SYCORAX", None)]
    fn test_parse_iau_frame(#[case] name: &str, #[case] exp: Option<DynFrame>) {
        let act = parse_iau_frame(name);
        assert_eq!(act, exp)
    }

    #[rstest]
    #[case(
        DynFrame::Iau(DynOrigin::Earth),
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
        DynFrame::Iau(DynOrigin::Moon),
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
    fn test_icrf_to_bodyfixed(#[case] frame: DynFrame, #[case] r_exp: DVec3, #[case] v_exp: DVec3) {
        let time = Utc::from_iso("2024-07-05T09:09:18.173")
            .unwrap()
            .to_dyn_time();
        let r = DVec3::new(-5530.01774359, -3487.0895338, -1850.03476185);
        let v = DVec3::new(1.29534407, -5.02456882, 5.6391936);
        let rot = DefaultRotationProvider
            .try_rotation(DynFrame::Icrf, frame, time)
            .unwrap();
        let (r_act, v_act) = rot.rotate_state(r, v);
        assert_approx_eq!(r_act, r_exp, rtol <= 1e-8);
        assert_approx_eq!(v_act, v_exp, rtol <= 1e-5);
    }

    #[rstest]
    #[case("MOD", DynFrame::Mod(ReferenceSystem::Iers1996))]
    #[case("mod", DynFrame::Mod(ReferenceSystem::Iers1996))]
    #[case("TOD", DynFrame::Tod(ReferenceSystem::Iers1996))]
    #[case("tod", DynFrame::Tod(ReferenceSystem::Iers1996))]
    #[case("PEF", DynFrame::Pef(ReferenceSystem::Iers1996))]
    #[case("pef", DynFrame::Pef(ReferenceSystem::Iers1996))]
    #[case("MOD(IERS1996)", DynFrame::Mod(ReferenceSystem::Iers1996))]
    #[case(
        "MOD(IERS2003)",
        DynFrame::Mod(ReferenceSystem::Iers2003(Iau2000Model::A))
    )]
    #[case(
        "mod(iers2003)",
        DynFrame::Mod(ReferenceSystem::Iers2003(Iau2000Model::A))
    )]
    #[case(
        "TOD(IERS2003)",
        DynFrame::Tod(ReferenceSystem::Iers2003(Iau2000Model::A))
    )]
    #[case(
        "PEF(IERS2003)",
        DynFrame::Pef(ReferenceSystem::Iers2003(Iau2000Model::A))
    )]
    #[case("MOD(IERS2010)", DynFrame::Mod(ReferenceSystem::Iers2010))]
    #[case("TOD(IERS2010)", DynFrame::Tod(ReferenceSystem::Iers2010))]
    #[case("PEF(IERS2010)", DynFrame::Pef(ReferenceSystem::Iers2010))]
    fn test_parse_equinox_frames(#[case] name: &str, #[case] exp: DynFrame) {
        let act: DynFrame = name.parse().unwrap();
        assert_eq!(act, exp);
    }

    #[test]
    fn test_frame_id() {
        assert_eq!(frame_id(&Icrf), frame_id(&DynFrame::Icrf));
        assert_eq!(frame_id(&J2000), frame_id(&DynFrame::J2000));
        assert_eq!(frame_id(&Cirf), frame_id(&DynFrame::Cirf));
        assert_eq!(frame_id(&Tirf), frame_id(&DynFrame::Tirf));
        assert_eq!(frame_id(&Itrf), frame_id(&DynFrame::Itrf));
        assert_eq!(
            frame_id(&Iau::new(Earth)),
            frame_id(&DynFrame::Iau(DynOrigin::Earth))
        );
    }

    #[rstest]
    #[case("J2000", DynFrame::J2000)]
    #[case("j2000", DynFrame::J2000)]
    #[case("EME2000", DynFrame::J2000)]
    fn test_parse_j2000(#[case] name: &str, #[case] exp: DynFrame) {
        let act: DynFrame = name.parse().unwrap();
        assert_eq!(act, exp);
    }

    #[test]
    fn test_j2000_quasi_inertial() {
        assert!(DynFrame::J2000.try_quasi_inertial().is_ok());
    }

    #[test]
    fn test_from_simple_frames() {
        assert_eq!(DynFrame::from(Icrf), DynFrame::Icrf);
        assert_eq!(DynFrame::from(J2000), DynFrame::J2000);
        assert_eq!(DynFrame::from(Cirf), DynFrame::Cirf);
        assert_eq!(DynFrame::from(Tirf), DynFrame::Tirf);
        assert_eq!(DynFrame::from(Itrf), DynFrame::Itrf);
        assert_eq!(DynFrame::from(Teme), DynFrame::Teme);
    }

    #[test]
    fn test_from_parameterized_frames() {
        use crate::iers::{Iers1996, Iers2003, Iers2010};

        assert_eq!(
            DynFrame::from(Mod(Iers1996)),
            DynFrame::Mod(ReferenceSystem::Iers1996)
        );
        assert_eq!(
            DynFrame::from(Tod(Iers2003::default())),
            DynFrame::Tod(ReferenceSystem::Iers2003(Iau2000Model::A))
        );
        assert_eq!(
            DynFrame::from(Pef(Iers2010)),
            DynFrame::Pef(ReferenceSystem::Iers2010)
        );
    }

    #[test]
    fn test_from_iau_frame() {
        assert_eq!(
            DynFrame::from(Iau::new(Earth)),
            DynFrame::Iau(DynOrigin::Earth)
        );
    }

    #[rstest]
    #[case(DynFrame::Icrf)]
    #[case(DynFrame::J2000)]
    #[case(DynFrame::Cirf)]
    #[case(DynFrame::Tirf)]
    #[case(DynFrame::Itrf)]
    #[case(DynFrame::Teme)]
    #[case(DynFrame::Mod(ReferenceSystem::Iers1996))]
    #[case(DynFrame::Mod(ReferenceSystem::Iers2003(Iau2000Model::A)))]
    #[case(DynFrame::Mod(ReferenceSystem::Iers2010))]
    #[case(DynFrame::Tod(ReferenceSystem::Iers1996))]
    #[case(DynFrame::Tod(ReferenceSystem::Iers2003(Iau2000Model::A)))]
    #[case(DynFrame::Tod(ReferenceSystem::Iers2010))]
    #[case(DynFrame::Pef(ReferenceSystem::Iers1996))]
    #[case(DynFrame::Pef(ReferenceSystem::Iers2003(Iau2000Model::A)))]
    #[case(DynFrame::Pef(ReferenceSystem::Iers2010))]
    #[case(DynFrame::Iau(DynOrigin::Earth))]
    fn test_abbreviation_round_trip(#[case] frame: DynFrame) {
        let abbr = frame.abbreviation();
        let parsed: DynFrame = abbr
            .parse()
            .unwrap_or_else(|e| panic!("failed to parse abbreviation '{}': {}", abbr, e));
        assert_eq!(parsed, frame);
    }
}
