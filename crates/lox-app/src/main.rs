// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use bevy::prelude::*;

mod time;
mod ui;

use time::TimePlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TimePlugin, UiPlugin))
        .run();
}
