use bevy::prelude::*;

/// Top-level game flow. Add new states here; systems gate on `in_state(..)`.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Boot,
    MainMenu,
    Playing,
    Paused,
    #[allow(dead_code)]
    Garage,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(Update, boot_to_menu.run_if(in_state(GameState::Boot)))
            .add_systems(Update, menu_input.run_if(in_state(GameState::MainMenu)))
            .add_systems(Update, pause_toggle);
    }
}

fn boot_to_menu(mut next: ResMut<NextState<GameState>>) {
    // Later (T-011): wait for the asset manifest to finish loading.
    next.set(GameState::MainMenu);
}

fn menu_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next.set(GameState::Playing);
    }
}

fn pause_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Playing => next.set(GameState::Paused),
            GameState::Paused => next.set(GameState::Playing),
            _ => {}
        }
    }
}
