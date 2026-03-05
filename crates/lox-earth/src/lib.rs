// SPDX-FileCopyrightText: 2023 Andrei Zisu <matzipan@gmail.com>
// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Earth orientation parameters, low-precision ephemeris, and tidal corrections.

/// Earth orientation parameters (EOP) parsing and interpolation.
pub mod eop;
/// Low-precision analytical ephemeris of the Earth.
pub mod ephemeris;
/// Trait implementations connecting [`eop::EopProvider`] to the time and frames crates.
pub mod providers;
/// Diurnal and subdiurnal tidal corrections to polar motion and UT1-UTC.
#[allow(dead_code)]
pub mod tides;
