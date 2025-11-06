#![allow(clippy::type_complexity)]

// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use bevy_ecs::{prelude::*, system::RegisteredSystemError};

use lox_bodies::DynOrigin;
use lox_core::coords::LonLatAlt;
use lox_frames::DynFrame;
use lox_time::{
    Time,
    deltas::TimeDelta,
    time_scales::{self, Tai},
    utc::Utc,
};

use crate::{
    orbits::{DynOrbit, DynTrajectory},
    propagators::OrbitPropagator,
};

#[derive(Component)]
struct Asset {
    name: String,
}

#[derive(Component)]
struct Orbit(DynOrbit);

#[derive(Component)]
struct Trajectory(Option<DynTrajectory>);

#[derive(Bundle)]
struct Spacecraft {
    meta: Asset,
    orbit: Orbit,
    trajectory: Trajectory,
}

#[derive(Resource)]
struct ScenarioInterval {
    start_time: Utc,
    end_time: Utc,
}

#[derive(Resource)]
struct TimeStep(TimeDelta);

impl ScenarioInterval {
    fn start_time(&self) -> Time<Tai> {
        self.start_time.to_time()
    }

    fn end_time(&self) -> Time<Tai> {
        self.end_time.to_time()
    }
}

pub struct Scenario(World);

fn propagate(mut query: Query<(&Orbit, &mut Trajectory), Or<(Changed<Orbit>, Added<Orbit>)>>) {
    query
        .par_iter_mut()
        .for_each(|(orbit, mut trajectory)| trajectory.0 = Some(orbit.0.propagate()));
}

impl Scenario {
    pub fn new(start_time: Utc, end_time: Utc) -> Self {
        let mut world = World::new();
        world.insert_resource(ScenarioInterval {
            start_time,
            end_time,
        });
        world.insert_resource(TimeStep(TimeDelta::from_seconds(60)));
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
        let scn = Scenario::new(start_time, end_time);
    }
}
