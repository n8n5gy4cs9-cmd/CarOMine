use super::*;
use crate::manifest::AssetManifest;
use crate::player::Player;
use crate::world::billboard::*;
use crate::world::layout::*;
use crate::world::scatter::Obstacle;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Goon, Hunter, Neighbour, Wolf, Moose }

pub struct Def {
    pub name: &'static str,
    pub sprite: &'static str,
    pub hp: f32,
    pub speed: f32,
    pub w: f32, pub h: f32,
    pub sight: f32,
    pub ranged: bool,
    pub range: f32,      // attack range
    pub damage: f32,
    pub rate: f32,
    pub accuracy: f32,   // ranged hit chance at point blank
}
pub fn def(k: Kind) -> Def {
    match k {
        Kind::Goon => Def { name: "ex-agency goon", sprite: "enemy.goon", hp: 60.0, speed: 3.2, w: 1.5, h: 2.0, sight: 55.0, ranged: true, range: 35.0, damage: 9.0, rate: 1.1, accuracy: 0.65 },
        Kind::Hunter => Def { name: "rogue hunter", sprite: "enemy.hunter", hp: 80.0, speed: 2.8, w: 1.5, h: 2.0, sight: 90.0, ranged: true, range: 70.0, damage: 20.0, rate: 2.2, accuracy: 0.7 },
        Kind::Neighbour => Def { name: "drunk neighbour", sprite: "enemy.neighbour", hp: 40.0, speed: 2.2, w: 1.5, h: 2.0, sight: 30.0, ranged: false, range: 1.8, damage: 8.0, rate: 0.9, accuracy: 1.0 },
        Kind::Wolf => Def { name: "wolf", sprite: "enemy.wolf", hp: 35.0, speed: 7.0, w: 1.4, h: 1.0, sight: 45.0, ranged: false, range: 1.6, damage: 10.0, rate: 0.7, accuracy: 1.0 },
        Kind::Moose => Def { name: "moose", sprite: "enemy.moose", hp: 220.0, speed: 6.0, w: 3.2, h: 2.4, sight: 35.0, ranged: false, range: 2.6, damage: 28.0, rate: 1.6, accuracy: 1.0 },
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State { Idle, Chase, Attack, Dead }

#[derive(Component)]
pub struct Enemy {
    pub kind: Kind,
    pub hp: f32,
    pub state: State,
    pub yaw: f32,
    pub cooldown: f32,
    pub attack_anim: f32,
    pub walk_t: f32,
    pub dead_t: f32,
    pub wander: Vec2,
    pub wander_t: f32,
    pub flash: f32,
    pub alerted: bool,
}

#[derive(Resource)]
pub struct EnemyAssets { pub mesh: std::collections::HashMap<u8, Handle<Mesh>>, pub tex: std::collections::HashMap<u8, Handle<Image>> }
fn key(k: Kind) -> u8 { k as u8 }
const KINDS: [Kind; 5] = [Kind::Goon, Kind::Hunter, Kind::Neighbour, Kind::Wolf, Kind::Moose];

pub fn load_enemy_assets(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut meshes: ResMut<Assets<Mesh>>) {
    let mut a = EnemyAssets { mesh: Default::default(), tex: Default::default() };
    for k in KINDS {
        let d = def(k);
        a.mesh.insert(key(k), meshes.add(feet_quad(d.w * 1.25, d.h * 1.25 * 64.0 / 48.0 * 0.75)));
        a.tex.insert(key(k), load_nearest(&server, man.path(d.sprite)));
    }
    commands.insert_resource(a);
}

pub fn spawn_enemy(commands: &mut Commands, a: &EnemyAssets, mats: &mut Assets<StandardMaterial>, k: Kind, pos: Vec2) {
    let d = def(k);
    let mut m = sprite_material(a.tex[&key(k)].clone());
    m.unlit = false;
    m.emissive = LinearRgba::gray(0.05);
    commands.spawn((
        Mesh3d(a.mesh[&key(k)].clone()),
        MeshMaterial3d(mats.add(m)),
        Transform::from_xyz(pos.x, height_at(pos.x, pos.y), pos.y),
        Billboard,
        AngleSprite { cols: 10, rows: 8, col: 0, facing: 0.0 },
        Hittable { radius: d.w * 0.32, height: d.h },
        Enemy { kind: k, hp: d.hp, state: State::Idle, yaw: 0.0, cooldown: 1.0, attack_anim: 0.0, walk_t: 0.0, dead_t: 0.0, wander: pos, wander_t: 0.0, flash: 0.0, alerted: false },
    ));
}

pub fn spawn_initial(mut commands: Commands, a: Option<Res<EnemyAssets>>, mut mats: ResMut<Assets<StandardMaterial>>, existing: Query<(), With<Enemy>>) {
    let Some(a) = a else { return };
    if !existing.is_empty() { return; }
    // a handful around the start area so there's something to shoot; wider spawns come with the mission system (T-066)
    let spots = [
        (Kind::Goon, Vec2::new(-10.0, -120.0)), (Kind::Goon, Vec2::new(5.0, -140.0)), (Kind::Goon, Vec2::new(-40.0, -160.0)),
        (Kind::Hunter, Vec2::new(140.0, 60.0)), (Kind::Hunter, Vec2::new(-120.0, 20.0)),
        (Kind::Neighbour, Vec2::new(30.0, 80.0)), (Kind::Neighbour, Vec2::new(90.0, 20.0)),
        (Kind::Wolf, Vec2::new(-90.0, 90.0)), (Kind::Wolf, Vec2::new(-100.0, 100.0)), (Kind::Wolf, Vec2::new(180.0, -60.0)),
        (Kind::Moose, Vec2::new(-60.0, -40.0)), (Kind::Moose, Vec2::new(200.0, 20.0)),
    ];
    for (k, p) in spots { spawn_enemy(&mut commands, &a, &mut mats, k, p); }
}

pub fn apply_hits(mut hits: MessageReader<HitEvent>, mut q: Query<&mut Enemy>) {
    for h in hits.read() {
        if let Ok(mut e) = q.get_mut(h.target) {
            if e.state == State::Dead { continue; }
            e.hp -= h.damage; e.flash = 0.12; e.alerted = true;
            if e.hp <= 0.0 { e.state = State::Dead; e.dead_t = 0.0; }
        }
    }
}

pub fn ai(
    time: Res<Time>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    obstacles: Query<(&GlobalTransform, &Obstacle), Without<Enemy>>,
    mut stats: ResMut<PlayerStats>,
    mut q: Query<(&mut Enemy, &mut Transform)>,
) {
    let Ok(pt) = player.single() else { return };
    let dt = time.delta_secs().min(0.05);
    let t = time.elapsed_secs();
    let pp = pt.translation;
    for (mut e, mut tr) in &mut q {
        let d = def(e.kind);
        e.cooldown = (e.cooldown - dt).max(0.0);
        e.attack_anim = (e.attack_anim - dt).max(0.0);
        e.flash = (e.flash - dt).max(0.0);
        if e.state == State::Dead { e.dead_t += dt; continue; }
        let to_p = Vec2::new(pp.x - tr.translation.x, pp.z - tr.translation.z);
        let dist = to_p.length();
        let alive_player = stats.hp > 0.0;
        // sight (cheap): distance, plus alerted memory
        if alive_player && (dist < d.sight * if e.alerted { 1.6 } else { 1.0 }) {
            if e.state == State::Idle { e.state = State::Chase; e.alerted = true; }
        } else if !alive_player || dist > d.sight * 2.5 { e.state = State::Idle; e.alerted = false; }
        let mut dir = Vec2::ZERO;
        match e.state {
            State::Idle => {
                e.wander_t -= dt;
                if e.wander_t <= 0.0 {
                    let h = crate::world::noise::hash2((t * 10.0) as i32, tr.translation.x as i32, 77);
                    let a = h * std::f32::consts::TAU;
                    e.wander = Vec2::new(tr.translation.x, tr.translation.z) + Vec2::new(a.cos(), a.sin()) * 8.0;
                    e.wander_t = 3.0 + h * 4.0;
                }
                let w = e.wander - Vec2::new(tr.translation.x, tr.translation.z);
                if w.length() > 1.0 { dir = w.normalize() * 0.35; }
            }
            State::Chase | State::Attack => {
                e.yaw = to_p.x.atan2(to_p.y);
                if dist <= d.range * if d.ranged { 0.9 } else { 1.0 } {
                    e.state = State::Attack;
                } else { e.state = State::Chase; dir = to_p.normalize_or_zero(); }
                if e.state == State::Attack && e.cooldown <= 0.0 {
                    e.cooldown = d.rate; e.attack_anim = 0.25;
                    let hit_chance = if d.ranged { d.accuracy * (1.0 - dist / (d.range * 1.3)).clamp(0.1, 1.0) } else { 0.9 };
                    let r = crate::world::noise::hash2((t * 1000.0) as i32, dist as i32, 99);
                    if r < hit_chance { stats.damage(d.damage); }
                }
            }
            State::Dead => {}
        }
        if dir != Vec2::ZERO {
            if e.state == State::Idle { e.yaw = dir.x.atan2(dir.y); }
            let sp = d.speed * dir.length().min(1.0) * if e.state == State::Idle { 0.5 } else { 1.0 };
            let mut np = tr.translation + Vec3::new(dir.x, 0.0, dir.y).normalize_or_zero() * sp * dt;
            for (g, o) in obstacles.iter() {
                let v = Vec2::new(np.x - g.translation().x, np.z - g.translation().z);
                let r = o.0 + 0.5;
                if v.length_squared() < r * r && v.length_squared() > 1e-6 {
                    let p = v.normalize() * r; np.x = g.translation().x + p.x; np.z = g.translation().z + p.y;
                }
            }
            // don't walk into deep lake ice cracks etc. (no-op now); follow terrain
            np.y = height_at(np.x, np.z);
            tr.translation = np;
            e.walk_t += dt * (sp * 1.4 + 1.0);
        }
    }
}

pub fn animate(mut q: Query<(&mut Enemy, &mut AngleSprite, &MeshMaterial3d<StandardMaterial>)>, mut mats: ResMut<Assets<StandardMaterial>>) {
    for (e, mut s, mh) in &mut q {
        s.facing = e.yaw;
        s.col = match e.state {
            State::Dead => 7 + ((e.dead_t / 0.18) as u32).min(2),
            _ if e.attack_anim > 0.0 => 5 + (e.attack_anim > 0.12) as u32,
            State::Chase | State::Attack | State::Idle if e.walk_t > 0.0 && e.state == State::Chase => 1 + (e.walk_t * 4.0) as u32 % 4,
            State::Idle => if e.walk_t.fract() > 0.0 && e.wander_t > 0.0 && (e.walk_t * 3.0) as u32 % 2 == 1 { 1 + (e.walk_t * 3.0) as u32 % 4 } else { 0 },
            _ => 0,
        };
        if let Some(mut m) = mats.get_mut(&mh.0) {
            m.base_color = if e.flash > 0.0 { Color::srgb(1.0, 0.35, 0.35) } else { Color::WHITE };
        }
    }
}

/// Dead enemies are flat on the ground: stop billboard-ing is unnecessary (sprite shows lying frames), but corpses despawn eventually.
pub fn car_runover(
    mut commands: Commands,
    cars: Query<(&Transform, &crate::vehicle::Car), Without<Enemy>>,
    mut q: Query<(Entity, &mut Enemy, &Transform)>,
    mut drops: ResMut<pickups::DropQueue>,
) {
    for (e_ent, mut e, et) in &mut q {
        if e.state == State::Dead {
            if e.dead_t > 45.0 { commands.entity(e_ent).despawn(); }
            continue;
        }
        for (ct, car) in &cars {
            if car.v_long.abs() > 5.0 && ct.translation.distance(et.translation) < 2.4 {
                e.hp -= car.v_long.abs() * 9.0; e.flash = 0.2; e.alerted = true;
                if e.hp <= 0.0 { e.state = State::Dead; e.dead_t = 0.0; }
            }
        }
        // drop loot once on death
        if e.state == State::Dead && e.dead_t == 0.0 { drops.0.push((et.translation, e.kind)); }
    }
}
