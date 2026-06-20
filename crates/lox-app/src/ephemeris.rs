// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use bevy::prelude::*;
use lox_bodies::Origin;
use lox_core::glam::DVec3;
use lox_ephem::{Ephemeris, spk::parser::Spk};
use lox_time::time_scales::Tdb;

use crate::time::{AppState, ScenarioTime, update_time};

static DE440S: &[u8] = include_bytes!("../../../data/spice/de440s.bsp");

const BODIES: [(Origin, &str); 10] = [
    (Origin::Sun, "Sun"),
    (Origin::Mercury, "Mercury"),
    (Origin::Earth, "Earth"),
    (Origin::Moon, "Moon"),
    (Origin::MarsBarycenter, "Mars"),
    (Origin::JupiterBarycenter, "Jupiter"),
    (Origin::SaturnBarycenter, "Saturn"),
    (Origin::UranusBarycenter, "Uranus"),
    (Origin::NeptuneBarycenter, "Neptune"),
    (Origin::PlutoBarycenter, "Pluto"),
];

#[derive(Debug, Component)]
struct BodyOrigin(Origin);

#[derive(Debug, Component)]
#[expect(unused)]
struct BodyName(String);

#[derive(Debug, Component)]
struct Position(DVec3);

fn spawn_bodies(
    time: Res<ScenarioTime>,
    ephemeris: Res<Ephem>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let t = time.current_time().to_scale(Tdb);
    for (origin, name) in BODIES {
        let position = ephemeris
            .0
            .position(t, Origin::SolarSystemBarycenter, origin)?;
        commands.spawn((
            BodyOrigin(origin),
            BodyName(name.to_owned()),
            Position(position),
        ));
    }
    Ok(())
}

fn update_bodies(
    time: Res<ScenarioTime>,
    ephemeris: Res<Ephem>,
    query: Query<(&BodyOrigin, &mut Position)>,
) -> Result<(), BevyError> {
    let t = time.current_time().to_scale(Tdb);
    for (origin, mut position) in query {
        position.0 = ephemeris
            .0
            .position(t, Origin::SolarSystemBarycenter, origin.0)?;
    }
    Ok(())
}

#[derive(Resource)]
struct Ephem(Spk);

impl Default for Ephem {
    fn default() -> Self {
        Self(Spk::from_bytes(DE440S).expect("embedded SPK kernel should be readable"))
    }
}

pub struct EphemerisPlugin;

impl Plugin for EphemerisPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Ephem>()
            .add_systems(Startup, spawn_bodies)
            .add_systems(
                Update,
                update_bodies
                    .run_if(in_state(AppState::Playing))
                    .after(update_time),
            );
    }
}
