// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Mean Keplerian elements representation.
//!
//! `MeanElements` is a plain data struct used by analytical propagators
//! (e.g. Kozai-based J2/J4) and by file-format adapters that need to
//! exchange mean orbital element sets (e.g. CCSDS OMM messages).

/// Mean Keplerian elements at a given epoch.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MeanElements {
    /// Semi-major axis \[m\].
    pub a: f64,
    /// Eccentricity.
    pub e: f64,
    /// Inclination \[rad\].
    pub i: f64,
    /// RAAN \[rad\].
    pub raan: f64,
    /// Argument of periapsis \[rad\].
    pub aop: f64,
    /// Mean anomaly \[rad\].
    pub m: f64,
}
