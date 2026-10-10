//! Life sim: needs (hunger/energy/warmth), home + sauna + shops via door "interact zones", Kela benefit, missions.
pub mod dialog;
pub mod missions;
pub mod needs;

use crate::state::GameState;
use crate::world::layout::*;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZoneKind { Home, Sauna, Market, AutoShop, Inspection }

#[derive(Clone, Copy)]
pub struct Zone { pub pos: Vec2, pub radius: f32, pub kind: ZoneKind }

impl Zone {
    pub fn prompt_key(&self) -> &'static str {
        match self.kind { ZoneKind::Home => "prompt.home", ZoneKind::Sauna => "prompt.sauna", ZoneKind::Market => "prompt.market", ZoneKind::AutoShop => "prompt.parts", ZoneKind::Inspection => "prompt.inspect" }
    }
}

/// Door stand-point for a house placed with `spawn_house` (pos, yaw, depth).
pub fn door(p: Vec2, yaw: f32, d: f32) -> Vec2 { let l = d / 2.0 + 1.9; Vec2::new(p.x + l * yaw.sin(), p.y + l * yaw.cos()) }

pub fn zones() -> Vec<Zone> {
    let q = std::f32::consts::FRAC_PI_2;
    vec![
        Zone { pos: door(CABIN, 0.0, 5.0), radius: 3.2, kind: ZoneKind::Home },
        Zone { pos: door(CABIN + Vec2::new(9.0, -4.0), 0.0, 3.4), radius: 3.0, kind: ZoneKind::Sauna },
        Zone { pos: door(VILLAGE + Vec2::new(22.0, 0.0), -q, 9.0), radius: 3.4, kind: ZoneKind::Market },
        Zone { pos: door(VILLAGE + Vec2::new(18.0, 42.0), -q, 6.0), radius: 3.4, kind: ZoneKind::AutoShop },
        Zone { pos: door(VILLAGE + Vec2::new(-24.0, -42.0), q, 7.0), radius: 3.6, kind: ZoneKind::Inspection },
    ]
}

#[derive(Resource, Default)]
pub struct Nearby(pub Option<Zone>);

pub struct LifePlugin;
impl Plugin for LifePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Nearby>().init_resource::<needs::Needs>().init_resource::<missions::Missions>().init_resource::<dialog::Dialog>()
            .add_systems(Startup, (dialog::spawn_ui, needs::spawn_prompt, missions::spawn_hud))
            .add_systems(OnEnter(GameState::Playing), spawn_lamps)
            .add_systems(Update, (needs::tick, needs::kela, find_zone, dialog::open, missions::tick, needs::prompt).chain().run_if(in_state(GameState::Playing)))
            .add_systems(Update, (dialog::input, dialog::refresh).chain().run_if(in_state(GameState::Dialog)));
    }
}

fn find_zone(player: Query<&Transform, With<crate::player::Player>>, driven: Query<(), With<crate::vehicle::Driven>>, mut near: ResMut<Nearby>) {
    near.0 = None;
    if !driven.is_empty() { return; }
    let Ok(p) = player.single() else { return };
    let pp = Vec2::new(p.translation.x, p.translation.z);
    near.0 = zones().into_iter().filter(|z| z.pos.distance(pp) < z.radius).min_by(|a, b| a.pos.distance(pp).partial_cmp(&b.pos.distance(pp)).unwrap());
}

/// A lit street lamp at each door so the entrances are findable at night.
fn spawn_lamps(mut commands: Commands, scatter: Option<Res<crate::world::scatter::ScatterAssets>>, mut done: Local<bool>) {
    let Some(sc) = scatter else { return };
    if *done { return; }
    *done = true;
    for z in zones() {
        let p = z.pos + Vec2::new(1.8, 0.0);
        let y = height_at(p.x, p.y);
        let e = commands.spawn((Mesh3d(sc.lamp.mesh.clone()), MeshMaterial3d(sc.lamp.mats[0].clone()), Transform::from_xyz(p.x, y, p.y), crate::world::billboard::Billboard)).id();
        commands.entity(e).with_child((PointLight { intensity: 400_000.0, range: 18.0, color: Color::srgb(1.0, 0.85, 0.55), shadow_maps_enabled: false, ..default() }, Transform::from_xyz(0.0, 3.6, 0.0)));
    }
}
