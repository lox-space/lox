// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Core types, constants, and utilities for the Lox Astrodynamics Toolkit.
//!
//! This crate provides the foundational building blocks used across Lox:
//! physical units, coordinate types, orbital elements, time representation,
//! mathematical utilities, and numerical constants.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

extern crate alloc;

pub mod anomalies;
pub mod comms;
pub mod coords;
pub mod elements;
pub mod f64;
pub mod glam;
pub mod i32;
pub mod i64;
pub mod math;
pub mod time;
pub mod types;
pub mod units;
pub mod utils;
