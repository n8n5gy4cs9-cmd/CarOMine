use super::*;
use crate::world::scatter::Obstacle;

const G: f32 = 9.81;

pub fn read_input(keys: Res<ButtonInput<KeyCode>>, kb: Res<Keybinds>, mut q: Query<&mut Car, With<Driven>>, time: Res<Time>) {
    let Ok(mut car) = q.single_mut() else { return };
    let dt = time.delta_secs();
    let steer = (keys.pressed(kb.left) as i32 - keys.pressed(kb.right) as i32) as f32;
    // keyboard steering eases in/out
    let rate = if steer == 0.0 { 6.0 } else { 3.5 };
    car.steer_in += (steer - car.steer_in) * (rate * dt).min(1.0);
    let fwd = keys.pressed(kb.forward);
    let back = keys.pressed(kb.back);
    // W = throttle (or brake if rolling backwards), S = brake (or reverse once stopped)
    car.throttle = 0.0;
    car.brake = 0.0;
    if car.gear >= 0 {
        if fwd { car.throttle = 1.0; }
        if back { if car.v_long > 0.6 { car.brake = 1.0; } else { car.gear = -1; } }
    }
    if car.gear < 0 {
        if back { car.throttle = 1.0; }
        if fwd { if car.v_long < -0.6 { car.brake = 1.0; } else { car.gear = 1; } }
    }
    car.handbrake = keys.pressed(kb.handbrake);
    if keys.just_pressed(kb.ignition) {
        if car.running { car.running = false; } else if car.fuel > 0.0 { car.running = true; car.rpm = car.spec.idle_rpm; }
    }
    if keys.just_pressed(kb.lights) { car.lights = !car.lights; }
    if keys.just_pressed(kb.wipers) { car.wipers = (car.wipers + 1) % 3; }
}

fn ftire(alpha: f32, fz: f32, mu: f32, c: f32, fx: f32) -> f32 {
    // friction circle: lateral capacity shrinks as longitudinal force uses grip
    let cap = ((mu * fz).powi(2) - fx.min(mu * fz).powi(2)).max(1.0).sqrt();
    cap * (c * alpha / cap).tanh()
}

pub fn drive(
    time: Res<Time>,
    obstacles: Query<(&GlobalTransform, &Obstacle), Without<Car>>,
    mut crashes: MessageWriter<Crash>,
    mut q: Query<(Entity, &mut Car, &mut Transform, Option<&Driven>)>,
) {
    let frame_dt = time.delta_secs().min(0.05);
    for (entity, mut car, mut t, driven) in &mut q {
        let steps = 4;
        let dt = frame_dt / steps as f32;
        for _ in 0..steps {
            step(&mut car, &mut t, dt, driven.is_some(), entity, &obstacles, &mut crashes);
        }
    }
}

fn step(car: &mut Car, t: &mut Transform, dt: f32, driven: bool, entity: Entity, obstacles: &Query<(&GlobalTransform, &Obstacle), Without<Car>>, crashes: &mut MessageWriter<Crash>) {
    let s = car.spec.clone();
    let (a, b) = (s.cg_front, s.wheelbase - s.cg_front);
    let pos = t.translation;
    let surf = surface_at(pos.x, pos.z);
    let mu = mu(surf, car.tyres);
    let m = s.mass;
    let fz_f = m * G * b / s.wheelbase;
    let fz_r = m * G * a / s.wheelbase;

    // steering (speed sensitive; slower on keyboard)
    let target = car.steer_in * s.max_steer / (1.0 + car.v_long.abs() / 14.0);
    car.steer += (target - car.steer) * (8.0 * dt).min(1.0);
    let delta = car.steer;

    // --- engine / gearbox (automatic, simple)
    let ratio = match car.gear { -1 => s.reverse, 0 => 0.0, g => s.gears[(g as usize - 1).min(4)] } * s.final_drive;
    let wheel_rpm = (car.v_long.abs() / s.wheel_r) * 60.0 / std::f32::consts::TAU;
    let mut eng_rpm = (wheel_rpm * ratio).max(if car.running { s.idle_rpm } else { 0.0 });
    if car.running && car.gear > 0 {
        if eng_rpm > s.redline * 0.88 && car.gear < 5 { car.gear += 1; }
        else if eng_rpm < s.redline * 0.32 && car.gear > 1 && car.throttle > 0.0 { car.gear -= 1; }
    }
    if car.running && car.throttle > 0.0 && car.v_long.abs() < 3.0 { eng_rpm = eng_rpm.max(s.idle_rpm + 2200.0 * car.throttle); }
    car.rpm += (eng_rpm.min(s.redline) - car.rpm) * (10.0 * dt).min(1.0);
    let can_drive = car.running && car.fuel > 0.0;
    let mut drive_f = 0.0;
    if can_drive && car.throttle > 0.0 && car.rpm < s.redline {
        let tq = s.torque_curve(car.rpm) * car.throttle;
        drive_f = tq * ratio * 0.85 / s.wheel_r;
        if car.gear < 0 { drive_f = -drive_f; }
        car.fuel = (car.fuel - s.fuel_burn * car.throttle * dt).max(0.0);
        if car.fuel <= 0.0 { car.running = false; }
    } else if can_drive {
        car.fuel = (car.fuel - s.fuel_burn * 0.12 * dt).max(0.0); // idle burn
        // engine braking
        drive_f = -car.v_long.signum() * (car.rpm / s.redline) * 700.0 * (ratio / 12.0);
    }
    if !car.running && car.rpm > 0.0 { car.rpm = (car.rpm - 4000.0 * dt).max(0.0); }

    // --- brakes (60/40), handbrake locks rear
    let sgn = if car.v_long.abs() < 0.05 { 0.0 } else { car.v_long.signum() };
    let brake_total = car.brake * s.brake_force;
    let mut fx_f = -sgn * brake_total * 0.6;
    let mut fx_r = drive_f - sgn * brake_total * 0.4;
    let hb = car.handbrake;
    if hb { fx_r -= sgn * s.brake_force * 0.9; }
    // grip limits
    let maxf = mu * fz_f; let maxr = mu * fz_r * if hb { 0.35 } else { 1.0 };
    fx_f = fx_f.clamp(-maxf, maxf);
    let wheelspin = fx_r.abs() > maxr;
    fx_r = fx_r.clamp(-maxr, maxr);
    // lateral
    let vx = if car.v_long.abs() < 0.5 { 0.5 * if car.v_long >= 0.0 { 1.0 } else { -1.0 } } else { car.v_long };
    let alpha_f = (car.v_lat + a * car.yaw_rate).atan2(vx.abs()) * vx.signum() - delta;
    let alpha_r = (car.v_lat - b * car.yaw_rate).atan2(vx.abs()) * vx.signum();
    let mut fy_f = -ftire(alpha_f, fz_f, mu, s.corner_stiff, fx_f.abs());
    let mut fy_r = -ftire(alpha_r, fz_r * if hb { 0.45 } else { 1.0 }, mu, s.corner_stiff, fx_r.abs());
    // low speed: fade lateral forces so car doesn't buzz
    let fade = ((car.v_long.abs() - 0.2) / 2.0).clamp(0.0, 1.0);
    fy_f *= fade; fy_r *= fade;
    car.slip = ((alpha_r.abs() + alpha_f.abs()) / 0.5).clamp(0.0, 1.0).max(if wheelspin { 0.6 } else { 0.0 });

    // resistances
    let rr = rolling(surf) * m * G * sgn;
    let drag = 0.5 * 1.2 * s.cd_a * car.v_long * car.v_long.abs();
    let fx_total = fx_f * delta.cos() + fx_r - fy_f * delta.sin() - rr - drag;
    let fy_total = fy_r + fy_f * delta.cos() + fx_f * delta.sin();
    let torque = a * (fy_f * delta.cos() + fx_f * delta.sin()) - b * fy_r;

    let dvx = fx_total / m + car.v_lat * car.yaw_rate;
    let dvy = fy_total / m - car.v_long * car.yaw_rate;
    car.v_long += dvx * dt;
    car.v_lat += dvy * dt;
    car.yaw_rate += torque / s.iz * dt;
    // standstill damping
    if car.v_long.abs() < 0.3 && car.throttle == 0.0 {
        car.v_long *= 1.0 - (6.0 * dt).min(1.0);
        car.v_lat *= 1.0 - (6.0 * dt).min(1.0);
        car.yaw_rate *= 1.0 - (6.0 * dt).min(1.0);
    }
    car.yaw += car.yaw_rate * dt;

    // world integration (x forward, y LEFT)
    let (sy, cy) = car.yaw.sin_cos();
    let fwd = Vec3::new(-sy, 0.0, -cy);
    let left = Vec3::new(-cy, 0.0, sy);
    let mut np = t.translation + (fwd * car.v_long + left * car.v_lat) * dt;

    // --- collisions with trees/rocks (two circles: front & rear of car)
    for off in [-1.5f32, 0.0, 1.5] {
        let c = np + fwd * off;
        for (g, o) in obstacles.iter() {
            let d = Vec2::new(c.x - g.translation().x, c.z - g.translation().z);
            let r = o.0 + 0.95;
            let l2 = d.length_squared();
            if l2 < r * r && l2 > 1e-6 {
                let n = d.normalize();
                let n3 = Vec3::new(n.x, 0.0, n.y);
                let world_v = fwd * car.v_long + left * car.v_lat;
                let closing = -world_v.dot(n3);
                np.x += n.x * (r - l2.sqrt()); np.z += n.y * (r - l2.sqrt());
                if closing > 0.0 {
                    let nv = world_v + n3 * closing * 1.35; // bounce
                    car.v_long = nv.dot(fwd) * 0.7; car.v_lat = nv.dot(left) * 0.7; car.yaw_rate *= 0.6;
                    if closing > 2.0 { crashes.write(Crash { car: entity, speed: closing, point: c }); }
                }
            }
        }
    }
    np.x = np.x.clamp(-WORLD_HALF + 5.0, WORLD_HALF - 5.0); np.z = np.z.clamp(-WORLD_HALF + 5.0, WORLD_HALF - 5.0);

    // --- terrain following
    let hw = s.track / 2.0; let hl = s.wheelbase / 2.0;
    let h = |lx: f32, lz: f32| { let p = np + left * (-lx) + fwd * (-lz); height_at(p.x, p.z) };
    // lx>0 = right, lz<0 = front (matches visuals)
    let (hfl, hfr, hrl, hrr) = (h(-hw, -hl), h(hw, -hl), h(-hw, hl), h(hw, hl));
    let hf = (hfl + hfr) / 2.0; let hr = (hrl + hrr) / 2.0; let hleft = (hfl + hrl) / 2.0; let hright = (hfr + hrr) / 2.0;
    car.pitch += (((hf - hr) / s.wheelbase).atan() - car.pitch) * (8.0 * dt).min(1.0);
    car.roll += (((hright - hleft) / s.track).atan() - car.roll) * (8.0 * dt).min(1.0);
    car.ground_y = (hf + hr) / 2.0;
    np.y = car.ground_y;
    t.translation = np;
    t.rotation = Quat::from_euler(EulerRot::YXZ, car.yaw, car.pitch, car.roll);
    // gravity assist on slopes (so cars roll back on hills)
    let n = normal_at(np.x, np.z);
    let slope_acc = Vec3::new(n.x, 0.0, n.z) * G; // downhill direction * g * sin
    car.v_long += slope_acc.dot(fwd) * dt * 0.9 * if car.brake > 0.1 || hb { 0.0 } else { 1.0 };
    car.v_lat += slope_acc.dot(left) * dt * 0.9;
    car.wheel_spin += car.v_long / s.wheel_r * dt;
    let _ = driven;
}

pub fn visuals(
    mut wheels: Query<(&WheelVisual, &mut Transform, &ChildOf), Without<Car>>,
    cars: Query<&Car>,
    mut lights: Query<(&mut SpotLight, &ChildOf), With<Headlight>>,
) {
    for (w, mut tr, p) in &mut wheels {
        if let Ok(c) = cars.get(p.parent()) {
            let steer = if w.front { c.steer } else { 0.0 };
            tr.rotation = Quat::from_rotation_y(steer) * Quat::from_rotation_x(-c.wheel_spin);
        }
    }
    for (mut l, p) in &mut lights {
        if let Ok(c) = cars.get(p.parent()) { l.intensity = if c.lights { 3_000_000.0 } else { 0.0 }; }
    }
}
