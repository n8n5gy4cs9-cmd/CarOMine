#!/usr/bin/env python3
"""T-032: first-person weapon + hand sheets, SDF-rendered (Doom / ROTT-style shaded pixel art), perspective camera.
Frame 192x144. cols: 0 idle, 1 fire (flash), 2 recoil, 3-6 reload stages (fists: 3 = left punch, 4 = right wind-up).
Camera space: x right, y up, camera at origin looking -Z; objects live around z=-0.8."""
import json, sys
import numpy as np
from PIL import Image
from assetlib import *
from sdfrender import *

W, H, COLS = 192, 144, 8
XR, YR = (-0.75, 0.75), (-0.5625, 0.5625)
def V(*a): return np.array(a, float)

GLOVE = Mat((46, 40, 38), noise="cloth", nscale=140, namp=0.07, spec=0.15)
GLOVE_HI = Mat((74, 64, 58), noise="cloth", nscale=140, namp=0.07, spec=0.2)
SLEEVE = Mat((52, 64, 88), noise="cloth", nscale=70, namp=0.08)
CUFF = Mat((30, 34, 46), noise="cloth", nscale=110, namp=0.05)
STEEL = Mat((88, 92, 104), noise="streak", nscale=150, namp=0.04, spec=0.9)
BLUED = Mat((34, 36, 44), noise="streak", nscale=120, namp=0.04, spec=0.85)
BRIGHT = Mat((170, 176, 188), spec=0.95)
WOOD = Mat((126, 78, 40), noise="streak", nscale=60, namp=0.13, spec=0.2)
WOOD_D = Mat((86, 52, 28), noise="streak", nscale=60, namp=0.12, spec=0.2)
POLY = Mat((30, 32, 36), noise="cloth", nscale=200, namp=0.05, spec=0.2)
BRASS = Mat((200, 160, 60), spec=0.9)
RED = Mat((176, 30, 28), spec=0.3)
FLASH = Mat((255, 236, 140), emissive=True); FLASH2 = Mat((255, 255, 226), emissive=True)

def hand(s, c, R=None, side=1, fingers=True, glove=GLOVE, sleeve_to=None, open_=0.0, k=1.35):
    """Gloved hand gripping something: back of hand + finger knuckles + curled fingers + thumb + cuff + sleeve (k = size factor)."""
    c = V(*c); R = np.eye(3) if R is None else R
    s.ellipsoid(c, V(0.034, 0.022, 0.05) * k, glove, rot=R)                                                    # back of hand
    for i in range(4):
        x = (i - 1.5) * 0.019 * k
        kn = c + R @ V(x * side, -0.004 * k, -0.045 * k)
        s.sphere(kn, 0.0105 * k, GLOVE_HI)                                                                     # knuckle
        if fingers: s.capsule(kn, kn + R @ V(x * side * 0.9, (-0.035 - open_ * 0.02) * k, (-0.016 + open_ * 0.03) * k), 0.0098 * k, 0.0088 * k, glove)
    s.capsule(c + R @ V(-0.026 * side * k, -0.004 * k, -0.02 * k), c + R @ V(-0.02 * side * k, -0.014 * k, -0.075 * k), 0.0115 * k, 0.0095 * k, glove)   # thumb
    wr = c + R @ V(0.0, 0.0, 0.06 * k)
    s.capsule(wr, wr + R @ V(0.0, 0.02, 0.07 * k), 0.034 * k, 0.038 * k, CUFF)                                 # cuff
    end = c + R @ V(0.02 * side, 0.1, 0.38) if sleeve_to is None else V(*sleeve_to)
    s.capsule(wr + R @ V(0.0, 0.02, 0.06 * k), end, 0.036 * k, 0.05 * k, SLEEVE)                              # forearm sleeve

def look_rot(yaw=0.0, pitch=0.0, roll=0.0): return rot_y(yaw) @ rot_x(pitch) @ rot_z(roll)

# ----------------------------------------------------------- weapons
def pistol(p):
    s = Scene()
    rec = p.get("recoil", 0.0); tilt = p.get("tilt", 0.0)
    off = V(0.03, -0.17 - p.get("dip", 0.0), -0.78 + 0.07 * rec)
    R = rot_x(0.05 + 0.18 * rec + tilt)
    g = Scene()
    slide_back = 0.05 * rec
    g.box(V(0, 0.0, -0.01 + slide_back), V(0.021, 0.025, 0.118), BLUED, rnd=0.008)                       # slide
    for i in range(6): g.box(V(0.0215, 0.002, 0.045 + slide_back + i * 0.009), V(0.002, 0.018, 0.003), STEEL, rnd=0.001)   # serrations
    g.box(V(0, 0.006, -0.121 + slide_back), V(0.012, 0.012, 0.012), STEEL, rnd=0.004)                    # muzzle crown
    g.box(V(0, -0.04, 0.0), V(0.019, 0.017, 0.105), POLY, rnd=0.006)                                      # frame
    g.box(V(0, 0.029, 0.09), V(0.006, 0.006, 0.006), BRIGHT, rnd=0.002)                                    # rear sight
    g.box(V(0, 0.03, -0.1), V(0.004, 0.007, 0.004), BRIGHT, rnd=0.002)                                     # front sight
    g.box(V(0, -0.11, 0.075), V(0.021, 0.06, 0.032), POLY, rot=rot_x(0.28), rnd=0.01)                     # grip
    g.capsule(V(0, -0.068, -0.02), V(0, -0.075, -0.05), 0.006, 0.006, BLUED)                              # trigger guard front
    g.box(V(0, -0.062, -0.037), V(0.003, 0.01, 0.025), BLUED, rnd=0.002)
    if p.get("mag_out", 0.0) > 0: g.box(V(0.0, -0.21 - 0.12 * p["mag_out"], 0.095), V(0.018, 0.045, 0.022), BLUED, rot=rot_x(0.28), rnd=0.006)
    s.merge(g.transform(R, off))
    # hands
    hr = off + R @ V(0.0, -0.11, 0.075)
    hand(s, hr, R @ rot_x(-0.1), side=1, sleeve_to=hr + V(0.3, -0.75, 0.14))
    hl = off + R @ V(-0.01, -0.15, 0.06) + V(-0.03, -0.02 - 0.12 * p.get("lhand", 0.0), 0.0)
    hand(s, hl, R @ rot_x(-0.2) @ rot_y(0.5), side=-1, sleeve_to=hl + V(-0.3, -0.75, 0.14))
    tip = off + R @ V(0, 0.005, -0.14)
    return s, tip

def shotgun(p):
    s = Scene()
    rec = p.get("recoil", 0.0); tilt = p.get("tilt", 0.0); pump = p.get("pump", 0.0)
    off = V(0.06, -0.19 - p.get("dip", 0.0), -0.82 + 0.1 * rec)
    R = rot_x(0.04 + 0.22 * rec + tilt) @ rot_y(-0.03)
    g = Scene()
    g.capsule(V(0, 0.02, -0.0), V(0, 0.02, -0.78), 0.0165, 0.0155, BLUED)                                # barrel
    g.capsule(V(0, -0.012, -0.0), V(0, -0.012, -0.62), 0.016, 0.016, BLUED)                              # magazine tube
    g.box(V(0, 0.034, -0.4), V(0.004, 0.006, 0.3), STEEL, rnd=0.002)                                      # vent rib
    g.sphere(V(0, 0.046, -0.77), 0.0075, BRASS)                                                           # bead
    g.box(V(0, 0.0, 0.07), V(0.026, 0.034, 0.1), BLUED, rnd=0.01)                                         # receiver
    g.box(V(0.0265, 0.005, 0.06), V(0.002, 0.012, 0.045), STEEL, rnd=0.001)                              # ejection port
    g.box(V(0, -0.012 + 0.0, -0.3 + pump * 0.14), V(0.03, 0.026, 0.1), WOOD, rnd=0.012)                  # pump forend
    for i in range(5): g.box(V(0, -0.038, -0.3 + pump * 0.14 + (i - 2) * 0.03), V(0.026, 0.003, 0.007), WOOD_D, rnd=0.001)   # grooves
    g.capsule(V(0, -0.02, 0.15), V(0, -0.05, 0.42), 0.032, 0.04, WOOD)                                   # stock toward camera
    g.box(V(0, -0.075, 0.1), V(0.02, 0.03, 0.04), WOOD, rot=rot_x(0.3), rnd=0.01)                        # pistol grip
    g.box(V(0, -0.05, 0.075), V(0.003, 0.02, 0.02), BLUED, rnd=0.001)                                    # trigger
    s.merge(g.transform(R, off))
    hr = off + R @ V(0.0, -0.085, 0.1)
    hand(s, hr, R @ rot_x(-0.1), side=1, sleeve_to=hr + V(0.3, -0.75, 0.14))
    hl = off + R @ V(0.0, -0.012 - 0.0, -0.3 + pump * 0.14) + V(0.0, -0.045, 0)
    hand(s, hl, R @ rot_x(0.05) @ rot_y(0.2), side=-1, sleeve_to=hl + V(-0.3, -0.75, 0.14))
    if p.get("shell", 0.0) > 0: s.capsule(off + V(-0.07, -0.06 - 0.02 * p["shell"], 0.0), off + V(-0.07, -0.02, 0.0), 0.011, 0.011, RED)
    tip = off + R @ V(0, 0.02, -0.8)
    return s, tip

def smg(p):
    s = Scene()
    rec = p.get("recoil", 0.0); tilt = p.get("tilt", 0.0)
    off = V(0.05, -0.17 - p.get("dip", 0.0), -0.8 + 0.05 * rec)
    R = rot_x(0.04 + 0.1 * rec + tilt) @ rot_y(-0.02) @ rot_z(0.02 * np.sin(rec * 30))
    g = Scene()
    g.box(V(0, 0.0, 0.0), V(0.028, 0.03, 0.17), POLY, rnd=0.009)                                           # receiver
    g.box(V(0, 0.034, 0.02), V(0.012, 0.006, 0.15), BLUED, rnd=0.003)                                      # top rail
    g.capsule(V(0, 0.004, -0.15), V(0, 0.004, -0.36), 0.019, 0.019, BLUED)                                 # shroud
    for i in range(7): g.box(V(0, 0.004, -0.17 - i * 0.026), V(0.0195, 0.0195, 0.004), STEEL, rnd=0.001)    # shroud rings
    g.capsule(V(0, 0.004, -0.36), V(0, 0.004, -0.4), 0.0075, 0.0075, STEEL)                                # muzzle
    g.box(V(0, -0.12 - p.get("mag_out", 0.0) * 0.14, 0.04), V(0.018, 0.085, 0.03), BLUED, rot=rot_x(0.12), rnd=0.008)  # magazine
    g.box(V(0, -0.075, 0.12), V(0.02, 0.05, 0.03), POLY, rot=rot_x(0.3), rnd=0.008)                       # pistol grip
    g.capsule(V(0, 0.01, 0.17), V(0, 0.0, 0.42), 0.007, 0.007, STEEL)                                      # wire stock
    g.box(V(0, 0.0, 0.43), V(0.02, 0.03, 0.01), POLY, rnd=0.004)
    g.box(V(0, -0.03, -0.2), V(0.016, 0.02, 0.05), POLY, rnd=0.006)                                        # foregrip
    g.box(V(0.0285, 0.012, 0.05 - 0.04 * rec), V(0.002, 0.008, 0.02), STEEL, rnd=0.001)                   # charging handle slot
    s.merge(g.transform(R, off))
    hr = off + R @ V(0.0, -0.075, 0.12)
    hand(s, hr, R @ rot_x(-0.1), side=1, sleeve_to=hr + V(0.3, -0.75, 0.14))
    hl = off + R @ V(0.0, -0.065, -0.2) + V(0, -0.1 * p.get("lhand", 0.0), 0.0)
    hand(s, hl, R @ rot_x(0.1) @ rot_y(0.25), side=-1, sleeve_to=hl + V(-0.3, -0.75, 0.14))
    tip = off + R @ V(0, 0.004, -0.42)
    return s, tip

def rifle(p):
    s = Scene()
    rec = p.get("recoil", 0.0); tilt = p.get("tilt", 0.0); bolt = p.get("bolt", 0.0)
    off = V(0.06, -0.2 - p.get("dip", 0.0), -0.84 + 0.12 * rec)
    R = rot_x(0.03 + 0.25 * rec + tilt) @ rot_y(-0.03)
    g = Scene()
    g.capsule(V(0, 0.02, 0.0), V(0, 0.02, -0.85), 0.016, 0.0125, BLUED)                                    # barrel
    g.box(V(0, -0.015, -0.34), V(0.026, 0.028, 0.34), WOOD, rnd=0.012)                                      # forend
    g.box(V(0, 0.0, 0.1), V(0.026, 0.034, 0.11), BLUED, rnd=0.008)                                          # receiver
    g.box(V(0, 0.062, 0.08), V(0.012, 0.012, 0.1), BLUED, rnd=0.004)                                        # scope mount
    g.capsule(V(0, 0.082, -0.05), V(0, 0.082, 0.17), 0.026, 0.026, POLY)                                     # scope tube
    g.capsule(V(0, 0.082, -0.07), V(0, 0.082, -0.1), 0.032, 0.032, POLY); g.capsule(V(0, 0.082, 0.15), V(0, 0.082, 0.19), 0.03, 0.03, POLY)   # bells
    g.sphere(V(0, 0.082, 0.205), 0.0165, Mat((40, 60, 80), spec=1.0))                                       # eyepiece glass
    g.capsule(V(0.03, 0.01, 0.1 + 0.05 * bolt), V(0.075, -0.012, 0.1 + 0.05 * bolt), 0.006, 0.006, STEEL); g.sphere(V(0.08, -0.012, 0.1 + 0.05 * bolt), 0.012, STEEL)   # bolt handle
    g.box(V(0, -0.03, 0.28), V(0.024, 0.045, 0.2), WOOD, rot=rot_x(-0.08), rnd=0.014)                       # stock
    g.box(V(0, -0.055, 0.08), V(0.02, 0.04, 0.03), WOOD, rot=rot_x(0.3), rnd=0.01)                         # grip
    g.sphere(V(0, 0.034, -0.83), 0.006, BRASS)
    s.merge(g.transform(R, off))
    hr = off + R @ V(0.0, -0.06, 0.09)
    hand(s, hr, R @ rot_x(-0.1), side=1, sleeve_to=hr + V(0.3, -0.75, 0.14))
    hl = off + R @ V(0.0, -0.045, -0.4) + V(0, -0.05 * p.get("lhand", 0.0), 0.0)
    hand(s, hl, R @ rot_x(0.05) @ rot_y(0.2), side=-1, sleeve_to=hl + V(-0.3, -0.75, 0.14))
    tip = off + R @ V(0, 0.02, -0.87)
    return s, tip

def fists(p):
    s = Scene()
    pr = p.get("punch_r", 0.0); pl = p.get("punch_l", 0.0)
    rbase = V(0.17, -0.12, -0.5); lbase = V(-0.17, -0.12, -0.5)
    rc = rbase + V(-0.1 * pr, 0.07 * pr, -0.38 * pr); lc = lbase + V(0.1 * pl, 0.07 * pl, -0.38 * pl)
    hand(s, rc, rot_x(0.15) @ rot_y(0.12 * pr), side=1, sleeve_to=rc + V(0.3, -0.75, 0.14))
    hand(s, lc, rot_x(0.15) @ rot_y(-0.12 * pl), side=-1, sleeve_to=lc + V(-0.3, -0.75, 0.14))
    return s, None

def muzzle(s, tip, big=1.0):
    if tip is None: return
    s.sphere(tip + V(0, 0, -0.05 * big), 0.05 * big, FLASH); s.sphere(tip + V(0, 0, -0.1 * big), 0.03 * big, FLASH2)
    for sx, sy in ((1, 0), (-1, 0), (0, 1), (0.7, 0.7), (-0.7, 0.7)): s.capsule(tip, tip + V(sx * 0.1 * big, sy * 0.1 * big, -0.06 * big), 0.014 * big, 0.002, FLASH)

WEAPONS = {
    "pistol": dict(fn=pistol, flash=0.8, reload=[dict(tilt=0.25, dip=0.04), dict(tilt=0.3, dip=0.06, mag_out=1.0), dict(tilt=0.28, dip=0.05, mag_out=0.3, lhand=0.6), dict(tilt=0.05, recoil=0.3)]),
    "shotgun": dict(fn=shotgun, flash=1.3, fire_extra=dict(pump=0.0), reload=[dict(tilt=0.35, dip=0.05, shell=1.0), dict(tilt=0.4, dip=0.07, shell=0.3), dict(tilt=0.2, dip=0.03), dict(pump=1.0)]),
    "smg": dict(fn=smg, flash=0.9, reload=[dict(tilt=0.2, dip=0.05, mag_out=0.5), dict(tilt=0.28, dip=0.09, mag_out=1.0), dict(tilt=0.25, dip=0.06, mag_out=0.2, lhand=0.5), dict(tilt=0.04)]),
    "rifle": dict(fn=rifle, flash=1.2, reload=[dict(tilt=0.2, dip=0.04, bolt=1.0), dict(tilt=0.28, dip=0.07, bolt=1.0, lhand=0.5), dict(tilt=0.2, dip=0.05, bolt=1.0), dict(tilt=0.03, bolt=0.0)]),
    "fists": dict(fn=fists, flash=0),
}

# view-model placement: (idle offset used inside each builder, final camera-space position, yaw toward the crosshair)
PLACE = {"pistol": ((0.03, -0.17, -0.78), (0.07, -0.085, -0.42), 0.14), "shotgun": ((0.06, -0.19, -0.82), (0.13, -0.15, -0.56), 0.12),
         "smg": ((0.05, -0.17, -0.80), (0.13, -0.135, -0.55), 0.12), "rifle": ((0.06, -0.20, -0.84), (0.14, -0.165, -0.58), 0.10)}

def place(name, sc, tip):
    if name not in PLACE: return sc, tip
    base, T, yaw = PLACE[name]; R = rot_y(yaw)
    t = V(*T) - R @ V(*base)
    sc2 = sc.transform(R, t)
    return sc2, (None if tip is None else R @ tip + t)

def frames_for(name, cfg):
    fn = cfg["fn"]; out = []
    if name == "fists":
        poses = [dict(), dict(punch_r=1.0), dict(punch_r=0.45), dict(punch_l=1.0), dict(punch_l=0.45), dict(), dict(), dict()]
        for p in poses: out.append(place(name, fn(p)[0], None)[0])
        return out
    def mk(p, flash=False):
        sc, tip = fn(p); sc, tip = place(name, sc, tip)
        if flash: muzzle(sc, tip, cfg["flash"])
        return sc
    out.append(mk(dict()))
    out.append(mk(dict(recoil=0.6), True))                                               # fire: recoil + flash
    out.append(mk(dict(recoil=1.0)))                                                     # recoil settle
    for rp in cfg["reload"]: out.append(mk(rp))                                          # 4 reload stages
    return out

def main(only=None):
    meta = {}
    for name, cfg in WEAPONS.items():
        if only and name not in only: continue
        sheet = Image.new("RGBA", (W * COLS, H), (0, 0, 0, 0))
        for i, sc in enumerate(frames_for(name, cfg)):
            img = render(sc, W, H, x_range=XR, y_range=YR, perspective=(0.0, 1.0), steps=60, light=(-0.4, 0.7, 0.6), ambient=0.34, outline=(8, 6, 12), edge=0.05)
            sheet.alpha_composite(img, (i * W, 0))
        rel = f"sprites/weapons/{name}.png"; sheet.save(out(rel))
        reg(f"weapon.{name}", rel, "sprite", f"First-person weapon sheet '{name}' with gloved hands: {COLS} frames {W}x{H} (0 idle, 1 fire, 2 recoil, 3-6 reload; fists: 1 right punch, 2 right retract, 3 left punch, 4 left retract)",
            source="generated (tools/gen_weapons.py, SDF-rendered)", notes="Replace keeping the 8x1 grid; HUD reads assets/data/sprites.json")
        meta[f"weapon.{name}"] = dict(frame=[W, H], cols=COLS, rows=1, anims=dict(idle=[0], fire=[1, 2], reload=[3, 4, 5, 6]))
        print(name, "ok", flush=True)
    p = out("data/sprites.json"); cur = json.load(open(p)) if p.exists() else {}; cur.update(meta); json.dump(cur, open(p, "w"), indent=1)

if __name__ == "__main__":
    main(sys.argv[1:] or None)
