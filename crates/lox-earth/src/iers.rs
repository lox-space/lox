// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Iers1996 {}
    impl Sealed for super::Iers2003 {}
    impl Sealed for super::Iers2010 {}
    impl Sealed for super::IersConvention {}
}

pub trait IersConventionId: sealed::Sealed {
    fn id(&self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iers1996;

impl IersConventionId for Iers1996 {
    fn id(&self) -> usize {
        0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Iau2000 {
    #[default]
    A = 1,
    B = 2,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Iers2003(pub Iau2000);

impl IersConventionId for Iers2003 {
    fn id(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iers2010;

impl IersConventionId for Iers2010 {
    fn id(&self) -> usize {
        3
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IersConvention {
    Iers1996,
    Iers2003(Iau2000),
    Iers2010,
}

impl IersConventionId for IersConvention {
    fn id(&self) -> usize {
        match self {
            IersConvention::Iers1996 => Iers1996.id(),
            IersConvention::Iers2003(iau2000) => Iers2003(*iau2000).id(),
            IersConvention::Iers2010 => Iers2010.id(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Iers1996, 0)]
    #[case(Iers2003(Iau2000::A), 1)]
    #[case(Iers2003(Iau2000::B), 2)]
    #[case(Iers2010, 3)]
    #[case(IersConvention::Iers1996, 0)]
    #[case(IersConvention::Iers2003(Iau2000::A), 1)]
    #[case(IersConvention::Iers2003(Iau2000::B), 2)]
    #[case(IersConvention::Iers2010, 3)]
    fn test_iers_convention_id<T: IersConventionId>(#[case] iers: T, #[case] exp: usize) {
        let act = iers.id();
        assert_eq!(act, exp);
    }
}
