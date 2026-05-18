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

use alloc::borrow::ToOwned;
use alloc::string::String;
use core::fmt::Display;
use core::str::FromStr;

use thiserror::Error;

/// Marker trait denoting a continuous astronomical time scale.
pub trait TimeScale {
    /// Returns the standard abbreviation of this time scale (e.g. `"TAI"`).
    fn abbreviation(&self) -> &'static str;
    /// Returns the full name of this time scale (e.g. `"International Atomic Time"`).
    fn name(&self) -> &'static str;
}

/// International Atomic Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Barycentric Coordinate Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Geocentric Coordinate Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Barycentric Dynamical Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// GPS time. A continuous atomic timescale used by the Global Positioning
/// System. Related to TAI by a constant offset: `TAI = GPS + 19s`.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
pub struct Gps;

impl TimeScale for Gps {
    fn abbreviation(&self) -> &'static str {
        "GPS"
    }
    fn name(&self) -> &'static str {
        "Global Positioning System time"
    }
}

impl Display for Gps {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Terrestrial Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Universal Time.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

// -- serde: serialize time scale ZSTs as their abbreviation --

macro_rules! impl_time_scale_serde {
    ($ty:ident, $abbrev:literal) => {
        #[cfg(feature = "serde")]
        impl From<$ty> for &'static str {
            fn from(_: $ty) -> Self {
                $abbrev
            }
        }

        #[cfg(feature = "serde")]
        impl TryFrom<String> for $ty {
            type Error = String;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                if s == $abbrev {
                    Ok($ty)
                } else {
                    Err(format!("expected \"{}\", got \"{}\"", $abbrev, s))
                }
            }
        }
    };
}

impl_time_scale_serde!(Tai, "TAI");
impl_time_scale_serde!(Tcb, "TCB");
impl_time_scale_serde!(Tcg, "TCG");
impl_time_scale_serde!(Tdb, "TDB");
impl_time_scale_serde!(Gps, "GPS");
impl_time_scale_serde!(Tt, "TT");
impl_time_scale_serde!(Ut1, "UT1");

/// Dynamic time scale selector for runtime-determined time scales.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DynTimeScale {
    /// GPS Time.
    Gps,
    /// International Atomic Time.
    #[default]
    Tai,
    /// Barycentric Coordinate Time.
    Tcb,
    /// Geocentric Coordinate Time.
    Tcg,
    /// Barycentric Dynamical Time.
    Tdb,
    /// Terrestrial Time.
    Tt,
    /// Universal Time.
    Ut1,
}

impl TimeScale for DynTimeScale {
    fn abbreviation(&self) -> &'static str {
        match self {
            DynTimeScale::Gps => Gps.abbreviation(),
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
            DynTimeScale::Gps => Gps.name(),
            DynTimeScale::Tai => Tai.name(),
            DynTimeScale::Tcb => Tcb.name(),
            DynTimeScale::Tcg => Tcg.name(),
            DynTimeScale::Tdb => Tdb.name(),
            DynTimeScale::Tt => Tt.name(),
            DynTimeScale::Ut1 => Ut1.name(),
        }
    }
}

impl From<Gps> for DynTimeScale {
    fn from(_: Gps) -> Self {
        Self::Gps
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

/// Error returned when parsing an unknown time scale abbreviation.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("unknown time scale: {0}")]
pub struct UnknownTimeScaleError(String);

impl FromStr for DynTimeScale {
    type Err = UnknownTimeScaleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gps" | "GPS" => Ok(DynTimeScale::Gps),
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

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

    #[test]
    fn gps_time_scale_abbreviation_and_name() {
        assert_eq!(Gps.abbreviation(), "GPS");
        assert_eq!(Gps.name(), "Global Positioning System time");
    }

    #[test]
    fn dyn_time_scale_gps_abbreviation() {
        assert_eq!(DynTimeScale::Gps.abbreviation(), "GPS");
    }

    #[test]
    fn dyn_time_scale_parses_gps_both_cases() {
        assert_eq!("GPS".parse::<DynTimeScale>().unwrap(), DynTimeScale::Gps);
        assert_eq!("gps".parse::<DynTimeScale>().unwrap(), DynTimeScale::Gps);
    }

    #[test]
    fn dyn_time_scale_rejects_unknown() {
        assert!("XYZ".parse::<DynTimeScale>().is_err());
    }
}
