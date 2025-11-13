// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_time::OffsetProvider;

#[macro_export]
macro_rules! transform_provider {
    ($provider:ident) => {
        impl $crate::transformations::RotationProvider for $provider {}
    };
}

#[derive(Copy, Clone, Debug, OffsetProvider)]
pub struct DefaultRotationProvider;

transform_provider!(DefaultRotationProvider);
