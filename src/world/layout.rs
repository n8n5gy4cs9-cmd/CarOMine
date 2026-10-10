//! Fixed landmarks + terrain function. World is 8x8 km centred on (0,0); Nurmijarvi village near origin.
use super::noise::*;
use bevy::prelude::*;

pub const SEED: u32 = 0xC4_20_26;
pub const WORLD_HALF: f32 = 4000.0;
pub const CABIN: Vec2 = Vec2::new(60.0, 40.0);
pub const VILLAGE: Vec2 = Vec2::new(-20.0, -180.0);
pub const LAKE_C: Vec2 = Vec2::new(380.0, 120.0);
pub const LAKE_R: f32 = 140.0;
pub const ROAD_HALF_W: f32 = 3.6;

/// Centreline of the main road (runs north-south, wanders).
pub fn road_x(z: f32) -> f32 {
    -30.0 + 45.0 * (z / 260.0).sin() + 18.0 * (z / 90.0).sin()
}
/// Short spur from the road to the cabin yard.
pub fn spur_dist(x: f32, z: f32) -> f32 {
    // segment from (road_x(40), 40) to CABIN
    let a = Vec2::new(road_x(CABIN.y), CABIN.y);
    let b = CABIN;
    let p = Vec2::new(x, z);
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}
pub fn road_dist(x: f32, z: f32) -> f32 {
    // distance to the sinuous road: approximate via horizontal offset scaled by slope
    let dz = 0.5;
    let slope = (road_x(z + dz) - road_x(z - dz)) / (2.0 * dz);
    let d_main = (x - road_x(z)).abs() / (1.0 + slope * slope).sqrt();
    d_main.min(spur_dist(x, z))
}
pub fn lake_dist(x: f32, z: f32) -> f32 {
    // wobbly shoreline
    let ang = (z - LAKE_C.y).atan2(x - LAKE_C.x);
    let r = LAKE_R * (1.0 + 0.25 * (ang * 3.0).sin() + 0.12 * (ang * 5.0 + 1.0).cos());
    Vec2::new(x - LAKE_C.x, z - LAKE_C.y).length() - r
}

fn base_height(x: f32, z: f32) -> f32 {
    let hills = fbm(x / 520.0, z / 520.0, SEED, 4);
    let bumps = fbm(x / 70.0, z / 70.0, SEED + 7, 3);
    4.0 + (hills - 0.5) * 38.0 + (bumps - 0.5) * 2.2
}

/// Terrain height in metres at world (x,z).
pub fn height_at(x: f32, z: f32) -> f32 {
    let mut h = base_height(x, z);
    // flatten for road
    let rd = road_dist(x, z);
    if rd < 16.0 {
        let rc = base_height(road_x(z), z) * 0.9 + 0.2;
        h += (rc - h) * (1.0 - smoothstep(ROAD_HALF_W, 16.0, rd));
    }
    // flatten yard around cabin and village
    for (c, r) in [(CABIN, 26.0), (VILLAGE, 60.0)] {
        let d = Vec2::new(x, z).distance(c);
        if d < r + 20.0 {
            let ch = base_height(c.x, c.y) * 0.9 + 0.2;
            h += (ch - h) * (1.0 - smoothstep(r - 6.0, r + 20.0, d));
        }
    }
    // lake basin
    let ld = lake_dist(x, z);
    if ld < 25.0 {
        let lh = 0.3;
        h = h + (lh - h) * (1.0 - smoothstep(-5.0, 25.0, ld)).max(0.0);
    }
    h
}

pub fn height_vec(p: Vec3) -> f32 { height_at(p.x, p.z) }

pub fn normal_at(x: f32, z: f32) -> Vec3 {
    let e = 0.6;
    let dx = height_at(x + e, z) - height_at(x - e, z);
    let dz = height_at(x, z + e) - height_at(x, z - e);
    Vec3::new(-dx, 2.0 * e, -dz).normalize()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Surface { Snow, PackedSnow, Ice, Gravel }

/// Ground type for friction / sound / tracks.
pub fn surface_at(x: f32, z: f32) -> Surface {
    if lake_dist(x, z) < -2.0 { return Surface::Ice; }
    let rd = road_dist(x, z);
    if rd < ROAD_HALF_W { return if fbm(x / 9.0, z / 9.0, SEED + 3, 2) > 0.62 { Surface::Ice } else { Surface::PackedSnow }; }
    Surface::Snow
}

/// Open-front garage shed next to the cabin: car building happens here.
pub const GARAGE: Vec2 = Vec2::new(CABIN.x - 26.0, CABIN.y + 4.0);
