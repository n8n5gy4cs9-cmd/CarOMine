//! CarOMine - by Crowelian 2026 + Sonnet.
#![allow(dead_code)] // scaffolding; modules get wired in as phases land
mod audio;
mod build;
mod combat;
mod console;
mod dev;
mod i18n;
mod life;
mod manifest;
mod menu;
mod player;
mod save;
mod settings;
mod state;
mod story;
mod vehicle;
mod world;

use bevy::light::DirectionalLight;
use bevy::prelude::*;
use manifest::ManifestPlugin;
use settings::SettingsPlugin;
use state::{GameState, StatePlugin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window { title: "CarOMine".into(), resolution: (1280, 720).into(), ..default() }),
                    ..default()
                })
                .set(AssetPlugin { file_path: asset_dir(), ..default() }),
        )
        .add_plugins((ManifestPlugin, SettingsPlugin, StatePlugin, world::WorldPlugin, player::PlayerPlugin, vehicle::VehiclePlugin, combat::CombatPlugin))
        .add_plugins((console::ConsolePlugin, build::BuildPlugin, audio::GameAudioPlugin, i18n::I18nPlugin, menu::MenuPlugin, life::LifePlugin, story::StoryPlugin, save::SavePlugin, dev::DevPlugin))
        .insert_resource(ClearColor(Color::srgb(0.55, 0.65, 0.78)))
        .add_systems(Startup, setup_lights)
        .run();
}

/// Prefer ./assets (dev / bundle with cwd set); else Bevy's default (next to the executable).
fn asset_dir() -> String {
    match std::env::current_dir() {
        Ok(d) if d.join("assets").is_dir() => d.join("assets").to_string_lossy().into_owned(),
        _ => "assets".into(),
    }
}

fn setup_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight { shadow_maps_enabled: false, illuminance: 0.0, color: Color::srgb(0.55, 0.68, 1.0), ..default() },
        Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::new(0.4, -1.0, 0.3), Vec3::Y),
        world::sky::Moon,
    ));
    commands.spawn((DirectionalLight { shadow_maps_enabled: true, illuminance: 10_000.0, ..default() }, Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::new(-1.0, 0.0, -0.5), Vec3::Y), world::sky::Sun));
}
