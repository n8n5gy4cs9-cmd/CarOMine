//! Car building + repair: part slots with condition, inventory, fault simulation, workshop UI.
//! Design goal: simpler than My Summer Car, but with REAL faults and real fixes (see manual in workshop).
pub mod sim;
pub mod workshop;

use crate::state::GameState;
use crate::vehicle::spec::TyreType;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Slot {
    Engine, Gearbox, Battery, SparkPlugs, FuelFilter, Radiator, Alternator, Belt, Brakes, Tyres,
    WiperBlades, Headlights, Fuse, Hood, DoorL, DoorR, Bumper, Windshield,
}
pub const N: usize = 18;
pub const ALL: [Slot; N] = [
    Slot::Engine, Slot::Gearbox, Slot::Battery, Slot::SparkPlugs, Slot::FuelFilter, Slot::Radiator, Slot::Alternator, Slot::Belt,
    Slot::Brakes, Slot::Tyres, Slot::WiperBlades, Slot::Headlights, Slot::Fuse, Slot::Hood, Slot::DoorL, Slot::DoorR, Slot::Bumper, Slot::Windshield,
];

pub struct SlotInfo {
    pub name: &'static str,
    pub fi: &'static str,
    pub price: i32,
    pub fix_cap: f32,         // condition a quick fix can restore to (0 = replace only)
    pub fix_label: &'static str,
    pub heavy: bool,          // needs the garage (engine hoist etc.)
    pub hint: &'static str,
}
impl Slot {
    pub fn idx(self) -> usize { ALL.iter().position(|s| *s == self).unwrap() }
    pub fn info(self) -> SlotInfo {
        match self {
            Slot::Engine => SlotInfo { name: "Engine", fi: "Moottori", price: 900, fix_cap: 0.0, fix_label: "", heavy: true, hint: "No engine, no go. Heavy: install at the garage." },
            Slot::Gearbox => SlotInfo { name: "Gearbox", fi: "Vaihteisto", price: 450, fix_cap: 0.0, fix_label: "", heavy: true, hint: "Without it the engine spins but the car stays put." },
            Slot::Battery => SlotInfo { name: "Battery", fi: "Akku", price: 60, fix_cap: 0.0, fix_label: "", heavy: false, hint: "Cold drains it. Charge it in the workshop. Click-click = too weak." },
            Slot::SparkPlugs => SlotInfo { name: "Spark plugs", fi: "Sytytystulpat", price: 15, fix_cap: 0.7, fix_label: "Clean plugs", heavy: false, hint: "Fouled plugs: cranks but won't catch, misfires, weak power." },
            Slot::FuelFilter => SlotInfo { name: "Fuel filter", fi: "Polttoainesuodatin", price: 12, fix_cap: 0.0, fix_label: "", heavy: false, hint: "Clogged: engine starves under load or stalls." },
            Slot::Radiator => SlotInfo { name: "Radiator", fi: "Jäähdytin", price: 120, fix_cap: 0.65, fix_label: "Patch leaks", heavy: true, hint: "Needs coolant! Empty or leaking = overheating." },
            Slot::Alternator => SlotInfo { name: "Alternator", fi: "Laturi", price: 90, fix_cap: 0.0, fix_label: "", heavy: true, hint: "Dead alternator: battery drains while driving." },
            Slot::Belt => SlotInfo { name: "Drive belt", fi: "Hihna", price: 14, fix_cap: 0.0, fix_label: "", heavy: false, hint: "Snaps when worn: alternator + water pump stop." },
            Slot::Brakes => SlotInfo { name: "Brake pads", fi: "Jarrupalat", price: 40, fix_cap: 0.0, fix_label: "", heavy: false, hint: "Worn pads = long stopping distance." },
            Slot::Tyres => SlotInfo { name: "Tyres", fi: "Renkaat", price: 80, fix_cap: 0.55, fix_label: "Patch puncture", heavy: false, hint: "Summer tyres are lethal on ice. Winter/studded recommended." },
            Slot::WiperBlades => SlotInfo { name: "Wiper blades", fi: "Pyyhkijänsulat", price: 12, fix_cap: 0.0, fix_label: "", heavy: false, hint: "Worn blades smear snow instead of clearing it." },
            Slot::Headlights => SlotInfo { name: "Headlight bulbs", fi: "Ajovalot", price: 10, fix_cap: 0.0, fix_label: "", heavy: false, hint: "Nights are long. Dim bulbs = fewer metres of view." },
            Slot::Fuse => SlotInfo { name: "Fuse box", fi: "Sulakkeet", price: 3, fix_cap: 0.0, fix_label: "", heavy: false, hint: "Blown fuse: lights and wipers dead." },
            Slot::Hood => SlotInfo { name: "Hood", fi: "Konepelti", price: 60, fix_cap: 0.8, fix_label: "Hammer out dents", heavy: false, hint: "Body damage is cosmetic... mostly (katsastus checks it)." },
            Slot::DoorL => SlotInfo { name: "Door (left)", fi: "Ovi vasen", price: 55, fix_cap: 0.8, fix_label: "Hammer out dents", heavy: false, hint: "" },
            Slot::DoorR => SlotInfo { name: "Door (right)", fi: "Ovi oikea", price: 55, fix_cap: 0.8, fix_label: "Hammer out dents", heavy: false, hint: "" },
            Slot::Bumper => SlotInfo { name: "Bumper", fi: "Puskuri", price: 40, fix_cap: 0.8, fix_label: "Hammer out dents", heavy: false, hint: "" },
            Slot::Windshield => SlotInfo { name: "Windshield", fi: "Tuulilasi", price: 80, fix_cap: 0.0, fix_label: "", heavy: false, hint: "Cracked glass fails the inspection." },
        }
    }
    /// Needed to get the car moving at all.
    pub fn essential(self) -> bool { matches!(self, Slot::Engine | Slot::Gearbox | Slot::Tyres) }
}

#[derive(Clone, Copy, Debug)]
pub struct PartState { pub installed: bool, pub cond: f32 }

#[derive(Component, Clone)]
pub struct CarParts {
    pub st: [PartState; N],
    pub tyre: TyreType,
    pub flat: bool,
    pub coolant: f32,      // 0..1
    pub battery: f32,      // charge 0..1
    pub temp: f32,         // engine temp, C
    pub frozen_line: bool,
    pub dry_gas_t: f32,    // seconds of anti-freeze protection left
    pub belt_snapped: bool,
    pub stall_t: f32,
    pub crank_t: f32,      // >0 while starter cranking (for sound/anim later)
    pub ambient: f32,      // outside temperature, C (set by the sim)
}
impl CarParts {
    pub fn get(&self, s: Slot) -> PartState { self.st[s.idx()] }
    pub fn ok(&self, s: Slot, min: f32) -> bool { let p = self.get(s); p.installed && p.cond >= min }
    pub fn set(&mut self, s: Slot, installed: bool, cond: f32) { self.st[s.idx()] = PartState { installed, cond }; }
    /// The "pile of boxes" starting state: body only.
    pub fn starter_shell() -> Self {
        let mut p = Self { st: [PartState { installed: false, cond: 0.0 }; N], tyre: TyreType::Summer, flat: false, coolant: 0.0, battery: 0.0, temp: -12.0, frozen_line: false, dry_gas_t: 0.0, belt_snapped: false, stall_t: 0.0, crank_t: 0.0, ambient: -10.0 };
        for (s, c) in [(Slot::Tyres, 0.5), (Slot::Brakes, 0.45), (Slot::Headlights, 0.8), (Slot::Fuse, 1.0), (Slot::Hood, 0.7), (Slot::DoorL, 0.8), (Slot::DoorR, 0.6), (Slot::Bumper, 0.75), (Slot::Windshield, 0.9)] { p.set(s, true, c); }
        p
    }
    pub fn fully_built() -> Self {
        let mut p = Self::starter_shell();
        for s in ALL { p.set(s, true, 1.0); }
        p.coolant = 1.0; p.battery = 1.0; p
    }
}

#[derive(Clone, Debug)]
pub struct StockPart { pub slot: Slot, pub cond: f32, pub tyre: TyreType }

#[derive(Resource)]
pub struct Inventory {
    pub parts: Vec<StockPart>,
    pub dry_gas: u32,
    pub coolant_cans: u32,
    pub fuel_cans: u32,
}
impl Default for Inventory {
    fn default() -> Self {
        let sp = |slot, cond| StockPart { slot, cond, tyre: TyreType::Summer };
        Self {
            // the "boxes": an old wreck's worth of parts, of varying quality
            parts: vec![sp(Slot::Engine, 0.7), sp(Slot::Gearbox, 0.8), sp(Slot::Battery, 0.9), sp(Slot::SparkPlugs, 0.3), sp(Slot::FuelFilter, 0.5), sp(Slot::Radiator, 0.8), sp(Slot::Alternator, 0.6), sp(Slot::Belt, 0.7)],
            dry_gas: 0, coolant_cans: 0, fuel_cans: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct Toasts { pub msgs: Vec<(String, f32)> }
impl Toasts { pub fn say(&mut self, s: impl Into<String>) { let s = s.into(); info!("TOAST: {s}"); self.msgs.push((s, 5.0)); if self.msgs.len() > 4 { self.msgs.remove(0); } } }

/// Pressed the ignition key.
#[derive(Message)]
pub struct Ignition;

pub struct BuildPlugin;
impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>().init_resource::<Toasts>().add_message::<Ignition>()
            .add_systems(Startup, workshop::spawn_ui)
            .add_systems(Update, (sim::tick_toasts, workshop::refresh_toast_ui, workshop::sync_visibility))
            .add_systems(Update, (sim::sim, sim::on_crash).chain().before(crate::vehicle::physics::drive).run_if(in_state(GameState::Playing)))
            .add_systems(Update, workshop::open_close.run_if(in_state(GameState::Playing).or_else(in_state(GameState::Garage))))
            .add_systems(Update, (workshop::input, workshop::refresh).chain().run_if(in_state(GameState::Garage)));
    }
}
