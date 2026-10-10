//! Vehicle: bicycle-model dynamics with friction circle (slippery winter!), terrain following,
//! enter/exit, crash events, lights, wipers + windshield snow.
pub mod physics;
pub mod spec;
pub mod windshield;

use crate::player::{Player, PlayerCamera};
use crate::settings::Keybinds;
use crate::state::GameState;
use crate::world::billboard::Billboard;
use crate::world::layout::*;
use bevy::prelude::*;
use spec::*;

#[derive(Component)]
pub struct Car {
    pub spec: CarSpec,
    pub tyres: TyreType,
    // body-frame state
    pub v_long: f32,
    pub v_lat: f32,
    pub yaw: f32,
    pub yaw_rate: f32,
    // inputs
    pub steer_in: f32,
    pub throttle: f32,
    pub brake: f32,
    pub handbrake: bool,
    // drivetrain
    pub steer: f32,
    pub gear: i32, // -1 reverse, 0 neutral, 1..5
    pub rpm: f32,
    pub running: bool,
    pub fuel: f32,
    pub lights: bool,
    pub wipers: u8, // 0 off, 1 slow, 2 fast
    pub pitch: f32,
    pub roll: f32,
    pub ground_y: f32,
    pub wheel_spin: f32,
    pub slip: f32, // 0..1 how much the tyres are sliding (for sound/particles/HUD)
    // derived by build::sim from installed parts / faults
    pub torque_mult: f32,
    pub brake_mult: f32,
    pub grip_mult: f32,
    pub electrics: bool,
    pub headlight_power: f32,
    pub wiper_power: f32,
    pub warn_temp: bool,
    pub warn_batt: bool,
}
impl Car {
    pub fn new(yaw: f32) -> Self {
        let spec = CarSpec::default();
        let fuel = 6.0;
        Self { spec, tyres: TyreType::Summer, v_long: 0.0, v_lat: 0.0, yaw, yaw_rate: 0.0, steer_in: 0.0, throttle: 0.0, brake: 0.0,
            handbrake: false, steer: 0.0, gear: 1, rpm: 0.0, running: false, fuel, lights: false, wipers: 0, pitch: 0.0, roll: 0.0, ground_y: 0.0,
            wheel_spin: 0.0, slip: 0.0, torque_mult: 1.0, brake_mult: 1.0, grip_mult: 1.0, electrics: true, headlight_power: 1.0, wiper_power: 1.0, warn_temp: false, warn_batt: false }
    }
    pub fn speed_kmh(&self) -> f32 { self.v_long.abs() * 3.6 }
}

/// Marker: this car is currently being driven by the player.
#[derive(Component)]
pub struct Driven;

#[derive(Component)]
pub struct WheelVisual { pub front: bool, pub side: f32 }

/// Fired on collision. `speed` = closing speed m/s. Damage system (T-084) listens.
#[derive(Message, Clone, Debug)]
pub struct Crash { pub car: Entity, pub speed: f32, pub point: Vec3 }

pub fn spawn_car(commands: &mut Commands, server: &AssetServer, man: &crate::manifest::AssetManifest, pos: Vec2, yaw: f32) -> Entity {
    let body = server.load(bevy::gltf::GltfAssetLabel::Scene(0).from_asset(man.path("car.body")));
    let wheel_scene = || server.load(bevy::gltf::GltfAssetLabel::Scene(0).from_asset(man.path("car.wheel")));
    let car = commands
        .spawn((Car::new(yaw), crate::build::CarParts::starter_shell(), Transform::from_xyz(pos.x, height_at(pos.x, pos.y), pos.y).with_rotation(Quat::from_rotation_y(yaw)), Visibility::default()))
        .with_children(|c| {
            c.spawn((WorldAssetRoot(body), Transform::default()));
            for (front, fz) in [(true, -1.35f32), (false, 1.35)] {
                for side in [-1.0f32, 1.0] {
                    c.spawn((WorldAssetRoot(wheel_scene()), Transform::from_xyz(side * 0.78, 0.32, fz), WheelVisual { front, side }));
                }
            }
            // headlights
            for sx in [-0.6f32, 0.6] {
                c.spawn((
                    SpotLight { intensity: 0.0, range: 80.0, outer_angle: 0.6, inner_angle: 0.3, color: Color::srgb(1.0, 0.95, 0.8), shadow_maps_enabled: false, ..default() },
                    Transform::from_xyz(sx, 0.75, -2.3).looking_to(Vec3::new(0.0, -0.08, -1.0), Vec3::Y),
                    Headlight,
                ));
            }
        })
        .id();
    car
}

#[derive(Component)]
pub struct Headlight;

pub fn spawn_start_car(mut commands: Commands, server: Res<AssetServer>, man: Res<crate::manifest::AssetManifest>, existing: Query<(), With<Car>>) {
    if !existing.is_empty() { return; }
    // park next to the cabin garage
    let road_side = Vec2::new(CABIN.x - 14.0, CABIN.y - 4.0);
    spawn_car(&mut commands, &server, &man, road_side, 0.3);
}

/// E near the car -> get in; E inside -> get out.
pub fn enter_exit(
    mut sfx: MessageWriter<crate::audio::Sfx>,
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    kb: Res<Keybinds>,
    mut player: Query<(Entity, &mut Player, &mut Transform, &mut Visibility), (Without<Car>, Without<PlayerCamera>)>,
    cam: Query<Entity, With<PlayerCamera>>,
    cars: Query<(Entity, &Transform, Option<&Driven>), With<Car>>,
) {
    if !keys.just_pressed(kb.interact) { return; }
    sfx.write(crate::audio::Sfx::new("sfx.door"));
    let Ok((pe, mut p, mut pt, mut pv)) = player.single_mut() else { return };
    let Ok(cam) = cam.single() else { return };
    // already driving -> exit
    if let Some((ce, ct, _)) = cars.iter().find(|(_, _, d)| d.is_some()) {
        commands.entity(ce).remove::<Driven>();
        let left = Vec3::new(-ct.rotation.to_euler(EulerRot::YXZ).0.cos(), 0.0, ct.rotation.to_euler(EulerRot::YXZ).0.sin());
        let out = ct.translation + left * 2.0;
        pt.translation = Vec3::new(out.x, height_at(out.x, out.z), out.z);
        p.vel = Vec3::ZERO;
        *pv = Visibility::Inherited;
        commands.entity(cam).insert(ChildOf(pe)).insert(Transform::from_xyz(0.0, crate::player::EYE_STAND, 0.0).with_rotation(Quat::from_rotation_x(p.pitch)));
        return;
    }
    // find nearby
    if let Some((ce, _, _)) = cars.iter().filter(|(_, t, _)| t.translation.distance(pt.translation) < 3.6).next() {
        commands.entity(ce).insert(Driven);
        *pv = Visibility::Hidden;
        // driver seat (LHD): left, slightly forward of centre
        commands.entity(cam).insert(ChildOf(ce)).insert(Transform::from_xyz(-0.38, 1.12, -0.15));
        p.vel = Vec3::ZERO;
    }
}

/// While driving: mouse looks around inside the car; the player entity follows the car (so chunks stream).
pub fn driver_look(
    motion: Res<bevy::input::mouse::AccumulatedMouseMotion>,
    settings: Res<crate::settings::Settings>,
    car: Query<&Transform, (With<Driven>, With<Car>, Without<PlayerCamera>, Without<Player>)>,
    mut player: Query<(&mut Transform, &mut Player), (Without<Car>, Without<PlayerCamera>)>,
    mut cam: Query<&mut Transform, (With<PlayerCamera>, Without<Car>, Without<Player>)>,
    mut look: Local<(f32, f32)>,
) {
    let (Ok(ct), Ok((mut pt, _)), Ok(mut c)) = (car.single(), player.single_mut(), cam.single_mut()) else { return };
    pt.translation = ct.translation;
    look.0 = (look.0 - motion.delta.x * settings.mouse_sensitivity).clamp(-2.3, 2.3);
    look.1 = (look.1 - motion.delta.y * settings.mouse_sensitivity).clamp(-1.2, 1.0);
    // gently recentre yaw so you naturally look forward
    c.rotation = Quat::from_rotation_y(look.0) * Quat::from_rotation_x(look.1);
}

pub struct VehiclePlugin;
impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Crash>()
            .init_resource::<windshield::Windshield>()
            .add_systems(OnEnter(GameState::Playing), (spawn_start_car, windshield::spawn_overlay))
            .add_systems(
                Update,
                (
                    enter_exit,
                    physics::read_input,
                    physics::drive,
                    physics::visuals,
                    driver_look,
                    windshield::update,
                    windshield::show,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
