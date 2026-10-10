//! Audio: one-shot SFX via `Sfx` messages, looping engine/tyre/wind, music state machine.
//! All sounds resolve through assets/manifest.json ids (sfx.*, music.*) - replace files freely.
use crate::manifest::AssetManifest;
use crate::settings::Settings;
use crate::state::GameState;
use crate::vehicle::{Car, Driven};
use crate::world::sky::{sky_state, TimeOfDay};
use crate::world::weather::Weather;
use bevy::audio::{AudioSink, AudioSinkPlayback, PlaybackSettings, SpatialListener, Volume};
use bevy::prelude::*;
use std::collections::HashMap;

/// Play a one-shot sound by manifest id (e.g. "sfx.pistol"). `pos` makes it 3D.
#[derive(Message, Clone)]
pub struct Sfx { pub id: &'static str, pub pos: Option<Vec3>, pub vol: f32 }
impl Sfx {
    pub fn new(id: &'static str) -> Self { Self { id, pos: None, vol: 1.0 } }
    pub fn at(id: &'static str, pos: Vec3) -> Self { Self { id, pos: Some(pos), vol: 1.0 } }
    pub fn vol(mut self, v: f32) -> Self { self.vol = v; self }
}

#[derive(Resource, Default)]
pub struct Sounds(pub HashMap<String, Handle<AudioSource>>);

#[derive(Component)] struct EngineLoop;
#[derive(Component)] struct TyreLoop;
#[derive(Component)] struct WindLoop;
#[derive(Component)] struct Music(&'static str);

pub struct GameAudioPlugin;
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Sounds>().add_message::<Sfx>()
            .add_systems(Startup, load_sounds)
            .add_systems(Update, (attach_listener, play_sfx, music, loops));
    }
}

fn load_sounds(server: Res<AssetServer>, man: Res<AssetManifest>, mut sounds: ResMut<Sounds>) {
    for (id, e) in man.0.iter() {
        if e.kind == "sfx" || e.kind == "music" { sounds.0.insert(id.clone(), server.load(e.path.clone())); }
    }
}

fn attach_listener(mut commands: Commands, cam: Query<Entity, (With<crate::player::PlayerCamera>, Without<SpatialListener>)>) {
    for e in &cam { commands.entity(e).insert(SpatialListener::new(0.3)); }
}

fn play_sfx(mut commands: Commands, mut ev: MessageReader<Sfx>, sounds: Res<Sounds>, settings: Res<Settings>) {
    for s in ev.read() {
        let Some(h) = sounds.0.get(s.id) else { warn!("no sound '{}'", s.id); continue };
        let mut ps = PlaybackSettings::DESPAWN;
        ps.volume = Volume::Linear(s.vol * settings.sfx_volume * settings.master_volume);
        if let Some(p) = s.pos {
            ps.spatial = true;
            commands.spawn((AudioPlayer::new(h.clone()), ps, Transform::from_translation(p)));
        } else {
            commands.spawn((AudioPlayer::new(h.clone()), ps));
        }
    }
}

fn want_music(state: GameState, night: bool, in_car: bool, combat: bool) -> &'static str {
    match state {
        GameState::MainMenu | GameState::Boot => "music.menu",
        _ if combat => "music.combat",
        _ if in_car => "music.drive",
        _ if night => "music.night",
        _ => "music.drive",
    }
}

fn music(
    mut commands: Commands,
    state: Res<State<GameState>>,
    tod: Res<TimeOfDay>,
    sounds: Res<Sounds>,
    settings: Res<Settings>,
    driven: Query<(), With<Driven>>,
    enemies: Query<(&crate::combat::enemy::Enemy, &Transform)>,
    player: Query<&Transform, With<crate::player::Player>>,
    cur: Query<(Entity, &Music)>,
    mut sinks: Query<&mut AudioSink, With<Music>>,
    mut hold: Local<f32>,
    time: Res<Time>,
) {
    let night = sky_state(tod.hour).day < 0.35;
    let combat_now = player.single().map(|p| enemies.iter().any(|(e, t)| e.state == crate::combat::enemy::State::Attack && t.translation.distance(p.translation) < 60.0)).unwrap_or(false);
    // combat music lingers 6s after the last fight
    if combat_now { *hold = 6.0; } else { *hold = (*hold - time.delta_secs()).max(0.0); }
    let want = want_music(*state.get(), night, !driven.is_empty(), *hold > 0.0);
    let vol = Volume::Linear(settings.music_volume * settings.master_volume);
    let mut have = false;
    for (e, m) in &cur {
        if m.0 == want { have = true; } else { commands.entity(e).despawn(); }
    }
    for mut s in &mut sinks { s.set_volume(vol); }
    if !have {
        if let Some(h) = sounds.0.get(want) {
            let mut ps = PlaybackSettings::LOOP; ps.volume = vol;
            commands.spawn((Music(want), AudioPlayer::new(h.clone()), ps));
        }
    }
}

/// Engine / tyre / wind loops.
fn loops(
    mut commands: Commands,
    sounds: Res<Sounds>,
    settings: Res<Settings>,
    weather: Res<Weather>,
    state: Res<State<GameState>>,
    cars: Query<(&Car, Option<&Driven>)>,
    mut eng: Query<&mut AudioSink, (With<EngineLoop>, Without<TyreLoop>, Without<WindLoop>)>,
    mut tyr: Query<&mut AudioSink, (With<TyreLoop>, Without<EngineLoop>, Without<WindLoop>)>,
    mut wind: Query<&mut AudioSink, (With<WindLoop>, Without<TyreLoop>, Without<EngineLoop>)>,
    existing: Query<(), Or<(With<EngineLoop>, With<TyreLoop>, With<WindLoop>)>>,
) {
    let playing = matches!(*state.get(), GameState::Playing);
    if existing.is_empty() && playing {
        let mut spawn = |id: &str, marker: usize, commands: &mut Commands| {
            if let Some(h) = sounds.0.get(id) {
                let mut ps = PlaybackSettings::LOOP; ps.volume = Volume::Linear(0.0);
                let mut e = commands.spawn((AudioPlayer::new(h.clone()), ps));
                match marker { 0 => { e.insert(EngineLoop); } 1 => { e.insert(TyreLoop); } _ => { e.insert(WindLoop); } }
            }
        };
        spawn("sfx.engine_idle", 0, &mut commands); spawn("sfx.tire_snow", 1, &mut commands); spawn("sfx.wind", 2, &mut commands);
        return;
    }
    let master = settings.sfx_volume * settings.master_volume;
    let car = cars.iter().find(|(_, d)| d.is_some()).or_else(|| cars.iter().next());
    if let Ok(mut s) = eng.single_mut() {
        match car {
            Some((c, d)) if c.running && playing => {
                s.set_speed(0.75 + (c.rpm / c.spec.redline) * 2.4);
                let inside = if d.is_some() { 0.55 } else { 0.35 };
                s.set_volume(Volume::Linear((0.35 + 0.65 * c.throttle) * inside * master));
            }
            _ => s.set_volume(Volume::Linear(0.0)),
        }
    }
    if let Ok(mut s) = tyr.single_mut() {
        match car {
            Some((c, d)) if d.is_some() && playing => {
                let sp = (c.v_long.abs() / 25.0).min(1.0);
                s.set_speed(0.7 + sp * 0.8);
                s.set_volume(Volume::Linear((sp * 0.5 + c.slip * 0.4) * master));
            }
            _ => s.set_volume(Volume::Linear(0.0)),
        }
    }
    if let Ok(mut s) = wind.single_mut() { s.set_volume(Volume::Linear(if playing { (0.12 + 0.3 * weather.intensity) * master } else { 0.0 })); }
}
