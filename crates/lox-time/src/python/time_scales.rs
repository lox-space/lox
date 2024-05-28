/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::prelude::{Tai, Tcb, Tcg, Tdb, TimeScale, Tt, Ut1};
use pyo3::exceptions::PyValueError;
use pyo3::PyErr;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum PyTimeScale {
    Tai,
    Tcb,
    Tcg,
    Tdb,
    Tt,
    Ut1,
}

impl FromStr for PyTimeScale {
    type Err = PyErr;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "TAI" => Ok(PyTimeScale::Tai),
            "TCB" => Ok(PyTimeScale::Tcb),
            "TCG" => Ok(PyTimeScale::Tcg),
            "TDB" => Ok(PyTimeScale::Tdb),
            "TT" => Ok(PyTimeScale::Tt),
            "UT1" => Ok(PyTimeScale::Ut1),
            _ => Err(PyValueError::new_err(format!(
                "invalid time scale: {}",
                name
            ))),
        }
    }
}

impl TimeScale for PyTimeScale {
    fn abbreviation(&self) -> &'static str {
        match self {
            PyTimeScale::Tai => Tai.abbreviation(),
            PyTimeScale::Tcb => Tcb.abbreviation(),
            PyTimeScale::Tcg => Tcg.abbreviation(),
            PyTimeScale::Tdb => Tdb.abbreviation(),
            PyTimeScale::Tt => Tt.abbreviation(),
            PyTimeScale::Ut1 => Ut1.abbreviation(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            PyTimeScale::Tai => Tai.name(),
            PyTimeScale::Tcb => Tcb.name(),
            PyTimeScale::Tcg => Tcg.name(),
            PyTimeScale::Tdb => Tdb.name(),
            PyTimeScale::Tt => Tt.name(),
            PyTimeScale::Ut1 => Ut1.name(),
        }
    }
}

impl Display for PyTimeScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("TAI", "International Atomic Time")]
    #[case("TT", "Terrestrial Time")]
    #[case("TCG", "Geocentric Coordinate Time")]
    #[case("TCB", "Barycentric Coordinate Time")]
    #[case("TDB", "Barycentric Dynamical Time")]
    #[case("UT1", "Universal Time")]
    #[should_panic(expected = "invalid time scale: NotATimeScale")]
    #[case("NotATimeScale", "not a time scale")]
    fn test_pytimescale(#[case] abbreviation: &'static str, #[case] name: &'static str) {
        let scale = PyTimeScale::from_str(abbreviation).unwrap();
        assert_eq!(scale.abbreviation(), abbreviation);
        assert_eq!(scale.name(), name);
        assert_eq!(scale.to_string(), abbreviation);
    }
}
