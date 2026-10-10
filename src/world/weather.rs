//! Snowfall: a recycled box of flakes around the camera. `Weather.intensity` 0..1.
use bevy::prelude::*;

#[derive(Resource)]
pub struct Weather { pub intensity: f32, pub wind: Vec2 }
impl Default for Weather { fn default() -> Self { Self { intensity: 0.6, wind: Vec2::new(1.2, 0.4) } } }

#[derive(Component)]
pub struct Flake { pub idx: usize, pub phase: f32 }

pub const FLAKES: usize = 2500;
const BOX: Vec3 = Vec3::new(50.0, 28.0, 50.0);

pub fn spawn_flakes(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(Cuboid::new(0.05, 0.05, 0.05));
    let mat = mats.add(StandardMaterial { base_color: Color::WHITE, unlit: true, ..default() });
    let mut s = 12345u32;
    let mut r = || { s = s.wrapping_mul(1664525).wrapping_add(1013904223); (s >> 8) as f32 / 16_777_216.0 };
    for i in 0..FLAKES {
        let p = Vec3::new((r() - 0.5) * BOX.x, r() * BOX.y, (r() - 0.5) * BOX.z);
        commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), Transform::from_translation(p), Flake { idx: i, phase: r() * 6.28 }));
    }
}

pub fn animate_flakes(
    time: Res<Time>,
    weather: Res<Weather>,
    cam: Query<&GlobalTransform, With<Camera3d>>,
    mut q: Query<(&mut Transform, &mut Visibility, &Flake)>,
) {
    let Some(c) = cam.iter().next() else { return };
    let cp = c.translation();
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let shown = (weather.intensity.clamp(0.0, 1.0) * FLAKES as f32) as usize;
    for (mut tr, mut vis, f) in &mut q {
        *vis = if f.idx < shown { Visibility::Inherited } else { Visibility::Hidden };
        if f.idx >= shown { continue; }
        tr.translation += Vec3::new(weather.wind.x + (t * 0.8 + f.phase).sin() * 0.5, -1.8 - (f.idx % 7) as f32 * 0.15, weather.wind.y + (t * 0.6 + f.phase).cos() * 0.5) * dt;
        // wrap inside box around camera
        let rel = tr.translation - cp;
        let wrapped = Vec3::new(
            (rel.x + BOX.x / 2.0).rem_euclid(BOX.x) - BOX.x / 2.0,
            (rel.y + 6.0).rem_euclid(BOX.y) - 6.0,
            (rel.z + BOX.z / 2.0).rem_euclid(BOX.z) - BOX.z / 2.0,
        );
        tr.translation = cp + wrapped;
    }
}
