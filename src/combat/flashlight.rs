//! Flashlight: handheld (fists / no weapon) or mounted on whatever gun is equipped. F = on/off, X = mount/unmount.
//! Battery drains while on; flickers when low; batteries (item.battery) recharge it.
use super::weapons::{Loadout, WEAPONS};
use crate::player::PlayerCamera;
use crate::settings::Keybinds;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

#[derive(Resource)]
pub struct Flashlight { pub on: bool, pub battery: f32, pub mounted: bool }
impl Default for Flashlight { fn default() -> Self { Self { on: false, battery: 1.0, mounted: true } } }

#[derive(Component)]
pub struct FlashlightBeam;

pub fn spawn_beam(mut commands: Commands, cam: Query<Entity, With<PlayerCamera>>, existing: Query<(), With<FlashlightBeam>>) {
    if !existing.is_empty() { return; }
    let Ok(c) = cam.single() else { return };
    let beam = commands.spawn((
        FlashlightBeam,
        SpotLight { intensity: 0.0, range: 70.0, outer_angle: 0.5, inner_angle: 0.12, color: Color::srgb(1.0, 0.96, 0.85), shadow_maps_enabled: true, ..default() },
        Transform::from_xyz(0.0, -0.2, -0.3),
    )).id();
    commands.entity(c).add_child(beam);
}

pub fn update(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    kb: Res<Keybinds>,
    motion: Res<AccumulatedMouseMotion>,
    lo: Res<Loadout>,
    mut fl: ResMut<Flashlight>,
    mut beam: Query<(&mut SpotLight, &mut Transform), With<FlashlightBeam>>,
    mut toasts: ResMut<crate::build::Toasts>,
    mut sway: Local<Vec2>,
    mut sfx: MessageWriter<crate::audio::Sfx>,
) {
    let dt = time.delta_secs();
    if keys.just_pressed(kb.flashlight) {
        if fl.battery <= 0.01 { toasts.say("Flashlight battery is dead. Find batteries."); }
        else { fl.on = !fl.on; sfx.write(crate::audio::Sfx::new("sfx.ui_click").vol(0.6)); }
    }
    if keys.just_pressed(KeyCode::KeyX) {
        fl.mounted = !fl.mounted;
        toasts.say(if fl.mounted { "Flashlight mounted on weapon." } else { "Flashlight in hand." });
    }
    if fl.on { fl.battery = (fl.battery - dt / 420.0).max(0.0); if fl.battery <= 0.0 { fl.on = false; toasts.say("Flashlight died."); } }
    let Ok((mut l, mut t)) = beam.single_mut() else { return };
    let gun = !WEAPONS[lo.current].melee;
    let on_gun = fl.mounted && gun;
    // target pose in camera space
    let target = if on_gun { Vec3::new(0.13, -0.11, -0.55) } else { Vec3::new(-0.32, -0.22, -0.3) };
    // lag/sway: light trails the camera a little when turning (more when hand-held)
    let lag = if on_gun { 0.015 } else { 0.05 };
    *sway = sway.lerp(Vec2::new(motion.delta.x, motion.delta.y) * lag, (12.0 * dt).min(1.0));
    t.translation = t.translation.lerp(target, (14.0 * dt).min(1.0));
    t.rotation = Quat::from_euler(EulerRot::YXZ, sway.x * 0.6, sway.y * 0.6, 0.0);
    let flicker = if fl.battery < 0.15 { 0.55 + 0.45 * ((time.elapsed_secs() * 37.0).sin() * (time.elapsed_secs() * 11.0).cos()).clamp(0.0, 1.0) } else { 1.0 };
    l.intensity = if fl.on { 2_500_000.0 * flicker } else { 0.0 };
    l.outer_angle = if on_gun { 0.42 } else { 0.55 };
}
