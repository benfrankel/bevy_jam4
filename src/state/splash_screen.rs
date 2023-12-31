use std::f32::consts::PI;

use bevy::asset::load_internal_binary_asset;
use bevy::prelude::*;
use bevy::render::texture::ImageSampler;
use bevy::render::texture::ImageType;
use bevy::ui::Val::*;
use bevy_asset_loader::prelude::*;
use iyes_progress::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::state::title_screen::TitleScreenAssets;
use crate::state::AppState::*;
use crate::AppRoot;

pub struct SplashScreenStatePlugin;

impl Plugin for SplashScreenStatePlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            SPLASH_SCREEN_IMAGE_HANDLE,
            "../../assets/image/ui/splash.png",
            |bytes, _path: String| {
                Image::from_buffer(
                    bytes,
                    ImageType::Extension("png"),
                    default(),
                    true,
                    ImageSampler::linear(),
                )
                .unwrap()
            }
        );

        app.register_type::<SplashScreenStartTime>()
            .register_type::<SplashImageFadeInOut>()
            .add_loading_state(LoadingState::new(SplashScreen))
            .add_collection_to_loading_state::<_, TitleScreenAssets>(SplashScreen)
            .add_plugins(ProgressPlugin::new(SplashScreen).continue_to(TitleScreen))
            .add_systems(OnEnter(SplashScreen), enter_splash_screen)
            .add_systems(OnExit(SplashScreen), exit_splash_screen)
            .add_systems(
                Update,
                update_splash_screen
                    .track_progress()
                    .run_if(in_state(SplashScreen)),
            );
    }
}

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct SplashScreenConfig {
    foreground_color: Color,
    background_color: Color,
    min_duration: f64,
}

const SPLASH_SCREEN_IMAGE_HANDLE: Handle<Image> =
    Handle::weak_from_u128(145948501136218819748366695396142082634);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct SplashScreenStartTime(f64);

fn enter_splash_screen(
    mut commands: Commands,
    root: Res<AppRoot>,
    config: Res<Config>,
    time: Res<Time>,
) {
    let config = &config.splash_screen;
    commands.insert_resource(ClearColor(config.background_color));
    commands.insert_resource(SplashScreenStartTime(time.elapsed_seconds_f64()));

    let screen = commands
        .spawn((
            Name::new("SplashScreen"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(root.ui)
        .id();

    commands
        .spawn((
            Name::new("SplashImage"),
            ImageBundle {
                style: Style {
                    margin: UiRect::all(Auto),
                    width: Percent(70.0),
                    ..default()
                },
                image: UiImage::new(SPLASH_SCREEN_IMAGE_HANDLE),
                ..default()
            },
            SplashImageFadeInOut(config.foreground_color),
        ))
        .set_parent(screen);
}

fn exit_splash_screen(mut commands: Commands, root: Res<AppRoot>) {
    commands.remove_resource::<SplashScreenStartTime>();
    commands.entity(root.ui).despawn_descendants();
}

#[derive(Component, Reflect)]
struct SplashImageFadeInOut(Color);

// TODO: Replace this with some Animation component
fn update_splash_screen(
    config: Res<Config>,
    time: Res<Time>,
    start: Res<SplashScreenStartTime>,
    mut color_query: Query<(&mut BackgroundColor, &SplashImageFadeInOut)>,
) -> Progress {
    let config = &config.splash_screen;
    let elapsed = (time.elapsed_seconds_f64() - start.0) / config.min_duration;

    for (mut color, fade) in &mut color_query {
        let t = elapsed as f32;
        let amplitude = 1.5;
        let alpha = (amplitude * (PI * t).sin() - amplitude + 1.0)
            .max(0.0)
            .powf(1.2);
        color.0 = fade.0.with_a(alpha);
    }

    (elapsed >= 1.0).into()
}
