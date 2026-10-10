use super::*;
use crate::combat::PlayerStats;
use crate::world::sky::{sky_state, TimeOfDay};
use crate::world::weather::Weather;

#[derive(Resource)]
pub struct Needs {
    pub hunger: f32, pub energy: f32, pub warmth: f32,
    pub food: u32, pub coffee: u32, pub logs: u32,
    pub sauna_hot: f32, // game minutes of heat left
    pub prev_hour: f32,
}
impl Default for Needs {
    fn default() -> Self { Self { hunger: 0.8, energy: 0.9, warmth: 1.0, food: 2, coffee: 3, logs: 4, sauna_hot: 0.0, prev_hour: 9.5 } }
}

#[derive(Component)] pub struct PromptText;

pub fn spawn_prompt(mut commands: Commands) {
    commands.spawn((PromptText, Text::new(""), TextFont { font_size: FontSize::Px(22.0), ..default() }, TextColor(Color::srgb(1.0, 0.95, 0.6)),
        Node { position_type: PositionType::Absolute, bottom: Val::Percent(24.0), left: Val::Percent(30.0), ..default() }, GlobalZIndex(31)));
}

pub fn tick(
    time: Res<Time>,
    tod: Res<TimeOfDay>,
    weather: Res<Weather>,
    mut needs: ResMut<Needs>,
    mut stats: ResMut<PlayerStats>,
    mut toasts: ResMut<crate::build::Toasts>,
    cars: Query<(&crate::vehicle::Car, Option<&crate::vehicle::Driven>)>,
    mut tod_mut: Local<f32>,
) {
    let _ = &mut tod_mut;
    if stats.hp <= 0.0 { return; }
    let dt = time.delta_secs();
    let gh = dt * tod.speed.max(1.0 / 60.0); // game hours (dev time-freeze still ages needs slowly)
    let night = 1.0 - sky_state(tod.hour).day;
    needs.hunger -= gh * 0.07;
    needs.energy -= gh * 0.055;
    let heater = cars.iter().any(|(c, d)| d.is_some() && c.running);
    if heater { needs.warmth += gh * 0.45; } else { needs.warmth -= gh * (0.06 + 0.05 * night + 0.06 * weather.intensity); }
    needs.sauna_hot = (needs.sauna_hot - gh * 60.0).max(0.0);
    needs.hunger = needs.hunger.clamp(0.0, 1.0); needs.energy = needs.energy.clamp(0.0, 1.0); needs.warmth = needs.warmth.clamp(0.0, 1.0);
    if needs.warmth < 0.08 { stats.hp = (stats.hp - 0.4 * dt).max(0.0); if (time.elapsed_secs() * 0.5) as u32 % 20 == 0 { toasts.say("You're freezing! Get warm: car heater, sauna, coffee."); } }
    if needs.hunger < 0.02 { stats.hp = (stats.hp - 0.3 * dt).max(0.0); }
    if needs.energy <= 0.0 { needs.energy = 0.5; stats.hp = (stats.hp - 12.0).max(1.0); needs.warmth = (needs.warmth - 0.3).max(0.0); toasts.say("You passed out from exhaustion... and woke up cold."); }
    // slow regen when comfortable
    if needs.hunger > 0.5 && needs.warmth > 0.5 && stats.hp < 100.0 { stats.hp = (stats.hp + 0.15 * dt).min(100.0); }
}

/// Kela pays unemployment benefit every morning at 08:00.
pub fn kela(tod: Res<TimeOfDay>, mut needs: ResMut<Needs>, mut stats: ResMut<PlayerStats>, mut toasts: ResMut<crate::build::Toasts>, mut missions: ResMut<missions::Missions>) {
    let (p, c) = (needs.prev_hour, tod.hour);
    if (p < 8.0 && c >= 8.0) || (p > c && c >= 8.0 && p < 24.0 && false) {
        stats.money += 35;
        toasts.say("Kela: unemployment benefit +35 EUR. (Thanks, Finland.)");
        missions.new_day();
    }
    needs.prev_hour = c;
}

pub fn prompt(
    near: Res<Nearby>,
    cars: Query<&Transform, With<crate::vehicle::Car>>,
    driven: Query<(), With<crate::vehicle::Driven>>,
    player: Query<&Transform, (With<crate::player::Player>, Without<crate::vehicle::Car>)>,
    missions: Res<missions::Missions>,
    mut q: Query<&mut Text, With<PromptText>>,
) {
    let Ok(mut t) = q.single_mut() else { return };
    let mut s = String::new();
    let car_near = player.single().ok().and_then(|p| cars.single().ok().map(|c| (p.translation.distance(c.translation), c.translation))).filter(|(d, _)| *d < 3.6 && driven.is_empty());
    if let Some(z) = near.0 { s = z.prompt().to_string(); }
    else if car_near.is_some() { s = "E: get in the car   T: workshop".into(); }
    else if !driven.is_empty() { s = String::new(); }
    let _ = &missions;
    t.0 = s;
}
