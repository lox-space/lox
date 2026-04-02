// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#[cfg(feature = "analysis")]
pub mod analysis;

#[cfg(feature = "bodies")]
pub mod bodies;

#[cfg(feature = "orbits")]
pub mod constellations;

#[cfg(feature = "comms")]
pub mod comms;

#[cfg(feature = "core")]
pub mod core;

#[cfg(feature = "earth")]
pub mod earth;

#[cfg(feature = "ephem")]
pub mod ephem;

#[cfg(feature = "frames")]
pub mod frames;

#[cfg(feature = "io")]
pub mod io;

#[cfg(feature = "itur")]
pub mod itur;

#[cfg(feature = "math")]
pub mod math;

#[cfg(feature = "orbits")]
pub mod orbits;

#[cfg(feature = "time")]
pub mod time;

// Re-export lox_time macros so users can write `lox_space::time!` / `lox_space::utc!`.
#[cfg(feature = "time")]
#[macro_export]
macro_rules! time {
    ($($args:tt)*) => { $crate::__private_time!($($args)*) };
}
#[cfg(feature = "time")]
#[doc(hidden)]
pub use lox_time::time as __private_time;

#[cfg(feature = "time")]
#[macro_export]
macro_rules! utc {
    ($($args:tt)*) => { $crate::__private_utc!($($args)*) };
}
#[cfg(feature = "time")]
#[doc(hidden)]
pub use lox_time::utc as __private_utc;

#[cfg(feature = "units")]
pub mod units;

pub mod prelude;

#[cfg(feature = "python")]
pub mod python;
