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
            .add_systems(Update, pause_toggle);
    }
}

fn boot_to_menu(mut next: ResMut<NextState<GameState>>) {
    // Later (T-011): wait for the asset manifest to finish loading.
    next.set(GameState::MainMenu);
}

fn pause_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        // Resuming is handled by the pause menu (Esc at its root screen) so submenus can use Esc to go back.
        if *state.get() == GameState::Playing { next.set(GameState::Paused); }
    }
}
