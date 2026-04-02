// SPDX-FileCopyrightText: 2016 Inigo del Portillo, Massachusetts Institute of Technology
// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MIT AND MPL-2.0

//! ITU-R P.1510-1: Annual and monthly mean surface temperature.
//!
//! Provides the annual and monthly mean surface temperature for any location on Earth.

use lox_core::units::{Angle, Temperature};

use crate::data::LazyGrid;

static T_ANNUAL: LazyGrid = LazyGrid::new("1510/v1_t_annual.bin.zst");

static T_MONTHS: [LazyGrid; 12] = [
    LazyGrid::new("1510/v1_t_month01.bin.zst"),
    LazyGrid::new("1510/v1_t_month02.bin.zst"),
    LazyGrid::new("1510/v1_t_month03.bin.zst"),
    LazyGrid::new("1510/v1_t_month04.bin.zst"),
    LazyGrid::new("1510/v1_t_month05.bin.zst"),
    LazyGrid::new("1510/v1_t_month06.bin.zst"),
    LazyGrid::new("1510/v1_t_month07.bin.zst"),
    LazyGrid::new("1510/v1_t_month08.bin.zst"),
    LazyGrid::new("1510/v1_t_month09.bin.zst"),
    LazyGrid::new("1510/v1_t_month10.bin.zst"),
    LazyGrid::new("1510/v1_t_month11.bin.zst"),
    LazyGrid::new("1510/v1_t_month12.bin.zst"),
];

/// Returns the annual mean surface temperature at the given location.
pub fn surface_mean_temperature(lat: Angle, lon: Angle) -> Temperature {
    Temperature::kelvin(T_ANNUAL.get().bilinear(lat.to_degrees(), lon.to_degrees()))
}

/// Returns the monthly mean surface temperature at the given location.
///
/// # Panics
///
/// Panics if `month` is not in the range 1–12.
pub fn surface_month_mean_temperature(lat: Angle, lon: Angle, month: u8) -> Temperature {
    assert!((1..=12).contains(&month), "month must be 1–12, got {month}");
    Temperature::kelvin(
        T_MONTHS[(month - 1) as usize]
            .get()
            .bilinear(lat.to_degrees(), lon.to_degrees()),
    )
}
