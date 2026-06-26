// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use bevy::prelude::*;

mod ephemeris;
mod scene;
mod time;
mod ui;

use crate::{ephemeris::EphemerisPlugin, scene::ScenePlugin, time::TimePlugin, ui::UiPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EphemerisPlugin,
            TimePlugin,
            UiPlugin,
            ScenePlugin,
        ))
        .run();
}
