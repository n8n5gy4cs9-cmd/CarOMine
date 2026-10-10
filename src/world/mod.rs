pub mod atmosphere;
pub mod billboard;
pub mod layout;
pub mod noise;
pub mod scatter;
pub mod sky;
pub mod terrain;
pub mod village;
pub mod weather;

use crate::state::GameState;
use bevy::prelude::*;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(atmosphere::AtmospherePlugin)
            .init_resource::<terrain::Chunks>()
            .init_resource::<sky::TimeOfDay>()
            .init_resource::<weather::Weather>()
            .add_systems(Startup, (terrain::setup_material, scatter::setup_scatter, weather::spawn_flakes, village::setup))
            .add_systems(
                Update,
                (
                    terrain::stream_chunks,
                    billboard::face_camera,
                    billboard::update_angle_sprites,
                    weather::animate_flakes,
                    sky::advance_time,
                    sky::update_sky,
                    village::spawn_village,
                    village::window_glow,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
