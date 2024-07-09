/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

/*!
    Module `time_scales` provides a marker trait denoting a continuous astronomical time scale,
    along with zero-sized implementations for the most commonly used scales.

    # Utc

    As a discontinuous time scale, [Utc] does not implement [TimeScale] and is treated by Lox
    exclusively as an IO format.
*/

use std::str::FromStr;

use thiserror::Error;

pub mod transformations;

/// Marker trait denoting a continuous astronomical time scale.
pub trait TimeScale {
    fn abbreviation(&self) -> &'static str;
    fn name(&self) -> &'static str;
}

/// International Atomic Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Tai;

impl TimeScale for Tai {
    fn abbreviation(&self) -> &'static str {
        "TAI"
    }
    fn name(&self) -> &'static str {
        "International Atomic Time"
    }
}

/// Barycentric Coordinate Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Tcb;

impl TimeScale for Tcb {
    fn abbreviation(&self) -> &'static str {
        "TCB"
    }
    fn name(&self) -> &'static str {
        "Barycentric Coordinate Time"
    }
}

/// Geocentric Coordinate Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Tcg;

impl TimeScale for Tcg {
    fn abbreviation(&self) -> &'static str {
        "TCG"
    }
    fn name(&self) -> &'static str {
        "Geocentric Coordinate Time"
    }
}

/// Barycentric Dynamical Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Tdb;

impl TimeScale for Tdb {
    fn abbreviation(&self) -> &'static str {
        "TDB"
    }
    fn name(&self) -> &'static str {
        "Barycentric Dynamical Time"
    }
}

/// Terrestrial Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Tt;

impl TimeScale for Tt {
    fn abbreviation(&self) -> &'static str {
        "TT"
    }
    fn name(&self) -> &'static str {
        "Terrestrial Time"
    }
}

/// Universal Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Ut1;

impl TimeScale for Ut1 {
    fn abbreviation(&self) -> &'static str {
        "UT1"
    }
    fn name(&self) -> &'static str {
        "Universal Time"
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum DynTimeScale {
    Tai,
    Tcb,
    Tcg,
    Tdb,
    Tt,
    Ut1,
}

impl TimeScale for DynTimeScale {
    fn abbreviation(&self) -> &'static str {
        match self {
            DynTimeScale::Tai => Tai.abbreviation(),
            DynTimeScale::Tcb => Tcb.abbreviation(),
            DynTimeScale::Tcg => Tcg.abbreviation(),
            DynTimeScale::Tdb => Tdb.abbreviation(),
            DynTimeScale::Tt => Tt.abbreviation(),
            DynTimeScale::Ut1 => Ut1.abbreviation(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            DynTimeScale::Tai => Tai.name(),
            DynTimeScale::Tcb => Tcb.name(),
            DynTimeScale::Tcg => Tcg.name(),
            DynTimeScale::Tdb => Tdb.name(),
            DynTimeScale::Tt => Tt.name(),
            DynTimeScale::Ut1 => Ut1.name(),
        }
    }
}

#[derive(Error, Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[error("invalid time scale: {0}")]
pub struct InvalidTimeScale(String);

impl FromStr for DynTimeScale {
    type Err = InvalidTimeScale;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "TAI" => Ok(DynTimeScale::Tai),
            "TCB" => Ok(DynTimeScale::Tcb),
            "TCG" => Ok(DynTimeScale::Tcg),
            "TDB" => Ok(DynTimeScale::Tdb),
            "TT" => Ok(DynTimeScale::Tt),
            "UT1" => Ok(DynTimeScale::Ut1),
            _ => Err(InvalidTimeScale(name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Tai, "TAI", "International Atomic Time")]
    #[case(Tcb, "TCB", "Barycentric Coordinate Time")]
    #[case(Tcg, "TCG", "Geocentric Coordinate Time")]
    #[case(Tdb, "TDB", "Barycentric Dynamical Time")]
    #[case(Tt, "TT", "Terrestrial Time")]
    #[case(Ut1, "UT1", "Universal Time")]
    fn test_time_scales<T: TimeScale>(
        #[case] scale: T,
        #[case] abbreviation: &'static str,
        #[case] name: &'static str,
    ) {
        assert_eq!(scale.abbreviation(), abbreviation);
        assert_eq!(scale.name(), name);
    }

    #[rstest]
    #[case("TAI", "International Atomic Time")]
    #[case("TT", "Terrestrial Time")]
    #[case("TCG", "Geocentric Coordinate Time")]
    #[case("TCB", "Barycentric Coordinate Time")]
    #[case("TDB", "Barycentric Dynamical Time")]
    #[case("UT1", "Universal Time")]
    #[should_panic(expected = "invalid time scale: NotATimeScale")]
    #[case("NotATimeScale", "not a time scale")]
    fn test_dyn_time_scale(#[case] abbreviation: &'static str, #[case] name: &'static str) {
        let scale = DynTimeScale::from_str(abbreviation).unwrap();
        assert_eq!(scale.abbreviation(), abbreviation);
        assert_eq!(scale.name(), name);
    }
}
