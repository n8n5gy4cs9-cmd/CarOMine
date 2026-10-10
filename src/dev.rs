//! Dev/test harness (env vars). Lets CI/AI run the game headless (xvfb + lavapipe) and grab a screenshot.
//!   CAROMINE_AUTOSTART=1        skip menu
//!   CAROMINE_SHOT=out.png       screenshot after CAROMINE_SHOT_FRAMES (default 150) frames in Playing, then exit
//!   CAROMINE_DRIVE=throttle,steer[,hb]  auto-enter car, engine on, lights on, scripted inputs (e.g. 1,0.3)  CAROMINE_WS=0..1 windshield snow  CAROMINE_WIPERS=0..2
//!   CAROMINE_CMDS="cmd1;cmd2"  run console commands at frame 30 and leave the console open  (CAROMINE_CONSOLE_OPEN=1 to keep it open)
//!   CAROMINE_HOUR=13.5          start hour       CAROMINE_POS=x,z,yaw_deg,pitch_deg   CAROMINE_SNOW=0..1
use crate::state::GameState;
use crate::world::{sky::TimeOfDay, weather::Weather};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

#[derive(Resource, Default)]
struct DevState { menu_frames: u32, frames: u32, shot_taken_at: Option<u32> }

pub struct DevPlugin;
impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DevState>().add_systems(Startup, apply_env).add_systems(Update, (drive, dev_car, dev_lineup, dev_cmds, dev_workshop, dev_ignite, dev_dialog, dev_logcar));
    }
}

fn apply_env(mut tod: ResMut<TimeOfDay>, mut w: ResMut<Weather>, mut fl: ResMut<crate::combat::flashlight::Flashlight>, mut story: ResMut<crate::story::Story>) {
    if std::env::var("CAROMINE_AUTOSTART").is_ok() && std::env::var("CAROMINE_INTRO").is_err() { story.intro_shown = true; }
    if std::env::var("CAROMINE_FLASH").is_ok() { fl.on = true; }
    if let Ok(h) = std::env::var("CAROMINE_HOUR") { if let Ok(h) = h.parse() { tod.hour = h; tod.speed = 0.0; } }
    if let Ok(s) = std::env::var("CAROMINE_SNOW") { if let Ok(s) = s.parse() { w.intensity = s; } }
}

fn drive(mut commands: Commands, mut d: ResMut<DevState>, state: Res<State<GameState>>, mut next: ResMut<NextState<GameState>>, mut exit: MessageWriter<AppExit>) {
    if std::env::var("CAROMINE_AUTOSTART").is_ok() && *state.get() == GameState::MainMenu { next.set(GameState::Playing); }
    if matches!(*state.get(), GameState::Boot | GameState::MainMenu) {
        // CAROMINE_SHOT_MENU=1: screenshot the main menu itself
        if std::env::var("CAROMINE_SHOT_MENU").is_ok() && *state.get() == GameState::MainMenu {
            d.menu_frames += 1;
            if let Ok(path) = std::env::var("CAROMINE_SHOT") {
                if d.menu_frames == 60 { commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path)); }
                if d.menu_frames == 90 { exit.write(AppExit::Success); }
            }
        }
        return;
    }
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
    mut cars: Query<(Entity, &mut crate::vehicle::Car, &mut crate::build::CarParts)>,
    cam: Query<Entity, With<crate::player::PlayerCamera>>,
    mut pv: Query<&mut Visibility, With<crate::player::Player>>,
) {
    let Ok(v) = std::env::var("CAROMINE_DRIVE") else { return };
    if d.frames != 20 { return; }
    let n: Vec<f32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let Ok((ce, mut car, mut parts)) = cars.single_mut() else { return };
    if std::env::var("CAROMINE_RAW").is_err() {
        // default for dev driving: a fully built, running car. CAROMINE_RAW=1 keeps the real starting state.
        *parts = crate::build::CarParts::fully_built();
        car.running = true; car.lights = true; car.rpm = 900.0; car.fuel = 40.0;
    }
    if let Ok(w) = std::env::var("CAROMINE_WIPERS") { car.wipers = w.parse().unwrap_or(0); }
    if let Ok(s) = std::env::var("CAROMINE_WS") { ws.set_all(s.parse().unwrap_or(0.0)); }
    commands.entity(ce).insert(crate::vehicle::Driven);
    if let Ok(c) = cam.single() { commands.entity(c).insert(ChildOf(ce)).insert(Transform::from_xyz(-0.38, 1.12, -0.15)); }
    for mut v in &mut pv { *v = Visibility::Hidden; }
    if std::env::var("CAROMINE_CHASE").is_ok() { commands.insert_resource(crate::vehicle::ChaseCam(true)); }
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

fn dev_cmds(d: Res<DevState>, mut out: MessageWriter<crate::console::ConsoleLine>, mut con: ResMut<crate::console::Console>) {
    if d.frames != 30 { return; }
    let Ok(v) = std::env::var("CAROMINE_CMDS") else { return };
    for c in v.split(';') { out.write(crate::console::ConsoleLine(c.trim().to_string())); con.log.push(format!("> {}", c.trim())); }
    if std::env::var("CAROMINE_CONSOLE_OPEN").is_ok() { con.open = true; }
}

/// CAROMINE_WORKSHOP=1: open the workshop menu at frame 25 (car + player must be adjacent: use CAROMINE_POS near car).
pub fn dev_workshop(d: Res<DevState>, mut next: ResMut<NextState<GameState>>) {
    if std::env::var("CAROMINE_WORKSHOP").is_ok() && d.frames == 25 { next.set(GameState::Garage); }
}

/// CAROMINE_IGNITE=60,90,...  press the ignition at these frames (needs CAROMINE_DRIVE to sit in the car).
fn dev_ignite(d: Res<DevState>, mut w: MessageWriter<crate::build::Ignition>) {
    let Ok(v) = std::env::var("CAROMINE_IGNITE") else { return };
    if v.split(',').filter_map(|s| s.trim().parse::<u32>().ok()).any(|f| f == d.frames) { w.write(crate::build::Ignition); }
}

/// CAROMINE_DIALOG=home|phone|sauna|market|parts: open that dialog at frame 25.
fn dev_dialog(d: Res<DevState>, mut insp: MessageWriter<crate::story::RunInspection>, mut next: ResMut<NextState<GameState>>, mut dlg: ResMut<crate::life::dialog::Dialog>, mut missions: ResMut<crate::life::missions::Missions>) {
    use crate::life::dialog::Screen;
    let Ok(v) = std::env::var("CAROMINE_DIALOG") else { return };
    if d.frames == 40 && v == "inspect" { insp.write(crate::story::RunInspection); }
    if d.frames != 25 { return; }
    dlg.screen = match v.as_str() { "phone" => Screen::Phone, "sauna" => Screen::Sauna, "market" => Screen::Market, "parts" => Screen::Parts, _ => Screen::Home };
    dlg.sel = 0; missions.ensure_board();
    if v == "inspect" { dlg.screen = Screen::Inspect; }
    next.set(GameState::Dialog);
}

/// CAROMINE_LOGCAR=1: print car telemetry once per ~0.5 s of game time.
fn dev_logcar(time: Res<Time>, cars: Query<(&crate::vehicle::Car, &Transform)>, mut acc: Local<f32>) {
    if std::env::var("CAROMINE_LOGCAR").is_err() { return; }
    *acc += time.delta_secs().min(0.05);
    if *acc < 0.5 { return; }
    *acc = 0.0;
    if let Ok((c, t)) = cars.single() {
        info!("CAR v={:5.1}km/h lat={:4.1} gear={} rpm={:4.0} yawrate={:5.2} slip={:.2} run={} pos=({:.0},{:.0})", c.speed_kmh(), c.v_lat, c.gear, c.rpm, c.yaw_rate, c.slip, c.running, t.translation.x, t.translation.z);
    }
}
