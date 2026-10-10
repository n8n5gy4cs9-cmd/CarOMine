//! CarOMine - by Crowelian 2026 + Sonnet.
mod state;

use bevy::prelude::*;
use state::{GameState, StatePlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CarOMine".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(StatePlugin)
        .insert_resource(ClearColor(Color::srgb(0.55, 0.65, 0.78)))
        .add_systems(Startup, setup_placeholder_scene)
        .add_systems(OnEnter(GameState::MainMenu), || info!("CarOMine main menu - press ENTER"))
        .add_systems(OnEnter(GameState::Playing), || info!("Playing"))
        .run();
}

/// Temporary scene so the window shows something (replaced by world/ in P3).
fn setup_placeholder_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-6.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight { shadow_maps_enabled: true, ..default() },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(mats.add(Color::srgb(0.93, 0.95, 1.0))),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 4.0))),
        MeshMaterial3d(mats.add(Color::srgb(0.7, 0.1, 0.1))),
        Transform::from_xyz(0.0, 0.6, 0.0),
    ));
}
