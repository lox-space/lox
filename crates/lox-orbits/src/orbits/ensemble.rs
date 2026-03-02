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

pub type DynEnsemble<K> = Ensemble<K, DynTimeScale, DynOrigin, DynFrame>;

impl<K, T, O, R> Ensemble<K, T, O, R>
where
    K: Eq + Hash,
    T: TimeScale,
    O: Origin,
    R: ReferenceFrame,
{
    pub fn new(map: HashMap<K, Trajectory<T, O, R>>) -> Self {
        Self(map)
    }

    pub fn get(&self, key: &K) -> Option<&Trajectory<T, O, R>> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: K, trajectory: Trajectory<T, O, R>) {
        self.0.insert(key, trajectory);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &Trajectory<T, O, R>)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
