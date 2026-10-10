//! Dev/test harness (env vars). Lets CI/AI run the game headless (xvfb + lavapipe) and grab a screenshot.
//!   CAROMINE_AUTOSTART=1        skip menu
//!   CAROMINE_SHOT=out.png       screenshot after CAROMINE_SHOT_FRAMES (default 150) frames in Playing, then exit
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
        app.init_resource::<DevState>().add_systems(Startup, apply_env).add_systems(Update, drive);
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
