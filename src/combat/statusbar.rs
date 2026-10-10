//! Doom-style status bar (on foot): AMMO | HEALTH% | ARMS | FACE | ARMOR% | EUR | needs bars.
//! Art: assets/sprites/ui/{statusbar,digits,faces}.png (tools/gen_hud.py). Layout numbers mirror tools/gen_hud.py (native 480x48, shown x2).
use super::flashlight::Flashlight;
use super::weapons::{Loadout, WEAPONS};
use super::*;
use crate::life::needs::Needs;
use crate::manifest::AssetManifest;
use bevy::prelude::*;

const S: f32 = 2.0;
const BAR: Vec2 = Vec2::new(480.0, 48.0);
pub const BAR_PX_H: f32 = BAR.y * S;

#[derive(Component)] pub struct BarRoot;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Group { Ammo, Health, Armor, Eur }
#[derive(Component)] pub struct Digit { pub group: Group, pub idx: usize }
#[derive(Component)] pub struct Arms(usize);
#[derive(Component)] pub struct FaceNode;
#[derive(Component)] pub struct NeedFill(u8);

fn n(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node { position_type: PositionType::Absolute, left: Val::Px(x * S), top: Val::Px(y * S), width: Val::Px(w * S), height: Val::Px(h * S), ..default() }
}

pub fn spawn(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut layouts: ResMut<Assets<TextureAtlasLayout>>, existing: Query<(), With<BarRoot>>) {
    if !existing.is_empty() { return; }
    let load = |id: &str| crate::world::billboard::load_nearest(&server, man.path(id));
    let digits = load("ui.digits"); let faces = load("ui.faces");
    let dl = layouts.add(TextureAtlasLayout::from_grid(UVec2::new(14, 16), 12, 1, None, None));
    let fl = layouts.add(TextureAtlasLayout::from_grid(UVec2::new(36, 44), 5, 6, None, None));
    let glyph = |idx: usize, l: &Handle<TextureAtlasLayout>, img: &Handle<Image>| { let mut i = ImageNode::new(img.clone()); i.texture_atlas = Some(TextureAtlas { layout: l.clone(), index: idx }); i };
    commands.spawn((BarRoot, ImageNode::new(load("ui.statusbar")).with_mode(NodeImageMode::Stretch),
        Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), bottom: Val::Px(0.0), width: Val::Px(BAR.x * S), height: Val::Px(BAR.y * S), margin: UiRect { left: Val::Px(-BAR.x * S / 2.0), ..default() }, ..default() },
        GlobalZIndex(11), Visibility::Inherited)).with_children(|r| {
        for (g, x0, count) in [(Group::Ammo, 10.0, 3usize), (Group::Health, 78.0, 4), (Group::Armor, 260.0, 4), (Group::Eur, 336.0, 4)] {
            for i in 0..count { r.spawn((Digit { group: g, idx: i }, glyph(0, &dl, &digits), n(x0 + i as f32 * 14.0, 13.0, 14.0, 16.0))); }
        }
        for i in 0..5 { r.spawn((Arms(i), glyph(i + 1, &dl, &digits), n(154.0 + i as f32 * 8.5, 14.0, 7.0, 8.0))); }
        r.spawn((FaceNode, glyph(0, &fl, &faces), n(208.0, 2.0, 36.0, 44.0)));
        for (i, c) in [(0u8, Color::srgb(0.95, 0.55, 0.15)), (1, Color::srgb(0.3, 0.6, 0.95)), (2, Color::srgb(0.95, 0.3, 0.2))] {
            r.spawn((NeedFill(i), BackgroundColor(c), n(406.0 + i as f32 * 23.0, 9.0, 16.0, 24.0)));
        }
    });
}

fn tier(hp: f32) -> usize { if hp > 80.0 { 0 } else if hp > 60.0 { 1 } else if hp > 40.0 { 2 } else if hp > 20.0 { 3 } else { 4 } }

#[allow(clippy::too_many_arguments)]
pub fn update(
    time: Res<Time>,
    stats: Res<PlayerStats>,
    lo: Res<Loadout>,
    needs: Res<Needs>,
    fl: Res<Flashlight>,
    driven: Query<(), With<crate::vehicle::Driven>>,
    mut root: Query<&mut Visibility, With<BarRoot>>,
    mut digits: Query<(&Digit, &mut ImageNode)>,
    mut arms: Query<(&Arms, &mut ImageNode), Without<Digit>>,
    mut face: Query<&mut ImageNode, (With<FaceNode>, Without<Digit>, Without<Arms>)>,
    mut fills: Query<(&NeedFill, &mut Node)>,
) {
    let _ = &fl;
    if let Ok(mut v) = root.single_mut() { *v = if driven.is_empty() { Visibility::Inherited } else { Visibility::Hidden }; }
    let w = &WEAPONS[lo.current];
    let ammo = if w.melee { None } else { Some(lo.mag[lo.current] as i32) };
    for (d, mut img) in &mut digits {
        let (val, count, pct) = match d.group {
            Group::Ammo => (ammo, 3, false), Group::Health => (Some(stats.hp.ceil() as i32), 3, true),
            Group::Armor => (Some(stats.armor.ceil() as i32), 3, true), Group::Eur => (Some(stats.money.clamp(0, 9999)), 4, false),
        };
        let mut glyph: Option<usize> = None;
        if pct && d.idx == count { glyph = Some(10); }                                   // the % sign
        else if d.idx < count {
            match val {
                Some(v) => { let s = format!("{:0width$}", v.clamp(0, 10_i32.pow(count as u32) - 1), width = count); let ch = s.as_bytes()[d.idx] - b'0';
                    // blank leading zeros (keep the last digit)
                    let lead_zero = s.as_bytes()[..=d.idx].iter().all(|b| *b == b'0') && d.idx + 1 < count; if !lead_zero { glyph = Some(ch as usize); } }
                None => { if d.idx == count - 1 { glyph = Some(11); } }                  // fists: '-'
            }
        }
        match glyph { Some(g) => { if let Some(a) = &mut img.texture_atlas { a.index = g; } img.color = Color::WHITE; } None => img.color = Color::NONE }
    }
    for (a, mut img) in &mut arms {
        img.color = if lo.current == a.0 { Color::WHITE } else if lo.owned[a.0] { Color::srgb(0.55, 0.55, 0.55) } else { Color::srgb(0.18, 0.18, 0.2) };
    }
    if let Ok(mut f) = face.single_mut() {
        let t = time.elapsed_secs();
        let (row, col) = if stats.hp <= 0.0 { (5, 0) }
            else if stats.god { (5, 1) }
            else if stats.hurt_flash > 0.35 { (tier(stats.hp), 3) }
            else if lo.since_shot < 0.5 && !w.melee { (tier(stats.hp), 4) }
            else { (tier(stats.hp), [0usize, 1, 0, 2][((t / 1.6) as usize) % 4]) };
        if let Some(a) = &mut f.texture_atlas { a.index = row * 5 + col; }
    }
    for (nf, mut node) in &mut fills {
        let v = match nf.0 { 0 => needs.hunger, 1 => needs.energy, _ => needs.warmth }.clamp(0.0, 1.0);
        node.height = Val::Px(24.0 * S * v); node.top = Val::Px((9.0 + 24.0 * (1.0 - v)) * S);
    }
}
