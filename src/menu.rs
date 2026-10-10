//! Main menu, pause menu, load screen, settings. Keyboard driven: Up/Down, Enter, Left/Right to adjust, Esc back.
use crate::console::Console;
use crate::save::{self, PendingLoad, SaveRequest};
use crate::i18n::{Strings, LANGS};
use crate::manifest::AssetManifest;
use crate::settings::Settings;
use crate::state::GameState;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen { Root, Load, Settings }

#[derive(Resource)]
pub struct Menu { pub screen: Screen, pub sel: usize, pub note: String }
impl Default for Menu { fn default() -> Self { Self { screen: Screen::Root, sel: 0, note: String::new() } } }

#[derive(Component)] struct MenuUi;
#[derive(Component)] struct MenuText;
#[derive(Component)] struct MenuCam;

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Menu>()
            .add_systems(Startup, spawn_ui)
            .add_systems(OnEnter(GameState::MainMenu), spawn_cam)
            .add_systems(OnExit(GameState::MainMenu), despawn_cam)
            .add_systems(OnEnter(GameState::Paused), reset_menu)
            .add_systems(OnEnter(GameState::MainMenu), reset_menu)
            .add_systems(Update, (input, refresh).chain());
    }
}

fn spawn_ui(mut commands: Commands) {
    commands.spawn((MenuUi, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), padding: UiRect::all(Val::Px(60.0)), ..default() },
        BackgroundColor(Color::srgba(0.02, 0.04, 0.09, 0.86)), GlobalZIndex(80), Visibility::Hidden))
        .with_children(|c| { c.spawn((MenuText, Text::new(""), TextFont { font_size: FontSize::Px(26.0), ..default() }, TextColor(Color::srgb(0.92, 0.95, 1.0)))); });
}
fn spawn_cam(mut commands: Commands) { commands.spawn((MenuCam, Camera2d)); }
fn despawn_cam(mut commands: Commands, q: Query<Entity, With<MenuCam>>) { for e in &q { commands.entity(e).despawn(); } }
fn reset_menu(mut m: ResMut<Menu>) { m.screen = Screen::Root; m.sel = 0; m.note.clear(); }

fn has_autosave() -> bool { save::read("autosave").is_some() }

/// Items for the current screen (label, value-suffix).
fn items(m: &Menu, state: GameState, s: &Settings, t: &Strings) -> Vec<String> {
    match (m.screen, state) {
        (Screen::Root, GameState::MainMenu) => {
            let mut v = vec![t.t("menu.new")];
            v.push(if has_autosave() { t.t("menu.continue") } else { t.t("menu.continue_none") });
            v.extend([t.t("menu.load"), t.t("menu.settings"), t.t("menu.quit")]);
            v
        }
        (Screen::Root, _) => vec![t.t("menu.resume"), t.tn("menu.save_slot", 1), t.tn("menu.save_slot", 2), t.tn("menu.save_slot", 3), t.t("menu.load"), t.t("menu.settings"), t.t("menu.quit_desktop")],
        (Screen::Load, _) => { let mut v: Vec<String> = save::SLOTS.iter().map(|s| format!("{:<9} {}", s, save::summary(s))).collect(); v.push(t.t("menu.back")); v }
        (Screen::Settings, _) => vec![
            format!("{:<16}< {:.0}% >", t.t("set.master"), s.master_volume * 100.0), format!("{:<16}< {:.0}% >", t.t("set.music"), s.music_volume * 100.0), format!("{:<16}< {:.0}% >", t.t("set.sfx"), s.sfx_volume * 100.0),
            format!("{:<16}< {:.1} >", t.t("set.sens"), s.mouse_sensitivity * 1000.0), format!("{:<16}< {:.0} >", t.t("set.fov"), s.fov_degrees), format!("{:<16}< {} >", t.t("set.invert"), if s.invert_y { t.t("set.yes") } else { t.t("set.no") }),
            format!("{:<16}< {} >", t.t("set.lang"), LANGS.iter().find(|(c, _)| *c == s.language).map(|(_, n)| *n).unwrap_or("English")), t.t("menu.back"),
        ],
    }
}

fn input(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    mut m: ResMut<Menu>,
    mut s: ResMut<Settings>,
    con: Res<Console>,
    mut pending: ResMut<PendingLoad>,
    mut save_req: MessageWriter<SaveRequest>,
    mut exit: MessageWriter<AppExit>,
    mut cam: Query<&mut Projection, With<crate::player::PlayerCamera>>,
    mut strings: ResMut<Strings>,
    man: Res<AssetManifest>,
) {
    let st = *state.get();
    if !matches!(st, GameState::MainMenu | GameState::Paused) || con.open { return; }
    let list = items(&m, st, &s, &strings);
    let n = list.len();
    if keys.just_pressed(KeyCode::ArrowDown) { m.sel = (m.sel + 1) % n; }
    if keys.just_pressed(KeyCode::ArrowUp) { m.sel = (m.sel + n - 1) % n; }
    let left = keys.just_pressed(KeyCode::ArrowLeft); let right = keys.just_pressed(KeyCode::ArrowRight);
    let enter = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space);
    if keys.just_pressed(KeyCode::Escape) && st == GameState::MainMenu && m.screen != Screen::Root { m.screen = Screen::Root; m.sel = 0; return; }
    if keys.just_pressed(KeyCode::Escape) && st == GameState::Paused { if m.screen != Screen::Root { m.screen = Screen::Root; m.sel = 0; } else { next.set(GameState::Playing); } return; }
    match (m.screen, st) {
        (Screen::Root, GameState::MainMenu) if enter => match m.sel {
            0 => next.set(GameState::Playing),
            1 => { if let Some(d) = save::read("autosave") { pending.0 = Some(d); next.set(GameState::Playing); } else { m.note = "No autosave yet.".into(); } }
            2 => { m.screen = Screen::Load; m.sel = 0; }
            3 => { m.screen = Screen::Settings; m.sel = 0; }
            _ => { exit.write(AppExit::Success); }
        },
        (Screen::Root, _) if enter => match m.sel {
            0 => next.set(GameState::Playing),
            1..=3 => { save_req.write(SaveRequest(format!("slot{}", m.sel))); m.note = format!("Saved to slot {}.", m.sel); }
            4 => { m.screen = Screen::Load; m.sel = 0; }
            5 => { m.screen = Screen::Settings; m.sel = 0; }
            _ => { exit.write(AppExit::Success); }
        },
        (Screen::Load, _) if enter => {
            if m.sel < save::SLOTS.len() {
                if let Some(d) = save::read(save::SLOTS[m.sel]) { pending.0 = Some(d); next.set(GameState::Playing); m.screen = Screen::Root; } else { m.note = "That slot is empty.".into(); }
            } else { m.screen = Screen::Root; m.sel = 0; }
        }
        (Screen::Settings, _) => {
            let d = if right { 1.0 } else if left { -1.0 } else { 0.0 };
            match m.sel {
                0 => s.master_volume = (s.master_volume + d * 0.05).clamp(0.0, 1.0),
                1 => s.music_volume = (s.music_volume + d * 0.05).clamp(0.0, 1.0),
                2 => s.sfx_volume = (s.sfx_volume + d * 0.05).clamp(0.0, 1.0),
                3 => s.mouse_sensitivity = (s.mouse_sensitivity + d * 0.0002).clamp(0.0004, 0.01),
                4 => { s.fov_degrees = (s.fov_degrees + d * 5.0).clamp(50.0, 120.0); if d != 0.0 { if let Ok(mut p) = cam.single_mut() { if let Projection::Perspective(pp) = &mut *p { pp.fov = s.fov_degrees.to_radians(); } } } }
                5 => if d != 0.0 || enter { s.invert_y = !s.invert_y },
                6 => if d != 0.0 || enter { let i = LANGS.iter().position(|(c, _)| *c == s.language).unwrap_or(0); s.language = LANGS[(i + 1) % LANGS.len()].0.to_string(); strings.set_lang(&man, &s.language); s.save(); },
                _ => if enter { s.save(); m.screen = Screen::Root; m.sel = 0; },
            }
            if d != 0.0 { s.save(); }
        }
        _ => {}
    }
}

fn refresh(
    state: Res<State<GameState>>,
    m: Res<Menu>,
    s: Res<Settings>,
    strings: Res<Strings>,
    con: Res<Console>,
    mut root: Query<&mut Visibility, With<MenuUi>>,
    mut text: Query<&mut Text, With<MenuText>>,
) {
    let st = *state.get();
    let show = matches!(st, GameState::MainMenu | GameState::Paused) && !con.open;
    if let Ok(mut v) = root.single_mut() { *v = if show { Visibility::Inherited } else { Visibility::Hidden }; }
    if !show { return; }
    let Ok(mut t) = text.single_mut() else { return };
    let title = match (st, m.screen) { (GameState::MainMenu, Screen::Root) => format!("{}\n{}\n", strings.t("menu.title"), strings.t("menu.tagline")), (_, Screen::Root) => format!("{}\n", strings.t("menu.paused")), (_, Screen::Load) => format!("{}\n", strings.t("menu.load_title")), (_, Screen::Settings) => format!("{}\n", strings.t("menu.settings_title")) };
    let mut out = format!("{title}\n");
    for (i, it) in items(&m, st, &s, &strings).iter().enumerate() { out += &format!("{} {}\n", if i == m.sel { ">" } else { " " }, it); }
    out += &format!("\n{}\n", m.note);
    if st == GameState::MainMenu && m.screen == Screen::Root {
        out += "\nWASD move  Shift run  Ctrl crouch  Space jump  E car/use  T workshop  I ignition  V wipers  L lights  H horn\nF flashlight  X mount light  1-5 weapons  LMB fire  R reload  `/F1 console  Esc pause\n\nby Crowelian 2026 + Sonnet";
    }
    t.0 = out;
}
