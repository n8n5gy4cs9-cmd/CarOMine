//! Streamed heightmap chunks around the camera.
use super::layout::*;
use super::noise::*;
use super::scatter::scatter_chunk;
use crate::manifest::AssetManifest;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::HashMap;

pub const CHUNK: f32 = 64.0;
pub const RES: usize = 32;
pub const VIEW_CHUNKS: i32 = 7;

#[derive(Resource, Default)]
pub struct Chunks(pub HashMap<IVec2, Entity>);

#[derive(Resource)]
pub struct TerrainMaterial(pub Handle<StandardMaterial>);

#[derive(Component)]
pub struct StreamAnchor; // the thing (camera/player) chunks stream around

pub fn setup_material(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let tex: Handle<Image> = server.load_builder().with_settings(|s: &mut ImageLoaderSettings| {
        s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::linear()
        });
    }).load(man.path("tex.snow"));
    commands.insert_resource(TerrainMaterial(mats.add(StandardMaterial {
        base_color_texture: Some(tex),
        perceptual_roughness: 0.95,
        reflectance: 0.15,
        ..default()
    })));
}

pub fn build_chunk_mesh(cx: i32, cz: i32) -> Mesh {
    let n = RES + 1;
    let step = CHUNK / RES as f32;
    let (ox, oz) = (cx as f32 * CHUNK, cz as f32 * CHUNK);
    let mut pos = Vec::with_capacity(n * n);
    let mut nor = Vec::with_capacity(n * n);
    let mut uv = Vec::with_capacity(n * n);
    let mut col: Vec<[f32; 4]> = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            let (x, z) = (ox + i as f32 * step, oz + j as f32 * step);
            let y = height_at(x, z);
            pos.push([x, y, z]);
            nor.push(normal_at(x, z).to_array());
            uv.push([x / 6.0, z / 6.0]);
            let v = 0.93 + 0.07 * value_noise(x / 5.0, z / 5.0, SEED + 11);
            let c = match surface_at(x, z) {
                Surface::Snow => [v * 0.97, v, v * 1.03, 1.0],
                Surface::PackedSnow => [0.62, 0.64, 0.70, 1.0],
                Surface::Ice => [0.55, 0.72, 0.9, 1.0],
                Surface::Gravel => [0.5, 0.48, 0.45, 1.0],
            };
            // blend road edge for a softer look
            col.push(c);
        }
    }
    let mut idx = Vec::with_capacity(RES * RES * 6);
    for j in 0..RES {
        for i in 0..RES {
            let a = (j * n + i) as u32;
            let (b, c, d) = (a + 1, a + n as u32, a + n as u32 + 1);
            idx.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nor)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
        .with_inserted_indices(Indices::U32(idx))
}

pub fn stream_chunks(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mat: Option<Res<TerrainMaterial>>,
    anchor: Query<&GlobalTransform, With<StreamAnchor>>,
    scatter: Option<Res<super::scatter::ScatterAssets>>,
) {
    let (Some(mat), Some(scatter)) = (mat, scatter) else { return };
    let Ok(t) = anchor.single() else { return };
    let p = t.translation();
    let c = IVec2::new((p.x / CHUNK).floor() as i32, (p.z / CHUNK).floor() as i32);
    // despawn far
    let far: Vec<IVec2> = chunks.0.keys().filter(|k| (k.x - c.x).abs() > VIEW_CHUNKS + 1 || (k.y - c.y).abs() > VIEW_CHUNKS + 1).cloned().collect();
    for k in far {
        if let Some(e) = chunks.0.remove(&k) { commands.entity(e).despawn(); }
    }
    // spawn nearest missing first (limit per frame)
    let mut want: Vec<IVec2> = Vec::new();
    for dz in -VIEW_CHUNKS..=VIEW_CHUNKS {
        for dx in -VIEW_CHUNKS..=VIEW_CHUNKS {
            let k = c + IVec2::new(dx, dz);
            if !chunks.0.contains_key(&k) && ((k.x.abs() as f32) * CHUNK) < WORLD_HALF && ((k.y.abs() as f32) * CHUNK) < WORLD_HALF { want.push(k); }
        }
    }
    want.sort_by_key(|k| (k.x - c.x).pow(2) + (k.y - c.y).pow(2));
    for k in want.into_iter().take(4) {
        let e = commands
            .spawn((Mesh3d(meshes.add(build_chunk_mesh(k.x, k.y))), MeshMaterial3d(mat.0.clone()), Transform::default(), Visibility::default()))
            .id();
        scatter_chunk(&mut commands, e, k, &scatter);
        chunks.0.insert(k, e);
    }
}
