//! First-person controller (walk / run / crouch / jump) with slippery-surface inertia.
use crate::settings::{Keybinds, Settings};
use crate::state::GameState;
use crate::world::layout::*;
use crate::world::scatter::Obstacle;
use crate::world::terrain::StreamAnchor;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

#[derive(Component)]
pub struct Player {
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub eye: f32,
}

#[derive(Component)]
pub struct PlayerCamera;

pub const EYE_STAND: f32 = 1.7;
pub const EYE_CROUCH: f32 = 1.0;

pub fn spawn_player(mut commands: Commands, settings: Res<Settings>) {
    let mut pos = Vec3::new(CABIN.x - 8.0, 0.0, CABIN.y - 10.0);
    let (mut yaw, mut pitch) = (std::f32::consts::PI, 0.0f32);
    if let Ok(v) = std::env::var("CAROMINE_POS") {
        let n: Vec<f32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect();
        if n.len() >= 2 { pos.x = n[0]; pos.z = n[1]; }
        if n.len() >= 3 { yaw = n[2].to_radians(); }
        if n.len() >= 4 { pitch = n[3].to_radians(); }
    }
    pos.y = height_at(pos.x, pos.z);
    commands
        .spawn((
            Player { vel: Vec3::ZERO, yaw, pitch, on_ground: true, eye: EYE_STAND },
            StreamAnchor,
            Transform::from_translation(pos),
            Visibility::default(),
        ))
        .with_children(|p| {
            p.spawn((
                PlayerCamera,
                Camera3d::default(),
                Projection::Perspective(PerspectiveProjection { fov: settings.fov_degrees.to_radians(), far: 2000.0, ..default() }),
                Transform::from_xyz(0.0, EYE_STAND, 0.0),
                DistanceFog {
                    color: Color::srgb(0.6, 0.7, 0.85),
                    falloff: FogFalloff::Linear { start: 120.0, end: 520.0 },
                    ..default()
                },
            ));
        });
}

pub fn despawn_player(mut commands: Commands, q: Query<Entity, With<Player>>) {
    for e in &q { commands.entity(e).despawn(); }
}

pub fn grab_cursor(mut q: Query<&mut CursorOptions, With<PrimaryWindow>>, grab: bool) {
    if let Ok(mut c) = q.single_mut() {
        c.grab_mode = if grab { CursorGrabMode::Locked } else { CursorGrabMode::None };
        c.visible = !grab;
    }
}
pub fn grab_on(q: Query<&mut CursorOptions, With<PrimaryWindow>>) { grab_cursor(q, true); }
pub fn grab_off(q: Query<&mut CursorOptions, With<PrimaryWindow>>) { grab_cursor(q, false); }

pub fn look(driving: Query<(), With<crate::vehicle::Driven>>, motion: Res<AccumulatedMouseMotion>, settings: Res<Settings>, mut q: Query<(&mut Player, &mut Transform), Without<PlayerCamera>>, mut cam: Query<&mut Transform, With<PlayerCamera>>) {
    if !driving.is_empty() { return; }
    let Ok((mut p, mut t)) = q.single_mut() else { return };
    let inv = if settings.invert_y { -1.0 } else { 1.0 };
    p.yaw -= motion.delta.x * settings.mouse_sensitivity;
    p.pitch = (p.pitch - motion.delta.y * settings.mouse_sensitivity * inv).clamp(-1.5, 1.5);
    t.rotation = Quat::from_rotation_y(p.yaw);
    if let Ok(mut c) = cam.single_mut() {
        c.rotation = Quat::from_rotation_x(p.pitch);
    }
}

pub fn walk(
    cheats: Res<crate::console::Cheats>,
    driving: Query<(), With<crate::vehicle::Driven>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    kb: Res<Keybinds>,
    obstacles: Query<(&GlobalTransform, &Obstacle), Without<Player>>,
    mut q: Query<(&mut Player, &mut Transform)>,
    mut cam: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    if !driving.is_empty() { return; }
    let Ok((mut p, mut t)) = q.single_mut() else { return };
    let dt = time.delta_secs().min(0.05);
    let fwd = Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos());
    let right = Vec3::new(p.yaw.cos(), 0.0, -p.yaw.sin());
    let mut wish = Vec3::ZERO;
    if keys.pressed(kb.forward) { wish += fwd; }
    if keys.pressed(kb.back) { wish -= fwd; }
    if keys.pressed(kb.right) { wish += right; }
    if keys.pressed(kb.left) { wish -= right; }
    let crouch = keys.pressed(kb.crouch);
    if cheats.fly {
        // noclip: fly along the look direction
        let look = Quat::from_rotation_y(p.yaw) * Quat::from_rotation_x(p.pitch);
        let mut v = Vec3::ZERO;
        if keys.pressed(kb.forward) { v += look * Vec3::NEG_Z; }
        if keys.pressed(kb.back) { v += look * Vec3::Z; }
        if keys.pressed(kb.right) { v += look * Vec3::X; }
        if keys.pressed(kb.left) { v -= look * Vec3::X; }
        if keys.pressed(kb.jump) { v += Vec3::Y; }
        if keys.pressed(kb.crouch) { v -= Vec3::Y; }
        let sp = 14.0 * cheats.speed_mult * if keys.pressed(kb.run) { 3.0 } else { 1.0 };
        t.translation += v.normalize_or_zero() * sp * dt;
        p.vel = Vec3::ZERO; p.on_ground = false;
        return;
    }
    let speed = cheats.speed_mult * if crouch { 1.8 } else if keys.pressed(kb.run) { 7.0 } else { 4.2 };
    let wish = wish.normalize_or_zero() * speed;
    let grip = match surface_at(t.translation.x, t.translation.z) { Surface::Ice => 1.2, Surface::PackedSnow => 7.0, _ => 9.0 };
    let accel = if p.on_ground { grip } else { 1.0 };
    let hv = Vec3::new(p.vel.x, 0.0, p.vel.z);
    let hv = hv + (wish - hv) * (accel * dt).min(1.0);
    p.vel.x = hv.x; p.vel.z = hv.z;
    if p.on_ground && keys.just_pressed(kb.jump) { p.vel.y = 5.0; p.on_ground = false; }
    p.vel.y -= 14.0 * dt;
    let mut np = t.translation + p.vel * dt;
    // obstacle push-out (trees, rocks...)
    for (g, o) in &obstacles {
        let d = Vec2::new(np.x - g.translation().x, np.z - g.translation().z);
        let r = o.0 + 0.35;
        if d.length_squared() < r * r && d.length_squared() > 1e-6 {
            let push = d.normalize() * r;
            np.x = g.translation().x + push.x; np.z = g.translation().z + push.y;
        }
    }
    let gh = height_at(np.x, np.z);
    if np.y <= gh { np.y = gh; p.vel.y = 0.0; p.on_ground = true; } else if np.y > gh + 0.05 { p.on_ground = false; }
    // keep inside world
    np.x = np.x.clamp(-WORLD_HALF + 5.0, WORLD_HALF - 5.0); np.z = np.z.clamp(-WORLD_HALF + 5.0, WORLD_HALF - 5.0);
    t.translation = np;
    let target = if crouch { EYE_CROUCH } else { EYE_STAND };
    p.eye += (target - p.eye) * (10.0 * dt).min(1.0);
    if let Ok(mut c) = cam.single_mut() { c.translation.y = p.eye; }
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (spawn_player_once, grab_on))
            .add_systems(OnEnter(GameState::Paused), grab_off)
            .add_systems(Update, (look, walk).chain().run_if(in_state(GameState::Playing)));
    }
}

fn spawn_player_once(mut commands: Commands, settings: Res<Settings>, existing: Query<(), With<Player>>) {
    if existing.is_empty() { spawn_player(commands.reborrow(), settings); }
}
