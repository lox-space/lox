/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use lox_bodies::{Earth, Jupiter, Mars, Mercury, Neptune, Pluto, Saturn, Uranus, Venus};
use lox_coords::frames::iau::BodyFixed;
use lox_coords::frames::Icrf;

use crate::LoxPyError;

// TODO: Add other supported IAU frames
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PyFrame {
    Icrf(Icrf),
    IauMercury(BodyFixed<Mercury>),
    IauVenus(BodyFixed<Venus>),
    IauEarth(BodyFixed<Earth>),
    IauMars(BodyFixed<Mars>),
    IauJupiter(BodyFixed<Jupiter>),
    IauSaturn(BodyFixed<Saturn>),
    IauUranus(BodyFixed<Uranus>),
    IauNeptune(BodyFixed<Neptune>),
    IauPluto(BodyFixed<Pluto>),
}

impl Display for PyFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            PyFrame::Icrf(frame) => write!(f, "{}", frame),
            PyFrame::IauMercury(frame) => write!(f, "{}", frame),
            PyFrame::IauVenus(frame) => write!(f, "{}", frame),
            PyFrame::IauEarth(frame) => write!(f, "{}", frame),
            PyFrame::IauMars(frame) => write!(f, "{}", frame),
            PyFrame::IauJupiter(frame) => write!(f, "{}", frame),
            PyFrame::IauSaturn(frame) => write!(f, "{}", frame),
            PyFrame::IauUranus(frame) => write!(f, "{}", frame),
            PyFrame::IauNeptune(frame) => write!(f, "{}", frame),
            PyFrame::IauPluto(frame) => write!(f, "{}", frame),
        }
    }
}

impl FromStr for PyFrame {
    type Err = LoxPyError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "icrf" | "ICRF" => Ok(PyFrame::Icrf(Icrf)),
            "iau_mercury" | "IAU_MERCURY" => Ok(PyFrame::IauMercury(BodyFixed(Mercury))),
            "iau_venus" | "IAU_VENUS" => Ok(PyFrame::IauVenus(BodyFixed(Venus))),
            "iau_earth" | "IAU_EARTH" => Ok(PyFrame::IauEarth(BodyFixed(Earth))),
            "iau_mars" | "IAU_MARS" => Ok(PyFrame::IauMars(BodyFixed(Mars))),
            "iau_jupiter" | "IAU_JUPITER" => Ok(PyFrame::IauJupiter(BodyFixed(Jupiter))),
            "iau_saturn" | "IAU_SATURN" => Ok(PyFrame::IauSaturn(BodyFixed(Saturn))),
            "iau_uranus" | "IAU_URANUS" => Ok(PyFrame::IauUranus(BodyFixed(Uranus))),
            "iau_neptune" | "IAU_NEPTUNE" => Ok(PyFrame::IauNeptune(BodyFixed(Neptune))),
            "iau_pluto" | "IAU_PLUTO" => Ok(PyFrame::IauPluto(BodyFixed(Pluto))),
            _ => Err(LoxPyError::InvalidFrame(name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("icrf", PyFrame::Icrf(Icrf))]
    #[case("iau_mercury", PyFrame::IauMercury(BodyFixed(Mercury)))]
    #[case("iau_venus", PyFrame::IauVenus(BodyFixed(Venus)))]
    #[case("iau_earth", PyFrame::IauEarth(BodyFixed(Earth)))]
    #[case("iau_mars", PyFrame::IauMars(BodyFixed(Mars)))]
    #[case("iau_jupiter", PyFrame::IauJupiter(BodyFixed(Jupiter)))]
    #[case("iau_saturn", PyFrame::IauSaturn(BodyFixed(Saturn)))]
    #[case("iau_uranus", PyFrame::IauUranus(BodyFixed(Uranus)))]
    #[case("iau_neptune", PyFrame::IauNeptune(BodyFixed(Neptune)))]
    #[case("iau_pluto", PyFrame::IauPluto(BodyFixed(Pluto)))]
    fn test_frames(#[case] name: &str, #[case] exp: PyFrame) {
        let upper = name.to_uppercase();
        let act = PyFrame::from_str(name).expect("frame should be valid");
        assert_eq!(act, exp);
        let act = PyFrame::from_str(&upper).expect("frame should be valid");
        assert_eq!(act, exp);
        assert_eq!(format!("{}", act), upper);
    }

    #[test]
    fn test_invalid_frame() {
        let frame = PyFrame::from_str("Flat Earth");
        assert!(frame.is_err());
    }
}
