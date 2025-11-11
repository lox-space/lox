#![allow(clippy::type_complexity)]

// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use bevy_ecs::{prelude::*, system::RegisteredSystemError, world::error::EntityMutableFetchError};

use lox_bodies::DynOrigin;
use lox_core::coords::{LonLatAlt, TimeStampedCartesian};
use lox_frames::DynFrame;
use lox_time::{
    Time,
    deltas::TimeDelta,
    intervals::UtcInterval,
    time_scales::{self, Tai},
    utc::Utc,
};

use crate::{
    orbits::{DynOrbit, DynTrajectory},
    propagators::DynPropagator,
};

#[derive(Component)]
struct Asset {
    name: String,
}

#[derive(Component)]
struct Orbit(DynOrbit);

#[derive(Component)]
struct InitialState(TimeStampedCartesian);

#[derive(Component)]
struct Propagator(Box<dyn DynPropagator>);

#[derive(Component, Default)]
struct Trajectory(Option<DynTrajectory>);

#[derive(Bundle)]
struct Spacecraft {
    meta: Asset,
    orbit: Orbit,
    trajectory: Trajectory,
}

#[derive(Resource)]
struct ScenarioData {
    interval: UtcInterval,
    origin: DynOrigin,
    frame: DynFrame,
}

pub struct Scenario(World);

fn propagate(
    scenario: Res<ScenarioData>,
    mut query: Query<(&Orbit, &mut Trajectory), Or<(Changed<Orbit>, Added<Orbit>)>>,
) {
    query.par_iter_mut().for_each(|(orbit, mut trajectory)| {
        trajectory.0 = Some(orbit.0.propagate(scenario.interval.to_time()))
    });
}

impl Scenario {
    pub fn new(start_time: Utc, end_time: Utc) -> Self {
        let mut world = World::new();
        world.insert_resource(ScenarioData {
            interval: UtcInterval::new(start_time, end_time),
            origin: DynOrigin::default(),
            frame: DynFrame::default(),
        });
        Self(world)
    }

    pub fn add_spacecraft(&mut self, name: String, orbit: DynOrbit) -> Entity {
        self.0
            .spawn(Spacecraft {
                meta: Asset { name },
                orbit: Orbit(orbit),
                trajectory: Trajectory(None),
            })
            .id()
    }

    pub fn insert_bundle<B: Bundle>(
        &mut self,
        id: Entity,
        bundle: B,
    ) -> Result<(), EntityMutableFetchError> {
        self.0.get_entity_mut(id)?.insert(bundle);
        Ok(())
    }

    pub fn all_spacecraft(&mut self) {
        let mut query = self.0.query::<(&Asset, &Orbit)>();
        for (asset, orbit) in query.iter(&self.0) {}
    }

    pub fn propagate(&mut self) -> Result<(), RegisteredSystemError> {
        self.0.run_system_cached(propagate)
    }
}

#[cfg(test)]
mod tests {
    use lox_time::utc;

    use super::*;

    #[test]
    fn test_scenario() {
        let start_time = utc!(2025, 11, 6).unwrap();
        let end_time = utc!(2025, 11, 6, 1).unwrap();
        let mut scn = Scenario::new(start_time, end_time);
        scn.propagate().unwrap();
    }
}
