// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::str::FromStr;

use lox_bodies::{DynOrigin, Origin, TryRotationalElements, UndefinedOriginPropertyError};
use lox_time::{Time, time_scales::DynTimeScale};
use thiserror::Error;

use crate::{
    Iau,
    frames::{Cirf, Icrf, Itrf, Tirf},
    providers::DefaultTransformProvider,
    traits::{
        NonBodyFixedFrameError, NonQuasiInertialFrameError, ReferenceFrame, TryBodyFixed,
        TryQuasiInertial,
    },
    transformations::{Rotation, TryTransform},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum DynFrame {
    #[default]
    Icrf,
    Cirf,
    Tirf,
    Itrf,
    Iau(DynOrigin),
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
        }
    }

    fn is_rotating(&self) -> bool {
        match self {
            DynFrame::Icrf | DynFrame::Cirf => false,
            DynFrame::Tirf | DynFrame::Itrf | DynFrame::Iau(_) => true,
        }
    }
}

impl TryQuasiInertial for DynFrame {
    fn try_quasi_inertial(&self) -> Result<(), NonQuasiInertialFrameError> {
        match self {
            DynFrame::Icrf => Ok(()),
            _ => Err(NonQuasiInertialFrameError(self.abbreviation())),
        }
    }
}

impl TryBodyFixed for DynFrame {
    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError> {
        match self {
            DynFrame::Iau(_) | DynFrame::Itrf => Ok(()),
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

#[derive(Debug, Error)]
pub enum DynTransformError {
    #[error("transformations between {0} and {1} require an EOP provider")]
    MissingEopProvider(String, String),
    #[error(transparent)]
    MissingUt1Provider(#[from] lox_time::offsets::MissingEopProviderError),
    #[error(transparent)]
    UndefinedRotationalElements(#[from] UndefinedOriginPropertyError),
}

impl TryTransform<DynFrame, DynFrame, DynTimeScale> for DefaultTransformProvider {
    type Error = DynTransformError;

    fn try_transform(
        &self,
        origin: DynFrame,
        target: DynFrame,
        time: Time<DynTimeScale>,
    ) -> Result<Rotation, Self::Error> {
        match (origin, target) {
            (DynFrame::Icrf, DynFrame::Icrf) => Ok(Rotation::IDENTITY),
            (DynFrame::Icrf, DynFrame::Iau(target)) => {
                Ok(self.try_transform(Icrf, Iau::try_new(target)?, time)?)
            }
            (DynFrame::Cirf, DynFrame::Cirf) => Ok(Rotation::IDENTITY),
            (DynFrame::Tirf, DynFrame::Tirf) => Ok(Rotation::IDENTITY),
            (DynFrame::Iau(origin), DynFrame::Icrf) => {
                Ok(self.try_transform(Iau::try_new(origin)?, Icrf, time)?)
            }
            (DynFrame::Iau(origin), DynFrame::Iau(target)) => {
                let origin = Iau::try_new(origin)?;
                let target = Iau::try_new(target)?;
                Ok(self
                    .try_transform(origin, Icrf, time)?
                    .compose(self.try_transform(Icrf, target, time)?))
            }
            (origin, target) => Err(DynTransformError::MissingEopProvider(
                origin.abbreviation(),
                target.abbreviation(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use glam::DVec3;
    use lox_bodies::DynOrigin;
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
        let rot = DefaultTransformProvider
            .try_transform(DynFrame::Icrf, frame, time)
            .unwrap();
        let (r_act, v_act) = rot.rotate_state(r, v);
        assert_approx_eq!(r_act, r_exp, rtol <= 1e-8);
        assert_approx_eq!(v_act, v_exp, rtol <= 1e-5);
    }
}
