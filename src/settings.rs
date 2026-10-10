//! User settings persisted to `settings.json` next to the executable's working dir.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub mouse_sensitivity: f32,
    pub fov_degrees: f32,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub render_distance: f32,
    pub invert_y: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { mouse_sensitivity: 0.0022, fov_degrees: 75.0, master_volume: 0.8, music_volume: 0.5, sfx_volume: 0.8, render_distance: 600.0, invert_y: false }
    }
}

const PATH: &str = "settings.json";

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(PATH).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }
    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(PATH, s);
        }
    }
}

/// Default key bindings (rebinding UI is a later task). Gameplay code reads these, never raw KeyCodes.
#[derive(Resource, Clone, Debug)]
pub struct Keybinds {
    pub forward: KeyCode, pub back: KeyCode, pub left: KeyCode, pub right: KeyCode,
    pub jump: KeyCode, pub run: KeyCode, pub crouch: KeyCode, pub interact: KeyCode,
    pub reload: KeyCode, pub flashlight: KeyCode,
    pub handbrake: KeyCode, pub wipers: KeyCode, pub lights: KeyCode, pub ignition: KeyCode, pub horn: KeyCode,
}
impl Default for Keybinds {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW, back: KeyCode::KeyS, left: KeyCode::KeyA, right: KeyCode::KeyD,
            jump: KeyCode::Space, run: KeyCode::ShiftLeft, crouch: KeyCode::ControlLeft, interact: KeyCode::KeyE,
            reload: KeyCode::KeyR, flashlight: KeyCode::KeyF,
            handbrake: KeyCode::Space, wipers: KeyCode::KeyV, lights: KeyCode::KeyL, ignition: KeyCode::KeyI, horn: KeyCode::KeyH,
        }
    }
}

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::load()).init_resource::<Keybinds>();
    }
}
