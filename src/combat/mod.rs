//! FPS layer: weapons, hitscan, enemies (billboard sprites), pickups, player health.
pub mod director;
pub mod enemy;
pub mod flashlight;
pub mod hud;
pub mod pickups;
pub mod weapons;

use crate::state::GameState;
use bevy::prelude::*;

#[derive(Resource)]
pub struct PlayerStats {
    pub hp: f32,
    pub armor: f32,
    pub money: i32,
    pub dead_timer: f32,
    pub hurt_flash: f32,
    pub god: bool,
}
impl Default for PlayerStats {
    fn default() -> Self { Self { hp: 100.0, armor: 0.0, money: 120, dead_timer: 0.0, hurt_flash: 0.0, god: false } }
}
impl PlayerStats {
    pub fn damage(&mut self, d: f32) {
        if self.god { return; }
        let absorbed = (d * 0.5).min(self.armor);
        self.armor -= absorbed;
        self.hp = (self.hp - (d - absorbed)).max(0.0);
        self.hurt_flash = 0.6;
    }
}

/// Anything the player's bullets can hit.
#[derive(Component)]
pub struct Hittable { pub radius: f32, pub height: f32 }

#[derive(Message)]
pub struct HitEvent { pub target: Entity, pub damage: f32, pub headshot: bool }

pub struct CombatPlugin;
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerStats>()
            .init_resource::<weapons::Loadout>()
            .init_resource::<flashlight::Flashlight>()
            .add_message::<HitEvent>()
            .add_systems(Startup, (enemy::load_enemy_assets, pickups::load_pickup_assets, weapons::load_weapon_assets))
            .add_systems(OnEnter(GameState::Playing), (enemy::spawn_initial, hud::spawn_hud))
            .add_systems(
                Update,
                (
                    flashlight::spawn_beam,
                    flashlight::update,
                    weapons::switch_and_fire,
                    director::run,
                    enemy::apply_hits,
                    enemy::ai,
                    enemy::animate,
                    enemy::car_runover,
                    pickups::collect,
                    hud::update_hud,
                    hud::update_weapon_sprite,
                    player_death,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn player_death(
    time: Res<Time>,
    mut stats: ResMut<PlayerStats>,
    mut player: Query<&mut Transform, With<crate::player::Player>>,
    mut cars: Query<&mut crate::vehicle::Car>,
) {
    stats.hurt_flash = (stats.hurt_flash - time.delta_secs()).max(0.0);
    if stats.hp <= 0.0 {
        stats.dead_timer += time.delta_secs();
        if stats.dead_timer > 3.0 {
            // wake up in the cabin, lighter wallet
            stats.hp = 100.0; stats.armor = 0.0; stats.dead_timer = 0.0; stats.money = (stats.money - 50).max(0);
            if let Ok(mut t) = player.single_mut() {
                let p = crate::world::layout::CABIN + Vec2::new(-4.0, 3.0);
                t.translation = Vec3::new(p.x, crate::world::layout::height_at(p.x, p.y), p.y);
            }
            for mut c in &mut cars { c.v_long = 0.0; c.v_lat = 0.0; }
        }
    }
}
