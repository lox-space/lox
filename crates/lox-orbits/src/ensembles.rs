/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::frames::ReferenceFrame;
use crate::origins::Origin;
use crate::trajectories::Trajectory;
use lox_time::TimeLike;
use std::collections::HashMap;

pub struct Ensemble<T: TimeLike, O: Origin, R: ReferenceFrame>(
    HashMap<String, Trajectory<T, O, R>>,
);

impl<T, O, R> Ensemble<T, O, R>
where
    T: TimeLike + Clone,
    O: Origin + Clone,
    R: ReferenceFrame + Clone,
{
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from_slice(slice: &[(String, Trajectory<T, O, R>)]) -> Self {
        Self(slice.iter().cloned().collect())
    }

    pub fn insert(mut self, name: String, trajectory: Trajectory<T, O, R>) -> Self {
        self.0.insert(name, trajectory);
        self
    }

    pub fn get(&self, name: &str) -> Option<&Trajectory<T, O, R>> {
        self.0.get(name)
    }
}

impl<T, O, R> Default for Ensemble<T, O, R>
where
    T: TimeLike + Clone,
    O: Origin + Clone,
    R: ReferenceFrame + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
