// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2024 Andrei Zisu <matzipan@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Parsers for astrodynamics data file formats.
//!
//! Currently this crate provides readers for NAIF SPICE text kernels (see the [`spice`] module).

pub mod spice;
