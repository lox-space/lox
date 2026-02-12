// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::str::FromStr;

use lox_bodies::{DynOrigin, Origin, TryRotationalElements};
use thiserror::Error;

use crate::{
    frames::{Cirf, Icrf, Itrf, Mod, Pef, Teme, Tirf, Tod},
    iers::ReferenceSystem,
    traits::{
        NonBodyFixedFrameError, NonQuasiInertialFrameError, ReferenceFrame, TryBodyFixed,
        TryQuasiInertial, frame_id,
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DynFrame {
    #[default]
    Icrf,
    Cirf,
    Tirf,
    Itrf,
    Iau(DynOrigin),
    Mod(ReferenceSystem),
    Tod(ReferenceSystem),
    Pef(ReferenceSystem),
    Teme,
}

impl ReferenceFrame for DynFrame {
    fn name(&self) -> String {
        match self {
            DynFrame::Icrf => Icrf.name(),
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
            DynFrame::Icrf | DynFrame::Cirf | DynFrame::Mod(_) | DynFrame::Tod(_) => Ok(()),
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

fn parse_iau_frame(s: &str) -> Option<DynFrame> {
    let (prefix, origin) = s.split_once("_")?;
    if prefix.to_lowercase() != "iau" {
        return None;
    }
    let origin: DynOrigin = origin.to_lowercase().parse().ok()?;
    let _ = origin.try_rotational_elements(0.0).ok()?;
    Some(DynFrame::Iau(origin))
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("no frame with name '{0}' is known")]
pub struct UnknownFrameError(String);

impl FromStr for DynFrame {
    type Err = UnknownFrameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "icrf" | "ICRF" => Ok(DynFrame::Icrf),
            "cirf" | "CIRF" => Ok(DynFrame::Cirf),
            "tirf" | "TIRF" => Ok(DynFrame::Tirf),
            "itrf" | "ITRF" => Ok(DynFrame::Itrf),
            "teme" | "TEME" => Ok(DynFrame::Teme),
            _ => {
                if let Some(frame) = parse_iau_frame(s) {
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

    #[test]
    fn test_frame_id() {
        assert_eq!(frame_id(&Icrf), frame_id(&DynFrame::Icrf));
        assert_eq!(frame_id(&Cirf), frame_id(&DynFrame::Cirf));
        assert_eq!(frame_id(&Tirf), frame_id(&DynFrame::Tirf));
        assert_eq!(frame_id(&Itrf), frame_id(&DynFrame::Itrf));
        assert_eq!(
            frame_id(&Iau::new(Earth)),
            frame_id(&DynFrame::Iau(DynOrigin::Earth))
        );
    }
}
