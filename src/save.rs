//! Save / load (JSON in ./saves/). Slots: "autosave", "slot1".."slot3". Console: save/load.
use crate::build::{CarParts, Inventory, Slot, StockPart, ALL, N};
use crate::combat::flashlight::Flashlight;
use crate::combat::weapons::Loadout;
use crate::combat::PlayerStats;
use crate::player::Player;
use crate::state::GameState;
use crate::vehicle::spec::TyreType;
use crate::vehicle::{Car, Driven};
use crate::world::layout::height_at;
use crate::world::sky::TimeOfDay;
use crate::world::weather::Weather;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SaveData {
    pub version: u32,
    pub stamp: String,
    pub player: [f32; 3], pub yaw: f32, pub pitch: f32,
    pub hp: f32, pub armor: f32, pub money: i32,
    pub owned: [bool; 5], pub mag: [u32; 5], pub reserve: [u32; 5], pub current: usize,
    pub flash_battery: f32,
    pub hour: f32, pub snow: f32,
    pub car_pos: [f32; 3], pub car_yaw: f32, pub car_fuel: f32,
    pub parts_installed: Vec<bool>, pub parts_cond: Vec<f32>,
    pub tyre: u8, pub flat: bool, pub coolant: f32, pub battery: f32, pub temp: f32, pub frozen: bool,
    pub inv_parts: Vec<(usize, f32, u8)>, pub dry_gas: u32, pub coolant_cans: u32,
}

pub const SLOTS: [&str; 4] = ["autosave", "slot1", "slot2", "slot3"];
fn path(slot: &str) -> String { format!("saves/{slot}.json") }

pub fn write(slot: &str, d: &SaveData) -> Result<(), String> {
    std::fs::create_dir_all("saves").map_err(|e| e.to_string())?;
    std::fs::write(path(slot), serde_json::to_string_pretty(d).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
pub fn read(slot: &str) -> Option<SaveData> { serde_json::from_str(&std::fs::read_to_string(path(slot)).ok()?).ok() }
pub fn summary(slot: &str) -> String {
    match read(slot) { Some(d) => format!("{}  {:02}:{:02}  EUR {}  HP {:.0}", d.stamp, d.hour as u32, (d.hour.fract() * 60.0) as u32, d.money, d.hp), None => "(empty)".into() }
}

/// Set by menus/console; applied once the world exists.
#[derive(Resource, Default)]
pub struct PendingLoad(pub Option<SaveData>);
/// Request a save into this slot (processed in Playing/Paused/Garage where the world exists).
#[derive(Message)]
pub struct SaveRequest(pub String);

#[derive(Resource)]
pub struct AutoSave(pub f32);

pub struct SavePlugin;
impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingLoad>().insert_resource(AutoSave(0.0)).add_message::<SaveRequest>()
            .add_systems(Update, (apply_load, do_save, autosave).run_if(not(in_state(GameState::Boot))));
    }
}

fn stamp() -> String {
    let s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("t+{}h{:02}m", s / 3600 % 24, s / 60 % 60)
}

#[allow(clippy::too_many_arguments)]
fn do_save(
    mut req: MessageReader<SaveRequest>,
    player: Query<(&Transform, &Player)>,
    car: Query<(&Transform, &Car, &CarParts)>,
    stats: Res<PlayerStats>, lo: Res<Loadout>, fl: Res<Flashlight>, tod: Res<TimeOfDay>, weather: Res<Weather>, inv: Res<Inventory>,
    mut toasts: ResMut<crate::build::Toasts>,
) {
    for r in req.read() {
        let (Ok((pt, p)), Ok((ct, c, parts))) = (player.single(), car.single()) else { continue };
        let d = SaveData {
            version: 1, stamp: stamp(),
            player: pt.translation.to_array(), yaw: p.yaw, pitch: p.pitch,
            hp: stats.hp, armor: stats.armor, money: stats.money,
            owned: lo.owned, mag: lo.mag, reserve: lo.reserve, current: lo.current,
            flash_battery: fl.battery, hour: tod.hour, snow: weather.intensity,
            car_pos: ct.translation.to_array(), car_yaw: c.yaw, car_fuel: c.fuel,
            parts_installed: ALL.iter().map(|s| parts.get(*s).installed).collect(), parts_cond: ALL.iter().map(|s| parts.get(*s).cond).collect(),
            tyre: parts.tyre.to_u8(), flat: parts.flat, coolant: parts.coolant, battery: parts.battery, temp: parts.temp, frozen: parts.frozen_line,
            inv_parts: inv.parts.iter().map(|s| (s.slot.idx(), s.cond, s.tyre.to_u8())).collect(), dry_gas: inv.dry_gas, coolant_cans: inv.coolant_cans,
        };
        match write(&r.0, &d) { Ok(()) => { if r.0 != "autosave" { toasts.say(format!("Game saved ({}).", r.0)); } } Err(e) => toasts.say(format!("Save failed: {e}")) }
    }
}

fn autosave(time: Res<Time>, mut t: ResMut<AutoSave>, state: Res<State<GameState>>, mut req: MessageWriter<SaveRequest>) {
    if *state.get() != GameState::Playing { return; }
    t.0 += time.delta_secs();
    if t.0 > 180.0 { t.0 = 0.0; req.write(SaveRequest("autosave".into())); }
}

#[allow(clippy::too_many_arguments)]
fn apply_load(
    mut pending: ResMut<PendingLoad>,
    mut commands: Commands,
    mut player: Query<(Entity, &mut Transform, &mut Player), Without<Car>>,
    mut car: Query<(Entity, &mut Transform, &mut Car, &mut CarParts), Without<Player>>,
    driven: Query<Entity, With<Driven>>,
    mut stats: ResMut<PlayerStats>, mut lo: ResMut<Loadout>, mut fl: ResMut<Flashlight>, mut tod: ResMut<TimeOfDay>, mut weather: ResMut<Weather>, mut inv: ResMut<Inventory>,
    cam: Query<Entity, With<crate::player::PlayerCamera>>,
    mut vis: Query<&mut Visibility, With<Player>>,
    mut toasts: ResMut<crate::build::Toasts>,
    state: Res<State<GameState>>,
) {
    if pending.0.is_none() || *state.get() != GameState::Playing { return; }
    let (Ok((pe, mut pt, mut p)), Ok((ce, mut ct, mut c, mut parts))) = (player.single_mut(), car.single_mut()) else { return };
    let d = pending.0.take().unwrap();
    // get out of the car if driving
    if driven.single().is_ok() {
        commands.entity(ce).remove::<Driven>();
        if let Ok(cam) = cam.single() { commands.entity(cam).insert(ChildOf(pe)).insert(Transform::from_xyz(0.0, crate::player::EYE_STAND, 0.0)); }
        for mut v in &mut vis { *v = Visibility::Inherited; }
    }
    pt.translation = Vec3::from_array(d.player); p.yaw = d.yaw; p.pitch = d.pitch; p.vel = Vec3::ZERO;
    stats.hp = d.hp.max(1.0); stats.armor = d.armor; stats.money = d.money;
    lo.owned = d.owned; lo.mag = d.mag; lo.reserve = d.reserve; lo.current = d.current.min(4); lo.reload_t = 0.0;
    fl.battery = d.flash_battery; fl.on = false;
    tod.hour = d.hour; weather.intensity = d.snow;
    ct.translation = Vec3::from_array(d.car_pos); ct.translation.y = height_at(ct.translation.x, ct.translation.z);
    c.yaw = d.car_yaw; c.v_long = 0.0; c.v_lat = 0.0; c.yaw_rate = 0.0; c.fuel = d.car_fuel; c.running = false;
    for (i, s) in ALL.iter().enumerate() { parts.set(*s, d.parts_installed.get(i).copied().unwrap_or(false), d.parts_cond.get(i).copied().unwrap_or(0.0)); }
    parts.tyre = TyreType::from_u8(d.tyre); parts.flat = d.flat; parts.coolant = d.coolant; parts.battery = d.battery; parts.temp = d.temp; parts.frozen_line = d.frozen;
    inv.parts = d.inv_parts.iter().filter(|(i, _, _)| *i < N).map(|(i, c, t)| StockPart { slot: ALL[*i], cond: *c, tyre: TyreType::from_u8(*t) }).collect();
    inv.dry_gas = d.dry_gas; inv.coolant_cans = d.coolant_cans;
    let _ = Slot::Engine;
    toasts.say("Game loaded.");
}
