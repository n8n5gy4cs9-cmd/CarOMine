//! HUD: health/armor/money, ammo, driving gauges, clock, crosshair, weapon sprite, damage flash.
use super::weapons::*;
use super::*;
use crate::manifest::AssetManifest;
use crate::vehicle::Car;
use bevy::prelude::*;

#[derive(Component)] pub struct HudRoot;
#[derive(Component)] pub struct HudStats;
#[derive(Component)] pub struct HudAmmo;
#[derive(Component)] pub struct HudDrive;
#[derive(Component)] pub struct HudWeapon;
#[derive(Component)] pub struct HudHurt;
#[derive(Component)] pub struct HudMsg;

fn txt(size: f32) -> (TextFont, TextColor) { (TextFont { font_size: FontSize::Px(size), ..default() }, TextColor(Color::WHITE)) }

pub fn spawn_hud(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, existing: Query<(), With<HudRoot>>) {
    if !existing.is_empty() { return; }
    let cross = server.load(man.path("ui.crosshair"));
    commands
        .spawn((HudRoot, Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, GlobalZIndex(10), Pickable::IGNORE))
        .with_children(|r| {
            r.spawn((HudHurt, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgba(0.8, 0.0, 0.0, 0.0))));
            r.spawn((ImageNode::new(cross), Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), top: Val::Percent(50.0), width: Val::Px(24.0), height: Val::Px(24.0), margin: UiRect { left: Val::Px(-12.0), top: Val::Px(-12.0), ..default() }, ..default() }));
            r.spawn((HudWeapon, ImageNode::default(), Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), bottom: Val::Px(0.0), width: Val::Px(480.0), height: Val::Px(384.0), margin: UiRect { left: Val::Px(-200.0), ..default() }, ..default() }));
            r.spawn((HudStats, Text::new(""), txt(22.0), Node { position_type: PositionType::Absolute, left: Val::Px(16.0), bottom: Val::Px(12.0), ..default() }));
            r.spawn((HudAmmo, Text::new(""), txt(22.0), Node { position_type: PositionType::Absolute, right: Val::Px(16.0), bottom: Val::Px(12.0), ..default() }));
            r.spawn((HudDrive, Text::new(""), txt(22.0), Node { position_type: PositionType::Absolute, right: Val::Px(16.0), top: Val::Px(10.0), ..default() }));
            r.spawn((HudMsg, Text::new(""), txt(44.0), Node { position_type: PositionType::Absolute, left: Val::Percent(35.0), top: Val::Percent(40.0), ..default() }));
        });
}

pub fn update_hud(
    stats: Res<PlayerStats>,
    lo: Res<Loadout>,
    tod: Res<crate::world::sky::TimeOfDay>,
    ws: Res<crate::vehicle::windshield::Windshield>,
    cars: Query<&Car, With<crate::vehicle::Driven>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<HudStats>>, Query<&mut Text, With<HudAmmo>>, Query<&mut Text, With<HudDrive>>, Query<&mut Text, With<HudMsg>>,
    )>,
    mut hurt: Query<&mut BackgroundColor, With<HudHurt>>,
) {
    if let Ok(mut t) = texts.p0().single_mut() { t.0 = format!("HP {:.0}  ARMOR {:.0}  EUR {}", stats.hp.ceil(), stats.armor.ceil(), stats.money); }
    if let Ok(mut t) = texts.p1().single_mut() {
        let w = &WEAPONS[lo.current];
        t.0 = if w.melee { w.name.to_string() } else if lo.reload_t > 0.0 { format!("{} - reloading...", w.name) } else { format!("{}  {}/{}", w.name, lo.mag[lo.current], lo.reserve[lo.current]) };
    }
    let (h, m) = (tod.hour as u32, ((tod.hour.fract()) * 60.0) as u32);
    if let Ok(mut t) = texts.p2().single_mut() {
        t.0 = match cars.single() {
            Ok(c) => format!("{:02}:{:02}\n{:3.0} km/h  gear {}\n{:4.0} rpm  fuel {:.0}L\n{} {}{}{}\nsnow on glass {:.0}%",
                h, m, c.speed_kmh(), match c.gear { -1 => "R".into(), g => g.to_string() }, c.rpm, c.fuel,
                c.tyres.name(), if c.running { "ENGINE " } else { "engine off(I) " }, if c.lights { "LIGHTS " } else { "" }, if c.wipers > 0 { "WIPERS" } else { "" }, ws.snow * 100.0),
            Err(_) => format!("{:02}:{:02}", h, m),
        };
    }
    if let Ok(mut t) = texts.p3().single_mut() { t.0 = if stats.hp <= 0.0 { "YOU DIED".into() } else { String::new() }; }
    if let Ok(mut b) = hurt.single_mut() { b.0 = Color::srgba(0.8, 0.0, 0.0, (stats.hurt_flash * 0.5).min(0.45) + if stats.hp <= 0.0 { 0.3 } else { 0.0 }); }
}

pub fn update_weapon_sprite(
    lo: Res<Loadout>,
    a: Option<Res<WeaponAssets>>,
    driving: Query<(), With<crate::vehicle::Driven>>,
    player: Query<&crate::player::Player>,
    time: Res<Time>,
    mut q: Query<(&mut ImageNode, &mut Node), With<HudWeapon>>,
) {
    let Some(a) = a else { return };
    let Ok((mut img, mut node)) = q.single_mut() else { return };
    let w = &WEAPONS[lo.current];
    img.image = a.sheets[lo.current].clone();
    let frame = if lo.since_shot < 0.05 { 1 } else if lo.since_shot < 0.14 { 2 } else { 0 };
    img.texture_atlas = Some(TextureAtlas { layout: a.layout.clone(), index: frame });
    let vis = driving.is_empty();
    img.color = if vis { Color::WHITE } else { Color::NONE };
    // walking bob + reload dip
    let speed = player.single().map(|p| Vec2::new(p.vel.x, p.vel.z).length()).unwrap_or(0.0);
    let bob = (time.elapsed_secs() * 9.0).sin() * (speed * 1.2).min(8.0);
    let dip = if lo.reload_t > 0.0 { 120.0 * (1.0 - (lo.reload_t - 0.55).abs() / 0.55).clamp(0.0, 1.0) } else { 0.0 };
    node.bottom = Val::Px(-20.0 + bob.abs() - dip);
    node.margin.left = Val::Px(-200.0 + bob * 1.5);
    let _ = w;
}
