// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Quantities specific to orbital mechanics.

use lox_approx::ApproxEq;

use crate::units::{quantity_impls, runtime_units};

/// Specific energy in J/kg (m²/s²), such as the specific orbital energy.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct SpecificEnergy(f64);

impl SpecificEnergy {
    /// Creates a new specific energy from an `f64` value in m²/s² (J/kg).
    pub const fn square_meters_per_second_squared(m2_per_s2: f64) -> Self {
        Self(m2_per_s2)
    }

    /// Creates a new specific energy from an `f64` value in km²/s².
    pub const fn square_kilometers_per_second_squared(km2_per_s2: f64) -> Self {
        Self(km2_per_s2 * 1e6)
    }

    /// Returns the value in m²/s² (J/kg).
    pub const fn to_square_meters_per_second_squared(self) -> f64 {
        self.0
    }

    /// Returns the value in km²/s².
    pub const fn to_square_kilometers_per_second_squared(self) -> f64 {
        self.0 / 1e6
    }
}

/// Specific angular momentum in m²/s.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, ApproxEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct SpecificAngularMomentum(f64);

impl SpecificAngularMomentum {
    /// Creates a new specific angular momentum from an `f64` value in m²/s.
    pub const fn square_meters_per_second(m2_per_s: f64) -> Self {
        Self(m2_per_s)
    }

    /// Creates a new specific angular momentum from an `f64` value in km²/s.
    pub const fn square_kilometers_per_second(km2_per_s: f64) -> Self {
        Self(km2_per_s * 1e6)
    }

    /// Returns the value in m²/s.
    pub const fn to_square_meters_per_second(self) -> f64 {
        self.0
    }

    /// Returns the value in km²/s.
    pub const fn to_square_kilometers_per_second(self) -> f64 {
        self.0 / 1e6
    }
}

quantity_impls!(SpecificAngularMomentum, SpecificEnergy);

runtime_units! {
    SpecificEnergy, SpecificEnergyUnit, si = SquareMetersPerSecondSquared {
        SquareMetersPerSecondSquared, "m²/s²" | "m2/s2" | "m^2/s^2" | "J/kg",
            square_meters_per_second_squared, to_square_meters_per_second_squared;
        SquareKilometersPerSecondSquared, "km²/s²" | "km2/s2" | "km^2/s^2",
            square_kilometers_per_second_squared, to_square_kilometers_per_second_squared;
    }
    SpecificAngularMomentum, SpecificAngularMomentumUnit, si = SquareMetersPerSecond {
        SquareMetersPerSecond, "m²/s" | "m2/s" | "m^2/s", square_meters_per_second,
            to_square_meters_per_second;
        SquareKilometersPerSecond, "km²/s" | "km2/s" | "km^2/s", square_kilometers_per_second,
            to_square_kilometers_per_second;
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_specific_energy_units() {
        let e = SpecificEnergy::square_kilometers_per_second_squared(-29.6);
        assert_eq!(e.to_square_meters_per_second_squared(), -29.6e6);
        assert_eq!(e.to_square_kilometers_per_second_squared(), -29.6);
        assert_eq!(e.to_string(), "-29600000 m²/s²");
        assert_eq!("-29.6 km²/s²".parse(), Ok(e));
        assert_eq!(
            "5 J/kg".parse(),
            Ok(SpecificEnergy::square_meters_per_second_squared(5.0))
        );
    }

    #[test]
    fn test_specific_angular_momentum_units() {
        let h = SpecificAngularMomentum::square_kilometers_per_second(52_000.0);
        assert_eq!(h.to_square_meters_per_second(), 5.2e10);
        assert_eq!(h.to_square_kilometers_per_second(), 52_000.0);
        assert_eq!(
            h.to(SpecificAngularMomentumUnit::SquareKilometersPerSecond),
            52_000.0
        );
        assert_eq!(h * 2.0 - h, h);
    }
}
