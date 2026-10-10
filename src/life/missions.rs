//! Agency gigs (phone in the cabin). Types: Hunt (clear a camp), Deliver (parcel to the village), Recover (briefcase from a guarded spot).
use super::*;
use crate::combat::enemy::{spawn_enemy, Enemy, EnemyAssets, Kind as EKind, State as EState};
use crate::combat::PlayerStats;
use crate::world::noise::hash2;
use crate::world::sky::TimeOfDay;
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub enum MKind { Hunt { target: u8, count: u32 }, Deliver { to: Vec2, label: &'static str, deadline_h: f32 }, Recover { at: Vec2, have: bool } }

#[derive(Clone, Debug)]
pub struct Mission { pub title: String, pub desc: String, pub reward: i32, pub kind: MKind, pub center: Vec2, pub spawned: bool, pub started_hour: f32 }

impl Mission {
    pub fn target(&self) -> Vec2 {
        match &self.kind { MKind::Deliver { to, .. } => *to, MKind::Recover { have: true, .. } => door(CABIN, 0.0, 5.0), _ => self.center }
    }
    pub fn hud_line(&self, from: Vec3) -> String {
        let t = self.target(); let d = Vec2::new(from.x, from.z).distance(t);
        let dir = Vec2::new(t.x - from.x, t.y - from.z);
        let ang = dir.x.atan2(-dir.y).to_degrees().rem_euclid(360.0);
        let compass = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"][((ang + 22.5) / 45.0) as usize % 8];
        format!("GIG: {} ({} EUR) - {:.0} m {}", self.title, self.reward, d, compass)
    }
}

#[derive(Resource, Default)]
pub struct Missions { pub active: Option<Mission>, pub board: Vec<Mission>, pub done: u32, pub day: u32, pub killed_marker: u32 }

#[derive(Component)] pub struct MissionTarget;
#[derive(Component)] pub struct Briefcase;
#[derive(Component)] pub struct Beacon;
#[derive(Component)] pub struct MissionHud;

impl Missions {
    pub fn new_day(&mut self) { self.day += 1; self.board.clear(); }
    pub fn ensure_board(&mut self) {
        if !self.board.is_empty() { return; }
        let d = self.day as i32;
        let pick = |salt: i32, min: f32, max: f32| -> Vec2 {
            for tries in 0..20 {
                let a = hash2(d, salt + tries, 11) * std::f32::consts::TAU; let r = min + hash2(d, salt + tries, 12) * (max - min);
                let p = CABIN + Vec2::new(a.cos(), a.sin()) * r;
                if lake_dist(p.x, p.y) > 20.0 && p.x.abs() < WORLD_HALF - 100.0 && p.y.abs() < WORLD_HALF - 100.0 { return p; }
            }
            CABIN + Vec2::new(200.0, 0.0)
        };
        let hunt_kinds = [(EKind::Goon as u8, "ex-agency goons", 4u32), (EKind::Hunter as u8, "rogue hunters", 3), (EKind::Wolf as u8, "wolves", 5)];
        let hk = hunt_kinds[(hash2(d, 1, 2) * 3.0) as usize % 3];
        self.board = vec![
            Mission { title: format!("Clear the camp: {} x{}", hk.1, hk.2), desc: format!("Intel says {} are camped out in the woods. Take them out.", hk.1), reward: 80 + hk.2 as i32 * 20, kind: MKind::Hunt { target: hk.0, count: hk.2 }, center: pick(10, 150.0, 450.0), spawned: false, started_hour: 0.0 },
            Mission { title: "Parcel for the auto shop".into(), desc: "Deliver a sealed parcel to the auto parts shop in the village. No questions. 6 hours.".into(), reward: 60, kind: MKind::Deliver { to: door(VILLAGE + Vec2::new(18.0, 42.0), -std::f32::consts::FRAC_PI_2, 6.0), label: "auto shop", deadline_h: 6.0 }, center: VILLAGE, spawned: false, started_hour: 0.0 },
            { let at = pick(30, 250.0, 600.0); Mission { title: "Recover the briefcase".into(), desc: "An old contact left a briefcase at a stash site, guarded by goons. Bring it home.".into(), reward: 160, kind: MKind::Recover { at, have: false }, center: at, spawned: false, started_hour: 0.0 } },
        ];
    }
}

pub fn spawn_hud(mut commands: Commands) {
    commands.spawn((MissionHud, Text::new(""), TextFont { font_size: FontSize::Px(18.0), ..default() }, TextColor(Color::srgb(1.0, 0.9, 0.4)), Node { position_type: PositionType::Absolute, top: Val::Px(34.0), left: Val::Percent(30.0), ..default() }, GlobalZIndex(12)));
}

#[allow(clippy::too_many_arguments)]
pub fn tick(
    mut commands: Commands,
    mut m: ResMut<Missions>,
    tod: Res<TimeOfDay>,
    mut stats: ResMut<PlayerStats>,
    mut toasts: ResMut<crate::build::Toasts>,
    ea: Option<Res<EnemyAssets>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    player: Query<&Transform, With<crate::player::Player>>,
    targets: Query<(Entity, &Enemy), With<MissionTarget>>,
    beacon: Query<Entity, With<Beacon>>,
    brief: Query<(Entity, &Transform), With<Briefcase>>,
    mut hud: Query<&mut Text, With<MissionHud>>,
    mut sfx: MessageWriter<crate::audio::Sfx>,
) {
    m.ensure_board();
    let pp = player.single().map(|p| p.translation).unwrap_or(Vec3::ZERO);
    let Some(mut mission) = m.active.clone() else {
        for e in &beacon { commands.entity(e).despawn(); }
        if let Ok(mut t) = hud.single_mut() { t.0.clear(); }
        return;
    };
    // beacon pillar at the objective
    let tgt = mission.target();
    if beacon.is_empty() {
        let mesh = meshes.add(Cylinder::new(0.5, 160.0));
        let mat = mats.add(StandardMaterial { base_color: Color::srgba(1.0, 0.85, 0.2, 0.35), unlit: true, alpha_mode: AlphaMode::Add, fog_enabled: false, ..default() });
        commands.spawn((Beacon, Mesh3d(mesh), MeshMaterial3d(mat), Transform::from_xyz(tgt.x, height_at(tgt.x, tgt.y) + 80.0, tgt.y)));
    } else if let Ok(e) = beacon.single() {
        commands.entity(e).insert(Transform::from_xyz(tgt.x, height_at(tgt.x, tgt.y) + 80.0, tgt.y));
    }
    let mut finished = false;
    match &mut mission.kind {
        MKind::Hunt { target, count } => {
            if !mission.spawned {
                if let Some(ea) = &ea {
                    let k = match *target { 0 => EKind::Goon, 1 => EKind::Hunter, 2 => EKind::Neighbour, 3 => EKind::Wolf, _ => EKind::Moose };
                    for i in 0..*count { let a = i as f32 / *count as f32 * std::f32::consts::TAU; let p = mission.center + Vec2::new(a.cos(), a.sin()) * 9.0; let e = spawn_enemy(&mut commands, ea, &mut mats, k, p); commands.entity(e).insert(MissionTarget); }
                    mission.spawned = true;
                }
            } else {
                let alive = targets.iter().filter(|(_, e)| e.state != EState::Dead).count();
                if alive == 0 { finished = true; }
            }
        }
        MKind::Deliver { to, deadline_h, .. } => {
            let el = (tod.hour - mission.started_hour).rem_euclid(24.0);
            if Vec2::new(pp.x, pp.z).distance(*to) < 4.5 { finished = true; }
            else if el > *deadline_h { toasts.say("The parcel is late... gig failed."); m.active = None; return; }
        }
        MKind::Recover { at, have } => {
            if !mission.spawned {
                if let Some(ea) = &ea {
                    for i in 0..3 { let a = i as f32 * 2.1; let p = *at + Vec2::new(a.cos(), a.sin()) * 8.0; let e = spawn_enemy(&mut commands, ea, &mut mats, EKind::Goon, p); commands.entity(e).insert(MissionTarget); }
                    // the briefcase: a billboard reusing the key sprite
                    mission.spawned = true;
                    commands.spawn((Briefcase, Transform::from_xyz(at.x, height_at(at.x, at.y) + 0.1, at.y), Visibility::default()));
                }
            }
            if !*have {
                for (e, t) in &brief { if t.translation.distance(pp) < 2.2 { *have = true; commands.entity(e).despawn(); toasts.say("Got the briefcase! Bring it home to the cabin."); sfx.write(crate::audio::Sfx::new("sfx.pickup")); } }
            } else if Vec2::new(pp.x, pp.z).distance(door(CABIN, 0.0, 5.0)) < 4.0 { finished = true; }
        }
    }
    if finished {
        stats.money += mission.reward; m.done += 1;
        toasts.say(format!("Gig complete: +{} EUR.", mission.reward));
        sfx.write(crate::audio::Sfx::new("sfx.ui_confirm"));
        for (e, _) in &targets { commands.entity(e).insert(Despawn45); }
        m.active = None; return;
    }
    if let Ok(mut t) = hud.single_mut() { t.0 = mission.hud_line(pp); }
    m.active = Some(mission);
}

/// Mission enemies linger as corpses (handled by enemy corpse timer) - marker kept for clarity.
#[derive(Component)] pub struct Despawn45;

pub fn abandon(m: &mut Missions, commands: &mut Commands, targets: &Query<Entity, With<MissionTarget>>, brief: &Query<Entity, With<Briefcase>>) {
    m.active = None;
    for e in targets.iter() { commands.entity(e).despawn(); }
    for e in brief.iter() { commands.entity(e).despawn(); }
}
