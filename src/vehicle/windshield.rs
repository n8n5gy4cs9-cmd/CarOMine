//! Snow piles up on the windshield (Amiga Toyota Celica style) unless the wipers run. Frost forms when parked cold.
use super::*;
use crate::manifest::AssetManifest;
use crate::world::weather::Weather;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Windshield {
    pub snow: f32,   // 0..1
    pub frost: f32,  // 0..1
    pub wiper_t: f32, // 0..1 sweep phase (triangle wave)
    pub wiper_dir: f32,
}

#[derive(Resource)]
pub struct WindshieldImages { pub snow: Vec<Handle<Image>>, pub frost: Handle<Image> }

#[derive(Component)] pub struct WsRoot;
#[derive(Component)] pub struct WsSnow;
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
            c.spawn((WsSnow, ImageNode::new(snow[0].clone()).with_mode(NodeImageMode::Stretch), Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }));
            c.spawn((WsWiper, ImageNode::new(wiper), Node { position_type: PositionType::Absolute, width: Val::Px(520.0), height: Val::Px(10.0), left: Val::Px(0.0), top: Val::Px(0.0), ..default() }));
        });
}

pub fn update(time: Res<Time>, weather: Res<Weather>, tod: Res<crate::world::sky::TimeOfDay>, mut ws: ResMut<Windshield>, car: Query<&Car, With<Driven>>, parked: Query<&Car, Without<Driven>>) {
    let dt = time.delta_secs();
    let driven = car.single().ok();
    let speed = driven.map(|c| c.v_long.abs()).unwrap_or(0.0);
    let wipers = driven.map(|c| c.wipers).unwrap_or(0);
    // snow accumulation (flakes land faster when driving into them)
    ws.snow += weather.intensity * (0.012 + speed * 0.0018) * dt;
    // wipers sweep
    if wipers > 0 || ws.wiper_t > 0.0 {
        let period = if wipers == 2 { 0.9 } else { 1.8 };
        if ws.wiper_dir == 0.0 { ws.wiper_dir = 1.0; }
        let step = dt / period * 2.0 * ws.wiper_dir;
        ws.wiper_t += step;
        if ws.wiper_t >= 1.0 { ws.wiper_t = 1.0; ws.wiper_dir = -1.0; }
        if ws.wiper_t <= 0.0 {
            ws.wiper_t = 0.0;
            ws.wiper_dir = if wipers > 0 { 1.0 } else { 0.0 };
        }
        // wiper moving clears snow
        if wipers > 0 { ws.snow -= dt * (0.35 + 0.25 * wipers as f32); }
    }
    ws.snow = ws.snow.clamp(0.0, 1.0);
    // frost: forms at night when engine is off, melts with engine (heater)
    let night = 1.0 - crate::world::sky::sky_state(tod.hour).day;
    let heater = driven.map(|c| c.running).unwrap_or(false) || parked.iter().any(|c| c.running);
    if heater { ws.frost -= dt * 0.03; } else { ws.frost += dt * 0.004 * night; }
    ws.frost = ws.frost.clamp(0.0, 1.0);
}

pub fn show(
    tod: Res<crate::world::sky::TimeOfDay>,
    ws: Res<Windshield>,
    imgs: Option<Res<WindshieldImages>>,
    driven: Query<(), With<Driven>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut root: Query<&mut Visibility, With<WsRoot>>,
    mut snow: Query<&mut ImageNode, (With<WsSnow>, Without<WsFrost>, Without<WsWiper>)>,
    mut frost: Query<&mut ImageNode, (With<WsFrost>, Without<WsSnow>, Without<WsWiper>)>,
    mut wiper: Query<(&mut Node, &mut UiTransform, &mut Visibility), (With<WsWiper>, Without<WsRoot>)>,
) {
    let Some(imgs) = imgs else { return };
    let in_car = !driven.is_empty();
    if let Ok(mut v) = root.single_mut() { *v = if in_car { Visibility::Inherited } else { Visibility::Hidden }; }
    // glass is lit by the world: dark at night (headlights / street light keep a floor)
    let lit = 0.35 + 0.65 * crate::world::sky::sky_state(tod.hour).day;
    if let Ok(mut s) = snow.single_mut() {
        let lvl = (ws.snow * 5.0).ceil() as usize;
        if lvl == 0 { s.color = Color::srgba(1.0, 1.0, 1.0, 0.0); } else { s.image = imgs.snow[(lvl - 1).min(4)].clone(); s.color = Color::srgb(lit, lit, lit * 1.05); }
    }
    if let Ok(mut f) = frost.single_mut() { f.color = Color::srgba(lit, lit, lit, ws.frost); }
    if let (Ok(w), Ok((mut node, mut ui, mut vis))) = (window.single(), wiper.single_mut()) {
        let active = ws.wiper_t > 0.0 || ws.wiper_dir != 0.0;
        *vis = if active { Visibility::Inherited } else { Visibility::Hidden };
        // wiper pivots bottom-centre-left; sweeps 5deg..170deg
        let size = Vec2::new(w.width(), w.height() * 0.78);
        let pivot = Vec2::new(size.x * 0.5, size.y * 0.97);
        let len = size.y * 0.8;
        let ang = (10.0 + ws.wiper_t * 160.0f32).to_radians();
        let centre = pivot + Vec2::new(ang.cos(), -ang.sin()) * (len * 0.5);
        node.width = Val::Px(len); node.height = Val::Px(8.0);
        node.left = Val::Px(centre.x - len * 0.5); node.top = Val::Px(centre.y - 4.0);
        ui.rotation = Rot2::radians(-ang);
    }
}
