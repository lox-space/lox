// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use lox_time::intervals::TimeInterval;
use lox_time::time_scales::Tai;

use crate::assets::AssetId;
use crate::imaging::aoi::AoiId;

/// Direction of the spacecraft's orbital motion at the time of an access window.
///
/// Determined from the sign of the geodetic-latitude rate at the window midpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PassDirection {
    /// Spacecraft moving northward at the access midpoint.
    Ascending,
    /// Spacecraft moving southward at the access midpoint.
    Descending,
}

/// A single access window — a time interval annotated with the spacecraft's
/// pass direction at the time of access.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccessWindow {
    /// The access time interval.
    pub interval: TimeInterval<Tai>,
    /// Spacecraft pass direction at the interval midpoint.
    pub direction: PassDirection,
}

type WindowMap = HashMap<(AssetId, AoiId), Vec<AccessWindow>>;

/// Results of an access analysis.
pub struct AccessResults {
    windows: WindowMap,
}

impl AccessResults {
    pub(super) fn new(windows: WindowMap) -> Self {
        Self { windows }
    }

    /// Returns access windows for a specific (spacecraft, AOI) pair.
    pub fn windows(&self, sc_id: &AssetId, aoi_id: &AoiId) -> &[AccessWindow] {
        self.windows
            .get(&(sc_id.clone(), aoi_id.clone()))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns all (spacecraft, AOI) pairs and their access windows.
    pub fn all_windows(&self) -> &WindowMap {
        &self.windows
    }

    /// Returns `true` if no access windows were found.
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Returns the number of (spacecraft, AOI) pairs.
    pub fn num_pairs(&self) -> usize {
        self.windows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use lox_time::Time;
    use lox_time::deltas::TimeDelta;

    #[test]
    fn pass_direction_variants_distinct() {
        assert_ne!(PassDirection::Ascending, PassDirection::Descending);
    }

    #[test]
    fn access_window_carries_interval_and_direction() {
        let start = Time::j2000(Tai);
        let end = start + TimeDelta::from_seconds(60);
        let interval = TimeInterval::new(start, end);
        let window = AccessWindow {
            interval,
            direction: PassDirection::Ascending,
        };
        assert_eq!(window.direction, PassDirection::Ascending);
        assert_eq!(window.interval.start(), start);
    }
}
