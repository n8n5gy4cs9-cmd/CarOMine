//! Car data. Everything that affects handling comes from here (later: derived from installed parts, T-100).
use crate::world::layout::Surface;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TyreType { Summer, Winter, Studded }
impl TyreType {
    pub fn name(self) -> &'static str { match self { Self::Summer => "summer", Self::Winter => "winter", Self::Studded => "studded" } }
}

/// Peak friction coefficient for surface x tyre. Based on RoR ground_models (snow 0.65/0.4, ice 0.3/0.2), tuned to be fun.
pub fn mu(surface: Surface, tyre: TyreType) -> f32 {
    use Surface::*; use TyreType::*;
    match (surface, tyre) {
        (Gravel, _) => 0.75,
        (Snow, Summer) => 0.30, (Snow, Winter) => 0.55, (Snow, Studded) => 0.60,
        (PackedSnow, Summer) => 0.34, (PackedSnow, Winter) => 0.62, (PackedSnow, Studded) => 0.68,
        (Ice, Summer) => 0.10, (Ice, Winter) => 0.20, (Ice, Studded) => 0.42,
    }
}
/// Extra rolling resistance coefficient (deep snow drags).
pub fn rolling(surface: Surface) -> f32 {
    match surface { Surface::Snow => 0.07, Surface::PackedSnow => 0.025, Surface::Ice => 0.012, Surface::Gravel => 0.02 }
}

#[derive(Clone, Debug)]
pub struct CarSpec {
    pub mass: f32,
    pub wheelbase: f32,
    pub cg_front: f32, // distance CG -> front axle
    pub track: f32,
    pub iz: f32,
    pub wheel_r: f32,
    pub peak_torque: f32,   // Nm
    pub idle_rpm: f32,
    pub redline: f32,
    pub gears: [f32; 5],
    pub reverse: f32,
    pub final_drive: f32,
    pub max_steer: f32,     // rad
    pub corner_stiff: f32,  // N/rad per axle
    pub brake_force: f32,   // N max total
    pub cd_a: f32,
    pub fuel_cap: f32,
    pub fuel_burn: f32,     // litres per second at full throttle
}
impl Default for CarSpec {
    fn default() -> Self {
        // Old 70s/80s RWD wagon, ~ 90hp
        Self {
            mass: 1250.0, wheelbase: 2.7, cg_front: 1.3, track: 1.4, iz: 1900.0, wheel_r: 0.32,
            peak_torque: 165.0, idle_rpm: 850.0, redline: 5600.0,
            gears: [3.7, 2.2, 1.45, 1.0, 0.8], reverse: 3.5, final_drive: 3.9,
            max_steer: 0.52, corner_stiff: 50_000.0, brake_force: 11_000.0, cd_a: 0.75,
            fuel_cap: 50.0, fuel_burn: 0.0045,
        }
    }
}
impl CarSpec {
    pub fn torque_curve(&self, rpm: f32) -> f32 {
        let x = (rpm / self.redline).clamp(0.0, 1.2);
        // peaks around 55% of redline, falls off toward redline
        let c = 1.0 - 1.6 * (x - 0.55).powi(2);
        self.peak_torque * c.clamp(0.25, 1.0)
    }
}
