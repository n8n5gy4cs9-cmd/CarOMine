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
    fl: Res<super::flashlight::Flashlight>,
    needs: Res<crate::life::needs::Needs>,
    tr: Res<crate::i18n::Strings>,
    tod: Res<crate::world::sky::TimeOfDay>,
    ws: Res<crate::vehicle::windshield::Windshield>,
    cars: Query<&Car, With<crate::vehicle::Driven>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<HudStats>>, Query<&mut Text, With<HudAmmo>>, Query<&mut Text, With<HudDrive>>, Query<&mut Text, With<HudMsg>>,
    )>,
    mut hurt: Query<&mut BackgroundColor, With<HudHurt>>,
) {
    if let Ok(mut t) = texts.p0().single_mut() { t.0 = format!("{} {:.0}  {} {:.0}  {} {}   {} {} {:.0}%\n{} {:.0}%  {} {:.0}%  {} {:.0}%", tr.t("hud.hp"), stats.hp.ceil(), tr.t("hud.armor"), stats.armor.ceil(), tr.t("hud.eur"), stats.money, tr.t("hud.light"), if fl.on { tr.t("hud.on") } else { tr.t("hud.off") }, fl.battery * 100.0, tr.t("hud.food"), needs.hunger * 100.0, tr.t("hud.energy"), needs.energy * 100.0, tr.t("hud.warm"), needs.warmth * 100.0); }
    if let Ok(mut t) = texts.p1().single_mut() {
        let w = &WEAPONS[lo.current];
        t.0 = if w.melee { w.name.to_string() } else if lo.reload_t > 0.0 { format!("{} - {}", w.name, tr.t("hud.reloading")) } else { format!("{}  {}/{}", w.name, lo.mag[lo.current], lo.reserve[lo.current]) };
    }
    let (h, m) = (tod.hour as u32, ((tod.hour.fract()) * 60.0) as u32);
    if let Ok(mut t) = texts.p2().single_mut() {
        t.0 = match cars.single() {
            Ok(c) => format!("{:02}:{:02}\n{:3.0} km/h  gear {}\n{:4.0} rpm  fuel {:.0}L\n{} {}{}{}\nsnow on glass {:.0}%",
                h, m, c.speed_kmh(), match c.gear { -1 => "R".into(), g => g.to_string() }, c.rpm, c.fuel,
                c.tyres.name(), if c.running { "ENGINE " } else { "engine off(I) " }, if c.lights { "LIGHTS " } else { "" }, if c.wipers > 0 { "WIPERS" } else { "" }, ws.snow * 100.0) + &format!("{}{}{}", if c.warn_temp { "\n!! ENGINE HOT !!" } else { "" }, if c.warn_batt { "\n!! BATTERY !!" } else { "" }, if !c.electrics { "\n!! NO POWER !!" } else { "" }),
            Err(_) => format!("{:02}:{:02}", h, m),
        };
    }
    if let Ok(mut t) = texts.p3().single_mut() { t.0 = if stats.hp <= 0.0 { tr.t("hud.died") } else { String::new() }; }
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
    // sheet (tools/gen_weapons.py): 0 idle, 1 fire, 2 recoil, 3-6 reload stages; fists: 1/2 right punch, 3/4 left punch
    let frame = if w.melee {
        let right = lo.shots % 2 == 1;
        if lo.since_shot < 0.12 { if right { 1 } else { 3 } } else if lo.since_shot < 0.28 { if right { 2 } else { 4 } } else { 0 }
    } else if lo.reload_t > 0.0 {
        let progress = (1.0 - lo.reload_t / 1.1).clamp(0.0, 0.999);
        3 + (progress * 4.0) as usize
    } else if lo.since_shot < 0.06 { 1 } else if lo.since_shot < 0.17 { 2 } else { 0 };
    img.texture_atlas = Some(TextureAtlas { layout: a.layout.clone(), index: frame });
    let vis = driving.is_empty();
    img.color = if vis { Color::WHITE } else { Color::NONE };
    // walking bob
    let speed = player.single().map(|p| Vec2::new(p.vel.x, p.vel.z).length()).unwrap_or(0.0);
    let bob = (time.elapsed_secs() * 9.0).sin() * (speed * 1.2).min(8.0);
    node.width = Val::Px(768.0); node.height = Val::Px(576.0);
    node.bottom = Val::Px(-30.0 + bob.abs());
    node.margin.left = Val::Px(-384.0 + bob * 1.5);
    let _ = w;
}
