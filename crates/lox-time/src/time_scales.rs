// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

/*!
    Module `time_scales` provides a marker trait denoting a continuous astronomical time scale,
    along with zero-sized implementations for the most commonly used scales.

    # Utc

    As a discontinuous time scale, [Utc] does not implement [TimeScale] and is treated by Lox
    exclusively as an IO format.
*/

use std::fmt::Display;
use std::str::FromStr;

use thiserror::Error;

/// Marker trait denoting a continuous astronomical time scale.
pub trait TimeScale {
    fn abbreviation(&self) -> &'static str;
    fn name(&self) -> &'static str;
}

/// International Atomic Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tai;

impl TimeScale for Tai {
    fn abbreviation(&self) -> &'static str {
        "TAI"
    }
    fn name(&self) -> &'static str {
        "International Atomic Time"
    }
}

impl Display for Tai {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Barycentric Coordinate Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tcb;

impl TimeScale for Tcb {
    fn abbreviation(&self) -> &'static str {
        "TCB"
    }
    fn name(&self) -> &'static str {
        "Barycentric Coordinate Time"
    }
}

impl Display for Tcb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Geocentric Coordinate Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tcg;

impl TimeScale for Tcg {
    fn abbreviation(&self) -> &'static str {
        "TCG"
    }
    fn name(&self) -> &'static str {
        "Geocentric Coordinate Time"
    }
}

impl Display for Tcg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Barycentric Dynamical Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tdb;

impl TimeScale for Tdb {
    fn abbreviation(&self) -> &'static str {
        "TDB"
    }
    fn name(&self) -> &'static str {
        "Barycentric Dynamical Time"
    }
}

impl Display for Tdb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Terrestrial Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tt;

impl TimeScale for Tt {
    fn abbreviation(&self) -> &'static str {
        "TT"
    }
    fn name(&self) -> &'static str {
        "Terrestrial Time"
    }
}

impl Display for Tt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Universal Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ut1;

impl TimeScale for Ut1 {
    fn abbreviation(&self) -> &'static str {
        "UT1"
    }
    fn name(&self) -> &'static str {
        "Universal Time"
    }
}

impl Display for Ut1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DynTimeScale {
    #[default]
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

impl From<Tai> for DynTimeScale {
    fn from(_: Tai) -> Self {
        Self::Tai
    }
}

impl From<Tcb> for DynTimeScale {
    fn from(_: Tcb) -> Self {
        Self::Tcb
    }
}

impl From<Tcg> for DynTimeScale {
    fn from(_: Tcg) -> Self {
        Self::Tcg
    }
}

impl From<Tdb> for DynTimeScale {
    fn from(_: Tdb) -> Self {
        Self::Tdb
    }
}

impl From<Tt> for DynTimeScale {
    fn from(_: Tt) -> Self {
        Self::Tt
    }
}

impl From<Ut1> for DynTimeScale {
    fn from(_: Ut1) -> Self {
        Self::Ut1
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("unknown time scale: {0}")]
pub struct UnknownTimeScaleError(String);

impl FromStr for DynTimeScale {
    type Err = UnknownTimeScaleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tai" | "TAI" => Ok(DynTimeScale::Tai),
            "tcb" | "TCB" => Ok(DynTimeScale::Tcb),
            "tcg" | "TCG" => Ok(DynTimeScale::Tcg),
            "tdb" | "TDB" => Ok(DynTimeScale::Tdb),
            "tt" | "TT" => Ok(DynTimeScale::Tt),
            "ut1" | "UT1" => Ok(DynTimeScale::Ut1),
            _ => Err(UnknownTimeScaleError(s.to_owned())),
        }
    }
}

impl Display for DynTimeScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
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
    fn test_time_scales<T: TimeScale + ToString>(
        #[case] scale: T,
        #[case] abbreviation: &'static str,
        #[case] name: &'static str,
    ) {
        assert_eq!(scale.abbreviation(), abbreviation);
        assert_eq!(scale.to_string(), abbreviation);
        assert_eq!(scale.name(), name);
    }

    #[rstest]
    #[case("TAI", "International Atomic Time")]
    #[case("TCB", "Barycentric Coordinate Time")]
    #[case("TCG", "Geocentric Coordinate Time")]
    #[case("TDB", "Barycentric Dynamical Time")]
    #[case("TT", "Terrestrial Time")]
    #[case("UT1", "Universal Time")]
    fn test_dyn_time_scale(#[case] abbreviation: &str, #[case] name: &str) {
        let scale: DynTimeScale = abbreviation.parse().unwrap();
        assert_eq!(scale.abbreviation(), abbreviation);
        assert_eq!(scale.to_string(), abbreviation);
        assert_eq!(scale.name(), name);
    }

    #[test]
    fn test_dyn_time_scale_invalid() {
        let scale: Result<DynTimeScale, UnknownTimeScaleError> = "NTS".parse();
        assert_eq!(scale, Err(UnknownTimeScaleError("NTS".to_owned())))
    }
}
