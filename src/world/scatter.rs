//! Deterministic trees / bushes / rocks per chunk (billboards).
use super::billboard::*;
use super::layout::*;
use super::noise::*;
use super::terrain::CHUNK;
use crate::manifest::AssetManifest;
use bevy::prelude::*;

pub struct Kind {
    pub mesh: Handle<Mesh>,
    pub mats: Vec<Handle<StandardMaterial>>,
    pub radius: f32,
}

#[derive(Resource)]
pub struct ScatterAssets {
    pub spruce: Kind, pub pine: Kind, pub birch: Kind, pub bare: Kind,
    pub bush: Kind, pub rock: Kind, pub fence: Kind, pub lamp: Kind,
}

/// Obstacle for player/vehicle collision (cylinder in XZ).
#[derive(Component)]
pub struct Obstacle(pub f32);

pub fn setup_scatter(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let mut kind = |id: &str, n: usize, w: f32, h: f32, radius: f32| -> Kind {
        let mesh = meshes.add(feet_quad(w, h));
        let m = (0..n).map(|v| mats.add(sprite_material(load_nearest(&server, man.path(&format!("{id}.{v}")))))).collect();
        Kind { mesh, mats: m, radius }
    };
    commands.insert_resource(ScatterAssets {
        spruce: kind("tree.spruce", 3, 5.0, 8.75, 0.45),
        pine: kind("tree.pine", 2, 5.0, 8.75, 0.35),
        birch: kind("tree.birch", 2, 4.0, 7.0, 0.3),
        bare: kind("tree.bare", 2, 4.0, 6.0, 0.3),
        bush: kind("bush", 3, 1.9, 1.3, 0.7),
        rock: kind("rock", 2, 1.9, 1.3, 0.8),
        fence: kind("fence", 1, 2.4, 1.6, 0.0),
        lamp: kind("streetlamp", 1, 1.2, 4.0, 0.15),
    });
}

fn put(commands: &mut Commands, parent: Entity, k: &Kind, variant: u32, pos: Vec3, scale: f32) {
    let v = (variant as usize) % k.mats.len();
    let e = commands
        .spawn((
            Mesh3d(k.mesh.clone()),
            MeshMaterial3d(k.mats[v].clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
            Billboard,
        ))
        .id();
    if k.radius > 0.0 { commands.entity(e).insert(Obstacle(k.radius * scale)); }
    commands.entity(parent).add_child(e);
}

pub fn scatter_chunk(commands: &mut Commands, parent: Entity, c: IVec2, a: &ScatterAssets) {
    let cell = 4.5;
    let n = (CHUNK / cell) as i32;
    let (ox, oz) = (c.x as f32 * CHUNK, c.y as f32 * CHUNK);
    for j in 0..n {
        for i in 0..n {
            let gi = c.x * n + i;
            let gj = c.y * n + j;
            let x = ox + (i as f32 + hash2(gi, gj, 1)) * cell;
            let z = oz + (j as f32 + hash2(gi, gj, 2)) * cell;
            if road_dist(x, z) < ROAD_HALF_W + 3.0 || lake_dist(x, z) < 6.0 { continue; }
            if Vec2::new(x, z).distance(CABIN) < 22.0 || Vec2::new(x, z).distance(VILLAGE) < 55.0 { continue; }
            let forest = fbm(x / 140.0, z / 140.0, SEED + 21, 3);
            let r = hash2(gi, gj, 3);
            let y = height_at(x, z);
            let pos = Vec3::new(x, y, z);
            let sc = 0.8 + 0.5 * hash2(gi, gj, 4);
            let var = (hash2(gi, gj, 5) * 8.0) as u32;
            let dens = if forest > 0.5 { 0.75 } else if forest > 0.42 { 0.25 } else { 0.04 };
            if r < dens {
                let t = hash2(gi, gj, 6);
                let k = if t < 0.55 { &a.spruce } else if t < 0.78 { &a.pine } else if t < 0.93 { &a.birch } else { &a.bare };
                put(commands, parent, k, var, pos, sc);
            } else if r < dens + 0.10 {
                put(commands, parent, &a.bush, var, pos, 0.8 + 0.6 * hash2(gi, gj, 7));
            } else if r < dens + 0.115 {
                put(commands, parent, &a.rock, var, pos, 0.8 + hash2(gi, gj, 8));
            }
        }
    }
}
