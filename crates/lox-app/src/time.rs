// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use bevy::prelude::*;
use bevy::winit::WinitSettings;
use lox_time::{
    deltas::TimeDelta,
    intervals::{Interval, TimeInterval},
    time_scales::Tai,
};

pub type SimTime = lox_time::Time<Tai>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default, States)]
pub enum AppState {
    #[default]
    Paused,
    Playing,
}

impl std::fmt::Display for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppState::Paused => write!(f, "paused"),
            AppState::Playing => write!(f, "playing"),
        }
    }
}

impl AppState {
    fn toggle(&self) -> Self {
        match self {
            AppState::Paused => AppState::Playing,
            AppState::Playing => AppState::Paused,
        }
    }
}

const SPEEDS: &[(f64, &str)] = &[
    (1.0, "1s/s"),
    (60.0, "1min/s"),
    (3600.0, "1h/s"),
    (86_400.0, "1d/s"),
    (604_800.0, "1w/s"),
    (2_592_000.0, "1mo/s"),
    (31_557_600.0, "1y/s"),
];

#[derive(Debug, Resource)]
pub struct ScenarioTime {
    interval: TimeInterval<Tai>,
    current_time: SimTime,
    tier: i32,
}

impl ScenarioTime {
    pub fn start_time(&self) -> SimTime {
        self.interval.start()
    }

    pub fn end_time(&self) -> SimTime {
        self.interval.end()
    }

    pub fn current_time(&self) -> SimTime {
        self.current_time
    }

    fn set_current_time(&mut self, time: SimTime) {
        self.current_time = time;
    }

    fn increase_tier(&mut self) {
        let max = SPEEDS.len() as i32 - 1;
        self.tier = (self.tier + 1).min(max);
    }

    fn decrease_tier(&mut self) {
        let min = -(SPEEDS.len() as i32);
        self.tier = (self.tier - 1).max(min);
    }

    fn tier_index(&self) -> usize {
        if self.tier >= 0 {
            self.tier as usize
        } else {
            (-self.tier - 1) as usize
        }
    }

    fn factor(&self) -> f64 {
        let mag = SPEEDS[self.tier_index()].0;
        if self.tier >= 0 { mag } else { -mag }
    }

    pub fn speed_label(&self) -> String {
        let label = SPEEDS[self.tier_index()].1;
        let arrow = if self.tier >= 0 { ">>" } else { "<<" };
        format!("{label} {arrow}")
    }
}

impl Default for ScenarioTime {
    fn default() -> Self {
        let start: SimTime = chrono::Utc::now().into();
        let end = start + TimeDelta::from_days(1);
        Self {
            interval: Interval::new(start, end),
            current_time: start,
            tier: 0,
        }
    }
}

fn handle_space(
    input: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if input.just_pressed(KeyCode::Space) {
        next_state.set(state.toggle());
    }
}

fn handle_arrow_keys(input: Res<ButtonInput<KeyCode>>, mut scenario_time: ResMut<ScenarioTime>) {
    if input.just_pressed(KeyCode::ArrowRight) {
        scenario_time.increase_tier();
    } else if input.just_pressed(KeyCode::ArrowLeft) {
        scenario_time.decrease_tier();
    }
}

pub fn update_time(
    delta: Res<Time>,
    mut scenario_time: ResMut<ScenarioTime>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let delta =
        scenario_time.factor() * TimeDelta::from_nanoseconds(delta.delta().as_nanos() as i64);
    let current_time = scenario_time.current_time() + delta;
    let start_time = scenario_time.start_time();
    let end_time = scenario_time.end_time();
    if current_time <= start_time {
        next_state.set(AppState::Paused);
        scenario_time.set_current_time(start_time);
        return;
    }
    if current_time >= end_time {
        next_state.set(AppState::Paused);
        scenario_time.set_current_time(end_time);
        return;
    }
    scenario_time.set_current_time(current_time);
}

fn use_continuous_updates(mut settings: ResMut<WinitSettings>) {
    *settings = WinitSettings::game();
}

fn use_reactive_updates(mut settings: ResMut<WinitSettings>) {
    *settings = WinitSettings::desktop_app();
}

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<ScenarioTime>()
            .insert_resource(WinitSettings::desktop_app())
            .add_systems(OnEnter(AppState::Playing), use_continuous_updates)
            .add_systems(OnEnter(AppState::Paused), use_reactive_updates)
            .add_systems(Update, (handle_space, handle_arrow_keys))
            .add_systems(Update, update_time.run_if(in_state(AppState::Playing)));
    }
}
