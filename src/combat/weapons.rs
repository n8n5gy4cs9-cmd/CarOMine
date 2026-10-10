use super::*;
use crate::manifest::AssetManifest;
use crate::player::PlayerCamera;
use crate::world::scatter::Obstacle;
use bevy::prelude::*;

pub struct WeaponDef {
    pub name: &'static str,
    pub id: &'static str,    // manifest id suffix
    pub damage: f32,
    pub pellets: u32,
    pub spread: f32,         // radians
    pub rate: f32,           // seconds between shots
    pub mag: u32,
    pub range: f32,
    pub auto: bool,
    pub melee: bool,
}
pub const WEAPONS: [WeaponDef; 5] = [
    WeaponDef { name: "Fists", id: "fists", damage: 14.0, pellets: 1, spread: 0.0, rate: 0.45, mag: 0, range: 2.2, auto: false, melee: true },
    WeaponDef { name: "Pistol", id: "pistol", damage: 22.0, pellets: 1, spread: 0.008, rate: 0.28, mag: 12, range: 120.0, auto: false, melee: false },
    WeaponDef { name: "Shotgun", id: "shotgun", damage: 11.0, pellets: 9, spread: 0.07, rate: 0.9, mag: 6, range: 40.0, auto: false, melee: false },
    WeaponDef { name: "SMG", id: "smg", damage: 11.0, pellets: 1, spread: 0.03, rate: 0.085, mag: 30, range: 90.0, auto: true, melee: false },
    WeaponDef { name: "Hunting rifle", id: "rifle", damage: 70.0, pellets: 1, spread: 0.0, rate: 1.1, mag: 5, range: 400.0, auto: false, melee: false },
];

#[derive(Resource)]
pub struct Loadout {
    pub current: usize,
    pub owned: [bool; 5],
    pub mag: [u32; 5],
    pub reserve: [u32; 5],
    pub cooldown: f32,
    pub reload_t: f32,
    pub since_shot: f32,
}
impl Default for Loadout {
    fn default() -> Self {
        Self { current: 0, owned: [true, true, false, false, false], mag: [0, 12, 0, 0, 0], reserve: [0, 24, 0, 0, 0], cooldown: 0.0, reload_t: 0.0, since_shot: 9.0 }
    }
}

#[derive(Resource)]
pub struct WeaponAssets { pub sheets: Vec<Handle<Image>>, pub layout: Handle<TextureAtlasLayout> }

pub fn load_weapon_assets(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut layouts: ResMut<Assets<TextureAtlasLayout>>) {
    let sheets = WEAPONS.iter().map(|w| crate::world::billboard::load_nearest(&server, man.path(&format!("weapon.{}", w.id)))).collect();
    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::new(160, 128), 3, 1, None, None));
    commands.insert_resource(WeaponAssets { sheets, layout });
}

pub fn switch_and_fire(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    kb: Res<crate::settings::Keybinds>,
    stats: Res<PlayerStats>,
    mut lo: ResMut<Loadout>,
    cam: Query<&GlobalTransform, With<PlayerCamera>>,
    in_car: Query<(), With<crate::vehicle::Driven>>,
    targets: Query<(Entity, &GlobalTransform, &Hittable)>,
    obstacles: Query<(&GlobalTransform, &Obstacle), Without<Hittable>>,
    mut hits: MessageWriter<HitEvent>,
    mut sfx: MessageWriter<crate::audio::Sfx>,
) {
    let dt = time.delta_secs();
    lo.cooldown = (lo.cooldown - dt).max(0.0);
    lo.since_shot += dt;
    if stats.hp <= 0.0 || !in_car.is_empty() { return; }
    for (i, k) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5].iter().enumerate() {
        if keys.just_pressed(*k) && lo.owned[i] && lo.current != i { lo.current = i; lo.cooldown = 0.25; lo.reload_t = 0.0; }
    }
    let cur = lo.current;
    let w = &WEAPONS[cur];
    // reload
    if lo.reload_t > 0.0 {
        lo.reload_t -= dt;
        if lo.reload_t <= 0.0 {
            let need = w.mag - lo.mag[cur];
            let take = need.min(lo.reserve[cur]);
            lo.mag[cur] += take; lo.reserve[cur] -= take;
        }
        return;
    }
    if !w.melee && (keys.just_pressed(kb.reload) || (lo.mag[cur] == 0 && mouse.just_pressed(MouseButton::Left))) && lo.mag[cur] < w.mag && lo.reserve[cur] > 0 {
        lo.reload_t = 1.1; sfx.write(crate::audio::Sfx::new("sfx.reload")); return;
    }
    let want = if w.auto { mouse.pressed(MouseButton::Left) } else { mouse.just_pressed(MouseButton::Left) };
    if !want || lo.cooldown > 0.0 { return; }
    if !w.melee {
        if lo.mag[cur] == 0 { return; }
        lo.mag[cur] -= 1;
    }
    lo.cooldown = w.rate; lo.since_shot = 0.0;
    sfx.write(crate::audio::Sfx::new(match w.id { "pistol" => "sfx.pistol", "shotgun" => "sfx.shotgun", "smg" => "sfx.smg", "rifle" => "sfx.rifle", _ => "sfx.hit" }));
    let Ok(c) = cam.single() else { return };
    let origin = c.translation();
    let fwd = c.forward().as_vec3();
    let (right, up) = (c.right().as_vec3(), c.up().as_vec3());
    let mut seed = (time.elapsed_secs() * 1000.0) as u32;
    let mut rnd = || { seed = seed.wrapping_mul(1664525).wrapping_add(1013904223); ((seed >> 8) as f32 / 16_777_216.0) * 2.0 - 1.0 };
    for _ in 0..w.pellets {
        let dir = (fwd + right * rnd() * w.spread + up * rnd() * w.spread).normalize();
        if let Some((e, head)) = raycast(origin, dir, w.range, &targets, &obstacles) {
            hits.write(HitEvent { target: e, damage: w.damage * if head { 2.0 } else { 1.0 }, headshot: head });
        }
    }
}

/// Hitscan against hittables; trees/rocks block.
pub fn raycast(
    o: Vec3, d: Vec3, range: f32,
    targets: &Query<(Entity, &GlobalTransform, &Hittable)>,
    obstacles: &Query<(&GlobalTransform, &Obstacle), Without<Hittable>>,
) -> Option<(Entity, bool)> {
    let d2 = Vec2::new(d.x, d.z);
    let dl2 = d2.length_squared().max(1e-6);
    let mut best: Option<(f32, Entity, bool)> = None;
    for (e, g, h) in targets.iter() {
        let c = g.translation();
        let rel = Vec2::new(c.x - o.x, c.z - o.z);
        let t2 = rel.dot(d2) / dl2; // param along ray (d unit => metres / |d2|)
        if t2 <= 0.0 { continue; }
        let t = t2; // d is normalised so t is metres
        if t > range { continue; }
        let closest = Vec2::new(o.x, o.z) + d2 * t2;
        if closest.distance(Vec2::new(c.x, c.z)) > h.radius { continue; }
        let y = o.y + d.y * t;
        if y < c.y || y > c.y + h.height { continue; }
        if best.map_or(true, |b| t < b.0) { best = Some((t, e, y > c.y + h.height * 0.78)); }
    }
    let (t_hit, e, head) = best?;
    // blocked by obstacle nearer than the hit?
    for (g, ob) in obstacles.iter() {
        let c = g.translation();
        let rel = Vec2::new(c.x - o.x, c.z - o.z);
        let t2 = rel.dot(d2) / dl2;
        if t2 <= 0.0 || t2 >= t_hit { continue; }
        let closest = Vec2::new(o.x, o.z) + d2 * t2;
        if closest.distance(Vec2::new(c.x, c.z)) < ob.0 * 0.8 && o.y + d.y * t2 < c.y + 2.5 { return None; }
    }
    Some((e, head))
}
