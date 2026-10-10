"""Quadruped rigs (wolf, moose). Faces +Z (toward camera at view angle 0), x = character's left."""
import numpy as np
from sdfrender import *
from rig_humanoid import V, ik2

def leg4(s, hip, foot, l1, l2, r1, r2, m, hoof):
    knee = ik2(hip, foot, l1, l2, V(0, 0, -1.0))
    s.capsule(hip, knee, r1, r1 * 0.7, m); s.capsule(knee, foot, r1 * 0.7, r2, m)
    s.capsule(foot + V(0, 0.02, 0), foot + V(0, -0.0, 0.05), r2, r2 * 0.9, hoof)

def gait(frame, sp):
    """frame index -> dict(phase, lunge, pain, death). walk frames 1-4."""
    if frame == 0: return dict(phase=0.0, amp=0.0)
    if 1 <= frame <= 4: return dict(phase=(frame - 1) * np.pi / 2, amp=1.0)
    if frame == 5: return dict(phase=0.0, amp=0.0, lunge=0.5)
    if frame == 6: return dict(phase=0.0, amp=0.0, lunge=1.0, open=1.0)
    if frame == 7: return dict(phase=0.0, amp=0.0, pain=1.0)
    return dict(phase=0.0, amp=0.0, death=(frame - 7) / 4.0, pain=0.5)

def wolf(g):
    s = Scene()
    fur = Mat((112, 108, 106), noise="streak", nscale=70, namp=0.2); fur2 = Mat((170, 164, 156), noise="streak", nscale=70, namp=0.2)
    dark = Mat((54, 50, 52), noise="streak", nscale=80, namp=0.15); claw = Mat((30, 28, 30), spec=0.4)
    amp, ph = g.get("amp", 0), g["phase"]; lunge = g.get("lunge", 0.0); pain = g.get("pain", 0.0)
    bob = 0.015 * abs(np.sin(ph)) * amp
    body_y = 0.62 - bob - 0.05 * lunge
    s.ellipsoid(V(0, body_y, -0.02), V(0.17, 0.2, 0.42), fur)                                       # barrel
    s.ellipsoid(V(0, body_y + 0.03, 0.26), V(0.19, 0.23, 0.2), fur)                                 # chest/shoulders
    s.ellipsoid(V(0, body_y - 0.06, 0.28), V(0.12, 0.12, 0.15), fur2)                               # lighter chest
    s.ellipsoid(V(0, body_y, -0.26), V(0.15, 0.17, 0.2), fur)                                       # haunch
    s.capsule(V(0, body_y + 0.1, 0.4), V(0, body_y + 0.2 + 0.05 * lunge, 0.58), 0.1, 0.08, fur)       # neck
    hc = V(0, body_y + 0.2 + 0.04 * lunge - pain * 0.05, 0.62 + 0.06 * lunge)
    s.ellipsoid(hc, V(0.09, 0.09, 0.12), fur)                                                         # skull
    jaw_open = g.get("open", 0.0) * 0.07
    s.capsule(hc + V(0, -0.02, 0.07), hc + V(0, -0.04, 0.23), 0.052, 0.034, fur2)                    # snout
    s.capsule(hc + V(0, -0.075 - jaw_open, 0.06), hc + V(0, -0.075 - jaw_open * 1.5, 0.2), 0.03, 0.022, fur2)  # lower jaw
    s.sphere(hc + V(0, -0.0, 0.245), 0.026, Mat((20, 18, 20), spec=0.6))                             # nose
    if jaw_open > 0:
        s.box(hc + V(0, -0.045 - jaw_open * 0.4, 0.17), V(0.03, 0.006, 0.05), Mat((190, 60, 70)), rnd=0.003)
        for sx in (-1, 1): s.capsule(hc + V(sx * 0.022, -0.04, 0.2), hc + V(sx * 0.022, -0.075 - jaw_open * 0.8, 0.21), 0.008, 0.003, Mat((245, 245, 235)))
    for sx in (-1, 1):
        s.ellipsoid(hc + V(sx * 0.065, 0.1, -0.02), V(0.03, 0.06, 0.025), dark, rot=rot_z(sx * -0.2))   # ears
        s.sphere(hc + V(sx * 0.052, 0.025, 0.085), 0.016, Mat((250, 210, 60), emissive=False, spec=0.9)); s.sphere(hc + V(sx * 0.052, 0.025, 0.097), 0.008, Mat((10, 10, 10)))
    tail_up = 0.0 if amp else 0.1
    s.capsule(V(0, body_y + 0.02, -0.4), V(0, body_y - 0.12 + tail_up, -0.68), 0.05, 0.075, fur); s.capsule(V(0, body_y - 0.12 + tail_up, -0.68), V(0, body_y - 0.28, -0.82), 0.075, 0.03, dark)
    # legs: diagonal gait
    for (zx, zz, front) in ((0.1, 0.27, True), (-0.1, 0.27, True), (0.1, -0.27, False), (-0.1, -0.27, False)):
        sgn = 1 if zx > 0 else -1
        pp = ph + (0 if (front and zx > 0) or (not front and zx < 0) else np.pi)
        sw = 0.22 * np.sin(pp) * amp; lift = max(0.0, np.cos(pp)) * 0.1 * amp
        reach = 0.1 * lunge if front else -0.08 * lunge
        hip = V(zx * 1.0, body_y - 0.06, zz); foot = V(zx * 1.05, 0.03 + lift, zz + sw + reach)
        leg4(s, hip, foot, 0.3, 0.3, 0.065, 0.032, fur if front else dark if not front else fur, claw)
    return s

def moose(g):
    s = Scene()
    brown = Mat((70, 48, 34), noise="streak", nscale=60, namp=0.16); light = Mat((150, 126, 100), noise="streak", nscale=60, namp=0.16)
    dark = Mat((34, 24, 20), noise="streak", nscale=60, namp=0.12); horn = Mat((205, 190, 150), noise="streak", nscale=50, namp=0.12, spec=0.1)
    hoof = Mat((24, 20, 20), spec=0.4)
    amp, ph = g.get("amp", 0), g["phase"]; lunge = g.get("lunge", 0.0); pain = g.get("pain", 0.0)
    bob = 0.03 * abs(np.sin(ph)) * amp
    by = 1.48 - bob - 0.08 * lunge
    s.ellipsoid(V(0, by, -0.1), V(0.4, 0.42, 0.85), brown)
    s.ellipsoid(V(0, by + 0.18, 0.5), V(0.3, 0.42, 0.42), brown)                                      # shoulder hump
    s.ellipsoid(V(0, by - 0.05, -0.62), V(0.34, 0.36, 0.34), dark)                                    # hindquarters
    s.capsule(V(0, by + 0.28, 0.7), V(0, by + 0.5 + 0.12 * lunge, 1.12 + 0.15 * lunge), 0.2, 0.15, brown)   # neck
    hc = V(0, by + 0.45 + 0.1 * lunge - pain * 0.1, 1.28 + 0.2 * lunge)
    s.ellipsoid(hc, V(0.15, 0.21, 0.3), dark, rot=rot_x(-0.45))                                      # long head
    s.ellipsoid(hc + V(0, -0.2, 0.28), V(0.13, 0.12, 0.16), light, rot=rot_x(-0.45))                  # bulbous muzzle
    s.sphere(hc + V(0, -0.15, 0.4), 0.05, Mat((20, 16, 16), spec=0.5))
    s.capsule(hc + V(0, -0.25, 0.12), hc + V(0, -0.58, 0.05), 0.04, 0.025, dark)                      # bell (dewlap)
    for sx in (-1, 1):
        s.ellipsoid(hc + V(sx * 0.18, 0.12, -0.12), V(0.05, 0.12, 0.05), brown, rot=rot_z(sx * -0.8))   # ears
        s.sphere(hc + V(sx * 0.12, 0.02, 0.18), 0.022, Mat((14, 12, 12), spec=0.9))
        # palmate antlers
        base = hc + V(sx * 0.1, 0.18, -0.05); beam = base + V(sx * 0.42, 0.18, 0.05)
        s.capsule(base, beam, 0.035, 0.03, horn)
        s.ellipsoid(beam + V(sx * 0.18, 0.06, 0.0), V(0.26, 0.035, 0.2), horn, rot=rot_z(sx * -0.2))     # palm
        for i in range(5):
            z = -0.15 + i * 0.075; s.capsule(beam + V(sx * 0.26, 0.07, z), beam + V(sx * 0.38, 0.2 + 0.03 * (i % 2), z + 0.03), 0.018, 0.008, horn)
        s.capsule(base + V(sx * 0.08, 0.04, 0.06), base + V(sx * 0.2, 0.28, 0.2), 0.02, 0.01, horn)       # brow tine
    s.capsule(V(0, by + 0.1, -0.9), V(0, by - 0.05, -1.0), 0.05, 0.03, dark)
    for (zx, zz, front) in ((0.18, 0.55, True), (-0.18, 0.55, True), (0.18, -0.6, False), (-0.18, -0.6, False)):
        pp = ph + (0 if (front and zx > 0) or (not front and zx < 0) else np.pi)
        sw = 0.4 * np.sin(pp) * amp; lift = max(0.0, np.cos(pp)) * 0.2 * amp
        reach = 0.2 * lunge if front else -0.15 * lunge
        hip = V(zx * 1.0, by - 0.18, zz); foot = V(zx * 1.05, 0.05 + lift, zz + sw + reach)
        knee = ik2(hip, foot, 0.62, 0.62, V(0, 0, -1.0))
        s.capsule(hip, knee, 0.11, 0.07, brown); s.capsule(knee, foot, 0.07, 0.045, light)
        s.capsule(foot + V(0, 0.03, 0), foot + V(0, 0.0, 0.07), 0.05, 0.045, hoof)
    return s

def apply_death(s, g):
    d = g.get("death", 0.0)
    if d <= 0: return s
    # fall onto the side: rotate about the z axis (lying on left side)
    s = s.transform(rot_z(d * (np.pi / 2 - 0.1)))
    return s.transform(None, (0.0, 0.0, 0.0))

QUADS = {
    "wolf": dict(fn=wolf, px=(112, 64), world=(1.75, 1.0), x_range=(-0.875, 0.875), y_range=(0.0, 1.0), zc=0.0),
    "moose": dict(fn=moose, px=(144, 112), world=(3.2, 2.5), x_range=(-1.6, 1.6), y_range=(0.0, 2.5), zc=0.35),
}

def render_quad(name, cfg, COLS, ROWS):
    from PIL import Image
    W, H = cfg["px"]; sheet = Image.new("RGBA", (W * COLS, H * ROWS), (0, 0, 0, 0))
    for fi in range(COLS):
        g = gait(fi, name)
        s = cfg["fn"](g)
        s = s.transform(None, (0, 0, -cfg.get("zc", 0.0))).scaled(cfg.get("fit", 0.88))   # centre on the pivot + shrink so side views fit the frame
        s = apply_death(s, g)
        if g.get("death", 0) > 0:
            # drop the lying body to the ground: crude lift so it rests at y~=body radius
            r = 0.18 if name == "wolf" else 0.4
            s = s.transform(None, (0, r * g["death"], 0)).scaled(1.0 - 0.08 * g["death"])
        for r_ in range(ROWS):
            rel = r_ * np.pi / 4
            img = render(s.transform(rot_y(-rel)), W, H, x_range=cfg["x_range"], y_range=cfg["y_range"])
            sheet.alpha_composite(img, (fi * W, r_ * H))
    return sheet
