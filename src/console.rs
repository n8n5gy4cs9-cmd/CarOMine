//! Developer console (toggle with ` / F1) + cheat codes. Type `help`.
//! Opening the console pauses the game (GameState::Paused) and restores it on close.
use crate::combat::enemy::{spawn_enemy, EnemyAssets, Enemy, Kind};
use crate::combat::weapons::{Loadout, WEAPONS};
use crate::combat::PlayerStats;
use crate::player::{Player, PlayerCamera};
use crate::settings::Settings;
use crate::state::GameState;
use crate::vehicle::spec::TyreType;
use crate::vehicle::Car;
use crate::world::layout::*;
use crate::world::sky::TimeOfDay;
use crate::world::weather::Weather;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Console {
    pub open: bool,
    pub input: String,
    pub log: Vec<String>,
    pub history: Vec<String>,
    pub hist_idx: Option<usize>,
    resume: Option<GameState>,
}

/// Cheat flags read by gameplay systems.
#[derive(Resource)]
pub struct Cheats {
    pub god: bool,
    pub fly: bool,
    pub infinite_ammo: bool,
    pub infinite_fuel: bool,
    pub speed_mult: f32,
    pub show_pos: bool,
}
impl Default for Cheats {
    fn default() -> Self { Self { god: false, fly: false, infinite_ammo: false, infinite_fuel: false, speed_mult: 1.0, show_pos: false } }
}

#[derive(Component)] struct ConsoleRoot;
#[derive(Component)] struct ConsoleLog;
#[derive(Component)] struct ConsoleInput;
#[derive(Component)] struct PosHud;

pub struct ConsolePlugin;
impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Console>().init_resource::<Cheats>()
            .add_systems(Startup, spawn_ui)
            .add_message::<ConsoleLine>().add_systems(Update, (toggle, typing, exec_cmds, refresh_ui, apply_cheats, pos_hud).chain());
    }
}

fn spawn_ui(mut commands: Commands) {
    let font = |s: f32| (TextFont { font_size: FontSize::Px(s), ..default() }, TextColor(Color::srgb(0.85, 1.0, 0.85)));
    commands.spawn((ConsoleRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(46.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::FlexEnd, padding: UiRect::all(Val::Px(8.0)), ..default() },
        BackgroundColor(Color::srgba(0.0, 0.03, 0.0, 0.88)), GlobalZIndex(100), Visibility::Hidden)).with_children(|c| {
        c.spawn((ConsoleLog, Text::new(""), font(16.0)));
        c.spawn((ConsoleInput, Text::new("> "), font(18.0)));
    });
    commands.spawn((PosHud, Text::new(""), TextFont { font_size: FontSize::Px(14.0), ..default() }, TextColor(Color::srgb(1.0, 1.0, 0.5)), Node { position_type: PositionType::Absolute, left: Val::Px(8.0), top: Val::Px(8.0), ..default() }, GlobalZIndex(50)));
}

fn toggle(keys: Res<ButtonInput<KeyCode>>, mut con: ResMut<Console>, state: Res<State<GameState>>, mut next: ResMut<NextState<GameState>>) {
    if !(keys.just_pressed(KeyCode::Backquote) || keys.just_pressed(KeyCode::F1)) { return; }
    if con.open {
        con.open = false;
        if let Some(s) = con.resume.take() { next.set(s); }
    } else {
        con.open = true;
        if con.log.is_empty() { con.log.push("CarOMine console. Type 'help'. Toggle with ` or F1.".into()); }
        if *state.get() == GameState::Playing { con.resume = Some(GameState::Playing); next.set(GameState::Paused); }
    }
}

#[derive(Message)]
pub struct ConsoleLine(pub String);

fn typing(mut ev: MessageReader<KeyboardInput>, mut con: ResMut<Console>, mut out: MessageWriter<ConsoleLine>) {
    if !con.open { ev.clear(); return; }
    let mut run: Option<String> = None;
    for e in ev.read() {
        if e.state != ButtonState::Pressed { continue; }
        match &e.logical_key {
            Key::Enter => { run = Some(std::mem::take(&mut con.input)); }
            Key::Backspace => { con.input.pop(); }
            Key::ArrowUp => {
                if !con.history.is_empty() {
                    let i = con.hist_idx.map(|i| i.saturating_sub(1)).unwrap_or(con.history.len() - 1);
                    con.hist_idx = Some(i); con.input = con.history[i].clone();
                }
            }
            Key::ArrowDown => {
                if let Some(i) = con.hist_idx {
                    if i + 1 < con.history.len() { con.hist_idx = Some(i + 1); con.input = con.history[i + 1].clone(); } else { con.hist_idx = None; con.input.clear(); }
                }
            }
            _ => {
                if matches!(e.key_code, KeyCode::Backquote | KeyCode::F1) { continue; }
                if let Some(t) = &e.text { for ch in t.chars() { if !ch.is_control() { con.input.push(ch); } } }
            }
        }
    }
    if let Some(line) = run {
        let line = line.trim().to_string();
        if line.is_empty() { return; }
        con.history.push(line.clone()); con.hist_idx = None;
        con.log.push(format!("> {line}"));
        out.write(ConsoleLine(line));
    }
}

fn refresh_ui(con: Res<Console>, mut root: Query<&mut Visibility, With<ConsoleRoot>>, mut log: Query<&mut Text, (With<ConsoleLog>, Without<ConsoleInput>)>, mut inp: Query<&mut Text, (With<ConsoleInput>, Without<ConsoleLog>)>, time: Res<Time>) {
    if let Ok(mut v) = root.single_mut() { *v = if con.open { Visibility::Inherited } else { Visibility::Hidden }; }
    if !con.open { return; }
    if let Ok(mut t) = log.single_mut() { let n = con.log.len(); t.0 = con.log[n.saturating_sub(14)..].join("\n"); }
    if let Ok(mut t) = inp.single_mut() { t.0 = format!("> {}{}", con.input, if (time.elapsed_secs() * 2.0) as u32 % 2 == 0 { "_" } else { " " }); }
}

fn apply_cheats(cheats: Res<Cheats>, mut stats: ResMut<PlayerStats>, mut lo: ResMut<Loadout>, mut cars: Query<&mut Car>) {
    stats.god = cheats.god;
    if cheats.god { stats.hp = stats.hp.max(1.0); }
    if cheats.infinite_ammo { for i in 1..5 { lo.mag[i] = WEAPONS[i].mag; } }
    if cheats.infinite_fuel { for mut c in &mut cars { c.fuel = c.spec.fuel_cap; } }
}

fn pos_hud(cheats: Res<Cheats>, player: Query<(&Transform, &Player)>, mut q: Query<&mut Text, With<PosHud>>) {
    let Ok(mut t) = q.single_mut() else { return };
    t.0 = if cheats.show_pos { player.single().map(|(tr, p)| format!("x {:.1}  y {:.1}  z {:.1}  yaw {:.0}", tr.translation.x, tr.translation.y, tr.translation.z, p.yaw.to_degrees())).unwrap_or_default() } else { String::new() };
}


#[derive(bevy::ecs::system::SystemParam)]
struct Res0<'w> {
    cheats: ResMut<'w, Cheats>,
    stats: ResMut<'w, PlayerStats>,
    lo: ResMut<'w, Loadout>,
    tod: ResMut<'w, TimeOfDay>,
    weather: ResMut<'w, Weather>,
    settings: ResMut<'w, Settings>,
    ws: ResMut<'w, crate::vehicle::windshield::Windshield>,
    flash: ResMut<'w, crate::combat::flashlight::Flashlight>,
    missions: ResMut<'w, crate::life::missions::Missions>,
    needs: ResMut<'w, crate::life::needs::Needs>,
    save_req: MessageWriter<'w, crate::save::SaveRequest>,
    pending: ResMut<'w, crate::save::PendingLoad>,
    inv: ResMut<'w, crate::build::Inventory>,
}

const HELP: &str = "commands: help clear pos | tp <x z|cabin|village|garage|lake|shop> | time <hour> | timespeed <x> | snow <0..1> | ws <0..1> windshield snow
 gig [n|tp|done|abandon] need food|energy|warm <0..1> | save/load [slot] | god fly speed <x> flashlight batteries heal armor money <n> | give <pistol|shotgun|smg|rifle|all> | ammo (inf) fuel (inf) | spawn <goon|hunter|neighbour|wolf|moose> [n] killall
 build all|shell parts repair fault <name> | car bring|goto|tyres <summer|winter|studded>|lights|start | fov <deg> sens <x> | quit | cheat codes: iddqd idkfa idclip dnstuff";

fn exec_cmds(
    mut lines: MessageReader<ConsoleLine>,
    mut con: ResMut<Console>,
    mut commands: Commands,
    r: Res0,
    ea: Option<Res<EnemyAssets>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut player: Query<(&mut Transform, &mut Player), Without<Car>>,
    mut cars: Query<(&mut Transform, &mut Car), Without<Player>>,
    mut bparts: Query<&mut crate::build::CarParts>,
    enemies: Query<Entity, With<Enemy>>,
    mut cam: Query<&mut Projection, With<PlayerCamera>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Res0 { mut cheats, mut stats, mut lo, mut tod, mut weather, mut settings, mut ws, mut flash, mut missions, mut needs, mut save_req, mut pending, mut inv } = r;
    for ConsoleLine(line) in lines.read() {
        let w: Vec<&str> = line.split_whitespace().collect();
        let a = |i: usize| w.get(i).copied().unwrap_or("");
        let num = |i: usize| w.get(i).and_then(|v| v.parse::<f32>().ok());
        let mut out: Vec<String> = vec![];
        let tog = |b: &mut bool| { *b = !*b; if *b { "ON" } else { "OFF" } };
        match a(0).to_lowercase().as_str() {
            "help" | "?" => out.extend(HELP.lines().map(String::from)),
            "clear" => con.log.clear(),
            "pos" => { cheats.show_pos = !cheats.show_pos; out.push(format!("position HUD {}", if cheats.show_pos { "ON" } else { "OFF" })); }
            "god" | "iddqd" => out.push(format!("god mode {}", tog(&mut cheats.god))),
            "fly" | "noclip" | "idclip" => out.push(format!("fly/noclip {} (look direction + WASD, Space up, Ctrl down)", tog(&mut cheats.fly))),
            "speed" => { cheats.speed_mult = num(1).unwrap_or(1.0).clamp(0.1, 20.0); out.push(format!("walk speed x{}", cheats.speed_mult)); }
            "batteries" | "flashlight" => { flash.battery = 1.0; if a(0) == "flashlight" { flash.on = !flash.on; } out.push(format!("flashlight battery 100%, light {}", if flash.on { "ON" } else { "off" })); }
            "save" => { let slot = if a(1).is_empty() { "slot1".to_string() } else if a(1).len() == 1 { format!("slot{}", a(1)) } else { a(1).to_string() }; save_req.write(crate::save::SaveRequest(slot.clone())); out.push(format!("saving to {slot}")); }
            "load" => { let slot = if a(1).is_empty() { "slot1".to_string() } else if a(1).len() == 1 { format!("slot{}", a(1)) } else { a(1).to_string() }; match crate::save::read(&slot) { Some(d) => { pending.0 = Some(d); out.push(format!("loading {slot}")); } None => out.push(format!("no save in {slot}")) } }
            "gig" => {
                missions.ensure_board();
                match a(1) {
                    "" => { for (i, m) in missions.board.iter().enumerate() { out.push(format!("{i}: {} [{} EUR] at {:.0},{:.0}", m.title, m.reward, m.center.x, m.center.y)); } if let Some(m) = &missions.active { out.push(format!("ACTIVE: {} at {:.0},{:.0}", m.title, m.center.x, m.center.y)); } }
                    "tp" => { if let (Some(m), Ok((mut t, _))) = (missions.active.clone(), player.single_mut()) { let d = m.target() + Vec2::new(0.0, 25.0); t.translation = Vec3::new(d.x, height_at(d.x, d.y) + 0.5, d.y); out.push("teleported near objective".into()); } else { out.push("no active gig".into()); } }
                    "done" => { if let Some(m) = missions.active.take() { stats.money += m.reward; out.push(format!("gig completed (cheat): +{} EUR", m.reward)); } }
                    "abandon" => { missions.active = None; out.push("abandoned".into()); }
                    n => { if let Ok(i) = n.parse::<usize>() { if i < missions.board.len() && missions.active.is_none() { let m = missions.board.remove(i); out.push(format!("accepted: {}", m.title)); missions.active = Some(m); } else { out.push("bad index or gig already active".into()); } } }
                }
            }
            "need" | "needs" => { let v = num(2).unwrap_or(1.0).clamp(0.0, 1.0); match a(1) { "food" | "hunger" => needs.hunger = v, "energy" => needs.energy = v, "warm" | "warmth" => needs.warmth = v, _ => { out.push(format!("hunger {:.0}% energy {:.0}% warmth {:.0}%   usage: need food|energy|warm <0..1>", needs.hunger * 100.0, needs.energy * 100.0, needs.warmth * 100.0)); } } }
            "heal" => { stats.hp = 100.0; out.push("healed".into()); }
            "armor" => { stats.armor = 100.0; out.push("armor 100".into()); }
            "money" => { stats.money += num(1).unwrap_or(1000.0) as i32; out.push(format!("money {}", stats.money)); }
            "ammo" => out.push(format!("infinite ammo {}", tog(&mut cheats.infinite_ammo))),
            "fuel" => { out.push(format!("infinite fuel {}", tog(&mut cheats.infinite_fuel))); for (_, mut c) in &mut cars { c.fuel = c.spec.fuel_cap; } }
            "give" | "idkfa" | "dnstuff" => {
                let all = matches!(a(0), "idkfa" | "dnstuff") || a(1) == "all";
                for (i, wd) in WEAPONS.iter().enumerate() {
                    if all || wd.id == a(1) || wd.name.to_lowercase().starts_with(&a(1).to_lowercase()) && !a(1).is_empty() {
                        lo.owned[i] = true; if !wd.melee { lo.mag[i] = wd.mag; lo.reserve[i] += wd.mag * 6; }
                        out.push(format!("given {}", wd.name));
                    }
                }
                if a(0) == "dnstuff" { stats.hp = 100.0; stats.armor = 100.0; stats.money += 5000; out.push("DNSTUFF: health, armor, money, all weapons".into()); }
                if out.is_empty() { out.push("usage: give <pistol|shotgun|smg|rifle|all>".into()); }
            }
            "time" => { if let Some(h) = num(1) { tod.hour = h.rem_euclid(24.0); out.push(format!("time {:.1}", tod.hour)); } else { out.push(format!("time is {:.2} (usage: time <0-24>)", tod.hour)); } }
            "timespeed" => { tod.speed = num(1).unwrap_or(1.0) / 60.0; out.push(format!("1 game hour = {:.1} real seconds", 1.0 / tod.speed.max(1e-6))); }
            "snow" | "weather" => { weather.intensity = num(1).unwrap_or(0.8).clamp(0.0, 1.0); out.push(format!("snowfall {:.2}", weather.intensity)); }
            "ws" => { ws.snow = num(1).unwrap_or(0.0).clamp(0.0, 1.0); out.push(format!("windshield snow {:.2}", ws.snow)); }
            "fov" => { if let (Some(f), Ok(mut p)) = (num(1), cam.single_mut()) { if let Projection::Perspective(pp) = &mut *p { pp.fov = f.clamp(30.0, 140.0).to_radians(); settings.fov_degrees = f; out.push(format!("fov {f}")); } } }
            "sens" => { settings.mouse_sensitivity = num(1).unwrap_or(0.0022) * 0.001; out.push("sensitivity set (value x0.001)".into()); }
            "tp" | "teleport" => {
                let dest = match a(1) {
                    "cabin" | "home" => Some(CABIN + Vec2::new(-8.0, -9.0)), "village" => Some(VILLAGE + Vec2::new(0.0, 10.0)),
                    "garage" => Some(GARAGE + Vec2::new(8.0, 0.0)), "lake" => Some(LAKE_C + Vec2::new(-LAKE_R - 12.0, 0.0)), "shop" => Some(VILLAGE + Vec2::new(10.0, 4.0)),
                    _ => match (num(1), num(2)) { (Some(x), Some(z)) => Some(Vec2::new(x, z)), _ => None },
                };
                if let (Some(d), Ok((mut t, _))) = (dest, player.single_mut()) { t.translation = Vec3::new(d.x, height_at(d.x, d.y) + 0.5, d.y); out.push(format!("teleported to {:.0},{:.0}", d.x, d.y)); } else { out.push("usage: tp <x z|cabin|village|garage|lake|shop>".into()); }
            }
            "spawn" => {
                let kind = match a(1) { "goon" => Some(Kind::Goon), "hunter" => Some(Kind::Hunter), "neighbour" | "neighbor" => Some(Kind::Neighbour), "wolf" => Some(Kind::Wolf), "moose" => Some(Kind::Moose), _ => None };
                match (kind, &ea, player.single()) {
                    (Some(k), Some(ea), Ok((t, _))) => {
                        let n = num(2).unwrap_or(1.0) as i32;
                        let f = t.rotation * Vec3::NEG_Z;
                        for i in 0..n.clamp(1, 50) { let p = t.translation + f * 10.0 + Vec3::new((i % 5) as f32 * 3.0 - 6.0, 0.0, (i / 5) as f32 * 3.0); spawn_enemy(&mut commands, ea, &mut mats, k, Vec2::new(p.x, p.z)); }
                        out.push(format!("spawned {n} {}", a(1)));
                    }
                    _ => out.push("usage: spawn <goon|hunter|neighbour|wolf|moose> [n]".into()),
                }
            }
            "killall" => { let n = enemies.iter().count(); for e in &enemies { commands.entity(e).despawn(); } out.push(format!("removed {n} enemies")); }
            "car" => match a(1) {
                "bring" => { if let (Ok((pt, _)), Ok((mut ct, mut c))) = (player.single(), cars.single_mut()) { let p = pt.translation + pt.rotation * Vec3::NEG_Z * 6.0; ct.translation = Vec3::new(p.x, height_at(p.x, p.z), p.z); c.v_long = 0.0; c.v_lat = 0.0; c.yaw_rate = 0.0; out.push("car brought".into()); } }
                "goto" => { if let (Ok((mut pt, _)), Ok((ct, _))) = (player.single_mut(), cars.single()) { pt.translation = ct.translation + Vec3::new(3.0, 1.0, 0.0); out.push("teleported to car".into()); } }
                "tyres" | "tires" => { if let Ok((_, mut c)) = cars.single_mut() { c.tyres = match a(2) { "winter" => TyreType::Winter, "studded" => TyreType::Studded, _ => TyreType::Summer }; out.push(format!("tyres: {}", c.tyres.name())); } }
                "lights" => { if let Ok((_, mut c)) = cars.single_mut() { c.lights = !c.lights; } }
                "start" => { if let Ok((_, mut c)) = cars.single_mut() { c.running = true; c.fuel = c.fuel.max(5.0); out.push("engine started".into()); } }
                _ => out.push("usage: car bring|goto|tyres <summer|winter|studded>|lights|start".into()),
            },
            "build" => { if let Ok(mut p) = bparts.single_mut() { match a(1) {
                "all" => { *p = crate::build::CarParts::fully_built(); out.push("car fully built (all parts 100%, coolant+battery full)".into()); }
                "shell" => { *p = crate::build::CarParts::starter_shell(); *inv = crate::build::Inventory::default(); out.push("car reset to the shell + boxes (new game state)".into()); }
                _ => out.push("usage: build all|shell".into()) } } }
            "repair" => { if let Ok(mut p) = bparts.single_mut() { for s in crate::build::ALL { let st = p.get(s); if st.installed { p.set(s, true, 1.0); } } p.flat = false; p.belt_snapped = false; p.frozen_line = false; p.coolant = 1.0; p.battery = 1.0; out.push("all installed parts repaired, fluids full".into()); } }
            "fault" => { if let Ok(mut p) = bparts.single_mut() {
                use crate::build::Slot;
                match a(1) {
                    "freeze" => p.frozen_line = true, "flat" => p.flat = true, "plugs" => p.set(Slot::SparkPlugs, true, 0.05), "filter" => p.set(Slot::FuelFilter, true, 0.03),
                    "battery" => p.battery = 0.0, "coolant" => p.coolant = 0.0, "belt" => { p.set(Slot::Belt, true, 0.0); p.belt_snapped = true; }
                    "fuse" => p.set(Slot::Fuse, true, 0.0), "alternator" => p.set(Slot::Alternator, true, 0.0), "radiator" => p.set(Slot::Radiator, true, 0.2),
                    "brakes" => p.set(Slot::Brakes, true, 0.05), "engine" => p.set(Slot::Engine, true, 0.04), "overheat" => p.temp = 112.0,
                    _ => { out.push("usage: fault freeze|flat|plugs|filter|battery|coolant|belt|fuse|alternator|radiator|brakes|engine|overheat".into()); }
                } if !a(1).is_empty() { out.push(format!("fault injected: {}", a(1))); } } }
            "parts" => { if let Ok(p) = bparts.single() { for s in crate::build::ALL { let st = p.get(s); out.push(format!("{:<16} {}", s.info().name, if st.installed { format!("{:.0}%", st.cond * 100.0) } else { "missing".into() })); } out.push(format!("coolant {:.0}% battery {:.0}% temp {:.0}C frozen {} flat {}", p.coolant * 100.0, p.battery * 100.0, p.temp, p.frozen_line, p.flat)); } }
            "quit" | "exit" => { exit.write(AppExit::Success); }
            "" => {}
            other => out.push(format!("unknown command '{other}' (try 'help')")),
        }
        for l in out { con.log.push(l); }
        while con.log.len() > 200 { con.log.remove(0); }
    }
    let _ = &mut player;
}
