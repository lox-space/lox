// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use lox_time::intervals::TimeInterval;
use lox_time::time_scales::Tai;

use crate::assets::AssetId;
use crate::imaging::aoi::AoiId;

type IntervalMap = HashMap<(AssetId, AoiId), Vec<TimeInterval<Tai>>>;

/// Results of an access analysis.
pub struct AccessResults {
    intervals: IntervalMap,
}

impl AccessResults {
    pub(super) fn new(intervals: IntervalMap) -> Self {
        Self { intervals }
    }

    /// Returns access intervals for a specific (spacecraft, AOI) pair.
    pub fn intervals(&self, sc_id: &AssetId, aoi_id: &AoiId) -> &[TimeInterval<Tai>] {
        self.intervals
            .get(&(sc_id.clone(), aoi_id.clone()))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns an iterator over all (spacecraft, AOI) pairs and their intervals.
    pub fn all_intervals(&self) -> &IntervalMap {
        &self.intervals
    }

    /// Returns `true` if no access intervals were found.
    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    /// Returns the number of (spacecraft, AOI) pairs.
    pub fn num_pairs(&self) -> usize {
        self.intervals.len()
    }
}
