// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Communication systems and link budget analysis.
//!
//! This crate provides types and calculations for modelling radio communication systems,
//! antenna gain patterns, and link budget analysis.

#![warn(missing_docs)]

pub mod antenna;
pub mod channel;
pub mod error;
pub mod link_budget;
pub mod pattern;
pub mod pfd;
pub mod receiver;
pub mod system;
pub mod transmitter;
pub mod utils;

pub use error::LinkBudgetError;

use lox_core::units::Kelvin;

/// Boltzmann constant in J/K (CODATA 2018 / SI 2019 exact value).
pub const BOLTZMANN_CONSTANT: f64 = 1.380_649e-23;

/// Reference room temperature in Kelvin (per ITU-R).
pub const ROOM_TEMPERATURE: Kelvin = 290.0;
