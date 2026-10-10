//! CarOMine - by Crowelian 2026 + Sonnet.
#![allow(dead_code)] // scaffolding; modules get wired in as phases land
mod dev;
mod manifest;
mod player;
mod settings;
mod state;
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
        .add_plugins((ManifestPlugin, SettingsPlugin, StatePlugin, world::WorldPlugin, player::PlayerPlugin, vehicle::VehiclePlugin, dev::DevPlugin))
        .insert_resource(ClearColor(Color::srgb(0.55, 0.65, 0.78)))
        .add_systems(Startup, setup_lights)
        .add_systems(OnEnter(GameState::MainMenu), spawn_menu)
        .add_systems(OnExit(GameState::MainMenu), despawn_menu)
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

#[derive(Component)]
struct MenuUi;

fn spawn_menu(mut commands: Commands) {
    commands.spawn((MenuUi, Camera2d));
    commands.spawn((
        MenuUi,
        Text::new("CarOMine\n\nPress ENTER to start\nWASD move  Shift run  Ctrl crouch  Space jump  Esc pause\n\nby Crowelian 2026 + Sonnet"),
        TextFont { font_size: FontSize::Px(28.0), ..default() },
        Node { position_type: PositionType::Absolute, left: Val::Px(60.0), top: Val::Px(60.0), ..default() },
    ));
}

fn despawn_menu(mut commands: Commands, q: Query<Entity, With<MenuUi>>) {
    for e in &q { commands.entity(e).despawn(); }
}
