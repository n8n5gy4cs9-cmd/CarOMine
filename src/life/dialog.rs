//! List dialogs opened at door zones: Home (cabin), Phone (gigs), Sauna, K-market, Auto parts shop.
use super::missions::{Briefcase, Missions, MissionTarget};
use super::needs::Needs;
use super::*;
use crate::build::{Inventory, Slot, StockPart, Toasts, ALL};
use crate::combat::flashlight::Flashlight;
use crate::combat::PlayerStats;
use crate::vehicle::spec::TyreType;
use crate::world::sky::TimeOfDay;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen { Home, Phone, Sauna, Market, Parts, Inspect, Intro, Ending }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    Close, Sleep, Eat, Coffee, Chop, Phone, Back, Accept(usize), Abandon,
    SaunaHeat, SaunaBathe, Inspect, BuyFood, BuyCoffee, BuyDryGas, BuyCoolant, BuyFuel, BuyBatteries, BuyPart(Slot, TyreType, i32), GoParts,
}

#[derive(Resource)]
pub struct Dialog { pub screen: Screen, pub sel: usize, pub msg: String }
impl Default for Dialog { fn default() -> Self { Self { screen: Screen::Home, sel: 0, msg: String::new() } } }

#[derive(Component)] pub struct DlgRoot;
#[derive(Component)] pub struct DlgText;

pub fn spawn_ui(mut commands: Commands) {
    commands.spawn((DlgRoot, Node { position_type: PositionType::Absolute, left: Val::Percent(12.0), top: Val::Percent(10.0), width: Val::Percent(76.0), height: Val::Percent(80.0), padding: UiRect::all(Val::Px(18.0)), ..default() },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.09, 0.95)), GlobalZIndex(70), Visibility::Hidden))
        .with_children(|c| { c.spawn((DlgText, Text::new(""), TextFont { font_size: FontSize::Px(20.0), ..default() }, TextColor(Color::srgb(0.95, 0.95, 0.9)))); });
}

fn title(s: Screen) -> &'static str {
    match s { Screen::Home => "HOME - your cabin, Nurmijarvi", Screen::Phone => "PHONE - agency gig board", Screen::Sauna => "SAUNA", Screen::Market => "K-MARKET", Screen::Parts => "AUTO PARTS - Nurmijarven Osa", Screen::Inspect => "KATSASTUS - vehicle inspection station", Screen::Intro => "CAROMINE", Screen::Ending => "THE END" }
}

pub struct View<'a> { pub needs: &'a Needs, pub stats: &'a PlayerStats, pub inv: &'a Inventory, pub missions: &'a Missions, pub hour: f32 }

pub fn rows(screen: Screen, v: &View) -> Vec<(String, Act)> {
    let mut r: Vec<(String, Act)> = vec![];
    match screen {
        Screen::Home => {
            let late = v.hour >= 19.0 || v.hour < 5.0 || v.needs.energy < 0.4;
            r.push((format!("Sleep until morning{}", if late { "" } else { "  (not tired yet)" }), Act::Sleep));
            r.push((format!("Eat from the fridge  (sausages: {})", v.needs.food), Act::Eat));
            r.push((format!("Make coffee  (bags: {})", v.needs.coffee), Act::Coffee));
            r.push((format!("Chop firewood  (+3 logs, costs energy; logs: {})", v.needs.logs), Act::Chop));
            r.push(("Pick up the phone: agency gigs".into(), Act::Phone));
        }
        Screen::Phone => {
            if let Some(m) = &v.missions.active { r.push((format!("ACTIVE: {} ({} EUR) - abandon it", m.title, m.reward), Act::Abandon)); }
            for (i, m) in v.missions.board.iter().enumerate() { r.push((format!("{}  [{} EUR]\n      {}", m.title, m.reward, m.desc), Act::Accept(i))); }
            r.push(("Hang up".into(), Act::Back));
        }
        Screen::Sauna => {
            r.push((format!("Light the stove  (2 logs; logs: {})  {}", v.needs.logs, if v.needs.sauna_hot > 0.0 { format!("-- hot: {:.0} min left", v.needs.sauna_hot) } else { "-- cold".into() }), Act::SaunaHeat));
            r.push(("Throw loyly and bathe  (1 hour)".into(), Act::SaunaBathe));
        }
        Screen::Market => {
            for (l, p, a) in [("Makkara (sausage) 3 EUR", 3, Act::BuyFood), ("Coffee bag 2 EUR", 2, Act::BuyCoffee), ("Dry gas 4 EUR", 4, Act::BuyDryGas), ("Coolant can 8 EUR", 8, Act::BuyCoolant), ("Fuel can (10 L) 15 EUR", 15, Act::BuyFuel), ("Flashlight batteries 4 EUR", 4, Act::BuyBatteries)] { let _ = p; r.push((l.to_string(), a)); }
            r.push(("Leave".into(), Act::Close));
        }
        Screen::Inspect => { r.push(("Run the inspection (30 EUR)".into(), Act::Inspect)); r.push(("Leave".into(), Act::Close)); }
        Screen::Intro | Screen::Ending => { r.push(("Continue".into(), Act::Close)); }
        Screen::Parts => {
            for s in ALL {
                let i = s.info();
                if s == Slot::Tyres { for (t, p) in [(TyreType::Summer, 80), (TyreType::Winter, 180), (TyreType::Studded, 260)] { r.push((format!("Tyres ({}) {} EUR", t.name(), p), Act::BuyPart(s, t, p))); } }
                else { r.push((format!("{} / {}  {} EUR", i.name, i.fi, i.price), Act::BuyPart(s, TyreType::Summer, i.price))); }
            }
            r.push(("Leave".into(), Act::Close));
        }
    }
    r
}

pub fn open(
    keys: Res<ButtonInput<KeyCode>>,
    kb: Res<crate::settings::Keybinds>,
    near: Res<Nearby>,
    mut dlg: ResMut<Dialog>,
    mut next: ResMut<NextState<GameState>>,
    mut needs: ResMut<Needs>,
    mut missions: ResMut<Missions>,
    mut cursor: Query<&mut bevy::window::CursorOptions, With<bevy::window::PrimaryWindow>>,
) {
    if !keys.just_pressed(kb.interact) { return; }
    let Some(z) = near.0 else { return };
    dlg.screen = match z.kind { ZoneKind::Home => Screen::Home, ZoneKind::Sauna => Screen::Sauna, ZoneKind::Market => Screen::Market, ZoneKind::AutoShop => Screen::Parts, ZoneKind::Inspection => Screen::Inspect };
    dlg.sel = 0; dlg.msg.clear();
    missions.ensure_board();
    let _ = &mut needs;
    next.set(GameState::Dialog);
    if let Ok(mut c) = cursor.single_mut() { c.grab_mode = bevy::window::CursorGrabMode::None; c.visible = true; }
}

#[allow(clippy::too_many_arguments)]
pub fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut dlg: ResMut<Dialog>,
    mut next: ResMut<NextState<GameState>>,
    mut needs: ResMut<Needs>,
    mut stats: ResMut<PlayerStats>,
    mut inv: ResMut<Inventory>,
    mut missions: ResMut<Missions>,
    mut tod: ResMut<TimeOfDay>,
    mut fl: ResMut<Flashlight>,
    mut toasts: ResMut<Toasts>,
    mut save: MessageWriter<crate::save::SaveRequest>,
    mut sfx: MessageWriter<crate::audio::Sfx>,
    mut inspect: MessageWriter<crate::story::RunInspection>,
    mut commands: Commands,
    targets: Query<Entity, With<MissionTarget>>,
    brief: Query<Entity, With<Briefcase>>,
) {
    let view = View { needs: &needs, stats: &stats, inv: &inv, missions: &missions, hour: tod.hour };
    let list = rows(dlg.screen, &view);
    let n = list.len().max(1);
    if keys.just_pressed(KeyCode::Escape) { if dlg.screen == Screen::Phone { dlg.screen = Screen::Home; dlg.sel = 0; } else { next.set(GameState::Playing); } return; }
    if keys.just_pressed(KeyCode::ArrowDown) { dlg.sel = (dlg.sel + 1) % n; sfx.write(crate::audio::Sfx::new("sfx.ui_click").vol(0.4)); }
    if keys.just_pressed(KeyCode::ArrowUp) { dlg.sel = (dlg.sel + n - 1) % n; sfx.write(crate::audio::Sfx::new("sfx.ui_click").vol(0.4)); }
    if !(keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space)) { return; }
    let act = list[dlg.sel.min(n - 1)].1;
    let mut msg = String::new();
    let mut buy = |cost: i32, stats: &mut PlayerStats| -> bool { if stats.money >= cost { stats.money -= cost; true } else { false } };
    match act {
        Act::Close => { next.set(GameState::Playing); }
        Act::Back => { dlg.screen = Screen::Home; dlg.sel = 0; }
        Act::Phone => { dlg.screen = Screen::Phone; dlg.sel = 0; }
        Act::Sleep => {
            if !(tod.hour >= 19.0 || tod.hour < 5.0 || needs.energy < 0.4) { msg = "You're not tired. (Sleep after 19:00 or when exhausted.)".into(); }
            else { tod.hour = 7.5; needs.energy = 1.0; needs.warmth = 0.85; needs.hunger = (needs.hunger - 0.2).max(0.05); stats.hp = (stats.hp + 40.0).min(100.0); needs.prev_hour = 7.4; save.write(crate::save::SaveRequest("autosave".into())); msg = "You sleep like a log. It's 07:30. Game autosaved.".into(); }
        }
        Act::Eat => { if needs.food > 0 { needs.food -= 1; needs.hunger = (needs.hunger + 0.4).min(1.0); msg = "Makkara with mustard. Hunger sated.".into(); } else { msg = "Fridge is empty. Buy groceries at the K-market.".into(); } }
        Act::Coffee => { if needs.coffee > 0 { needs.coffee -= 1; needs.energy = (needs.energy + 0.25).min(1.0); needs.warmth = (needs.warmth + 0.2).min(1.0); msg = "Strong Finnish coffee. Energy and warmth up.".into(); } else { msg = "No coffee! This is an emergency. K-market.".into(); } }
        Act::Chop => { if needs.energy > 0.1 { needs.energy -= 0.05; needs.logs += 3; msg = "Thunk. Thunk. +3 logs.".into(); } else { msg = "Too tired to swing the axe.".into(); } }
        Act::SaunaHeat => { if needs.logs >= 2 { needs.logs -= 2; needs.sauna_hot = 30.0; msg = "The stove roars. Sauna will be hot for ~30 minutes.".into(); } else { msg = "Not enough logs (need 2). Chop some at the cabin.".into(); } }
        Act::SaunaBathe => {
            if needs.sauna_hot <= 0.0 { msg = "The stove is cold. Light it first.".into(); }
            else { needs.warmth = 1.0; needs.energy = (needs.energy + 0.1).min(1.0); stats.hp = (stats.hp + 20.0).min(100.0); tod.hour = (tod.hour + 1.0).rem_euclid(24.0); sfx.write(crate::audio::Sfx::new("sfx.sauna_hiss")); msg = "Loyly! You feel reborn. (1 hour passed.)".into(); }
        }
        Act::Accept(i) => {
            if missions.active.is_some() { msg = "Finish or abandon your current gig first.".into(); }
            else if let Some(mut m) = missions.board.get(i).cloned() { m.started_hour = tod.hour; toasts.say(format!("Gig accepted: {}", m.title)); missions.active = Some(m); missions.board.remove(i); next.set(GameState::Playing); }
        }
        Act::Abandon => { super::missions::abandon(&mut missions, &mut commands, &targets, &brief); msg = "Gig abandoned.".into(); }
        Act::BuyFood => { if buy(3, &mut stats) { needs.food += 1; msg = "+1 makkara in the fridge.".into(); } else { msg = "Not enough money.".into(); } }
        Act::BuyCoffee => { if buy(2, &mut stats) { needs.coffee += 1; msg = "+1 coffee bag.".into(); } else { msg = "Not enough money.".into(); } }
        Act::BuyDryGas => { if buy(4, &mut stats) { inv.dry_gas += 1; msg = "Bought dry gas (use in the workshop > Fluids).".into(); } else { msg = "Not enough money.".into(); } }
        Act::BuyCoolant => { if buy(8, &mut stats) { inv.coolant_cans += 1; msg = "Bought a coolant can.".into(); } else { msg = "Not enough money.".into(); } }
        Act::BuyFuel => { if buy(15, &mut stats) { inv.fuel_cans += 1; msg = "Bought a fuel can (10 L, use in workshop > Fluids).".into(); } else { msg = "Not enough money.".into(); } }
        Act::BuyBatteries => { if buy(4, &mut stats) { fl.battery = 1.0; msg = "Fresh batteries in the flashlight.".into(); } else { msg = "Not enough money.".into(); } }
        Act::GoParts => {}
        Act::Inspect => { inspect.write(crate::story::RunInspection); }
        Act::BuyPart(s, t, p) => { if buy(p, &mut stats) { inv.parts.push(StockPart { slot: s, cond: 1.0, tyre: t }); msg = format!("Bought {} - it's in your boxes. Install via the workshop (T).", s.info().name); } else { msg = format!("Need {p} EUR."); } }
    }
    if !msg.is_empty() { dlg.msg = msg; }
    // keep selection in range after list changes
    dlg.sel = dlg.sel.min(n.saturating_sub(1));
}

#[allow(clippy::too_many_arguments)]
pub fn refresh(
    dlg: Res<Dialog>, needs: Res<Needs>, stats: Res<PlayerStats>, inv: Res<Inventory>, missions: Res<Missions>, tod: Res<TimeOfDay>,
    state: Res<State<GameState>>,
    mut root: Query<&mut Visibility, With<DlgRoot>>, mut text: Query<&mut Text, With<DlgText>>,
) {
    if let Ok(mut v) = root.single_mut() { *v = if *state.get() == GameState::Dialog { Visibility::Inherited } else { Visibility::Hidden }; }
    let Ok(mut t) = text.single_mut() else { return };
    let view = View { needs: &needs, stats: &stats, inv: &inv, missions: &missions, hour: tod.hour };
    let mut s = format!("{}      EUR {}   hunger {:.0}%  energy {:.0}%  warmth {:.0}%\n\n", title(dlg.screen), stats.money, needs.hunger * 100.0, needs.energy * 100.0, needs.warmth * 100.0);
    for (i, (l, _)) in rows(dlg.screen, &view).iter().enumerate() { s += &format!("{} {}\n", if i == dlg.sel { ">" } else { " " }, l); }
    match dlg.screen {
        Screen::Intro => s = INTRO.to_string() + "\n\n> Continue",
        Screen::Ending => s = ENDING.to_string() + "\n\n> Continue (free roam)",
        Screen::Inspect => { s += &format!("\nThe inspector checks:\n{}\n", crate::story::CHECK_LIST); s += &format!("\n{}\n\n[Enter] do it  [Esc] leave", dlg.msg); }
        _ => s += &format!("\n{}\n\n[Up/Down] choose  [Enter] do it  [Esc] leave", dlg.msg),
    }
    t.0 = s;
}

const INTRO: &str = "NURMIJARVI, FINLAND. DECEMBER.\n\n\
You were the Agency's best field operative. Then budget cuts, a misplaced briefcase and one\n\
very awkward Christmas party later, you are unemployed, living in a cabin in the countryside\n\
with a pile of boxes in the garage: the remains of your old car.\n\n\
The Agency wants the briefcase back. Their ex-colleagues have started turning up in the woods.\n\
Kela pays 35 euros every morning. Coffee is 2. The sauna is warm. The roads are not.\n\n\
Build the car. Pass the katsastus. Drive to Helsinki.\n\n\
  T  workshop (next to the car)    E  doors / car    F  flashlight    ` console\n\
  Garage shed is west of the cabin. Engine and gearbox are in the boxes.";
const ENDING: &str = "The Director falls. The Agency's files burn in the Helsinki snow.\n\n\
You drive home through the long blue dusk, wipers going, studded tyres crunching on the ice.\n\
Somewhere in Nurmijarvi a sauna is heating up.\n\n\
          CAROMINE\n          by Crowelian 2026 + Sonnet\n\nThanks for playing. You can keep roaming, taking gigs and tuning the car.";
