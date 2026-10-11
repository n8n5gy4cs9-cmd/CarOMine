//! Instrument cluster as real 3D geometry on the dash (driver view): tacho, speedo, fuel, temperature, oil, volts, warning lights, clock/outside temp/odometer.
//! Art: assets/sprites/ui/dash_*.png (tools/gen_dash.py). Needles rotate about their centre; values are smoothed like real analog gauges.
use super::*;
use crate::build::{CarParts, Slot};
use crate::manifest::AssetManifest;
use crate::world::sky::TimeOfDay;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NeedleKind { Tacho, Speedo, Fuel, Temp, Oil, Volt }

#[derive(Component)] pub struct Needle { pub kind: NeedleKind, pub value: f32 }
#[derive(Component)] pub struct WarnIcon(pub usize);
#[derive(Component)] pub struct DialFace;
#[derive(Component)] pub struct DashText;
/// Marks a car whose 3D instrument cluster has been built.
#[derive(Component)] pub struct Dash3d;

/// Cluster placement in car space: behind the steering wheel, tilted up towards the driver's eye.
const CLUSTER_POS: Vec3 = Vec3::new(-0.38, 1.17, -0.72);
const CLUSTER_TILT: f32 = -0.30;
const S: f32 = 0.44 / 648.0; // metres per original art pixel (plate is 648x232 px)

/// Local position of a rect given in plate pixels (top-left origin) on the plate.
fn at(x: f32, y: f32, w: f32, h: f32) -> Vec3 { Vec3::new((x + w / 2.0 - 324.0) * S, (116.0 - (y + h / 2.0)) * S, 0.0) }

/// Small on-screen digital readout (clock, outside temp, gear, odometer) - the rest of the cluster is real 3D geometry on the dash.
pub fn spawn(mut commands: Commands, existing: Query<(), With<DashText>>) {
    if !existing.is_empty() { return; }
    commands.spawn((DashText, Text::new(""), TextFont { font_size: FontSize::Px(13.0), ..default() }, TextColor(Color::srgb(1.0, 0.72, 0.3)),
        Node { position_type: PositionType::Absolute, left: Val::Px(8.0), bottom: Val::Px(6.0), ..default() }, Visibility::Hidden, GlobalZIndex(6)));
}

/// Build the 3D cluster (plate, 6 dials, needles, 9 warning lights) as children of each car.
pub fn attach(
    mut commands: Commands,
    server: Res<AssetServer>,
    man: Res<AssetManifest>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    cars: Query<Entity, (With<Car>, Without<Dash3d>)>,
) {
    for car in &cars {
        let nearest = |id: &str| crate::world::billboard::load_nearest(&server, man.path(id));
        let unlit = |tex: Handle<Image>| StandardMaterial { base_color_texture: Some(tex), unlit: true, alpha_mode: AlphaMode::Mask(0.5), cull_mode: None, double_sided: true, ..default() };
        let (big, small, icons) = (nearest("ui.dash_needle"), nearest("ui.dash_needle_small"), nearest("ui.dash_icons"));
        let dials = [("ui.dash_tacho", 8.0, 10.0, 204.0, NeedleKind::Tacho), ("ui.dash_speedo", 220.0, 10.0, 204.0, NeedleKind::Speedo),
                     ("ui.dash_fuel", 436.0, 10.0, 96.0, NeedleKind::Fuel), ("ui.dash_temp", 540.0, 10.0, 96.0, NeedleKind::Temp),
                     ("ui.dash_oil", 436.0, 110.0, 96.0, NeedleKind::Oil), ("ui.dash_volt", 540.0, 110.0, 96.0, NeedleKind::Volt)];
        let plate = meshes.add(Cuboid::new(648.0 * S + 0.02, 232.0 * S + 0.02, 0.02));
        let plate_mat = mats.add(StandardMaterial { base_color: Color::srgb(0.07, 0.065, 0.06), perceptual_roughness: 0.9, ..default() });
        let (mut face_mats, mut needle_mats) = (Vec::new(), Vec::new());
        for (id, ..) in dials.iter() { face_mats.push(mats.add(unlit(nearest(id)))); }
        for (_, _, _, s, _) in dials.iter() { needle_mats.push(mats.add(unlit(if *s > 150.0 { big.clone() } else { small.clone() }))); }
        let icon_quad = meshes.add(Rectangle::new(32.0 * S, 32.0 * S));
        commands.entity(car).insert(Dash3d).with_children(|c| {
            c.spawn((Transform::from_translation(CLUSTER_POS).with_rotation(Quat::from_rotation_x(CLUSTER_TILT)), Visibility::default())).with_children(|r| {
                r.spawn((Mesh3d(plate), MeshMaterial3d(plate_mat), Transform::from_xyz(0.0, 0.0, -0.011)));
                for (i, (_, x, y, s, kind)) in dials.iter().enumerate() {
                    let q = meshes.add(Rectangle::new(s * S, s * S));
                    let p = at(*x, *y, *s, *s);
                    r.spawn((DialFace, Mesh3d(q.clone()), MeshMaterial3d(face_mats[i].clone()), Transform::from_translation(p)));
                    r.spawn((Needle { kind: *kind, value: 0.0 }, Mesh3d(q), MeshMaterial3d(needle_mats[i].clone()), Transform::from_translation(p + Vec3::Z * 0.003)));
                }
                for i in 0..9 {
                    let mut m = unlit(icons.clone());
                    m.uv_transform = bevy::math::Affine2::from_scale_angle_translation(Vec2::new(1.0 / 9.0, 1.0), 0.0, Vec2::new(i as f32 / 9.0, 0.0));
                    m.base_color = Color::srgb(0.17, 0.17, 0.19);
                    r.spawn((WarnIcon(i), Mesh3d(icon_quad.clone()), MeshMaterial3d(mats.add(m)), Transform::from_translation(at(150.0 + i as f32 * 40.0, -34.0, 32.0, 32.0) + Vec3::Z * 0.002)));
                }
            });
        });
    }
}

/// Needle angle (degrees clockwise from straight up) for a gauge value.
fn angle(kind: NeedleKind, v: f32) -> f32 {
    match kind {
        NeedleKind::Tacho => -135.0 + 270.0 * (v / 7000.0).clamp(0.0, 1.03),
        NeedleKind::Speedo => -135.0 + 270.0 * (v / 180.0).clamp(0.0, 1.03),
        NeedleKind::Fuel | NeedleKind::Oil => -45.0 + 90.0 * v.clamp(0.0, 1.0),
        NeedleKind::Temp => -45.0 + 90.0 * ((v - 40.0) / 80.0).clamp(0.0, 1.0),
        NeedleKind::Volt => -45.0 + 90.0 * ((v - 10.0) / 6.0).clamp(0.0, 1.0),
    }
}

pub fn update(
    time: Res<Time>,
    tod: Res<TimeOfDay>,
    chase: Res<ChaseCam>,
    car: Query<(&Car, &CarParts), With<Driven>>,
    mut needles: Query<(&mut Needle, &mut Transform)>,
    icons: Query<(&WarnIcon, &MeshMaterial3d<StandardMaterial>)>,
    faces: Query<&MeshMaterial3d<StandardMaterial>, Or<(With<DialFace>, With<Needle>)>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut text: Query<(&mut Text, &mut Visibility), With<DashText>>,
) {
    let Ok((c, p)) = car.single() else { if let Ok((_, mut v)) = text.single_mut() { *v = Visibility::Hidden; } return };
    let dt = time.delta_secs().min(0.05);
    let electrics = c.electrics;
    let charging = c.running && p.ok(Slot::Alternator, 0.1) && !p.belt_snapped;
    let volts = if !p.get(Slot::Battery).installed { 10.0 } else { 11.2 + 1.1 * p.battery + if charging { 1.6 } else { 0.0 } };
    // dials are backlit when the lights are on / dim when the electrics are dead
    let lvl = if !electrics { 0.28 } else if c.lights { 1.0 } else { 0.85 };
    for h in &faces { if let Some(mut m) = mats.get_mut(&h.0) { m.base_color = Color::srgb(lvl, lvl, lvl); } }
    for (mut n, mut tr) in &mut needles {
        let target = match n.kind {
            NeedleKind::Tacho => if c.running { c.rpm } else { 0.0 },
            NeedleKind::Speedo => c.speed_kmh(),
            NeedleKind::Fuel => if electrics { c.fuel / c.spec.fuel_cap } else { 0.0 },
            NeedleKind::Temp => if electrics { p.temp } else { 40.0 },
            NeedleKind::Oil => if c.running { c.oil } else { 0.0 },
            NeedleKind::Volt => if electrics { volts } else { 10.0 },
        };
        // analog inertia (+ a little jitter on the tacho when idling rough)
        let k = if n.kind == NeedleKind::Speedo { 6.0 } else { 9.0 };
        n.value += (target - n.value) * (k * dt).min(1.0);
        let jitter = if n.kind == NeedleKind::Tacho && c.running { (time.elapsed_secs() * 31.0).sin() * 18.0 } else { 0.0 };
        tr.rotation = Quat::from_rotation_z(-angle(n.kind, n.value + jitter).to_radians()); // clockwise as seen by the driver
    }
    let off = Color::srgb(0.17, 0.17, 0.19);
    let (red, amber, green, blue) = (Color::srgb(1.0, 0.18, 0.15), Color::srgb(1.0, 0.68, 0.1), Color::srgb(0.25, 0.95, 0.4), Color::srgb(0.4, 0.65, 1.0));
    let key_on_engine_off = electrics && !c.running;
    let check = p.ok(Slot::Engine, 0.0) && (p.get(Slot::SparkPlugs).cond < 0.35 || p.get(Slot::FuelFilter).cond < 0.3 || p.get(Slot::Engine).cond < 0.4);
    for (w, h) in &icons {
        let on = match w.0 {
            0 => (c.warn_batt || key_on_engine_off || (electrics && volts < 12.0)).then_some(red),
            1 => ((c.running && c.oil < 0.25) || key_on_engine_off).then_some(red),
            2 => (electrics && check).then_some(amber),
            3 => (electrics && (c.handbrake || p.get(Slot::Brakes).cond < 0.2)).then_some(red),
            4 => (electrics && (c.warn_temp || p.temp > 100.0)).then_some(red),
            5 => (electrics && c.fuel < 8.0).then_some(amber),
            6 => (electrics && c.lights).then_some(green),
            7 => (electrics && p.ambient < 3.5).then_some(blue),
            8 => (electrics && c.wipers > 0).then_some(green),
            _ => None,
        };
        if let Some(mut m) = mats.get_mut(&h.0) { m.base_color = on.unwrap_or(off); }
    }
    if let Ok((mut t, mut v)) = text.single_mut() {
        *v = if chase.0 { Visibility::Hidden } else { Visibility::Inherited };
        let (hh, mm) = (tod.hour as u32, (tod.hour.fract() * 60.0) as u32);
        let gear = match c.gear { -1 => "R".to_string(), 0 => "N".to_string(), g => g.to_string() };
        t.0 = if electrics { format!("{:+.0}C {:02}:{:02} G{} {:.0}km", p.ambient, hh, mm, gear, c.odo_m / 1000.0) } else { String::new() };
    }
}
