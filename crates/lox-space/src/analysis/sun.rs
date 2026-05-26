// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Analytical Sun ephemeris adapter.
//!
//! Wraps [`lox_earth::ephemeris::apparent_sun_position`] behind the
//! [`Ephemeris`](lox_ephem::Ephemeris) trait so that Earth-centred analyses can run without an
//! SPK file.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use lox_bodies::{NaifId, Origin};
use lox_core::coords::Cartesian;
use lox_core::glam::DVec3;
use lox_earth::ephemeris::apparent_sun_position;
use lox_ephem::Ephemeris;
use lox_time::{Time, time_scales::Tdb};

/// Error returned by [`AnalyticalSunEphemeris`] for unsupported body pairs.
#[derive(Debug)]
pub struct AnalyticalSunEphemerisError {
    origin_id: NaifId,
    target_id: NaifId,
}

impl Display for AnalyticalSunEphemerisError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "analytical ephemeris only supports Earth<->Sun queries (got {}->{})",
            self.origin_id, self.target_id
        )
    }
}

impl Error for AnalyticalSunEphemerisError {}

/// Ephemeris adapter that uses [`apparent_sun_position`] for Earth↔Sun
/// queries and returns an error for anything else.
///
/// This allows power-budget and eclipse analyses to run without loading
/// an SPK kernel, at the cost of being limited to Earth-centred scenarios.
pub struct AnalyticalSunEphemeris;

impl Ephemeris for AnalyticalSunEphemeris {
    type Error = AnalyticalSunEphemerisError;

    fn state<O1: Origin, O2: Origin>(
        &self,
        time: Time<Tdb>,
        origin: O1,
        target: O2,
    ) -> Result<Cartesian, Self::Error> {
        let origin_id = origin.id();
        let target_id = target.id();
        // Earth = 399, Sun = 10
        if origin_id == NaifId(399) && target_id == NaifId(10) {
            let pos = apparent_sun_position(time);
            Ok(Cartesian::from_vecs(pos, DVec3::ZERO))
        } else if origin_id == NaifId(10) && target_id == NaifId(399) {
            let pos = apparent_sun_position(time);
            Ok(Cartesian::from_vecs(-pos, DVec3::ZERO))
        } else {
            Err(AnalyticalSunEphemerisError {
                origin_id,
                target_id,
            })
        }
    }
}
