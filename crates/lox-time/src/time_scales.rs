/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module `time_scales` provides a marker trait with associated constants denoting a continuous
//! astronomical time scale, along with zero-sized implementations for the most commonly used
//! scales.

/// Marker trait with associated constants denoting a continuous astronomical time scale.
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
