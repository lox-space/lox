// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};

use glam::{DMat3, DVec3};
use lox_units::Angle;

use crate::iers::{cip::CipCoords, ecliptic::MeanObliquity, nutation::Nutation};

pub mod cio;
pub mod cip;
pub mod earth_rotation;
pub mod ecliptic;
pub mod fundamental;
pub mod nutation;
pub mod polar_motion;
pub mod precession;
pub mod tio;

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Iers1996 {}
    impl Sealed for super::Iers2003 {}
    impl Sealed for super::Iers2010 {}
    impl Sealed for super::ReferenceSystem {}
}

pub trait IersSystem: sealed::Sealed {
    fn id(&self) -> usize;
    fn name(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Iers1996;

impl IersSystem for Iers1996 {
    fn id(&self) -> usize {
        0
    }

    fn name(&self) -> String {
        "IERS1996".to_owned()
    }
}

impl Display for Iers1996 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Iau2000Model {
    #[default]
    A = 1,
    B = 2,
}

impl Display for Iau2000Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Iau2000Model::A => "IAU2000A".fmt(f),
            Iau2000Model::B => "IAU2000B".fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Iers2003(pub Iau2000Model);

impl IersSystem for Iers2003 {
    fn id(&self) -> usize {
        self.0 as usize
    }

    fn name(&self) -> String {
        format!("IERS2003/{}", self.0)
    }
}

impl Display for Iers2003 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Iers2010;

impl IersSystem for Iers2010 {
    fn id(&self) -> usize {
        3
    }

    fn name(&self) -> String {
        "IERS2010".to_owned()
    }
}

impl Display for Iers2010 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReferenceSystem {
    Iers1996,
    Iers2003(Iau2000Model),
    Iers2010,
}

impl IersSystem for ReferenceSystem {
    fn id(&self) -> usize {
        match self {
            ReferenceSystem::Iers1996 => Iers1996.id(),
            ReferenceSystem::Iers2003(iau2000) => Iers2003(*iau2000).id(),
            ReferenceSystem::Iers2010 => Iers2010.id(),
        }
    }

    fn name(&self) -> String {
        match self {
            ReferenceSystem::Iers1996 => Iers1996.to_string(),
            ReferenceSystem::Iers2003(model) => Iers2003(*model).to_string(),
            ReferenceSystem::Iers2010 => Iers2010.to_string(),
        }
    }
}

impl Display for ReferenceSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

impl From<Iers1996> for ReferenceSystem {
    fn from(_: Iers1996) -> Self {
        ReferenceSystem::Iers1996
    }
}

impl From<Iers2003> for ReferenceSystem {
    fn from(sys: Iers2003) -> Self {
        ReferenceSystem::Iers2003(sys.0)
    }
}

impl From<Iers2010> for ReferenceSystem {
    fn from(_: Iers2010) -> Self {
        ReferenceSystem::Iers2010
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Corrections(pub Angle, pub Angle);

impl Corrections {
    pub fn is_zero(&self) -> bool {
        self.0.is_zero() && self.1.is_zero()
    }
}

impl Add<Corrections> for Nutation {
    type Output = Self;

    fn add(self, rhs: Corrections) -> Self::Output {
        Self {
            dpsi: self.dpsi + rhs.0,
            deps: self.deps + rhs.1,
        }
    }
}

impl AddAssign<Corrections> for Nutation {
    fn add_assign(&mut self, rhs: Corrections) {
        self.dpsi += rhs.0;
        self.deps += rhs.1;
    }
}

impl Add<Corrections> for CipCoords {
    type Output = Self;

    fn add(self, rhs: Corrections) -> Self::Output {
        Self {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

impl AddAssign<Corrections> for CipCoords {
    fn add_assign(&mut self, rhs: Corrections) {
        self.x += rhs.0;
        self.y += rhs.1;
    }
}

impl ReferenceSystem {
    pub fn ecliptic_corrections(
        &self,
        corr: Corrections,
        nut: Nutation,
        epsa: MeanObliquity,
        rpb: DMat3,
    ) -> Corrections {
        match self {
            ReferenceSystem::Iers1996 => corr,
            ReferenceSystem::Iers2003(_) | ReferenceSystem::Iers2010 => {
                let Corrections(dx, dy) = corr;
                let rbpn = nut.nutation_matrix(epsa) * rpb;
                let v1 = DVec3::new(dx.as_f64(), dy.as_f64(), 0.0);
                let v2 = rbpn * v1;
                Corrections(Angle::new(v2.x / epsa.0.sin()), Angle::new(v2.y))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Iers1996, 0)]
    #[case(Iers2003(Iau2000Model::A), 1)]
    #[case(Iers2003(Iau2000Model::B), 2)]
    #[case(Iers2010, 3)]
    #[case(ReferenceSystem::Iers1996, 0)]
    #[case(ReferenceSystem::Iers2003(Iau2000Model::A), 1)]
    #[case(ReferenceSystem::Iers2003(Iau2000Model::B), 2)]
    #[case(ReferenceSystem::Iers2010, 3)]
    fn test_iers_convention_id<T: IersSystem>(#[case] iers: T, #[case] exp: usize) {
        let act = iers.id();
        assert_eq!(act, exp);
    }
}
