//! Decals on the snow: tyre tracks, footprints, blood splats. Ring-buffered quads (oldest are recycled), no per-frame allocation.
use super::layout::*;
use crate::manifest::AssetManifest;
use crate::player::Player;
use crate::vehicle::Car;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind { Tyre, Foot, Blood }

struct Pool { cap: usize, ents: Vec<Entity>, next: usize, mat: Handle<StandardMaterial> }

#[derive(Resource)]
pub struct Tracks { quad: Handle<Mesh>, tyre: Pool, foot: Pool, blood: Pool }

pub struct TracksPlugin;
impl Plugin for TracksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, (car_tracks, foot_tracks).run_if(in_state(crate::state::GameState::Playing)));
    }
}

fn setup(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let tex = server.load(man.path("fx.puff"));
    let mut mk = |c: Color| mats.add(StandardMaterial { base_color: c, base_color_texture: Some(tex.clone()), alpha_mode: AlphaMode::Blend, unlit: false, perceptual_roughness: 1.0, reflectance: 0.0, depth_bias: 2.0, cull_mode: None, double_sided: true, ..default() });
    let pool = |cap, mat| Pool { cap, ents: Vec::new(), next: 0, mat };
    commands.insert_resource(Tracks {
        quad: meshes.add(Rectangle::new(1.0, 1.0)),
        tyre: pool(700, mk(Color::srgba(0.22, 0.24, 0.3, 0.55))),
        foot: pool(260, mk(Color::srgba(0.3, 0.33, 0.4, 0.45))),
        blood: pool(80, mk(Color::srgba(0.45, 0.02, 0.03, 0.8))),
    });
}

impl Tracks {
    /// Lay a flat decal at `pos` on the terrain, oriented by `yaw`, size (width, length) in metres.
    pub fn put(&mut self, commands: &mut Commands, kind: Kind, pos: Vec3, yaw: f32, size: Vec2) {
        let (mesh, pool) = match kind { Kind::Tyre => (self.quad.clone(), &mut self.tyre), Kind::Foot => (self.quad.clone(), &mut self.foot), Kind::Blood => (self.quad.clone(), &mut self.blood) };
        let n = normal_at(pos.x, pos.z);
        let rot = Quat::from_rotation_arc(Vec3::Z, n) * Quat::from_rotation_z(-yaw);        // Rectangle faces +Z: tilt onto the slope, then spin
        let t = Transform::from_translation(Vec3::new(pos.x, height_at(pos.x, pos.z) + 0.03 + 0.002 * (pool.next % 7) as f32, pos.z)).with_rotation(rot).with_scale(Vec3::new(size.x, size.y, 1.0));
        if pool.ents.len() < pool.cap {
            let e = commands.spawn((Mesh3d(mesh), MeshMaterial3d(pool.mat.clone()), t, bevy::light::NotShadowCaster, bevy::light::NotShadowReceiver)).id();
            pool.ents.push(e);
        } else {
            commands.entity(pool.ents[pool.next]).insert(t);
            pool.next = (pool.next + 1) % pool.cap;
        }
    }
}

fn car_tracks(mut commands: Commands, mut tracks: Option<ResMut<Tracks>>, cars: Query<(&GlobalTransform, &Car)>, mut last: Local<Vec3>) {
    let Some(tr) = tracks.as_mut() else { return };
    for (g, c) in &cars {
        let p = g.translation();
        if p.distance(*last) < 0.55 || c.v_long.abs() < 1.2 { continue; }
        *last = p;
        if matches!(surface_at(p.x, p.z), Surface::Gravel) { continue; }
        let yaw = c.yaw;
        let (fwd, right) = (Vec3::new(-yaw.sin(), 0.0, -yaw.cos()), Vec3::new(yaw.cos(), 0.0, -yaw.sin()));
        // slide marks are wider/darker via a second pass when the tyres break loose
        for (zo, so) in [(-1.35f32, -0.78f32), (-1.35, 0.78), (1.35, -0.78), (1.35, 0.78)] {
            let w = p + fwd * (-zo) + right * so;
            tr.put(&mut commands, Kind::Tyre, w, yaw + c.v_lat.atan2(c.v_long.abs().max(0.1)) * 0.5, Vec2::new(0.24 + 0.14 * c.slip, 0.7));
        }
    }
}

fn foot_tracks(mut commands: Commands, mut tracks: Option<ResMut<Tracks>>, player: Query<(&Transform, &Player)>, driven: Query<(), With<crate::vehicle::Driven>>, mut acc: Local<(f32, bool, Vec3)>) {
    let (Some(tr), Ok((t, p))) = (tracks.as_mut(), player.single()) else { return };
    if !driven.is_empty() || !p.on_ground { return; }
    let sp = Vec2::new(p.vel.x, p.vel.z).length();
    if sp < 0.6 { return; }
    acc.0 += t.translation.distance(acc.2); acc.2 = t.translation;
    if acc.0 < 0.75 { return; }
    acc.0 = 0.0; acc.1 = !acc.1;
    let side = if acc.1 { 1.0 } else { -1.0 };
    let right = Vec3::new(p.yaw.cos(), 0.0, -p.yaw.sin());
    tr.put(&mut commands, Kind::Foot, t.translation + right * 0.13 * side, p.yaw, Vec2::new(0.14, 0.3));
}
