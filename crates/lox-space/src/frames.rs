// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

// Re-export lox_frames submodule contents at this level to avoid stutter
// (e.g. `lox_space::frames::Icrf` instead of `lox_space::frames::frames::Icrf`).
pub use lox_frames::dynamic;
pub use lox_frames::dynamic::*;
pub use lox_frames::frames::*;
pub use lox_frames::iau;
pub use lox_frames::iers;
pub use lox_frames::providers;
pub use lox_frames::rotations;
pub use lox_frames::traits;
pub use lox_frames::traits::*;

#[cfg(feature = "python")]
/// Python bindings exposed via PyO3.
pub mod python;
