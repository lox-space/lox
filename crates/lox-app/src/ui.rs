// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use bevy::prelude::*;

use crate::time::{AppState, ScenarioTime, update_time};

#[derive(Component)]
struct TimeDisplay;

#[derive(Component)]
struct PlayPauseDisplay;

#[derive(Component)]
struct FactorDisplay;

fn update_time_display(
    scenario_time: Res<ScenarioTime>,
    mut query: Query<&mut Text, With<TimeDisplay>>,
) {
    let mut text = query.single_mut().expect("time display is present");
    text.0 = format!("{}", scenario_time.current_time());
}

fn update_play_pause_display(
    state: Res<State<AppState>>,
    mut query: Query<&mut Text, With<PlayPauseDisplay>>,
) {
    let mut text = query.single_mut().expect("play/pause display is present");
    text.0 = format!("{}", state.get())
}

fn update_factor_display(
    scenario_time: Res<ScenarioTime>,
    mut query: Query<&mut Text, With<FactorDisplay>>,
) {
    let mut text = query.single_mut().expect("factor display is present");
    text.0 = scenario_time.speed_label();
}

fn spawn_ui(mut commands: Commands) {
    commands.spawn(Camera2d);
    let container = Node {
        flex_direction: FlexDirection::Column,
        width: percent(100.0),
        height: percent(100.0),
        justify_content: JustifyContent::Center,
        ..default()
    };
    let play_pause_text = (
        Node::DEFAULT,
        Text::default(),
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        PlayPauseDisplay,
    );
    let factor_text = (
        Node::DEFAULT,
        Text::default(),
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        FactorDisplay,
    );
    let time_text = (
        Node::DEFAULT,
        Text::default(),
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        TimeDisplay,
    );

    commands.spawn((
        container,
        children![play_pause_text, factor_text, time_text],
    ));
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui)
            .add_systems(Update, update_time_display.after(update_time))
            .add_systems(Update, update_play_pause_display)
            .add_systems(Update, update_factor_display);
    }
}
