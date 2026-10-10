//! Instrument cluster (driver view): tacho, speedo, fuel, temperature, oil, volts, warning lights, clock/outside temp/odometer.
//! Art: assets/sprites/ui/dash_*.png (tools/gen_dash.py). Needles rotate about their centre; values are smoothed like real analog gauges.
use super::*;
use crate::build::{CarParts, Slot};
use crate::manifest::AssetManifest;
use crate::world::sky::TimeOfDay;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NeedleKind { Tacho, Speedo, Fuel, Temp, Oil, Volt }

#[derive(Component)] pub struct DashRoot;
#[derive(Component)] pub struct Needle { pub kind: NeedleKind, pub value: f32 }
#[derive(Component)] pub struct WarnIcon(pub usize);
#[derive(Component)] pub struct DashText;

const K: f32 = 0.8;                       // overall cluster scale
const PLATE: Vec2 = Vec2::new(648.0 * K, 232.0 * K);

fn node(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node { position_type: PositionType::Absolute, left: Val::Px(x * K), top: Val::Px(y * K), width: Val::Px(w * K), height: Val::Px(h * K), ..default() }
}

pub fn spawn(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut layouts: ResMut<Assets<TextureAtlasLayout>>, existing: Query<(), With<DashRoot>>) {
    if !existing.is_empty() { return; }
    let nearest = |id: &str| crate::world::billboard::load_nearest(&server, man.path(id));
    let big = nearest("ui.dash_needle"); let small = nearest("ui.dash_needle_small");
    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::new(32, 32), 9, 1, None, None));
    let icons = nearest("ui.dash_icons");
    // (dial id, x, y, size, needle kind)
    let dials = [("ui.dash_tacho", 8.0, 10.0, 204.0, NeedleKind::Tacho), ("ui.dash_speedo", 220.0, 10.0, 204.0, NeedleKind::Speedo),
                 ("ui.dash_fuel", 436.0, 10.0, 96.0, NeedleKind::Fuel), ("ui.dash_temp", 540.0, 10.0, 96.0, NeedleKind::Temp),
                 ("ui.dash_oil", 436.0, 110.0, 96.0, NeedleKind::Oil), ("ui.dash_volt", 540.0, 110.0, 96.0, NeedleKind::Volt)];
    commands
        .spawn((DashRoot, Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), bottom: Val::Px(6.0), width: Val::Px(PLATE.x), height: Val::Px(PLATE.y + 46.0 * K), margin: UiRect { left: Val::Px(-PLATE.x / 2.0 - 100.0), ..default() }, ..default() }, Visibility::Hidden, GlobalZIndex(6)))
        .with_children(|r| {
            // warning light strip
            for i in 0..9 {
                let mut img = ImageNode::new(icons.clone()); img.texture_atlas = Some(TextureAtlas { layout: layout.clone(), index: i }); img.color = Color::srgb(0.17, 0.17, 0.19);
                r.spawn((WarnIcon(i), img, node(150.0 + i as f32 * 40.0, 0.0, 32.0, 32.0)));
            }
            // plate (dark moulded plastic with a lighter lip)
            r.spawn((node(0.0, 40.0, PLATE.x, PLATE.y), BackgroundColor(Color::srgba(0.07, 0.06, 0.055, 0.94)), Outline::new(Val::Px(3.0), Val::Px(0.0), Color::srgb(0.28, 0.25, 0.22))));
            for (id, x, y, s, kind) in dials {
                r.spawn((ImageNode::new(nearest(id)), node(x, 40.0 + y, s, s)));
                r.spawn((Needle { kind, value: 0.0 }, ImageNode::new(if s > 150.0 { big.clone() } else { small.clone() }), node(x, 40.0 + y, s, s), UiTransform::default()));
            }
            r.spawn((DashText, Text::new(""), TextFont { font_size: FontSize::Px(13.0), ..default() }, TextColor(Color::srgb(1.0, 0.72, 0.3)), node(420.0, 40.0 + 205.0, 228.0, 22.0)));
        });
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
    mut root: Query<&mut Visibility, With<DashRoot>>,
    mut needles: Query<(&mut Needle, &mut UiTransform)>,
    mut icons: Query<(&WarnIcon, &mut ImageNode)>,
    mut text: Query<&mut Text, With<DashText>>,
) {
    let Ok(mut vis) = root.single_mut() else { return };
    let Ok((c, p)) = car.single() else { *vis = Visibility::Hidden; return };
    *vis = if chase.0 { Visibility::Hidden } else { Visibility::Inherited };
    let dt = time.delta_secs().min(0.05);
    let electrics = c.electrics;
    let charging = c.running && p.ok(Slot::Alternator, 0.1) && !p.belt_snapped;
    let volts = if !p.get(Slot::Battery).installed { 10.0 } else { 11.2 + 1.1 * p.battery + if charging { 1.6 } else { 0.0 } };
    for (mut n, mut ui) in &mut needles {
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
        ui.rotation = Rot2::radians(angle(n.kind, n.value + jitter).to_radians());
    }
    let off = Color::srgb(0.17, 0.17, 0.19);
    let (red, amber, green, blue) = (Color::srgb(1.0, 0.18, 0.15), Color::srgb(1.0, 0.68, 0.1), Color::srgb(0.25, 0.95, 0.4), Color::srgb(0.4, 0.65, 1.0));
    let key_on_engine_off = electrics && !c.running;
    let check = p.ok(Slot::Engine, 0.0) && (p.get(Slot::SparkPlugs).cond < 0.35 || p.get(Slot::FuelFilter).cond < 0.3 || p.get(Slot::Engine).cond < 0.4);
    for (w, mut img) in &mut icons {
        let on = match w.0 {
            0 => (c.warn_batt || key_on_engine_off || (electrics && volts < 12.0)).then_some(red),
            1 => ((c.running && c.oil < 0.25) || key_on_engine_off).then_some(red),
            2 => (electrics && (check || key_on_engine_off && (time.elapsed_secs() % 1.0) < 0.5 && false)).then_some(amber),
            3 => (electrics && (c.handbrake || p.get(Slot::Brakes).cond < 0.2)).then_some(red),
            4 => (electrics && (c.warn_temp || p.temp > 100.0)).then_some(red),
            5 => (electrics && c.fuel < 8.0).then_some(amber),
            6 => (electrics && c.lights).then_some(green),
            7 => (electrics && p.ambient < 3.5).then_some(blue),
            8 => (electrics && c.wipers > 0).then_some(green),
            _ => None,
        };
        img.color = on.unwrap_or(off);
    }
    if let Ok(mut t) = text.single_mut() {
        let (h, m) = (tod.hour as u32, (tod.hour.fract() * 60.0) as u32);
        let gear = match c.gear { -1 => "R".to_string(), 0 => "N".to_string(), g => g.to_string() };
        t.0 = if electrics { format!("{:+.0}C {:02}:{:02} G{} {:.0}km", p.ambient, h, m, gear, c.odo_m / 1000.0) } else { String::new() };
    }
}
