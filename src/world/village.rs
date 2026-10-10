//! Buildings (3D near-field; far-LOD facade billboards later) + village layout.
use super::layout::*;
use super::scatter::Obstacle;
use crate::manifest::AssetManifest;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

#[derive(Resource)]
pub struct BuildingAssets {
    wall_red: Handle<StandardMaterial>,
    wall_wood: Handle<StandardMaterial>,
    wall_cream: Handle<StandardMaterial>,
    wall_brick: Handle<StandardMaterial>,
    wall_yellow: Handle<StandardMaterial>,
    roof: Handle<StandardMaterial>,
    snow: Handle<StandardMaterial>,
    door: Handle<StandardMaterial>,
    pub window: Handle<StandardMaterial>,
    floor: Handle<StandardMaterial>,
    unit_box: Handle<Mesh>,
    window_quad: Handle<Mesh>,
}

#[derive(Clone, Copy)]
pub enum Wall { Red, Wood, Cream, Brick, Yellow }

pub fn setup(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let tex = |id: &str| -> Handle<Image> {
        server.load_builder().with_settings(|s: &mut ImageLoaderSettings| {
            s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor { address_mode_u: ImageAddressMode::Repeat, address_mode_v: ImageAddressMode::Repeat, ..ImageSamplerDescriptor::linear() });
        }).load(man.path(id))
    };
    let mut m = |t: &str, tint: Color| mats.add(StandardMaterial { base_color_texture: Some(tex(t)), base_color: tint, perceptual_roughness: 0.9, reflectance: 0.1, ..default() });
    let wall_red = m("tex.wood_red", Color::WHITE);
    let wall_wood = m("tex.wood", Color::WHITE);
    let wall_cream = m("tex.wood", Color::srgb(1.6, 1.55, 1.3));
    let wall_brick = m("tex.brick", Color::WHITE);
    let wall_yellow = m("tex.wood", Color::srgb(1.7, 1.4, 0.7));
    let roof = m("tex.roof_dark", Color::WHITE);
    let floor = m("tex.floor_wood", Color::WHITE);
    let snow = mats.add(StandardMaterial { base_color: Color::srgb(0.95, 0.97, 1.0), perceptual_roughness: 1.0, ..default() });
    let door = mats.add(StandardMaterial { base_color: Color::srgb(0.25, 0.17, 0.1), ..default() });
    let window = mats.add(StandardMaterial { base_color: Color::srgb(0.25, 0.35, 0.45), emissive: LinearRgba::BLACK, perceptual_roughness: 0.2, ..default() });
    commands.insert_resource(BuildingAssets { wall_red, wall_wood, wall_cream, wall_brick, wall_yellow, roof, snow, door, window, floor, unit_box: meshes.add(Cuboid::new(1.0, 1.0, 1.0)), window_quad: meshes.add(Rectangle::new(1.0, 1.0)) });
}

/// Gable roof, ridge along X. w = length (x), d = depth (z), h = ridge height, flat normals.
fn gable(w: f32, d: f32, h: f32) -> Mesh {
    let (hx, hz) = (w / 2.0, d / 2.0);
    let a = Vec3::new(-hx, 0.0, -hz); let b = Vec3::new(hx, 0.0, -hz); let c = Vec3::new(hx, 0.0, hz); let dd = Vec3::new(-hx, 0.0, hz);
    let r0 = Vec3::new(-hx, h, 0.0); let r1 = Vec3::new(hx, h, 0.0);
    let mut pos: Vec<[f32; 3]> = vec![]; let mut nor: Vec<[f32; 3]> = vec![]; let mut uv: Vec<[f32; 2]> = vec![]; let mut idx: Vec<u32> = vec![];
    let mut tri = |p: [Vec3; 3], uvs: [[f32; 2]; 3]| {
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let base = pos.len() as u32;
        for (i, v) in p.iter().enumerate() { pos.push(v.to_array()); nor.push(n.to_array()); uv.push(uvs[i]); }
        idx.extend_from_slice(&[base, base + 1, base + 2]);
    };
    // south slope (z>0)
    tri([dd, c, r1], [[0.0, 0.0], [w / 3.0, 0.0], [w / 3.0, 1.5]]); tri([dd, r1, r0], [[0.0, 0.0], [w / 3.0, 1.5], [0.0, 1.5]]);
    // north slope
    tri([b, a, r0], [[0.0, 0.0], [w / 3.0, 0.0], [w / 3.0, 1.5]]); tri([b, r0, r1], [[0.0, 0.0], [w / 3.0, 1.5], [0.0, 1.5]]);
    // gables
    tri([c, b, r1], [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]); tri([a, dd, r0], [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]);
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos).with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nor)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv).with_inserted_indices(Indices::U32(idx))
}

pub struct HouseSpec { pub pos: Vec2, pub yaw: f32, pub w: f32, pub d: f32, pub h: f32, pub wall: Wall, pub windows: u32, pub door: bool, pub open_front: bool }

pub fn spawn_house(commands: &mut Commands, a: &BuildingAssets, meshes: &mut Assets<Mesh>, mats: &mut Assets<StandardMaterial>, s: &HouseSpec) {
    let wall_mat = match s.wall { Wall::Red => &a.wall_red, Wall::Wood => &a.wall_wood, Wall::Cream => &a.wall_cream, Wall::Brick => &a.wall_brick, Wall::Yellow => &a.wall_yellow };
    // per-building material copy so texture tiles at a sensible rate
    let mut wm = mats.get(wall_mat).cloned().unwrap_or_default();
    wm.uv_transform = bevy::math::Affine2::from_scale(Vec2::new(s.w.max(s.d) / 3.0, s.h / 3.0));
    let wall_mat = mats.add(wm);
    let ground = height_at(s.pos.x, s.pos.y);
    let rot = Quat::from_rotation_y(s.yaw);
    let roof_h = s.d * 0.32;
    commands.spawn((Transform::from_xyz(s.pos.x, ground, s.pos.y).with_rotation(rot), Visibility::default())).with_children(|c| {
        let t = 0.25; // wall thickness
        if s.open_front {
            // back wall (-x), side walls, no front (+x)
            c.spawn((Mesh3d(a.unit_box.clone()), MeshMaterial3d(wall_mat.clone()), Transform::from_xyz(-s.w / 2.0 + t / 2.0, s.h / 2.0, 0.0).with_scale(Vec3::new(t, s.h, s.d))));
            for sz in [-1.0f32, 1.0] {
                c.spawn((Mesh3d(a.unit_box.clone()), MeshMaterial3d(wall_mat.clone()), Transform::from_xyz(0.0, s.h / 2.0, sz * (s.d / 2.0 - t / 2.0)).with_scale(Vec3::new(s.w, s.h, t))));
            }
            c.spawn((Mesh3d(a.unit_box.clone()), MeshMaterial3d(a.floor.clone()), Transform::from_xyz(0.0, 0.05, 0.0).with_scale(Vec3::new(s.w, 0.1, s.d))));
        } else {
            c.spawn((Mesh3d(a.unit_box.clone()), MeshMaterial3d(wall_mat.clone()), Transform::from_xyz(0.0, s.h / 2.0, 0.0).with_scale(Vec3::new(s.w, s.h, s.d))));
        }
        // roof + snow cap
        let roof_mesh = meshes.add(gable(s.w + 0.8, s.d + 0.8, roof_h));
        c.spawn((Mesh3d(roof_mesh), MeshMaterial3d(a.roof.clone()), Transform::from_xyz(0.0, s.h, 0.0)));
        let snow_mesh = meshes.add(gable(s.w + 0.9, s.d + 0.9, roof_h + 0.12));
        c.spawn((Mesh3d(snow_mesh), MeshMaterial3d(a.snow.clone()), Transform::from_xyz(0.0, s.h + 0.02, 0.0)));
        if !s.open_front {
            // windows on both long faces (front = +z, back = -z) ; door on +z centre
            let n = s.windows.max(1);
            for side in [1.0f32, -1.0] {
                for i in 0..n {
                    let x = (i as f32 + 0.5) / n as f32 * s.w - s.w / 2.0;
                    if s.door && side > 0.0 && x.abs() < 1.1 { continue; }
                    c.spawn((Mesh3d(a.window_quad.clone()), MeshMaterial3d(a.window.clone()),
                        Transform::from_xyz(x, s.h * 0.58, side * (s.d / 2.0 + 0.02)).with_rotation(if side > 0.0 { Quat::IDENTITY } else { Quat::from_rotation_y(std::f32::consts::PI) }).with_scale(Vec3::new(1.0, 1.1, 1.0))));
                }
            }
            if s.door {
                c.spawn((Mesh3d(a.window_quad.clone()), MeshMaterial3d(a.door.clone()), Transform::from_xyz(0.0, 1.05, s.d / 2.0 + 0.03).with_scale(Vec3::new(1.1, 2.1, 1.0))));
            }
        }
    });
    // colliders: circles along the footprint perimeter
    let rotm = |v: Vec2| Vec2::new(v.x * s.yaw.cos() + v.y * s.yaw.sin(), -v.x * s.yaw.sin() + v.y * s.yaw.cos());
    let step = 1.8;
    let mut add = |lx: f32, lz: f32, commands: &mut Commands| {
        let w = s.pos + rotm(Vec2::new(lx, lz));
        commands.spawn((Transform::from_xyz(w.x, ground, w.y), GlobalTransform::default(), Obstacle(1.1)));
    };
    let nx = (s.w / step).ceil() as i32; let nz = (s.d / step).ceil() as i32;
    for i in 0..=nx {
        let x = -s.w / 2.0 + s.w * i as f32 / nx as f32;
        if !s.open_front || x < -s.w / 2.0 + 1.0 || true { if !(s.open_front && x > -s.w / 2.0 + 0.5 && false) { add(x, s.d / 2.0, commands); add(x, -s.d / 2.0, commands); } }
    }
    for j in 1..nz {
        let z = -s.d / 2.0 + s.d * j as f32 / nz as f32;
        add(-s.w / 2.0, z, commands);
        if !s.open_front { add(s.w / 2.0, z, commands); }
    }
}

pub fn spawn_village(mut commands: Commands, a: Option<Res<BuildingAssets>>, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>, mut done: Local<bool>) {
    let Some(a) = a else { return };
    if *done { return; }
    *done = true;
    use Wall::*;
    let yaw_to_road = |p: Vec2| { let rx = road_x(p.y); if rx > p.x { -std::f32::consts::FRAC_PI_2 } else { std::f32::consts::FRAC_PI_2 } };
    let _ = yaw_to_road;
    let mut h = |commands: &mut Commands, x: f32, z: f32, yaw: f32, w: f32, d: f32, hh: f32, wall: Wall, win: u32, door: bool, open: bool| {
        spawn_house(commands, &a, &mut meshes, &mut mats, &HouseSpec { pos: Vec2::new(x, z), yaw, w, d, h: hh, wall, windows: win, door, open_front: open });
    };
    // --- home: cabin, sauna, garage (open shed; its open side faces +x i.e. yaw 0 means open front toward +x)
    h(&mut commands, CABIN.x, CABIN.y, 0.0, 7.0, 5.0, 2.6, Red, 3, true, false);
    h(&mut commands, CABIN.x + 9.0, CABIN.y - 4.0, 0.0, 4.0, 3.4, 2.3, Wood, 1, true, false);
    h(&mut commands, GARAGE.x, GARAGE.y, 0.0, 9.0, 7.0, 3.0, Red, 0, false, true);
    // --- village along the road: K-market, church, gas station-ish shed, houses
    let v = VILLAGE;
    h(&mut commands, v.x + 22.0, v.y, -std::f32::consts::FRAC_PI_2, 14.0, 9.0, 3.8, Cream, 5, true, false); // K-market
    h(&mut commands, v.x - 22.0, v.y + 6.0, std::f32::consts::FRAC_PI_2, 8.0, 6.0, 4.5, Cream, 2, true, false); // church hall
    h(&mut commands, v.x + 24.0, v.y + 22.0, -std::f32::consts::FRAC_PI_2, 7.0, 6.0, 2.8, Yellow, 3, true, false);
    h(&mut commands, v.x - 24.0, v.y - 18.0, std::f32::consts::FRAC_PI_2, 7.0, 5.5, 2.8, Red, 3, true, false);
    h(&mut commands, v.x + 25.0, v.y - 22.0, -std::f32::consts::FRAC_PI_2, 8.0, 5.5, 2.8, Wood, 3, true, false);
    h(&mut commands, v.x - 26.0, v.y + 28.0, std::f32::consts::FRAC_PI_2, 6.0, 5.0, 2.7, Yellow, 2, true, false);
    h(&mut commands, v.x + 18.0, v.y + 42.0, -std::f32::consts::FRAC_PI_2, 9.0, 6.0, 3.0, Brick, 4, true, false); // auto parts shop
    h(&mut commands, v.x - 24.0, v.y - 42.0, std::f32::consts::FRAC_PI_2, 12.0, 7.0, 3.4, Brick, 3, true, false); // katsastus (inspection station)
}

/// Windows glow warm at night.
pub fn window_glow(tod: Res<super::sky::TimeOfDay>, a: Option<Res<BuildingAssets>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let Some(a) = a else { return };
    let night = 1.0 - super::sky::sky_state(tod.hour).day;
    if let Some(mut m) = mats.get_mut(&a.window) { m.emissive = LinearRgba::new(3.0, 2.1, 0.9, 1.0) * night; }
}
