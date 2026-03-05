// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;
use std::hash::Hash;

use lox_bodies::{DynOrigin, Origin};
use lox_frames::{DynFrame, ReferenceFrame};
use lox_time::time_scales::{DynTimeScale, TimeScale};

use super::Trajectory;

/// A collection of named trajectories keyed by an identifier type.
#[derive(Debug, Clone)]
pub struct Ensemble<K, T: TimeScale, O: Origin, R: ReferenceFrame>(
    pub HashMap<K, Trajectory<T, O, R>>,
)
where
    K: Eq + Hash;

/// A dynamically-typed ensemble with runtime time scale, origin, and frame.
pub type DynEnsemble<K> = Ensemble<K, DynTimeScale, DynOrigin, DynFrame>;

impl<K, T, O, R> Ensemble<K, T, O, R>
where
    K: Eq + Hash,
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    /// Creates a new ensemble from a map of trajectories.
    pub fn new(map: HashMap<K, Trajectory<T, O, R>>) -> Self {
        Self(map)
    }

    /// Returns a reference to the trajectory for the given key, if present.
    pub fn get(&self, key: &K) -> Option<&Trajectory<T, O, R>> {
        self.0.get(key)
    }

    /// Inserts a trajectory with the given key.
    pub fn insert(&mut self, key: K, trajectory: Trajectory<T, O, R>) {
        self.0.insert(key, trajectory);
    }

    /// Returns an iterator over all key-trajectory pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &Trajectory<T, O, R>)> {
        self.0.iter()
    }

    /// Returns the number of trajectories in the ensemble.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the ensemble contains no trajectories.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_bodies::DynOrigin;
    use lox_frames::DynFrame;
    use lox_time::time_scales::DynTimeScale;

    type TestEnsemble = Ensemble<String, DynTimeScale, DynOrigin, DynFrame>;

    fn make_trajectory() -> Trajectory<DynTimeScale, DynOrigin, DynFrame> {
        Trajectory::from_csv_dyn(
            &lox_test_utils::read_data_file("trajectory_lunar.csv"),
            DynOrigin::Earth,
            DynFrame::Icrf,
        )
        .unwrap()
    }

    #[test]
    fn test_new_empty() {
        let ensemble = TestEnsemble::new(HashMap::new());
        assert!(ensemble.is_empty());
        assert_eq!(ensemble.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut ensemble = TestEnsemble::new(HashMap::new());
        let traj = make_trajectory();
        ensemble.insert("sc1".to_string(), traj);
        assert_eq!(ensemble.len(), 1);
        assert!(!ensemble.is_empty());
        assert!(ensemble.get(&"sc1".to_string()).is_some());
        assert!(ensemble.get(&"sc2".to_string()).is_none());
    }

    #[test]
    fn test_iter() {
        let mut map = HashMap::new();
        map.insert("sc1".to_string(), make_trajectory());
        let ensemble = TestEnsemble::new(map);
        let keys: Vec<_> = ensemble.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(keys, vec!["sc1".to_string()]);
    }
}
