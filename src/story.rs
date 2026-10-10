//! Story spine: intro -> build the car -> get it running -> winter tyres -> katsastus -> drive to Helsinki -> boss -> ending.
use crate::build::{CarParts, Slot, Toasts};
use crate::combat::enemy::{spawn_enemy, Enemy, EnemyAssets, Kind as EKind, State as EState};
use crate::combat::PlayerStats;
use crate::life::dialog::{Dialog, Screen};
use crate::state::GameState;
use crate::vehicle::spec::TyreType;
use crate::world::layout::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Resource, Serialize, Deserialize, Clone, Default, Debug)]
pub struct Story { pub intro_shown: bool, pub ever_ran: bool, pub inspected: bool, pub hq_spawned: bool, pub boss_dead: bool, pub ended: bool }

pub fn objective_key(s: &Story, p: &CarParts) -> &'static str {
    if !(p.get(Slot::Engine).installed && p.get(Slot::Gearbox).installed) { return "obj.build"; }
    if !s.ever_ran { return "obj.run"; }
    if p.tyre == TyreType::Summer { return "obj.tyres"; }
    if !s.inspected { return "obj.inspect"; }
    if !s.boss_dead { return "obj.helsinki"; }
    "obj.free"
}

pub const CHECK_LIST: &str = "  engine/gearbox/radiator/battery/belt present, brakes >= 50%, tyres fitted & not flat,\n  winter tyres, lights + fuse, windshield, wiper blades, body panels, emissions (plugs, engine)";

#[derive(Component)] pub struct Boss;
#[derive(Component)] struct HqBeacon;
#[derive(Component)] pub struct StoryHud;

pub struct StoryPlugin;
impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Story>().add_message::<RunInspection>()
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (track, hq, intro, hud).run_if(in_state(GameState::Playing)))
            .add_systems(Update, inspection.run_if(in_state(GameState::Dialog)));
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((StoryHud, Text::new(""), TextFont { font_size: FontSize::Px(16.0), ..default() }, TextColor(Color::srgb(0.7, 0.9, 1.0)), Node { position_type: PositionType::Absolute, top: Val::Px(8.0), left: Val::Px(12.0), ..default() }, GlobalZIndex(12)));
}

fn track(mut story: ResMut<Story>, car: Query<&Car>, mut toasts: ResMut<Toasts>) {
    if let Ok(c) = car.single() { if c.running && !story.ever_ran { story.ever_ran = true; toasts.say("It runs! Now make it winter-proof."); } }
}
use crate::vehicle::Car;

fn hud(story: Res<Story>, tr: Res<crate::i18n::Strings>, parts: Query<&CarParts>, mut q: Query<&mut Text, With<StoryHud>>) {
    let (Ok(p), Ok(mut t)) = (parts.single(), q.single_mut()) else { return };
    t.0 = format!("{}: {}", tr.t("hud.objective"), tr.t(objective_key(&story, p)));
}

/// Intro on a fresh game.
fn intro(mut story: ResMut<Story>, mut dlg: ResMut<Dialog>, mut next: ResMut<NextState<GameState>>, pending: Res<crate::save::PendingLoad>, time: Res<Time>) {
    if story.intro_shown || pending.0.is_some() || time.elapsed_secs() < 1.0 { return; }
    story.intro_shown = true;
    dlg.screen = Screen::Intro; dlg.sel = 0; dlg.msg.clear();
    next.set(GameState::Dialog);
}

#[derive(Message)]
pub struct RunInspection;

/// The katsastus: checks the car like the real thing (brakes, lights, glass, tyres, body, emissions).
pub fn checks(p: &CarParts) -> Vec<(String, bool)> {
    let ok = |s: Slot, min: f32| p.ok(s, min);
    vec![
        ("Engine, gearbox, radiator, battery, belt present".into(), ok(Slot::Engine, 0.2) && ok(Slot::Gearbox, 0.2) && ok(Slot::Radiator, 0.2) && ok(Slot::Battery, 0.0) && ok(Slot::Belt, 0.1)),
        ("Brakes (pads >= 50%)".into(), ok(Slot::Brakes, 0.5)),
        ("Tyres: fitted, not flat, >= 35% tread".into(), ok(Slot::Tyres, 0.35) && !p.flat),
        ("Winter tyres (mandatory in winter)".into(), p.tyre != TyreType::Summer),
        ("Lights + fuse working".into(), ok(Slot::Headlights, 0.5) && ok(Slot::Fuse, 0.05)),
        ("Windshield not cracked (>= 60%)".into(), ok(Slot::Windshield, 0.6)),
        ("Wiper blades fitted".into(), ok(Slot::WiperBlades, 0.2)),
        ("Body: hood, bumper, doors >= 40%".into(), ok(Slot::Hood, 0.4) && ok(Slot::Bumper, 0.4) && ok(Slot::DoorL, 0.4) && ok(Slot::DoorR, 0.4)),
        ("Emissions: spark plugs + engine >= 40%".into(), ok(Slot::SparkPlugs, 0.4) && ok(Slot::Engine, 0.4)),
    ]
}

fn inspection(mut ev: MessageReader<RunInspection>, mut story: ResMut<Story>, parts: Query<&CarParts>, mut stats: ResMut<PlayerStats>, mut dlg: ResMut<Dialog>, mut toasts: ResMut<Toasts>) {
    for _ in ev.read() {
        let Ok(p) = parts.single() else { continue };
        if stats.money < 30 { dlg.msg = "The inspection fee is 30 EUR.".into(); continue; }
        stats.money -= 30;
        let res = checks(p);
        let fails: Vec<&str> = res.iter().filter(|(_, ok)| !*ok).map(|(n, _)| n.as_str()).collect();
        if fails.is_empty() {
            story.inspected = true; stats.money += 100;
            dlg.msg = "HYVAKSYTTY - PASSED! The inspector shakes your hand. Old colleague slips you 100 EUR: 'Helsinki is waiting.'".into();
            toasts.say("Katsastus passed! New objective: drive south to Helsinki.");
        } else {
            dlg.msg = format!("HYLATTY - FAILED. Fix:\n  - {}", fails.join("\n  - "));
        }
    }
}

/// Lazily build the Agency HQ compound + boss when the player gets within range after passing inspection.
#[allow(clippy::too_many_arguments)]
fn hq(
    mut commands: Commands,
    mut story: ResMut<Story>,
    ea: Option<Res<EnemyAssets>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    player: Query<&Transform, With<crate::player::Player>>,
    boss: Query<(&Enemy, &Transform), With<Boss>>,
    beacon: Query<(), With<HqBeacon>>,
    bassets: Option<Res<crate::world::village::BuildingAssets>>,
    mut stats: ResMut<PlayerStats>,
    mut dlg: ResMut<Dialog>,
    mut next: ResMut<NextState<GameState>>,
) {
    if !story.inspected { return; }
    if beacon.is_empty() && !story.boss_dead {
        let mesh = meshes.add(Cylinder::new(0.9, 220.0));
        let mat = mats.add(StandardMaterial { base_color: Color::srgba(1.0, 0.15, 0.1, 0.4), unlit: true, alpha_mode: AlphaMode::Add, fog_enabled: false, ..default() });
        commands.spawn((HqBeacon, Mesh3d(mesh), MeshMaterial3d(mat), Transform::from_xyz(HQ.x, height_at(HQ.x, HQ.y) + 110.0, HQ.y)));
    }
    let Ok(pt) = player.single() else { return };
    if !story.hq_spawned && Vec2::new(pt.translation.x, pt.translation.z).distance(HQ) < 700.0 {
        if let (Some(ea), Some(ba)) = (&ea, &bassets) {
            story.hq_spawned = true;
            use crate::world::village::{spawn_house, HouseSpec, Wall};
            for (dx, dz, w, d, wall) in [(-22.0, -10.0, 16.0, 10.0, Wall::Brick), (22.0, -10.0, 16.0, 10.0, Wall::Brick), (0.0, 22.0, 24.0, 12.0, Wall::Cream)] {
                spawn_house(&mut commands, ba, &mut meshes, &mut mats, &HouseSpec { pos: HQ + Vec2::new(dx, dz), yaw: 0.0, w, d, h: 5.0, wall, windows: 6, door: true, open_front: false });
            }
            for i in 0..7 { let a = i as f32 * 0.9; let p = HQ + Vec2::new(a.cos() * 14.0, a.sin() * 14.0); let e = spawn_enemy(&mut commands, ea, &mut mats, if i % 3 == 0 { EKind::Hunter } else { EKind::Goon }, p); let _ = e; }
            let b = spawn_enemy(&mut commands, ea, &mut mats, EKind::Director, HQ + Vec2::new(0.0, 4.0));
            commands.entity(b).insert((Boss, Name::new("The Director")));
            commands.entity(b).entry::<Enemy>().and_modify(|mut e| { e.alerted = true; });
        }
    }
    if story.hq_spawned && !story.boss_dead {
        if let Ok((e, _)) = boss.single() { if e.state == EState::Dead { story.boss_dead = true; stats.money += 1000; dlg.screen = Screen::Ending; dlg.sel = 0; dlg.msg.clear(); next.set(GameState::Dialog); story.ended = true; } }
    }
}
