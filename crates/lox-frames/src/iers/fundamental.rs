// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Fundamental astronomical arguments according to various IAU conventions.
//!
//! This module provides functions for calculating fundamental arguments used in
//! nutation, precession, and other Earth orientation calculations. The arguments
//! are primarily the Delaunay arguments (l, l', F, D, Ω) and planetary mean longitudes.
//!
//! ## Naming Convention
//!
//! Functions use canonical single-letter names from astronomical literature:
//! - `l`: Moon's mean anomaly
//! - `lp`: Sun's mean anomaly (l-prime)
//! - `f`: Moon's mean argument of latitude
//! - `d`: Mean elongation of Moon from Sun
//! - `omega`: Mean longitude of Moon's ascending node
//! - `pa`: General accumulated precession in longitude
//! - `{planet}_l`: Planetary mean longitudes
//!
//! Each function is suffixed with the convention name (e.g., `_iers03`, `_mhb2000`, `_simon1994`).

pub mod iers03;
pub mod mhb2000;
pub mod simon1994;
