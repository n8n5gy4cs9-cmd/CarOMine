//! Build-engine style sprites: camera-facing (Y-axis) quads, alpha-masked, optional 8-angle sheets.
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

/// Y-axis billboard. Rotates to face the active 3D camera.
#[derive(Component)]
pub struct Billboard;

/// 8-angle sheet sprite: picks UV cell by view angle relative to `facing` (yaw radians of the entity's forward).
#[derive(Component)]
pub struct AngleSprite {
    pub cols: u32,
    pub rows: u32, // == 8 angles
    pub col: u32,  // current animation column
    pub facing: f32,
}

pub fn load_nearest(server: &AssetServer, path: String) -> Handle<Image> {
    server.load_builder().with_settings(|s: &mut ImageLoaderSettings| s.sampler = ImageSampler::nearest()).load(path)
}

pub fn sprite_material(tex: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: Some(tex),
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
    }
}

/// Quad with bottom-centre pivot (origin at feet).
pub fn feet_quad(w: f32, h: f32) -> Mesh {
    Mesh::from(Rectangle::new(w, h)).translated_by(Vec3::new(0.0, h / 2.0, 0.0))
}

pub fn face_camera(cam: Query<&GlobalTransform, With<Camera3d>>, mut q: Query<(&GlobalTransform, &mut Transform), With<Billboard>>) {
    let Some(c) = cam.iter().next() else { return };
    let cp = c.translation();
    for (gt, mut t) in &mut q {
        let d = cp - gt.translation();
        t.rotation = Quat::from_rotation_y(d.x.atan2(d.z));
    }
}

/// Select the sheet cell for 8-angle sprites each frame (uses per-entity material uv_transform).
pub fn update_angle_sprites(
    cam: Query<&GlobalTransform, With<Camera3d>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    q: Query<(&GlobalTransform, &AngleSprite, &MeshMaterial3d<StandardMaterial>)>,
) {
    let Some(c) = cam.iter().next() else { return };
    let cp = c.translation();
    for (gt, sp, mh) in &q {
        let to_cam = cp - gt.translation();
        let view = to_cam.x.atan2(to_cam.z); // world yaw pointing at camera
        let rel = (view - sp.facing).rem_euclid(std::f32::consts::TAU);
        let row = ((rel / (std::f32::consts::TAU / 8.0)).round() as u32) % 8;
        if let Some(mut m) = mats.get_mut(&mh.0) {
            let (sx, sy) = (1.0 / sp.cols as f32, 1.0 / sp.rows as f32);
            m.uv_transform = bevy::math::Affine2::from_scale_angle_translation(Vec2::new(sx, sy), 0.0, Vec2::new(sp.col as f32 * sx, row as f32 * sy));
        }
    }
}
