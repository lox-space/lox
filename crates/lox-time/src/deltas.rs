// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! [`TimeDelta`] and the [`ToDelta`] trait for time representations relative to J2000.

#[doc(inline)]
pub use lox_core::time::deltas::*;

/// A unifying trait for types that can be converted into a [TimeDelta], the time since J2000.
pub trait ToDelta {
    /// Transforms the value into a [TimeDelta].
    fn to_delta(&self) -> TimeDelta;
}
