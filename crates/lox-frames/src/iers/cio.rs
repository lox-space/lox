// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-FileCopyrightText: 2024 Angus Morrison <github@angus-morrison.com>
//
// SPDX-License-Identifier: MPL-2.0

//! Module cio exposes functions for calculating the Celestial Intermediate Origin (CIO) locator, s.

use lox_core::units::Angle;
use lox_test_utils::ApproxEq;

pub mod iau2006;

#[derive(Debug, Clone, Copy, Default, PartialEq, ApproxEq)]
pub struct CioLocator(pub Angle);
