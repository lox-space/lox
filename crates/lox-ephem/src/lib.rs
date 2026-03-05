// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![warn(missing_docs)]

//! Ephemeris providers for solar system bodies, including SPK/DAF file support.

use arrayvec::ArrayVec;
use glam::DVec3;
use lox_bodies::Origin;
use lox_core::coords::Cartesian;
use lox_time::{Time, time_scales::Tdb};

/// SPICE SPK/DAF file parser and ephemeris implementation.
pub mod spk;

/// Returns the state (position and velocity) of `target` relative to `origin`
/// at the given TDB epoch, in ICRF, in SI units (meters, m/s).
///
/// Implementations handle path resolution for non-adjacent body pairs
/// (e.g. Earth relative to Mars goes via the barycenters automatically).
pub trait Ephemeris {
    /// The error type returned by ephemeris lookups.
    type Error: std::error::Error + Send + Sync;

    /// Returns the state (position and velocity) of `target` relative to `origin`.
    fn state<O1: Origin, O2: Origin>(
        &self,
        time: Time<Tdb>,
        origin: O1,
        target: O2,
    ) -> Result<Cartesian, Self::Error>;

    /// Returns only the position of `target` relative to `origin`.
    /// Default implementation delegates to `state()`.
    fn position<O1: Origin, O2: Origin>(
        &self,
        time: Time<Tdb>,
        origin: O1,
        target: O2,
    ) -> Result<DVec3, Self::Error> {
        Ok(self.state(time, origin, target)?.position())
    }
}

fn ancestors(id: i32) -> ArrayVec<i32, 8> {
    let mut ancestors = ArrayVec::new();
    ancestors.push(id);
    let mut current = id;
    while current != 0 {
        current /= 100;
        ancestors.push(current);
    }
    ancestors
}

pub(crate) fn path_from_ids(origin: i32, target: i32) -> ArrayVec<i32, 8> {
    let ancestors_origin = ancestors(origin);
    let ancestors_target = ancestors(target);
    let n = ancestors_target.len();
    let mut path = ancestors_origin;

    ancestors_target
        .into_iter()
        .take(n - 1)
        .rev()
        .for_each(|id| path.push(id));

    if *path.first().unwrap() != 0 && *path.last().unwrap() != 0 {
        let idx = path.iter().position(|&id| id == 0).unwrap();
        if path[idx - 1] == path[idx + 1] {
            // Remove the duplicate common ancestor: [... A, 0, A, ...] → [... A, ...]
            path.remove(idx + 1);
            path.remove(idx);
        }
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ancestors() {
        let expected_0: ArrayVec<i32, 8> = [0].into_iter().collect();
        let expected_3: ArrayVec<i32, 8> = [3, 0].into_iter().collect();
        let expected_399: ArrayVec<i32, 8> = [399, 3, 0].into_iter().collect();
        assert_eq!(ancestors(0), expected_0);
        assert_eq!(ancestors(3), expected_3);
        assert_eq!(ancestors(399), expected_399);
    }

    #[test]
    fn test_path_from_ids() {
        assert_eq!(path_from_ids(399, 499).as_slice(), [399, 3, 0, 4, 499]);
        assert_eq!(path_from_ids(399, 0).as_slice(), [399, 3, 0]);
        assert_eq!(path_from_ids(0, 399).as_slice(), [0, 3, 399]);
        assert_eq!(path_from_ids(399, 3).as_slice(), [399, 3]);
        assert_eq!(path_from_ids(3, 399).as_slice(), [3, 399]);
        assert_eq!(path_from_ids(399, 301).as_slice(), [399, 3, 301]);
    }
}
