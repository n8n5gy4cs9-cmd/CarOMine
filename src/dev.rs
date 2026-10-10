//! Dev/test harness (env vars). Lets CI/AI run the game headless (xvfb + lavapipe) and grab a screenshot.
//!   CAROMINE_AUTOSTART=1        skip menu
//!   CAROMINE_SHOT=out.png       screenshot after CAROMINE_SHOT_FRAMES (default 150) frames in Playing, then exit
//!   CAROMINE_DRIVE=throttle,steer[,hb]  auto-enter car, engine on, lights on, scripted inputs (e.g. 1,0.3)  CAROMINE_WS=0..1 windshield snow  CAROMINE_WIPERS=0..2
//!   CAROMINE_HOUR=13.5          start hour       CAROMINE_POS=x,z,yaw_deg,pitch_deg   CAROMINE_SNOW=0..1
use crate::state::GameState;
use crate::world::{sky::TimeOfDay, weather::Weather};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

#[derive(Resource, Default)]
struct DevState { frames: u32, shot_taken_at: Option<u32> }

pub struct DevPlugin;
impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DevState>().add_systems(Startup, apply_env).add_systems(Update, (drive, dev_car, dev_lineup));
    }
}

fn apply_env(mut tod: ResMut<TimeOfDay>, mut w: ResMut<Weather>) {
    if let Ok(h) = std::env::var("CAROMINE_HOUR") { if let Ok(h) = h.parse() { tod.hour = h; tod.speed = 0.0; } }
    if let Ok(s) = std::env::var("CAROMINE_SNOW") { if let Ok(s) = s.parse() { w.intensity = s; } }
}

fn drive(mut commands: Commands, mut d: ResMut<DevState>, state: Res<State<GameState>>, mut next: ResMut<NextState<GameState>>, mut exit: MessageWriter<AppExit>) {
    if std::env::var("CAROMINE_AUTOSTART").is_ok() && *state.get() == GameState::MainMenu { next.set(GameState::Playing); }
    if *state.get() != GameState::Playing { return; }
    d.frames += 1;
    let Ok(path) = std::env::var("CAROMINE_SHOT") else { return };
    let at: u32 = std::env::var("CAROMINE_SHOT_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(150);
    if d.frames == at && d.shot_taken_at.is_none() {
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        d.shot_taken_at = Some(d.frames);
    }
    if let Some(f) = d.shot_taken_at { if d.frames > f + 20 { exit.write(AppExit::Success); } }
}

fn dev_car(
    mut commands: Commands,
    d: Res<DevState>,
    mut ws: ResMut<crate::vehicle::windshield::Windshield>,
    mut cars: Query<(Entity, &mut crate::vehicle::Car)>,
    cam: Query<Entity, With<crate::player::PlayerCamera>>,
    mut pv: Query<&mut Visibility, With<crate::player::Player>>,
) {
    let Ok(v) = std::env::var("CAROMINE_DRIVE") else { return };
    if d.frames != 20 { return; }
    let n: Vec<f32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let Ok((ce, mut car)) = cars.single_mut() else { return };
    car.running = true; car.lights = true; car.rpm = 900.0;
    if let Ok(w) = std::env::var("CAROMINE_WIPERS") { car.wipers = w.parse().unwrap_or(0); }
    if let Ok(s) = std::env::var("CAROMINE_WS") { ws.snow = s.parse().unwrap_or(0.0); }
    commands.entity(ce).insert(crate::vehicle::Driven);
    if let Ok(c) = cam.single() { commands.entity(c).insert(ChildOf(ce)).insert(Transform::from_xyz(-0.38, 1.12, -0.15)); }
    for mut v in &mut pv { *v = Visibility::Hidden; }
    commands.insert_resource(crate::vehicle::physics::DevDrive { throttle: n.first().copied().unwrap_or(0.0), steer: n.get(1).copied().unwrap_or(0.0), handbrake: n.get(2).copied().unwrap_or(0.0) > 0.5 });
}

/// CAROMINE_LINEUP=1: spawn one of every enemy in a row in front of the player (AI frozen by giving them no sight: they are Dead-free but idle until alerted).
fn dev_lineup(
    mut commands: Commands,
    d: Res<DevState>,
    a: Option<Res<crate::combat::enemy::EnemyAssets>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    player: Query<&Transform, With<crate::player::Player>>,
) {
    if std::env::var("CAROMINE_LINEUP").is_err() || d.frames != 5 { return; }
    let (Some(a), Ok(pt)) = (a, player.single()) else { return };
    use crate::combat::enemy::*;
    let fwd = pt.rotation * Vec3::NEG_Z; let right = pt.rotation * Vec3::X;
    for (i, k) in [Kind::Goon, Kind::Hunter, Kind::Neighbour, Kind::Wolf, Kind::Moose].iter().enumerate() {
        let p = pt.translation + fwd * 9.0 + right * (i as f32 - 2.0) * 3.2;
        spawn_enemy(&mut commands, &a, &mut mats, *k, Vec2::new(p.x, p.z));
    }
}
