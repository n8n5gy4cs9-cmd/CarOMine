//! Fault simulation: translates part conditions into car behaviour and back (wear, damage).
use super::*;
use crate::combat::PlayerStats;
use crate::vehicle::{Car, Crash, Driven};
use crate::world::noise::hash2;
use crate::world::sky::{sky_state, TimeOfDay};
use bevy::prelude::*;

pub fn tick_toasts(time: Res<Time>, mut t: ResMut<Toasts>) {
    let dt = time.delta_secs();
    for m in &mut t.msgs { m.1 -= dt; }
    t.msgs.retain(|m| m.1 > 0.0);
}

fn rnd(t: f32, salt: i32) -> f32 { hash2((t * 1000.0) as i32, salt, 31337) }

pub fn sim(
    time: Res<Time>,
    tod: Res<TimeOfDay>,
    mut ign: MessageReader<Ignition>,
    mut toasts: ResMut<Toasts>,
    mut q: Query<(&mut Car, &mut CarParts, Option<&Driven>)>,
) {
    let dt = time.delta_secs().min(0.1);
    let t = time.elapsed_secs();
    let night = 1.0 - sky_state(tod.hour).day;
    let amb = -8.0 - 12.0 * night;
    let ignitions = ign.read().count();
    for (mut car, mut p, driven) in &mut q {
        let speed = car.v_long.abs();
        // ---------- ignition
        for _ in 0..ignitions {
            if driven.is_none() { continue; }
            if car.running { car.running = false; toasts.say("Engine off."); continue; }
            p.crank_t = 1.0;
            if !p.get(Slot::Battery).installed || p.battery < 0.03 { toasts.say(if p.get(Slot::Battery).installed { "Nothing happens. The battery is dead. (Charge it in the workshop: T)" } else { "Nothing happens. There is no battery." }); continue; }
            if p.battery < 0.2 { toasts.say("click-click-click... the battery is too weak."); p.battery = (p.battery - 0.01).max(0.0); continue; }
            if !p.get(Slot::Engine).installed { toasts.say("The starter spins freely... there is no engine!"); p.battery -= 0.03; continue; }
            p.battery = (p.battery - if p.temp < -5.0 { 0.09 } else { 0.05 }).max(0.0);
            let e = p.get(Slot::Engine).cond;
            if e < 0.05 { toasts.say("The engine is seized solid."); }
            else if car.fuel <= 0.0 { toasts.say("Cranks and cranks... out of fuel."); }
            else if p.frozen_line { toasts.say("Cranks, coughs, won't catch. Fuel line frozen? (Dry gas helps)"); }
            else if !p.get(Slot::SparkPlugs).installed { toasts.say("Cranks, but there is no spark. (Spark plugs missing)"); }
            else if p.get(Slot::SparkPlugs).cond < 0.15 { toasts.say("Cranks but won't catch. The spark plugs are fouled."); }
            else if !p.get(Slot::FuelFilter).installed { toasts.say("Fuel line is open at the filter - fuel everywhere. (Install a fuel filter)"); }
            else if p.temp < -8.0 && p.get(Slot::SparkPlugs).cond < 0.5 && rnd(t, 1) < 0.45 { toasts.say("Almost caught... try again. (Cold + tired plugs)"); }
            else { car.running = true; car.rpm = car.spec.idle_rpm; toasts.say("Engine started."); }
        }
        if p.crank_t > 0.0 { p.crank_t -= dt; }
        // ---------- electrics
        let batt_ok = p.get(Slot::Battery).installed && p.battery > 0.03;
        let fuse_ok = p.ok(Slot::Fuse, 0.05);
        let electrics = batt_ok && fuse_ok;
        if !electrics { car.lights = false; car.wipers = 0; }
        let charging = car.running && p.ok(Slot::Alternator, 0.1) && !p.belt_snapped;
        if p.get(Slot::Battery).installed {
            if charging { p.battery += 0.006 * dt; }
            else if car.running { p.battery -= 0.0004 * dt; }
            if car.lights && electrics { p.battery -= 0.0007 * dt; }
            if car.wipers > 0 && electrics { p.battery -= 0.0004 * car.wipers as f32 * dt; }
            p.battery = p.battery.clamp(0.0, 1.0);
        }
        if car.running && p.battery < 0.02 && !charging { /* ignition coil dies */ car.running = false; toasts.say("Engine dies - the battery is flat! (alternator/belt?)"); }
        // ---------- belt wear
        if car.running && p.get(Slot::Belt).installed && !p.belt_snapped {
            let mut b = p.get(Slot::Belt); b.cond -= 0.0005 * dt * (1.0 + car.rpm / 3000.0); p.st[Slot::Belt.idx()] = b;
            if b.cond <= 0.0 { p.belt_snapped = true; toasts.say("SNAP! The drive belt broke. No charging, no water pump."); }
        }
        if p.belt_snapped && p.get(Slot::Belt).installed && p.get(Slot::Belt).cond > 0.05 { p.belt_snapped = false; }
        // ---------- engine temperature
        let airflow = (speed / 18.0).clamp(0.0, 1.0);
        let rad_eff = if p.get(Slot::Radiator).installed { 0.3 + 0.7 * p.get(Slot::Radiator).cond } else { 0.0 };
        let pump = if p.belt_snapped || !p.get(Slot::Belt).installed { 0.15 } else { 1.0 };
        let cool_ok = if p.coolant > 0.15 { 1.0 } else { 0.1 };
        let k = 0.04 * (0.45 + 0.55 * airflow) * rad_eff * cool_ok * pump + 0.002;
        let heat = if car.running { 1.4 + 3.2 * car.throttle * (car.rpm / car.spec.redline) } else { 0.0 };
        p.temp += (heat - k * (p.temp - amb)) * dt;
        if p.get(Slot::Radiator).installed && p.get(Slot::Radiator).cond < 0.4 && p.coolant > 0.0 { p.coolant = (p.coolant - 0.004 * (0.4 - p.get(Slot::Radiator).cond) * dt).max(0.0); }
        if p.temp > 108.0 && rnd(t, 2) < 0.02 { toasts.say("Engine is overheating! Stop and let it cool. (Coolant? Radiator? Belt?)"); }
        if p.temp > 118.0 && p.get(Slot::Engine).installed { let mut e = p.get(Slot::Engine); e.cond = (e.cond - 0.012 * dt).max(0.0); p.st[Slot::Engine.idx()] = e; }
        if p.temp > 135.0 && car.running { car.running = false; let mut e = p.get(Slot::Engine); e.cond = (e.cond * 0.2).min(0.05); p.st[Slot::Engine.idx()] = e; toasts.say("The engine seized from overheating!"); }
        // ---------- fuel line freezing (cold nights, no dry gas)
        if p.dry_gas_t > 0.0 { p.dry_gas_t -= dt; p.frozen_line = false; }
        else if !car.running && amb < -14.0 && !p.frozen_line && rnd(t, 3) < 0.0006 { p.frozen_line = true; }
        // ---------- derived multipliers
        let mut tm = 1.0f32;
        let plugs = p.get(Slot::SparkPlugs);
        tm *= 0.5 + 0.5 * (plugs.cond / 0.6).min(1.0);
        tm *= 0.45 + 0.55 * p.get(Slot::Engine).cond.min(1.0);
        let filt = p.get(Slot::FuelFilter);
        if filt.cond < 0.3 && car.throttle > 0.6 { tm *= 0.35 + filt.cond; }
        if p.temp > 108.0 { tm *= 0.7; }
        if !p.get(Slot::Engine).installed || !p.get(Slot::Gearbox).installed { tm = 0.0; }
        car.torque_mult = tm;
        car.brake_mult = 0.3 + 0.7 * p.get(Slot::Brakes).cond;
        let tyre_cond = p.get(Slot::Tyres).cond;
        car.grip_mult = if !p.get(Slot::Tyres).installed { 0.25 } else if p.flat { 0.45 } else { 0.8 + 0.2 * tyre_cond };
        car.tyres = p.tyre;
        car.electrics = electrics;
        car.headlight_power = if p.get(Slot::Headlights).installed { 0.3 + 0.7 * p.get(Slot::Headlights).cond } else { 0.0 };
        car.wiper_power = if p.get(Slot::WiperBlades).installed { 0.2 + 0.8 * p.get(Slot::WiperBlades).cond } else { 0.0 };
        // ---------- stalls from bad fuel supply / plugs
        if car.running {
            p.stall_t += dt;
            if p.stall_t > 3.0 {
                p.stall_t = 0.0;
                let starve = filt.installed && filt.cond < 0.08 || !filt.installed;
                let misfire = plugs.cond < 0.1;
                if (starve || misfire) && rnd(t, 4) < 0.35 { car.running = false; toasts.say(if starve { "Engine dies - starved of fuel. (Fuel filter clogged?)" } else { "Engine dies - misfiring. (Spark plugs?)" }); }
            }
        }
        if !p.get(Slot::Engine).installed || !p.get(Slot::Gearbox).installed { /* nothing to drive */ }
        // ---------- wear: brakes + tyres
        if car.brake > 0.1 && speed > 1.0 { let mut b = p.get(Slot::Brakes); b.cond = (b.cond - 0.0015 * dt * speed / 8.0).max(0.0); p.st[Slot::Brakes.idx()] = b; }
        if car.slip > 0.5 && speed > 3.0 { let mut b = p.get(Slot::Tyres); let hard = match p.tyre { TyreType::Studded => 0.5, _ => 1.0 }; b.cond = (b.cond - 0.0004 * dt * hard).max(0.05); p.st[Slot::Tyres.idx()] = b; }
        // dash warnings
        if car.running && p.temp > 105.0 { car.warn_temp = true } else { car.warn_temp = false }
        car.warn_batt = car.running && !charging;
    }
}

/// Crashes break specific parts.
pub fn on_crash(mut crashes: MessageReader<Crash>, mut q: Query<(&Car, &mut CarParts)>, mut toasts: ResMut<Toasts>, time: Res<Time>, mut stats: ResMut<PlayerStats>) {
    let t = time.elapsed_secs();
    for c in crashes.read() {
        let Ok((car, mut p)) = q.get_mut(c.car) else { continue };
        let sev = ((c.speed - 2.5) / 14.0).clamp(0.0, 1.0);
        if sev <= 0.0 { continue; }
        let front = car.v_long >= -0.5;
        let hurt = |p: &mut CarParts, s: Slot, d: f32| { let mut x = p.get(s); if x.installed { x.cond = (x.cond - d).max(0.02); p.st[s.idx()] = x; } };
        if front {
            hurt(&mut p, Slot::Hood, sev * 0.9); hurt(&mut p, Slot::Bumper, sev * 1.1); hurt(&mut p, Slot::Headlights, sev * 0.5);
            hurt(&mut p, Slot::Radiator, sev * 0.8);
            if sev > 0.5 { hurt(&mut p, Slot::Engine, sev * 0.25); hurt(&mut p, Slot::Windshield, sev * 0.7); }
            if p.get(Slot::Radiator).installed && p.get(Slot::Radiator).cond < 0.4 { toasts.say("Radiator damaged - coolant is leaking!"); }
        } else {
            hurt(&mut p, Slot::Bumper, sev * 0.9); hurt(&mut p, Slot::DoorL, sev * 0.3); hurt(&mut p, Slot::DoorR, sev * 0.3);
        }
        if sev > 0.3 && rnd(t, 5) < 0.5 { hurt(&mut p, if rnd(t, 6) < 0.5 { Slot::DoorL } else { Slot::DoorR }, sev * 0.6); }
        if sev > 0.55 && rnd(t, 7) < 0.6 { p.flat = true; toasts.say("A tyre blew out in the crash!"); }
        if sev > 0.45 && p.ok(Slot::Fuse, 0.05) && rnd(t, 8) < 0.7 { hurt(&mut p, Slot::Fuse, 1.0); toasts.say("A fuse blew! Lights and wipers are dead."); }
        if sev > 0.2 { stats.damage(sev * 30.0); }
        toasts.say(format!("CRASH! ({:.0} km/h impact)", c.speed * 3.6));
    }
}
