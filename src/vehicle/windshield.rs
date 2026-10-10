//! Snow piles up on the windshield (Amiga Toyota Celica style) unless the wipers run. Frost forms when parked cold.
//! The glass is split into COLS vertical strips: snow accumulates per strip and the wiper blade clears the strips it sweeps over,
//! leaving a fan-shaped cleared arc that re-fills with snow over time.
use super::*;
use crate::manifest::AssetManifest;
use crate::world::weather::Weather;
use bevy::prelude::*;

pub const COLS: usize = 20;

#[derive(Resource)]
pub struct Windshield {
    pub snow: f32,        // average coverage 0..1 (HUD)
    pub cols: [f32; COLS],
    pub frost: f32,
    pub wiper_t: f32,     // 0..1 sweep phase
    pub wiper_dir: f32,
}
impl Default for Windshield { fn default() -> Self { Self { snow: 0.0, cols: [0.0; COLS], frost: 0.0, wiper_t: 0.0, wiper_dir: 0.0 } } }
impl Windshield {
    /// Dev/console: set uniform coverage.
    pub fn set_all(&mut self, v: f32) { self.cols = [v.clamp(0.0, 1.0); COLS]; self.snow = v.clamp(0.0, 1.0); }
}

#[derive(Resource)]
pub struct WindshieldImages { pub snow: Vec<Handle<Image>>, pub frost: Handle<Image> }

#[derive(Component)] pub struct WsRoot;
#[derive(Component)] pub struct WsStrip(pub usize);
#[derive(Component)] pub struct WsFrost;
#[derive(Component)] pub struct WsWiper;

pub fn spawn_overlay(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, existing: Query<(), With<WsRoot>>) {
    if !existing.is_empty() { return; }
    let snow: Vec<Handle<Image>> = (1..=5).map(|i| server.load(man.path(&format!("ui.windshield_snow_{i}")))).collect();
    let frost = server.load(man.path("ui.windshield_frost"));
    let wiper = server.load(man.path("ui.wiper"));
    commands.insert_resource(WindshieldImages { snow: snow.clone(), frost: frost.clone() });
    commands
        .spawn((WsRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(78.0), ..default() }, Visibility::Hidden, GlobalZIndex(5)))
        .with_children(|c| {
            c.spawn((WsFrost, ImageNode::new(frost).with_mode(NodeImageMode::Stretch), Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }));
            for i in 0..COLS {
                // each strip shows its own slice of the snow image
                let (x0, x1) = (i as f32 / COLS as f32, (i + 1) as f32 / COLS as f32);
                let mut img = ImageNode::new(snow[0].clone()).with_mode(NodeImageMode::Stretch);
                img.rect = Some(Rect::new(x0 * 512.0, 0.0, x1 * 512.0, 256.0));
                img.color = Color::srgba(1.0, 1.0, 1.0, 0.0);
                c.spawn((WsStrip(i), img, Node { position_type: PositionType::Absolute, left: Val::Percent(x0 * 100.0), width: Val::Percent(100.0 / COLS as f32), height: Val::Percent(100.0), ..default() }));
            }
            c.spawn((WsWiper, ImageNode::new(wiper), Node { position_type: PositionType::Absolute, width: Val::Px(520.0), height: Val::Px(10.0), left: Val::Px(0.0), top: Val::Px(0.0), ..default() }));
        });
}

/// Wiper sweep angle in degrees for a phase 0..1.
fn wiper_angle(t: f32) -> f32 { 10.0 + t * 160.0 }
/// Angle at which strip i is swept (strip 0 = left edge). The blade starts on the right (10deg) and sweeps to the left (170deg).
fn strip_angle(i: usize) -> f32 { wiper_angle(1.0 - (i as f32 + 0.5) / COLS as f32) }

pub fn update(mut sfx: MessageWriter<crate::audio::Sfx>, time: Res<Time>, weather: Res<Weather>, tod: Res<crate::world::sky::TimeOfDay>, mut ws: ResMut<Windshield>, car: Query<&Car, With<Driven>>, parked: Query<&Car, Without<Driven>>) {
    let dt = time.delta_secs().min(0.05);
    let driven = car.single().ok();
    let speed = driven.map(|c| c.v_long.abs()).unwrap_or(0.0);
    let wipers = driven.map(|c| c.wipers).unwrap_or(0);
    let power = driven.map(|c| c.wiper_power).unwrap_or(1.0);
    // snow accumulation per strip (flakes land faster when driving into them; slightly uneven)
    let rate = weather.intensity * (0.012 + speed * 0.0018) * dt;
    for (i, c) in ws.cols.iter_mut().enumerate() { *c += rate * (0.8 + 0.4 * ((i * 7 % 5) as f32 / 4.0)); }
    // wiper sweep
    if wipers > 0 || ws.wiper_t > 0.0 {
        let period = if wipers == 2 { 0.9 } else { 1.8 };
        if ws.wiper_dir == 0.0 { ws.wiper_dir = 1.0; }
        let before = wiper_angle(ws.wiper_t);
        ws.wiper_t += dt / period * 2.0 * ws.wiper_dir;
        if ws.wiper_t >= 1.0 { ws.wiper_t = 1.0; ws.wiper_dir = -1.0; sfx.write(crate::audio::Sfx::new("sfx.wiper").vol(0.6)); }
        if ws.wiper_t <= 0.0 { ws.wiper_t = 0.0; ws.wiper_dir = if wipers > 0 { 1.0 } else { 0.0 }; }
        let after = wiper_angle(ws.wiper_t);
        let (lo, hi) = (before.min(after) - 1.0, before.max(after) + 1.0);
        if wipers > 0 {
            for i in 0..COLS { let a = strip_angle(i); if a >= lo && a <= hi { ws.cols[i] *= 1.0 - 0.9 * power; } }
        }
    }
    for c in ws.cols.iter_mut() { *c = c.clamp(0.0, 1.0); }
    ws.snow = ws.cols.iter().sum::<f32>() / COLS as f32;
    // frost: forms at night when engine is off, melts with engine (heater)
    let night = 1.0 - crate::world::sky::sky_state(tod.hour).day;
    let heater = driven.map(|c| c.running).unwrap_or(false) || parked.iter().any(|c| c.running);
    if heater { ws.frost -= dt * 0.03; } else { ws.frost += dt * 0.004 * night; }
    ws.frost = ws.frost.clamp(0.0, 1.0);
}

pub fn show(
    chase: Res<super::ChaseCam>,
    tod: Res<crate::world::sky::TimeOfDay>,
    ws: Res<Windshield>,
    imgs: Option<Res<WindshieldImages>>,
    driven: Query<(), With<Driven>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut root: Query<&mut Visibility, With<WsRoot>>,
    mut strips: Query<(&WsStrip, &mut ImageNode), (Without<WsFrost>, Without<WsWiper>)>,
    mut frost: Query<&mut ImageNode, (With<WsFrost>, Without<WsStrip>, Without<WsWiper>)>,
    mut wiper: Query<(&mut Node, &mut UiTransform, &mut Visibility), (With<WsWiper>, Without<WsRoot>)>,
) {
    let Some(imgs) = imgs else { return };
    let in_car = !driven.is_empty() && !chase.0;
    if let Ok(mut v) = root.single_mut() { *v = if in_car { Visibility::Inherited } else { Visibility::Hidden }; }
    // glass is lit by the world: dark at night (headlights / street light keep a floor)
    let lit = 0.35 + 0.65 * crate::world::sky::sky_state(tod.hour).day;
    for (s, mut img) in &mut strips {
        let c = ws.cols[s.0];
        let lvl = (c * 5.0).ceil() as usize;
        if lvl == 0 { img.color = Color::srgba(1.0, 1.0, 1.0, 0.0); } else { img.image = imgs.snow[(lvl - 1).min(4)].clone(); img.color = Color::srgb(lit, lit, lit * 1.05); }
    }
    if let Ok(mut f) = frost.single_mut() { f.color = Color::srgba(lit, lit, lit, ws.frost); }
    if let (Ok(w), Ok((mut node, mut ui, mut vis))) = (window.single(), wiper.single_mut()) {
        let active = ws.wiper_t > 0.0 || ws.wiper_dir != 0.0;
        *vis = if active { Visibility::Inherited } else { Visibility::Hidden };
        let size = Vec2::new(w.width(), w.height() * 0.78);
        let pivot = Vec2::new(size.x * 0.5, size.y * 0.97);
        let len = size.y * 0.8;
        let ang = wiper_angle(ws.wiper_t).to_radians();
        let centre = pivot + Vec2::new(ang.cos(), -ang.sin()) * (len * 0.5);
        node.width = Val::Px(len); node.height = Val::Px(8.0);
        node.left = Val::Px(centre.x - len * 0.5); node.top = Val::Px(centre.y - 4.0);
        ui.rotation = Rot2::radians(-ang);
    }
}
