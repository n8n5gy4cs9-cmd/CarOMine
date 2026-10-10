//! Population director: keeps a few wanderers near the player (more at night / in storms), culls far idle ones.
use super::enemy::*;
use crate::life::missions::MissionTarget;
use crate::story::Boss;
use crate::world::layout::*;
use crate::world::noise::{fbm, hash2};
use crate::world::sky::{sky_state, TimeOfDay};
use crate::world::weather::Weather;
use bevy::prelude::*;

pub fn run(
    mut commands: Commands,
    time: Res<Time>,
    tod: Res<TimeOfDay>,
    weather: Res<Weather>,
    ea: Option<Res<EnemyAssets>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    player: Query<&Transform, With<crate::player::Player>>,
    enemies: Query<(Entity, &Enemy, &Transform, Option<&MissionTarget>, Option<&Boss>)>,
    mut acc: Local<f32>,
) {
    let (Some(ea), Ok(pt)) = (ea, player.single()) else { return };
    *acc += time.delta_secs();
    let pp = Vec2::new(pt.translation.x, pt.translation.z);
    // cull far idle wanderers
    for (e, en, t, mt, boss) in &enemies {
        if mt.is_some() || boss.is_some() { continue; }
        let far = Vec2::new(t.translation.x, t.translation.z).distance(pp) > 380.0;
        if far && (en.state == State::Idle || en.state == State::Dead) { commands.entity(e).despawn(); }
    }
    if *acc < 7.0 { return; }
    *acc = 0.0;
    let night = 1.0 - sky_state(tod.hour).day;
    let cap = (5.0 + 5.0 * night + 3.0 * weather.intensity) as usize;
    let alive = enemies.iter().filter(|(_, e, t, _, _)| e.state != State::Dead && Vec2::new(t.translation.x, t.translation.z).distance(pp) < 380.0).count();
    if alive >= cap { return; }
    // pick a spawn spot 90..170 m away that is on land, off-road and not in a settlement
    let seed = (time.elapsed_secs() * 100.0) as i32;
    for tries in 0..12 {
        let a = hash2(seed, tries, 201) * std::f32::consts::TAU;
        let r = 90.0 + hash2(seed, tries, 202) * 80.0;
        let p = pp + Vec2::new(a.cos(), a.sin()) * r;
        if p.x.abs() > WORLD_HALF - 40.0 || p.y.abs() > WORLD_HALF - 40.0 { continue; }
        if lake_dist(p.x, p.y) < 8.0 || road_dist(p.x, p.y) < 6.0 || p.distance(CABIN) < 60.0 || p.distance(VILLAGE) < 70.0 || p.distance(HQ) < 60.0 { continue; }
        let forest = fbm(p.x / 140.0, p.y / 140.0, SEED + 21, 3);
        let pick = hash2(seed, tries, 203);
        let kind = if p.distance(VILLAGE) < 400.0 { if pick < 0.5 { Kind::Goon } else if pick < 0.8 { Kind::Neighbour } else { Kind::Hunter } }
            else if forest > 0.5 { if pick < 0.45 { Kind::Wolf } else if pick < 0.75 { Kind::Hunter } else { Kind::Moose } }
            else if pick < 0.5 { Kind::Moose } else { Kind::Goon };
        let group = if kind == Kind::Wolf { 3 } else { 1 + (pick * 2.0) as i32 };
        for g in 0..group { spawn_enemy(&mut commands, &ea, &mut mats, kind, p + Vec2::new(g as f32 * 2.5, 0.0)); }
        return;
    }
}
