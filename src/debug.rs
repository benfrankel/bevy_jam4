use bevy::core::FrameCount;
use bevy::diagnostic::EntityCountDiagnosticsPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::diagnostic::SystemInformationDiagnosticsPlugin;
use bevy::ecs::schedule::LogLevel;
use bevy::ecs::schedule::ScheduleBuildSettings;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_editor_pls::EditorPlugin;
use iyes_progress::prelude::*;
use strum::IntoEnumIterator;

use crate::simulation::LinesAddedEvent;
use crate::simulation::Simulation;
use crate::state::AppState;

pub struct DebugPlugin {
    pub frame_time_diagnostics: bool,
    pub system_information_diagnostics: bool,
    pub entity_count_diagnostics: bool,
    pub ambiguity_detection: bool,
    pub debug_picking: bool,
    pub editor: bool,
    pub start: AppState,
    pub extend_loading_screen: f32,
    pub cheats: bool,
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Diagnostics
        if self.frame_time_diagnostics {
            app.add_plugins(FrameTimeDiagnosticsPlugin);
        }
        if self.system_information_diagnostics {
            app.add_plugins(SystemInformationDiagnosticsPlugin);
        }
        if self.entity_count_diagnostics {
            app.add_plugins(EntityCountDiagnosticsPlugin);
        }

        // Logging
        app.add_plugins(LogDiagnosticsPlugin::default());
        if self.ambiguity_detection {
            for (_, schedule) in app.world.resource_mut::<Schedules>().iter_mut() {
                schedule.set_build_settings(ScheduleBuildSettings {
                    ambiguity_detection: LogLevel::Warn,
                    ..default()
                });
            }
        }
        for state in AppState::iter() {
            app.add_systems(OnEnter(state), move |frame: Res<FrameCount>| {
                info!("[Frame {}] Entering {state:?}", frame.0)
            })
            .add_systems(OnExit(state), move |frame: Res<FrameCount>| {
                info!("[Frame {}] Exiting {state:?}", frame.0)
            });
        }

        if self.debug_picking {
            use bevy_mod_picking::debug::DebugPickingMode::*;
            app.insert_resource(State::new(Disabled)).add_systems(
                Update,
                (
                    (|mut next: ResMut<NextState<_>>| next.set(Normal))
                        .run_if(in_state(Disabled).and_then(input_just_pressed(DEBUG_TOGGLE_KEY))),
                    (|mut next: ResMut<NextState<_>>| next.set(Disabled))
                        .run_if(in_state(Normal).and_then(input_just_pressed(DEBUG_TOGGLE_KEY))),
                ),
            );
        }

        // Extend loading screen
        if self.extend_loading_screen > 0.0 {
            let delay = self.extend_loading_screen;
            let fake_task = move |mut start: Local<f32>, time: Res<Time>| -> Progress {
                let elapsed = time.elapsed_seconds();
                if *start == 0.0 {
                    *start = elapsed;
                }
                (elapsed - *start >= delay).into()
            };
            app.add_systems(
                Update,
                (
                    (|| Progress::from(false))
                        .track_progress()
                        .run_if(in_state(AppState::TitleScreen)),
                    fake_task
                        .track_progress()
                        .run_if(in_state(AppState::LoadingScreen)),
                ),
            );
        }

        // Skip to custom start state
        *app.world.resource_mut::<State<AppState>>() = State::new(self.start);

        // Editor
        if self.editor {
            app.add_plugins(EditorPlugin::new().in_new_window(Window {
                mode: WindowMode::Windowed,
                title: "bevy_editor_pls".to_string(),
                focused: false,
                ..default()
            }));
        }

        app.add_systems(Update, debug_start);
        app.add_systems(Update, debug_end);

        if self.cheats {
            app.init_resource::<CheatSettings>();
            app.add_systems(
                Update,
                |keyboard_input: Res<Input<KeyCode>>,
                 mut cheat_settings: ResMut<CheatSettings>,
                 mut simulation: ResMut<Simulation>| {
                    if keyboard_input.just_pressed(KeyCode::F5) {
                        cheat_settings.generate_lines = !cheat_settings.generate_lines;
                    }
                    if keyboard_input.just_pressed(KeyCode::F6) {
                        simulation.lines *= 4.0;
                    }
                },
            );
            app.add_systems(
                Update,
                |mut cheat_settings: ResMut<CheatSettings>,
                 time: Res<Time>,
                 mut events: EventWriter<LinesAddedEvent>| {
                    if cheat_settings.generate_lines
                        && cheat_settings.timer.tick(time.delta()).just_finished()
                    {
                        events.send(LinesAddedEvent { count: 1.0 });
                    }
                },
            );
        }
    }
}

#[derive(Resource)]
pub struct CheatSettings {
    pub generate_lines: bool,
    pub timer: Timer,
}

impl Default for CheatSettings {
    fn default() -> Self {
        Self {
            generate_lines: false,
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        }
    }
}

const DEBUG_TOGGLE_KEY: KeyCode = KeyCode::F3;

fn debug_start(world: &mut World) {
    let frame = world.resource::<FrameCount>().0;
    let prefix = format!("[Frame {frame} start] ");
    let _ = prefix;
}

fn debug_end(world: &mut World) {
    let frame = world.resource::<FrameCount>().0;
    let prefix = format!("[Frame {frame} end] ");
    let _ = prefix;
}

impl Default for DebugPlugin {
    fn default() -> Self {
        Self {
            frame_time_diagnostics: true,
            system_information_diagnostics: true,
            entity_count_diagnostics: true,
            ambiguity_detection: true,
            debug_picking: true,
            editor: true,
            extend_loading_screen: 0.0,
            start: default(),
            cheats: true,
        }
    }
}
