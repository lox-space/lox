// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
//
// SPDX-License-Identifier: MPL-2.0

// Re-export lox_time submodule contents at this level to avoid stutter
// (e.g. `lox_space::time::Time` instead of `lox_space::time::time::Time`).
pub use lox_time::calendar_dates;
pub use lox_time::deltas;
pub use lox_time::intervals;
pub use lox_time::julian_dates;
pub use lox_time::offsets;
pub use lox_time::series;
pub use lox_time::subsecond;
pub use lox_time::time::*;
pub use lox_time::time_of_day;
pub use lox_time::time_scales;
pub use lox_time::utc;

#[cfg(feature = "python")]
pub mod python;
