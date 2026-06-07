// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use bevy::{math::DVec3, prelude::*};
use lox_bodies::DynOrigin;
use lox_ephem::{Ephemeris, spk::parser::Spk};
use lox_time::time_scales::Tdb;

use crate::time::{AppState, ScenarioTime, update_time};

static DE440S: &[u8] = include_bytes!("../../../data/spice/de440s.bsp");

const BODIES: [(DynOrigin, &str); 10] = [
    (DynOrigin::Sun, "Sun"),
    (DynOrigin::Mercury, "Mercury"),
    (DynOrigin::Earth, "Earth"),
    (DynOrigin::Moon, "Moon"),
    (DynOrigin::MarsBarycenter, "Mars"),
    (DynOrigin::JupiterBarycenter, "Jupiter"),
    (DynOrigin::SaturnBarycenter, "Saturn"),
    (DynOrigin::UranusBarycenter, "Uranus"),
    (DynOrigin::NeptuneBarycenter, "Neptune"),
    (DynOrigin::PlutoBarycenter, "Pluto"),
];

#[derive(Debug, Component)]
struct Origin(DynOrigin);

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
            .position(t, DynOrigin::SolarSystemBarycenter, origin)?;
        commands.spawn((
            Origin(origin),
            BodyName(name.to_owned()),
            Position(position),
        ));
    }
    Ok(())
}

fn update_bodies(
    time: Res<ScenarioTime>,
    ephemeris: Res<Ephem>,
    query: Query<(&Origin, &mut Position)>,
) -> Result<(), BevyError> {
    let t = time.current_time().to_scale(Tdb);
    for (origin, mut position) in query {
        position.0 = ephemeris
            .0
            .position(t, DynOrigin::SolarSystemBarycenter, origin.0)?;
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
