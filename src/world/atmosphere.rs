//! Atmosphere & lighting polish: bloom/tonemapping/volumetric light on the camera, weather+mist fog,
//! stars, sun/moon discs, aurora borealis, and soft particle puffs (breath, exhaust, steam, spray, chimney smoke).
use super::billboard::*;
use super::layout::*;
use super::sky::{sky_state, Moon, Sun, TimeOfDay};
use super::weather::Weather;
use crate::manifest::AssetManifest;
use crate::player::PlayerCamera;
use crate::vehicle::Car;
use bevy::asset::RenderAssetUsages;
use bevy::light::{FogVolume, VolumetricFog, VolumetricLight};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

pub struct AtmospherePlugin;
impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_sky_objects, setup_fx_assets))
            .add_systems(Update, (camera_fx, sky_objects, fog_weather, emit_puffs, update_puffs, follow_fog_volume).run_if(in_state(crate::state::GameState::Playing)));
    }
}

// ---------- camera post-processing + volumetric
fn camera_fx(mut commands: Commands, cam: Query<Entity, (With<PlayerCamera>, Without<Bloom>)>, sun: Query<Entity, (With<Sun>, Without<VolumetricLight>)>, vol: Query<(), With<FogVolume>>) {
    for e in &cam {
        commands.entity(e).insert((
            Bloom { intensity: 0.12, ..Bloom::NATURAL },
            bevy::render::view::Tonemapping::TonyMcMapface,
            VolumetricFog { ambient_intensity: 0.0, ..default() },
            Msaa::Off,
        ));
    }
    for e in &sun { commands.entity(e).insert(VolumetricLight); }
    if vol.is_empty() {
        commands.spawn((FogVolume { density_factor: 0.004, fog_color: Color::srgb(0.8, 0.88, 1.0), absorption: 0.12, scattering: 0.35, ..default() }, Transform::from_scale(Vec3::new(500.0, 60.0, 500.0))));
    }
}
fn follow_fog_volume(cam: Query<&GlobalTransform, With<PlayerCamera>>, mut v: Query<&mut Transform, With<FogVolume>>, weather: Res<Weather>, tod: Res<TimeOfDay>, mut fv: Query<&mut FogVolume>) {
    let (Some(c), Ok(mut t)) = (cam.iter().next(), v.single_mut()) else { return };
    t.translation = Vec3::new(c.translation().x, c.translation().y + 10.0, c.translation().z);
    if let Ok(mut f) = fv.single_mut() {
        let mist = mist_amount(tod.hour);
        f.density_factor = 0.003 + 0.012 * weather.intensity + 0.01 * mist;
    }
}

/// Morning/evening mist 0..1 (peaks ~7:30 and ~16:30 when it's cold and still).
pub fn mist_amount(hour: f32) -> f32 {
    let bump = |c: f32, w: f32| (-((hour - c) / w).powi(2)).exp();
    (bump(7.5, 1.4) + 0.7 * bump(16.5, 1.2)).min(1.0)
}

fn fog_weather(tod: Res<TimeOfDay>, weather: Res<Weather>, mut fog: Query<&mut DistanceFog>) {
    let st = sky_state(tod.hour);
    let mist = mist_amount(tod.hour);
    let w = weather.intensity;
    for mut f in &mut fog {
        let start = 120.0 * (1.0 - w * 0.9) * (1.0 - 0.5 * mist);
        let end = (520.0 - 440.0 * w * w) * (1.0 - 0.45 * mist);
        f.falloff = FogFalloff::Linear { start: start.max(5.0), end: end.max(45.0) };
        // blizzard whiteout tints fog towards snow-white; mist is slightly warm in the morning
        let sky = st.sky.to_srgba();
        let white = Vec3::new(0.78, 0.82, 0.9) * (0.25 + 0.75 * st.day);
        let base = Vec3::new(sky.red, sky.green, sky.blue);
        let c = base.lerp(white, (w * 0.7 + mist * 0.25).min(0.85));
        f.color = Color::srgb(c.x, c.y, c.z);
        f.directional_light_color = Color::srgba(1.0, 0.85, 0.65, 0.4 * st.day);
        f.directional_light_exponent = 30.0;
    }
}

// ---------- sky objects: stars, sun, moon, aurora
#[derive(Component)] struct SkyDome;
#[derive(Component)] struct StarMat;
#[derive(Component)] struct SunDisc;
#[derive(Component)] struct MoonDisc;
#[derive(Component)] struct Aurora { phase: f32, layer: f32 }

#[derive(Resource)]
struct SkyAssets { star: Handle<StandardMaterial>, sun: Handle<StandardMaterial>, moon: Handle<StandardMaterial>, aurora: Handle<StandardMaterial> }

fn setup_sky_objects(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let unlit = |c: Color, mode: AlphaMode, tex: Option<Handle<Image>>| StandardMaterial { base_color: c, base_color_texture: tex, unlit: true, fog_enabled: false, alpha_mode: mode, cull_mode: None, double_sided: true, ..default() };
    let disc = server.load(man.path("fx.disc"));
    let star = mats.add(unlit(Color::srgba(1.0, 1.0, 1.0, 0.0), AlphaMode::Blend, None));
    let sun = mats.add(unlit(Color::srgba(1.0, 0.9, 0.7, 1.0), AlphaMode::Add, Some(disc.clone())));
    let moon = mats.add(unlit(Color::srgba(0.8, 0.88, 1.0, 1.0), AlphaMode::Add, Some(disc)));
    let aurora = mats.add(unlit(Color::WHITE, AlphaMode::Add, None));
    commands.insert_resource(SkyAssets { star: star.clone(), sun: sun.clone(), moon: moon.clone(), aurora: aurora.clone() });
    let quad = meshes.add(Rectangle::new(1.0, 1.0));
    commands.spawn((SkyDome, Transform::default(), Visibility::default())).with_children(|c| {
        // stars
        let mut s = 99u32;
        let mut r = || { s = s.wrapping_mul(1664525).wrapping_add(1013904223); (s >> 8) as f32 / 16_777_216.0 };
        for _ in 0..900 {
            let (u, v) = (r() * std::f32::consts::TAU, r() * 0.95 + 0.04);
            let (cy, sy) = (v.asin().cos(), v);
            let p = Vec3::new(u.cos() * cy, sy, u.sin() * cy) * 900.0;
            let size = 2.0 + r() * r() * 5.0;
            c.spawn((Mesh3d(quad.clone()), MeshMaterial3d(star.clone()), Transform::from_translation(p).looking_at(Vec3::ZERO, Vec3::Y).with_scale(Vec3::splat(size)), StarMat));
        }
        c.spawn((Mesh3d(quad.clone()), MeshMaterial3d(sun), Transform::default().with_scale(Vec3::splat(150.0)), SunDisc));
        c.spawn((Mesh3d(quad.clone()), MeshMaterial3d(moon), Transform::default().with_scale(Vec3::splat(110.0)), MoonDisc));
        // aurora curtains to the north (-z)
        for layer in 0..3 {
            let mesh = meshes.add(aurora_mesh(layer as f32));
            c.spawn((Mesh3d(mesh), MeshMaterial3d(aurora.clone()), Transform::from_xyz(0.0, 110.0 + layer as f32 * 35.0, -820.0 + layer as f32 * 40.0), Aurora { phase: layer as f32 * 2.1, layer: layer as f32 }));
        }
    });
}

fn aurora_mesh(seed: f32) -> Mesh {
    let n = 48; let (mut pos, mut col, mut idx): (Vec<[f32; 3]>, Vec<[f32; 4]>, Vec<u32>) = (vec![], vec![], vec![]);
    for i in 0..=n {
        let t = i as f32 / n as f32; let x = (t - 0.5) * 1500.0;
        let z = (t * 7.0 + seed * 3.0).sin() * 60.0 + (t * 19.0 + seed).sin() * 18.0;
        let top = 190.0 + (t * 11.0 + seed * 5.0).sin() * 50.0;
        let fade = (t * std::f32::consts::PI).sin().powf(0.7); // fade at the ends
        // bottom edge: bright green; top: purple/transparent
        pos.push([x, 0.0, z]); col.push([0.15, 1.0, 0.55, 1.0 * fade]);
        pos.push([x, top * 0.5, z + 8.0]); col.push([0.1, 0.8, 0.6, 0.55 * fade]);
        pos.push([x, top, z + 16.0]); col.push([0.5, 0.25, 0.9, 0.0]);
        if i < n { let b = (i * 3) as u32; for k in 0..2u32 { idx.extend_from_slice(&[b + k, b + 3 + k, b + k + 1, b + k + 1, b + 3 + k, b + 4 + k]); } }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos).with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col).with_inserted_indices(Indices::U32(idx))
}

fn sky_objects(
    time: Res<Time>,
    tod: Res<TimeOfDay>,
    weather: Res<Weather>,
    assets: Option<Res<SkyAssets>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    cam: Query<&GlobalTransform, With<PlayerCamera>>,
    sun: Query<&GlobalTransform, (With<Sun>, Without<SkyDome>)>,
    moon: Query<&GlobalTransform, (With<Moon>, Without<SkyDome>)>,
    mut dome: Query<&mut Transform, (With<SkyDome>, Without<SunDisc>, Without<MoonDisc>, Without<Aurora>)>,
    mut sun_disc: Query<&mut Transform, (With<SunDisc>, Without<MoonDisc>, Without<SkyDome>, Without<Aurora>)>,
    mut moon_disc: Query<&mut Transform, (With<MoonDisc>, Without<SunDisc>, Without<SkyDome>, Without<Aurora>)>,
    mut aur: Query<(&mut Transform, &Aurora), (Without<SunDisc>, Without<MoonDisc>, Without<SkyDome>)>,
) {
    let (Some(a), Some(c)) = (assets, cam.iter().next()) else { return };
    let st = sky_state(tod.hour);
    let night = (1.0 - st.day).clamp(0.0, 1.0);
    let clear = (1.0 - weather.intensity * 0.9).clamp(0.0, 1.0);
    let cp = c.translation();
    if let Ok(mut t) = dome.single_mut() { t.translation = cp; }
    let t_now = time.elapsed_secs();
    if let Some(mut m) = mats.get_mut(&a.star) { m.base_color = Color::srgba(1.0, 1.0, 1.0, (night * night * clear).min(0.95)); }
    // sun & moon: direction opposite the light's forward
    if let (Ok(s), Ok(mut d)) = (sun.single(), sun_disc.single_mut()) {
        let dir = -s.forward().as_vec3();
        d.translation = dir * 800.0; d.look_at(Vec3::ZERO, Vec3::Y);
        let vis = (dir.y + 0.08).clamp(0.0, 1.0).min(1.0);
        d.scale = Vec3::splat(if dir.y > -0.1 { 150.0 } else { 0.0 });
        if let Some(mut m) = mats.get_mut(&a.sun) { m.base_color = Color::srgba(1.0, 0.82 + 0.1 * dir.y.max(0.0), 0.6 + 0.3 * dir.y.max(0.0), (vis * 6.0).min(1.0) * clear.max(0.3)); }
    }
    if let (Ok(m), Ok(mut d)) = (moon.single(), moon_disc.single_mut()) {
        // the Moon light shines from a fixed direction; show the disc there (clamp above horizon)
        let mut dir = -m.forward().as_vec3(); dir.y = dir.y.abs().max(0.25); dir = dir.normalize();
        d.translation = dir * 800.0; d.look_at(Vec3::ZERO, Vec3::Y);
        if let Some(mut mm) = mats.get_mut(&a.moon) { mm.base_color = Color::srgba(0.8, 0.88, 1.0, (night * night * clear).min(1.0)); }
    }
    // aurora: night only, clear sky; breathing curtains
    for (mut t, au) in &mut aur {
        t.rotation = Quat::from_rotation_y((t_now * 0.05 + au.phase).sin() * 0.08);
        t.scale = Vec3::new(1.0, 0.8 + 0.35 * (t_now * 0.3 + au.phase).sin(), 1.0);
    }
    if let Some(mut m) = mats.get_mut(&a.aurora) {
        let pulse = 0.65 + 0.35 * (t_now * 0.2).sin();
        m.base_color = Color::srgba(1.0, 1.0, 1.0, (night.powi(3) * clear * pulse).clamp(0.0, 1.0));
    }
}

// ---------- puffs (soft particles)
#[derive(Component)]
pub struct Puff { vel: Vec3, life: f32, max: f32, s0: f32, s1: f32, a0: f32 }

#[derive(Resource)]
pub struct FxAssets { mesh: Handle<Mesh>, tex: Handle<Image> }
fn setup_fx_assets(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(FxAssets { mesh: meshes.add(Rectangle::new(1.0, 1.0)), tex: server.load(man.path("fx.puff")) });
}

pub fn spawn_puff(commands: &mut Commands, fx: &FxAssets, mats: &mut Assets<StandardMaterial>, pos: Vec3, vel: Vec3, life: f32, s0: f32, s1: f32, alpha: f32, tint: Color) {
    let mat = mats.add(StandardMaterial { base_color: tint.with_alpha(alpha), base_color_texture: Some(fx.tex.clone()), alpha_mode: AlphaMode::Blend, unlit: true, cull_mode: None, double_sided: true, ..default() });
    commands.spawn((Mesh3d(fx.mesh.clone()), MeshMaterial3d(mat), Transform::from_translation(pos).with_scale(Vec3::splat(s0)), Billboard, Puff { vel, life, max: life, s0, s1, a0: alpha }));
}

fn update_puffs(mut commands: Commands, time: Res<Time>, mut mats: ResMut<Assets<StandardMaterial>>, mut q: Query<(Entity, &mut Puff, &mut Transform, &MeshMaterial3d<StandardMaterial>)>, weather: Res<Weather>) {
    let dt = time.delta_secs();
    for (e, mut p, mut t, m) in &mut q {
        p.life -= dt;
        if p.life <= 0.0 { commands.entity(e).despawn(); continue; }
        let k = 1.0 - p.life / p.max;
        t.translation += (p.vel + Vec3::new(weather.wind.x, 0.0, weather.wind.y) * 0.4 * k) * dt;
        t.scale = Vec3::splat(p.s0 + (p.s1 - p.s0) * k);
        if let Some(mut mm) = mats.get_mut(&m.0) { let c = mm.base_color.to_srgba(); mm.base_color = Color::srgba(c.red, c.green, c.blue, p.a0 * (1.0 - k) * (k * 8.0).min(1.0)); }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_puffs(
    mut commands: Commands,
    time: Res<Time>,
    fx: Option<Res<FxAssets>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    count: Query<(), With<Puff>>,
    cam: Query<&GlobalTransform, With<PlayerCamera>>,
    cars: Query<(&GlobalTransform, &Car, &crate::build::CarParts)>,
    driven: Query<(), With<crate::vehicle::Driven>>,
    mut acc: Local<[f32; 4]>,
) {
    let Some(fx) = fx else { return };
    if count.iter().count() > 260 { return; }
    let dt = time.delta_secs();
    for a in acc.iter_mut() { *a += dt; }
    let white = Color::srgb(0.95, 0.97, 1.0);
    // breath vapour (first person, on foot)
    if acc[0] > 3.2 && driven.is_empty() {
        acc[0] = 0.0;
        if let Some(c) = cam.iter().next() {
            let f = c.forward().as_vec3();
            spawn_puff(&mut commands, &fx, &mut mats, c.translation() + f * 0.45 - Vec3::Y * 0.12, f * 0.5 + Vec3::Y * 0.2, 1.4, 0.08, 0.32, 0.16, white);
        }
    }
    // chimney smoke at the cabin
    if acc[1] > 0.7 {
        acc[1] = 0.0;
        let p = Vec3::new(CABIN.x + 1.8, height_at(CABIN.x, CABIN.y) + 4.3, CABIN.y - 0.8);
        spawn_puff(&mut commands, &fx, &mut mats, p, Vec3::new(0.15, 1.1, 0.05), 5.0, 0.4, 2.6, 0.4, Color::srgb(0.75, 0.77, 0.82));
    }
    // vehicles
    for (g, car, parts) in &cars {
        let (back, up) = (g.back().as_vec3(), Vec3::Y);
        let speed = car.v_long.abs();
        // exhaust
        if car.running && acc[2] > 0.14 / (0.4 + car.throttle) {
            acc[2] = 0.0;
            let dark = parts.get(crate::build::Slot::Engine).cond < 0.3 || parts.get(crate::build::Slot::SparkPlugs).cond < 0.15;
            let tint = if dark { Color::srgb(0.18, 0.18, 0.2) } else { white };
            let p = g.translation() + back * 2.3 + up * 0.35 + g.right().as_vec3() * 0.5;
            spawn_puff(&mut commands, &fx, &mut mats, p, back * (0.6 + car.throttle * 1.4) + up * 0.5, 1.5, 0.18, 1.0, if dark { 0.5 } else { 0.35 }, tint);
        }
        // steam from the hood when overheating
        if parts.temp > 103.0 && acc[3] > 0.1 {
            acc[3] = 0.0;
            let p = g.translation() - back * 1.6 + up * 1.0;
            spawn_puff(&mut commands, &fx, &mut mats, p, up * 1.8 + back * 0.3, 1.8, 0.3, 1.6, 0.5, white);
        }
        // tyre spray when sliding on snow
        if car.slip > 0.45 && speed > 5.0 && acc[2] > 0.05 {
            for s in [-0.8f32, 0.8] {
                let p = g.translation() + back * 1.3 + g.right().as_vec3() * s + up * 0.2;
                spawn_puff(&mut commands, &fx, &mut mats, p, back * 1.5 + up * 1.2 + g.right().as_vec3() * s, 0.9, 0.15, 0.7, 0.45, white);
            }
        }
    }
}
