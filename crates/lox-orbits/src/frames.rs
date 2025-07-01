/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::str::FromStr;

use lox_bodies::{DynOrigin, Origin, RotationalElements, TryRotationalElements};
use lox_time::{
    Time,
    julian_dates::JulianDate,
    time_scales::TryToScale,
    time_scales::{Tdb, TimeScale},
};
use thiserror::Error;

use crate::{
    frames::iau::{IauFrameTransformationError, icrf_to_iau},
    frames::iers::{cirf_to_tirf, icrf_to_cirf, tirf_to_itrf},
    rotations::Rotation,
};

pub mod iau;
pub mod iers;

pub trait ReferenceFrame {
    fn name(&self) -> String;
    fn abbreviation(&self) -> String;
    fn is_rotating(&self) -> bool;
}

pub trait QuasiInertial: ReferenceFrame {}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{0} is not a quasi-inertial frame")]
pub struct NonQuasiInertialFrameError(String);

pub trait TryQuasiInertial: ReferenceFrame {
    fn try_quasi_inertial(&self) -> Result<(), NonQuasiInertialFrameError>;
}

impl<T: QuasiInertial> TryQuasiInertial for T {
    fn try_quasi_inertial(&self) -> Result<(), NonQuasiInertialFrameError> {
        Ok(())
    }
}

pub trait BodyFixed: ReferenceFrame {}

#[derive(Clone, Debug, Error)]
#[error("{0} is not a body-fixed frame")]
pub struct NonBodyFixedFrameError(String);

pub trait TryBodyFixed: ReferenceFrame {
    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError>;
}

impl<T: BodyFixed> TryBodyFixed for T {
    fn try_body_fixed(&self) -> Result<(), NonBodyFixedFrameError> {
        Ok(())
    }
}

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

pub trait TryRotateTo<T: TimeScale, R: ReferenceFrame, P> {
    type Error;

    fn try_rotation(
        &self,
        frame: R,
        time: Time<T>,
        provider: Option<&P>,
    ) -> Result<Rotation, Self::Error>;
}

impl<T, P> TryRotateTo<T, DynFrame, P> for DynFrame
where
    T: TimeScale + TryToScale<Tdb, P> + Copy,
{
    // FIXME
    type Error = IauFrameTransformationError;

    fn try_rotation(
        &self,
        frame: DynFrame,
        time: Time<T>,
        provider: Option<&P>,
    ) -> Result<Rotation, Self::Error> {
        // FIXME
        let seconds_j2000 = time.seconds_since_j2000();
        let centuries_j2000 = time.centuries_since_j2000();
        match self {
            DynFrame::Icrf => match frame {
                DynFrame::Icrf => Ok(Rotation::IDENTITY),
                DynFrame::Cirf => Ok(icrf_to_cirf(centuries_j2000)),
                DynFrame::Tirf => {
                    Ok(icrf_to_cirf(centuries_j2000).compose(&cirf_to_tirf(seconds_j2000)))
                }
                DynFrame::Itrf => Ok(icrf_to_cirf(centuries_j2000)
                    .compose(&cirf_to_tirf(seconds_j2000))
                    .compose(&tirf_to_itrf(centuries_j2000))),
                DynFrame::Iau(target) => icrf_to_iau(time, target, provider),
            },
            DynFrame::Cirf => match frame {
                DynFrame::Icrf => Ok(icrf_to_cirf(centuries_j2000).transpose()),
                DynFrame::Cirf => Ok(Rotation::IDENTITY),
                DynFrame::Tirf => Ok(cirf_to_tirf(seconds_j2000)),
                DynFrame::Itrf => {
                    Ok(cirf_to_tirf(seconds_j2000).compose(&tirf_to_itrf(centuries_j2000)))
                }
                DynFrame::Iau(_) => Ok(self
                    .try_rotation(DynFrame::Icrf, time, provider)?
                    .compose(&DynFrame::Icrf.try_rotation(frame, time, provider)?)),
            },
            DynFrame::Tirf => match frame {
                DynFrame::Icrf => Ok(cirf_to_tirf(seconds_j2000)
                    .transpose()
                    .compose(&icrf_to_cirf(centuries_j2000).transpose())),
                DynFrame::Cirf => Ok(cirf_to_tirf(seconds_j2000).transpose()),
                DynFrame::Tirf => Ok(Rotation::IDENTITY),
                DynFrame::Itrf => Ok(tirf_to_itrf(centuries_j2000)),
                DynFrame::Iau(_) => Ok(self
                    .try_rotation(DynFrame::Icrf, time, provider)?
                    .compose(&DynFrame::Icrf.try_rotation(frame, time, provider)?)),
            },
            DynFrame::Itrf => match frame {
                DynFrame::Icrf => Ok(tirf_to_itrf(centuries_j2000)
                    .transpose()
                    .compose(&cirf_to_tirf(seconds_j2000).transpose())
                    .compose(&icrf_to_cirf(centuries_j2000).transpose())),
                DynFrame::Cirf => Ok(tirf_to_itrf(centuries_j2000)
                    .transpose()
                    .compose(&cirf_to_tirf(seconds_j2000).transpose())),
                DynFrame::Tirf => Ok(tirf_to_itrf(centuries_j2000).transpose()),
                DynFrame::Itrf => Ok(Rotation::IDENTITY),
                DynFrame::Iau(_) => Ok(self
                    .try_rotation(DynFrame::Icrf, time, provider)?
                    .compose(&DynFrame::Icrf.try_rotation(frame, time, provider)?)),
            },
            DynFrame::Iau(origin) => match frame {
                DynFrame::Icrf => Ok(icrf_to_iau(time, *origin, provider)?.transpose()),
                DynFrame::Cirf => Ok(self
                    .try_rotation(DynFrame::Icrf, time, provider)?
                    .compose(&DynFrame::Icrf.try_rotation(frame, time, provider)?)),
                DynFrame::Tirf => Ok(self
                    .try_rotation(DynFrame::Icrf, time, provider)?
                    .compose(&DynFrame::Icrf.try_rotation(frame, time, provider)?)),
                DynFrame::Itrf => Ok(self
                    .try_rotation(DynFrame::Icrf, time, provider)?
                    .compose(&DynFrame::Icrf.try_rotation(frame, time, provider)?)),
                DynFrame::Iau(target) => {
                    if *origin == target {
                        Ok(Rotation::IDENTITY)
                    } else {
                        Ok(self
                            .try_rotation(DynFrame::Icrf, time, provider)?
                            .compose(&DynFrame::Icrf.try_rotation(frame, time, provider)?))
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use glam::DVec3;
    use lox_math::assert_close;
    use lox_math::is_close::IsClose;
    use lox_time::utc::Utc;
    use rstest::rstest;

    #[rstest]
    #[case("IAU_EARTH", Some(DynFrame::Iau(DynOrigin::Earth)))]
    #[case("FOO_EARTH", None)]
    #[case("IAU_RUPERT", None)]
    #[case("IAU_SYCORAX", None)]
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
        let time = Utc::from_iso("2024-07-05T09:09:18.173").unwrap().to_time();
        let r = DVec3::new(-5530.01774359, -3487.0895338, -1850.03476185);
        let v = DVec3::new(1.29534407, -5.02456882, 5.6391936);
        let rot = DynFrame::Icrf.try_rotation(frame, time, None::<&()>);
        let (r_act, v_act) = rot.unwrap().rotate_state(r, v);
        assert_close!(r_act, r_exp, 1e-8);
        assert_close!(v_act, v_exp, 1e-5);
    }
}
