use super::enemy::Kind;
use super::weapons::*;
use super::*;
use crate::manifest::AssetManifest;
use crate::world::billboard::*;
use crate::world::layout::*;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PickupKind { Medkit, AmmoPistol, AmmoShells, AmmoRifle, Money, Coffee, Sausage }

#[derive(Component)]
pub struct Pickup(pub PickupKind);

#[derive(Resource, Default)]
pub struct DropQueue(pub Vec<(Vec3, Kind)>);

#[derive(Resource)]
pub struct PickupAssets { mesh: Handle<Mesh>, mats: Vec<(PickupKind, Handle<StandardMaterial>)> }

pub fn load_pickup_assets(mut commands: Commands, server: Res<AssetServer>, man: Res<AssetManifest>, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    commands.init_resource::<DropQueue>();
    let mesh = meshes.add(feet_quad(0.6, 0.6));
    let defs = [(PickupKind::Medkit, "item.medkit"), (PickupKind::AmmoPistol, "item.ammo_pistol"), (PickupKind::AmmoShells, "item.ammo_shells"), (PickupKind::AmmoRifle, "item.ammo_rifle"), (PickupKind::Money, "item.money"), (PickupKind::Coffee, "item.coffee"), (PickupKind::Sausage, "item.sausage")];
    let list = defs.iter().map(|(k, id)| {
        let mut m = sprite_material(load_nearest(&server, man.path(id)));
        m.emissive = LinearRgba::gray(0.25);
        (*k, mats.add(m))
    }).collect();
    commands.insert_resource(PickupAssets { mesh, mats: list });
}

pub fn spawn_pickup(commands: &mut Commands, a: &PickupAssets, k: PickupKind, pos: Vec3) {
    let m = a.mats.iter().find(|(kk, _)| *kk == k).unwrap().1.clone();
    commands.spawn((Mesh3d(a.mesh.clone()), MeshMaterial3d(m), Transform::from_translation(pos + Vec3::Y * 0.1), Billboard, Pickup(k)));
}

pub fn collect(
    mut commands: Commands,
    mut drops: ResMut<DropQueue>,
    a: Option<Res<PickupAssets>>,
    mut stats: ResMut<PlayerStats>,
    mut lo: ResMut<Loadout>,
    player: Query<&Transform, With<crate::player::Player>>,
    q: Query<(Entity, &Transform, &Pickup)>,
    mut started: Local<bool>,
) {
    let Some(a) = a else { return };
    if !*started {
        *started = true;
        // starter loot around the cabin
        for (i, k) in [PickupKind::Medkit, PickupKind::AmmoPistol, PickupKind::Coffee, PickupKind::Sausage, PickupKind::Money].iter().enumerate() {
            let p = CABIN + Vec2::new(-3.0 + i as f32 * 1.4, 6.0);
            spawn_pickup(&mut commands, &a, *k, Vec3::new(p.x, height_at(p.x, p.y), p.y));
        }
        // a shotgun-shell box near the village road for early fun
        let p = VILLAGE + Vec2::new(8.0, 6.0);
        spawn_pickup(&mut commands, &a, PickupKind::AmmoShells, Vec3::new(p.x, height_at(p.x, p.y), p.y));
        lo.owned[2] = true; // TODO: shotgun should be found/bought; unlocked for the vertical slice
        lo.reserve[2] = 12; lo.mag[2] = 6;
        lo.owned[3] = true; lo.reserve[3] = 90; lo.mag[3] = 30;
        lo.owned[4] = true; lo.reserve[4] = 15; lo.mag[4] = 5;
    }
    // enemy drops
    for (pos, kind) in drops.0.drain(..) {
        let h = crate::world::noise::hash2(pos.x as i32, pos.z as i32, 5);
        let k = match (kind, h) {
            (_, x) if x < 0.3 => PickupKind::Money,
            (Kind::Hunter, x) if x < 0.7 => PickupKind::AmmoRifle,
            (Kind::Goon, x) if x < 0.7 => PickupKind::AmmoPistol,
            (_, x) if x < 0.85 => PickupKind::Medkit,
            _ => PickupKind::Sausage,
        };
        spawn_pickup(&mut commands, &a, k, pos);
    }
    let Ok(pt) = player.single() else { return };
    for (e, t, p) in &q {
        if t.translation.distance(pt.translation) > 1.8 { continue; }
        let used = match p.0 {
            PickupKind::Medkit if stats.hp < 100.0 => { stats.hp = (stats.hp + 40.0).min(100.0); true }
            PickupKind::Medkit => false,
            PickupKind::AmmoPistol => { lo.reserve[1] += 12; true }
            PickupKind::AmmoShells => { lo.reserve[2] += 8; true }
            PickupKind::AmmoRifle => { lo.reserve[4] += 5; true }
            PickupKind::Money => { stats.money += 20; true }
            PickupKind::Coffee | PickupKind::Sausage => { stats.hp = (stats.hp + 10.0).min(100.0); true }
        };
        if used { commands.entity(e).despawn(); }
    }
}
