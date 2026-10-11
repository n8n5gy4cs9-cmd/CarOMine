//! Workshop menu (press T next to your car). Pages: Parts, Fluids & fixes, Manual. Paused while open (GameState::Garage).
use super::*;
use crate::combat::PlayerStats;
use crate::player::Player;
use crate::vehicle::{Car, Driven};
use crate::world::layout::GARAGE;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page { Parts, Fluids, Manual }

#[derive(Resource)]
pub struct Workshop { pub page: Page, pub sel: usize, pub msg: String, pub buy_tyre: TyreType }
impl Default for Workshop { fn default() -> Self { Self { page: Page::Parts, sel: 0, msg: String::new(), buy_tyre: TyreType::Winter } } }

#[derive(Component)] pub struct WsRootUi;
#[derive(Component)] pub struct WsLeft;
#[derive(Component)] pub struct WsRight;
#[derive(Component)] pub struct ToastUi;

const TYRE_PRICE: [(TyreType, i32); 3] = [(TyreType::Summer, 80), (TyreType::Winter, 180), (TyreType::Studded, 260)];

pub fn spawn_ui(mut commands: Commands) {
    commands.init_resource::<Workshop>();
    let f = |s: f32| (TextFont { font_size: FontSize::Px(s), ..default() }, TextColor(Color::srgb(0.95, 0.95, 0.9)));
    commands.spawn((WsRootUi, Node { position_type: PositionType::Absolute, left: Val::Percent(4.0), top: Val::Percent(5.0), width: Val::Percent(92.0), height: Val::Percent(88.0), padding: UiRect::all(Val::Px(14.0)), column_gap: Val::Px(30.0), ..default() },
        BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.95)), GlobalZIndex(60), Visibility::Hidden)).with_children(|c| {
        c.spawn((WsLeft, Text::new(""), f(17.0), Node { width: Val::Percent(48.0), ..default() }));
        c.spawn((WsRight, Text::new(""), f(17.0), Node { width: Val::Percent(48.0), ..default() }));
    });
    commands.spawn((ToastUi, Text::new(""), f(22.0), Node { position_type: PositionType::Absolute, bottom: Val::Px(110.0), left: Val::Percent(10.0), width: Val::Percent(80.0), ..default() }, GlobalZIndex(30)));
}

pub fn refresh_toast_ui(t: Res<Toasts>, mut q: Query<&mut Text, With<ToastUi>>) {
    if let Ok(mut x) = q.single_mut() { x.0 = t.msgs.iter().map(|m| m.0.as_str()).collect::<Vec<_>>().join("\n"); }
}

fn near_garage(p: Vec3) -> bool { Vec2::new(p.x, p.z).distance(GARAGE) < 32.0 }

pub fn open_close(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    player: Query<&Transform, (With<Player>, Without<Car>)>,
    cars: Query<(&Transform, Option<&Driven>), With<Car>>,
    mut toasts: ResMut<Toasts>,
    mut ws: ResMut<Workshop>,
    mut cursor: Query<&mut bevy::window::CursorOptions, With<bevy::window::PrimaryWindow>>,
) {
    let esc = keys.just_pressed(KeyCode::Escape);
    match state.get() {
        GameState::Playing if keys.just_pressed(KeyCode::KeyT) => {
            let (Ok(pt), Ok((ct, driven))) = (player.single(), cars.single()) else { return };
            if driven.is_some() { toasts.say("Get out of the car first (E)."); return; }
            if ct.translation.distance(pt.translation) > 7.0 { toasts.say("Stand next to your car to open the workshop (T)."); return; }
            ws.msg.clear();
            next.set(GameState::Garage);
            if let Ok(mut c) = cursor.single_mut() { c.grab_mode = bevy::window::CursorGrabMode::None; c.visible = true; }
        }
        GameState::Garage if keys.just_pressed(KeyCode::KeyT) || esc => {
            next.set(GameState::Playing);
        }
        _ => {}
    }
}

fn bar(c: f32) -> String { let n = (c.clamp(0.0, 1.0) * 10.0).round() as usize; format!("[{}{}] {:3.0}%", "#".repeat(n), ".".repeat(10 - n), c * 100.0) }

pub fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut ws: ResMut<Workshop>,
    mut inv: ResMut<Inventory>,
    mut stats: ResMut<PlayerStats>,
    player: Query<&Transform, (With<Player>, Without<Car>)>,
    mut cars: Query<&mut Car>,
    mut parts: Query<&mut CarParts>,
    mut sfx: MessageWriter<crate::audio::Sfx>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyF) { sfx.write(crate::audio::Sfx::new("sfx.ratchet")); }
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::Tab) { sfx.write(crate::audio::Sfx::new("sfx.ui_click").vol(0.5)); }
    let (Ok(mut p), Ok(mut car), Ok(pt)) = (parts.single_mut(), cars.single_mut(), player.single()) else { return };
    if keys.just_pressed(KeyCode::Tab) { ws.page = match ws.page { Page::Parts => Page::Fluids, Page::Fluids => Page::Manual, Page::Manual => Page::Parts }; ws.sel = 0; ws.msg.clear(); }
    let rows = if ws.page == Page::Parts { N } else { 1 };
    if keys.just_pressed(KeyCode::ArrowDown) { ws.sel = (ws.sel + 1) % rows; }
    if keys.just_pressed(KeyCode::ArrowUp) { ws.sel = (ws.sel + rows - 1) % rows; }
    if keys.just_pressed(KeyCode::Digit1) && ws.page == Page::Parts { ws.buy_tyre = TyreType::Summer; }
    if keys.just_pressed(KeyCode::Digit2) && ws.page == Page::Parts { ws.buy_tyre = TyreType::Winter; }
    if keys.just_pressed(KeyCode::Digit3) && ws.page == Page::Parts { ws.buy_tyre = TyreType::Studded; }
    match ws.page {
        Page::Parts => {
            let slot = ALL[ws.sel];
            let info = slot.info();
            let st = p.get(slot);
            if keys.just_pressed(KeyCode::Enter) {
                if info.heavy && !near_garage(pt.translation) { ws.msg = "Heavy job! Drive the car to the garage shed west of the cabin first.".into(); return; }
                if st.installed {
                    // remove -> inventory (the car can't be running)
                    car.running = false;
                    inv.parts.push(StockPart { slot, cond: st.cond, tyre: p.tyre });
                    p.set(slot, false, 0.0);
                    if slot == Slot::Tyres { p.flat = false; }
                    ws.msg = format!("Removed {}.", info.name);
                } else {
                    // install best matching stock part (for tyres: the chosen type if present)
                    let pick = inv.parts.iter().enumerate().filter(|(_, s)| s.slot == slot).max_by(|a, b| a.1.cond.partial_cmp(&b.1.cond).unwrap()).map(|(i, _)| i);
                    match pick {
                        Some(i) => { let s = inv.parts.remove(i); p.set(slot, true, s.cond); if slot == Slot::Tyres { p.tyre = s.tyre; p.flat = false; } if slot == Slot::Belt { p.belt_snapped = false; } ws.msg = format!("Installed {} ({:.0}% condition).", info.name, s.cond * 100.0); }
                        None => ws.msg = format!("No {} in your boxes. Buy one (B).", info.name),
                    }
                }
            }
            if keys.just_pressed(KeyCode::KeyF) {
                if info.fix_cap <= 0.0 { ws.msg = format!("{} can't be fixed - replace it.", info.name); }
                else if !st.installed { ws.msg = "Install it first.".into(); }
                else if st.cond >= info.fix_cap { ws.msg = "Already as good as a quick fix gets it.".into(); }
                else { p.set(slot, true, info.fix_cap); if slot == Slot::Tyres { p.flat = false; } ws.msg = format!("{} -> condition {:.0}%.", info.fix_label, info.fix_cap * 100.0); }
            }
            if keys.just_pressed(KeyCode::KeyB) {
                let price = if slot == Slot::Tyres { TYRE_PRICE.iter().find(|(t, _)| *t == ws.buy_tyre).unwrap().1 } else { info.price };
                if stats.money < price { ws.msg = format!("Not enough money ({} EUR needed).", price); }
                else { stats.money -= price; inv.parts.push(StockPart { slot, cond: 1.0, tyre: ws.buy_tyre }); ws.msg = format!("Bought new {} for {} EUR. Install it with Enter.", info.name, price); }
            }
        }
        Page::Fluids => {
            if keys.just_pressed(KeyCode::Digit1) {
                if inv.coolant_cans == 0 { if stats.money >= 8 { stats.money -= 8; inv.coolant_cans += 1; } else { ws.msg = "No coolant and no money (8 EUR).".into(); return; } }
                inv.coolant_cans -= 1; p.coolant = 1.0; ws.msg = "Radiator topped up with coolant (-8 EUR if bought).".into();
            }
            if keys.just_pressed(KeyCode::Digit2) && inv.fuel_cans > 0 {
                inv.fuel_cans -= 1; car.fuel = (car.fuel + 10.0).min(car.spec.fuel_cap); ws.msg = format!("Poured a fuel can. Tank: {:.0} L", car.fuel); return;
            }
            if keys.just_pressed(KeyCode::Digit2) {
                if stats.money >= 15 { stats.money -= 15; car.fuel = (car.fuel + 10.0).min(car.spec.fuel_cap); ws.msg = format!("+10 L fuel (15 EUR). Tank: {:.0} L", car.fuel); } else { ws.msg = "15 EUR needed.".into(); }
            }
            if keys.just_pressed(KeyCode::Digit3) && inv.dry_gas > 0 { inv.dry_gas -= 1; p.dry_gas_t = 900.0; p.frozen_line = false; ws.msg = "Dry gas added: fuel line won't freeze for ~15 min.".into(); return; }
            if keys.just_pressed(KeyCode::Digit3) {
                if stats.money >= 4 { stats.money -= 4; p.dry_gas_t = 900.0; p.frozen_line = false; ws.msg = "Dry gas added: fuel line won't freeze for ~15 min.".into(); } else { ws.msg = "4 EUR needed.".into(); }
            }
            if keys.just_pressed(KeyCode::Digit4) {
                if !p.get(Slot::Battery).installed { ws.msg = "No battery installed.".into(); } else { p.battery = 1.0; ws.msg = "Battery charged to 100% on the workshop charger.".into(); }
            }
            if keys.just_pressed(KeyCode::Digit5) {
                if p.flat { p.flat = false; ws.msg = "Wheel changed: puncture fixed.".into(); } else { ws.msg = "No flat tyre.".into(); }
            }
            if keys.just_pressed(KeyCode::Digit6) {
                // switch tyre type from stock (quick swap of whole set)
                ws.msg = "Use the Parts page: remove tyres, install the set you want.".into();
            }
        }
        Page::Manual => {}
    }
}

pub fn refresh(
    ws: Res<Workshop>,
    inv: Res<Inventory>,
    stats: Res<PlayerStats>,
    player: Query<&Transform, (With<Player>, Without<Car>)>,
    cars: Query<&Car>,
    parts: Query<&CarParts>,
    state: Res<State<GameState>>,
    mut root: Query<&mut Visibility, With<WsRootUi>>,
    mut left: Query<&mut Text, (With<WsLeft>, Without<WsRight>)>,
    mut right: Query<&mut Text, (With<WsRight>, Without<WsLeft>)>,
) {
    if let Ok(mut v) = root.single_mut() { *v = if *state.get() == GameState::Garage { Visibility::Inherited } else { Visibility::Hidden }; }
    let (Ok(p), Ok(car), Ok(pt)) = (parts.single(), cars.single(), player.single()) else { return };
    let (Ok(mut l), Ok(mut r)) = (left.single_mut(), right.single_mut()) else { return };
    let tabs = |cur: Page| format!("{}  {}  {}   (Tab, T/Esc)  EUR {}\n\n",
        if cur == Page::Parts { "[PARTS]" } else { " parts " }, if cur == Page::Fluids { "[FLUIDS & FIXES]" } else { " fluids & fixes " }, if cur == Page::Manual { "[MANUAL]" } else { " manual " }, stats.money);
    match ws.page {
        Page::Parts => {
            let mut s = tabs(ws.page);
            for (i, slot) in ALL.iter().enumerate() {
                let st = p.get(*slot); let info = slot.info();
                let boxes = inv.parts.iter().filter(|x| x.slot == *slot).count();
                s += &format!("{} {:<16} {}{}\n", if i == ws.sel { ">" } else { " " }, info.name, if st.installed { bar(st.cond) } else { "-- missing --".into() }, if boxes > 0 { format!("   (+{boxes} in boxes)") } else { String::new() });
            }
            l.0 = s;
            let slot = ALL[ws.sel]; let info = slot.info(); let st = p.get(slot);
            let mut t = format!("{} / {}\n\n{}\n\n", info.name, info.fi, info.hint);
            t += &format!("Status: {}\n", if st.installed { format!("installed, {}", bar(st.cond)) } else { "NOT installed".into() });
            if info.heavy { t += &format!("Heavy part - {}\n", if near_garage(pt.translation) { "you are at the garage: OK" } else { "needs the garage shed (west of the cabin)" }); }
            t += &format!("\n[Enter] {}\n", if st.installed { "Remove (to boxes)" } else { "Install from boxes" });
            if info.fix_cap > 0.0 { t += &format!("[F] {} (up to {:.0}%)\n", info.fix_label, info.fix_cap * 100.0); }
            let price = if slot == Slot::Tyres { TYRE_PRICE.iter().find(|(t, _)| *t == ws.buy_tyre).unwrap().1 } else { info.price };
            t += &format!("[B] Buy new: {} EUR\n", price);
            if slot == Slot::Tyres { t += &format!("[1] summer 80  [2] winter 180  [3] studded 260   (buying: {})\n   Installed type: {}{}\n", ws.buy_tyre.name(), p.tyre.name(), if p.flat { "  ** FLAT **" } else { "" }); }
            t += &format!("\nCar: coolant {:.0}%  battery {:.0}%  fuel {:.0} L  engine temp {:.0}C{}\n", p.coolant * 100.0, p.battery * 100.0, car.fuel, p.temp, if p.frozen_line { "  ** FUEL LINE FROZEN **" } else { "" });
            t += &format!("\n{}\n", ws.msg);
            r.0 = t;
        }
        Page::Fluids => {
            let s = tabs(ws.page);
            l.0 = s + &format!("[1] Fill coolant (can: 8 EUR)       now {:.0}%\n[2] Fuel +10 L (15 EUR)             now {:.0}/{:.0} L\n[3] Dry gas (4 EUR) - stops frozen fuel line\n[4] Charge battery (free)           now {:.0}%\n[5] Change flat wheel               {}\n", p.coolant * 100.0, car.fuel, car.spec.fuel_cap, p.battery * 100.0, if p.flat { "FLAT!" } else { "ok" });
            r.0 = format!("Boxes: {} spare parts\nCoolant cans: {}\n\n{}\n", inv.parts.len(), inv.coolant_cans, ws.msg);
        }
        Page::Manual => {
            l.0 = tabs(ws.page) + MANUAL_A;
            r.0 = MANUAL_B.to_string();
        }
    }
}

const MANUAL_A: &str = "KORJAAMO-OPAS - Workshop guide, vol. 1 (symptoms)\n\n\
Turn key (I) and ...\n\
 nothing at all            -> no battery, or it's totally flat. Install + charge.\n\
 click-click-click         -> battery too weak (cold eats it). Charge it.\n\
 starter spins, no engine  -> no engine installed (!)\n\
 cranks, won't catch       -> fouled plugs / frozen fuel line / no fuel\n\
 \"almost caught\"           -> cold + tired plugs: try again, or clean them\n\
 starts, then dies         -> clogged fuel filter or misfiring plugs\n\
While driving ...\n\
 power drops under load    -> clogged filter, fouled plugs, or overheating\n\
 temp >108C                -> coolant empty/leaking, radiator, broken belt\n\
 battery warning           -> alternator dead or belt snapped\n\
 lights/wipers dead        -> blown fuse (crash!) or flat battery\n\
 car pulls + feels spongy  -> flat tyre\n";
const MANUAL_B: &str = "Fix cheat-sheet\n\n\
 Plugs fouled     -> [F] clean (70%) or buy new (15)\n\
 Filter clogged   -> replace (12)\n\
 Frozen line      -> Fluids page: dry gas (4)\n\
 Overheating      -> Fluids: coolant, patch/replace radiator, new belt\n\
 Flat tyre        -> Fluids: change wheel; or patch (F) in Parts\n\
 Dented panels    -> [F] hammer them out\n\
 Blown fuse       -> buy a fuse (3)\n\n\
WINTER\n\
 Summer tyres on ice are a death wish.\n\
 Winter 180 EUR, studded 260 EUR: worth every cent.\n\
 Wipers + washer: snow piles on the glass fast when driving.\n\
 Heavy parts (engine, gearbox, radiator, alternator) only at the\n garage shed west of the cabin.\n";

/// Show the workshop panel only in `GameState::Garage`. Runs in every state (see life::dialog::sync_visibility for why).
pub fn sync_visibility(state: Res<State<GameState>>, mut root: Query<&mut Visibility, With<WsRootUi>>) {
    if let Ok(mut v) = root.single_mut() {
        let want = if *state.get() == GameState::Garage { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want { *v = want; }
    }
}
