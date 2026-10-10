//! Day/night cycle: sun, ambient, sky colour, fog. Finnish winter: short day, long blue dusk, dark nights.
use super::noise::smoothstep;
use bevy::light::{GlobalAmbientLight, DirectionalLight};
use bevy::prelude::*;

#[derive(Resource)]
pub struct TimeOfDay {
    pub hour: f32,      // 0..24
    pub speed: f32,     // game hours per real second
}
impl Default for TimeOfDay {
    fn default() -> Self { Self { hour: 9.5, speed: 1.0 / 60.0 } } // 1 game hour = 1 min
}

#[derive(Component)]
pub struct Sun;

/// Sun elevation factor in -1..1 (winter: day is ~8..16h, low sun).
pub fn sun_height(hour: f32) -> f32 {
    let a = (hour - 12.0) / 12.0 * std::f32::consts::TAU / 2.0; // -pi..pi
    (a.cos() - 0.62) / 0.38 * 0.9 // >0 only ~ 8..16h
}

pub fn advance_time(time: Res<Time>, mut tod: ResMut<TimeOfDay>) {
    tod.hour = (tod.hour + time.delta_secs() * tod.speed).rem_euclid(24.0);
}

fn lerp3(a: Vec3, b: Vec3, t: f32) -> Vec3 { a + (b - a) * t }

pub struct SkyState { pub day: f32, pub sky: Color, pub ambient: Color, pub ambient_brightness: f32, pub sun_lux: f32, pub sun_color: Color }

pub fn sky_state(hour: f32) -> SkyState {
    let h = sun_height(hour);
    let day = smoothstep(-0.15, 0.25, h);
    let dusk = (1.0 - (h / 0.25).abs().min(1.0)).max(0.0) * smoothstep(-0.4, 0.0, h); // peak at horizon
    let night = Vec3::new(0.015, 0.025, 0.07);
    let dayc = Vec3::new(0.62, 0.72, 0.86);
    let dusk_c = Vec3::new(0.85, 0.50, 0.42);
    let twilight = Vec3::new(0.18, 0.24, 0.45);
    let mut sky = lerp3(night, lerp3(twilight, dayc, smoothstep(0.0, 0.5, h)), smoothstep(-0.25, 0.05, h));
    sky = lerp3(sky, dusk_c, dusk * 0.55);
    let amb = lerp3(Vec3::new(0.25, 0.30, 0.55), Vec3::new(0.75, 0.82, 0.95), day);
    SkyState {
        day,
        sky: Color::srgb(sky.x, sky.y, sky.z),
        ambient: Color::srgb(amb.x, amb.y, amb.z),
        ambient_brightness: 40.0 + 260.0 * day,
        sun_lux: 14_000.0 * smoothstep(-0.05, 0.4, h),
        sun_color: Color::srgb(1.0, 0.88 - 0.2 * dusk, 0.75 - 0.3 * dusk),
    }
}

pub fn update_sky(
    tod: Res<TimeOfDay>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sun: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut fog: Query<&mut DistanceFog>,
) {
    let st = sky_state(tod.hour);
    clear.0 = st.sky;
    ambient.color = st.ambient;
    ambient.brightness = st.ambient_brightness;
    let h = sun_height(tod.hour);
    if let Ok((mut t, mut l)) = sun.single_mut() {
        // arc east->south->west; elevation from h
        let az = (tod.hour - 12.0) / 12.0 * std::f32::consts::PI;
        let elev = h.max(-0.2) * 0.45; // radians, low winter sun
        t.rotation = Quat::from_euler(EulerRot::YXZ, az + std::f32::consts::PI, -elev.max(0.02) - 0.05, 0.0);
        l.illuminance = st.sun_lux;
        l.color = st.sun_color;
    }
    for mut f in &mut fog {
        f.color = st.sky;
        f.falloff = FogFalloff::Linear { start: 120.0, end: 520.0 };
    }
}
