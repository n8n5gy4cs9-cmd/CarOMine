#!/usr/bin/env python3
"""T-033: world sprites (trees, bushes, rocks, fences, lamps, items) rendered from 3D models with tools/sdfrender.py.
Manifest ids / paths / aspect ratios are unchanged so the game picks them up as drop-in replacements."""
import sys, json
import numpy as np
from PIL import Image
from assetlib import *
from sdfrender import *
from rig_humanoid import V

def rnd(seed): return np.random.default_rng(seed)

BARK = Mat((92, 62, 40), noise="streak", nscale=60, namp=0.16)
BARK_D = Mat((54, 38, 30), noise="streak", nscale=70, namp=0.14)
NEEDLE = Mat((34, 74, 48), noise="cloth", nscale=45, namp=0.14, snow=0.55)
NEEDLE2 = Mat((46, 96, 58), noise="cloth", nscale=45, namp=0.14, snow=0.55)
PINE_N = Mat((52, 92, 52), noise="cloth", nscale=40, namp=0.14, snow=0.5)
BIRCH = Mat((226, 224, 214), noise="streak", nscale=30, namp=0.08)
BIRCH_MARK = Mat((24, 22, 24))
TWIG = Mat((78, 58, 44), noise="streak", nscale=60, namp=0.12, snow=0.7)
ROCK = Mat((104, 106, 114), noise="cloth", nscale=28, namp=0.2, snow=0.65)
LEAF = Mat((40, 78, 44), noise="cloth", nscale=34, namp=0.16, snow=0.6)
WOODF = Mat((120, 88, 58), noise="streak", nscale=55, namp=0.12, snow=0.7)
SNOW = Mat((240, 244, 252), noise="cloth", nscale=22, namp=0.05)
METAL = Mat((104, 108, 120), noise="streak", nscale=80, namp=0.06, spec=0.6)
LAMPG = Mat((255, 236, 160), emissive=True)

def spruce(seed):
    r = rnd(seed); s = Scene(); H = 8.75
    h = 7.6 + r.random() * 0.9
    s.capsule(V(0, 0.0, 0), V(0, h * 0.95, 0), 0.22, 0.04, BARK)
    n = 9 + int(r.integers(0, 3))
    for i in range(n):
        t = i / (n - 1); y = 1.2 + t * (h - 1.8); rad = 2.35 * (1 - t) ** 0.9 + 0.12
        k = 8 + int(3 * (1 - t))
        for j in range(k):
            a = j / k * np.tau if hasattr(np, "tau") else j / k * 2 * np.pi
            a += r.random() * 0.4 + i * 0.7
            ln = rad * (0.85 + 0.3 * r.random())
            tip = V(np.cos(a) * ln, y - ln * 0.42 - 0.1 * r.random(), np.sin(a) * ln)
            s.capsule(V(0, y + 0.05, 0), tip, 0.2 * (1 - t) + 0.06, 0.05, NEEDLE if (i + j) % 2 else NEEDLE2)
            s.ellipsoid(V(tip[0] * 0.62, tip[1] * 0.5 + (y + 0.05) * 0.5 - 0.12, tip[2] * 0.62), V(ln * 0.38, 0.15 + 0.1 * (1 - t), ln * 0.38), NEEDLE if (i + j) % 2 else NEEDLE2)
    s.capsule(V(0, h * 0.8, 0), V(0, h + 0.4, 0), 0.07, 0.015, NEEDLE2)
    return s, (5.0, 8.75), (96, 168)

def pine(seed):
    r = rnd(seed + 50); s = Scene()
    s.capsule(V(0, 0, 0), V(0.2 * (r.random() - .5), 7.0, 0), 0.2, 0.1, Mat((150, 90, 54), noise="streak", nscale=50, namp=0.16))
    top = 7.0
    for i in range(6):
        a = i * 1.05 + r.random() * 0.5; y = top - 0.6 - i * 0.55 - r.random() * 0.4; ln = 1.0 + r.random() * 0.9
        end = V(np.cos(a) * ln, y + 0.3, np.sin(a) * ln)
        s.capsule(V(0, y - 0.3, 0), end, 0.07, 0.04, BARK)
        s.ellipsoid(end + V(0, 0.15, 0), V(0.95 + 0.4 * r.random(), 0.38, 0.95 + 0.4 * r.random()), PINE_N)
    s.ellipsoid(V(0, top + 0.5, 0), V(1.0, 0.5, 1.0), PINE_N)
    return s, (5.0, 8.75), (96, 168)

def birch(seed):
    r = rnd(seed + 90); s = Scene(); h = 6.4
    s.capsule(V(0, 0, 0), V(0.1, h, 0), 0.17, 0.07, BIRCH)
    for i in range(14): y = 0.5 + r.random() * (h - 1.0); s.box(V(0.1 * y / h, y, 0.15 - 0.02 * r.random()), V(0.09 + 0.05 * r.random(), 0.025 + 0.02 * r.random(), 0.04), BIRCH_MARK, rnd=0.01)
    for i in range(9):
        y = 2.6 + r.random() * 3.2; a = r.random() * 6.28; ln = 0.9 + r.random() * 1.1
        e = V(np.cos(a) * ln, y + ln * 0.6, np.sin(a) * ln * 0.5)
        s.capsule(V(0.1 * y / h, y, 0), e, 0.045, 0.012, TWIG)
        for q in range(3): s.capsule(e * (0.5 + 0.2 * q) + V(0, y * (0.5 - 0.2 * q) * 0, 0), e + V(0.3 * (q - 1), 0.4, 0.1 * q), 0.018, 0.006, TWIG)
    return s, (4.0, 7.0), (96, 168)

def bare(seed):
    r = rnd(seed + 120); s = Scene()
    s.capsule(V(0, 0, 0), V(0, 3.2, 0), 0.22, 0.13, Mat((50, 40, 36), noise="streak", nscale=50, namp=0.15, snow=0.25))
    def branch(p, d, ln, rad, depth):
        e = p + d * ln
        s.capsule(p, e, rad, rad * 0.6, TWIG)
        if depth <= 0: return
        for k in range(2 + int(r.integers(0, 2))):
            a = (r.random() - 0.5) * 2.2; b = (r.random() - 0.5) * 2.2
            nd = d + V(a, 0.1, b) * 0.9; nd = nd / np.linalg.norm(nd)
            branch(e, nd, ln * 0.72, rad * 0.62, depth - 1)
    for k in range(4): a = k * 1.6 + r.random(); branch(V(0, 2.6 + 0.4 * r.random(), 0), V(np.cos(a) * 0.9, 1.0, np.sin(a) * 0.9) / 1.6, 1.3, 0.1, 3)
    return s, (4.0, 6.0), (96, 144)

def bush(seed):
    r = rnd(seed + 150); s = Scene()
    for i in range(7):
        x = (r.random() - 0.5) * 1.1; y = 0.28 + r.random() * 0.25; s.sphere(V(x, y, (r.random() - 0.5) * 0.4), 0.3 + r.random() * 0.18, LEAF)
    for i in range(10): s.capsule(V((r.random() - 0.5) * 0.9, 0.1, 0), V((r.random() - 0.5) * 1.3, 0.6 + 0.3 * r.random(), 0), 0.02, 0.008, TWIG)
    if seed % 3 == 1:
        for i in range(8): s.sphere(V((r.random() - 0.5) * 0.9, 0.25 + 0.4 * r.random(), 0.28), 0.04, Mat((190, 30, 40), spec=0.6))
    return s, (1.9, 1.3), (80, 56)

def rock(seed):
    r = rnd(seed + 170); s = Scene()
    for i in range(5): s.ellipsoid(V((r.random() - 0.5) * 0.9, 0.3 + r.random() * 0.1, (r.random() - 0.5) * 0.3), V(0.4 + r.random() * 0.3, 0.3 + r.random() * 0.25, 0.4), ROCK, rot=rot_y(r.random() * 3))
    return s, (1.9, 1.3), (80, 56)

def drift(seed):
    s = Scene()
    for i in range(4): s.ellipsoid(V((i - 1.5) * 0.7, 0.1, 0), V(0.9, 0.36 - 0.05 * abs(i - 1.5), 0.5), SNOW)
    return s, (2.6, 0.9), (80, 28)

def fence(seed):
    s = Scene()
    for x in (-0.95, 0.0, 0.95): s.box(V(x, 0.62, 0), V(0.05, 0.62, 0.05), WOODF, rnd=0.015); s.box(V(x, 1.27, 0), V(0.075, 0.025, 0.075), SNOW, rnd=0.01)
    for y in (0.5, 0.95): s.box(V(0, y, 0.03), V(1.05, 0.045, 0.02), WOODF, rnd=0.008); s.box(V(0, y + 0.05, 0.03), V(1.05, 0.012, 0.025), SNOW, rnd=0.005)
    return s, (2.4, 1.6), (96, 64)

def lamp(seed):
    s = Scene()
    s.capsule(V(0, 0, 0), V(0, 3.6, 0), 0.06, 0.04, METAL); s.box(V(0.25, 3.7, 0), V(0.3, 0.035, 0.05), METAL, rnd=0.01)
    s.box(V(0.45, 3.62, 0), V(0.14, 0.06, 0.1), METAL, rnd=0.02); s.sphere(V(0.45, 3.55, 0), 0.07, LAMPG)
    return s, (1.2, 4.0), (36, 120)

def roadpole(seed):
    s = Scene()
    s.capsule(V(0, 0, 0), V(0, 1.7, 0), 0.045, 0.04, Mat((236, 238, 240), noise="cloth", nscale=40, namp=0.05, snow=0.0))
    s.capsule(V(0, 1.45, 0), V(0, 1.7, 0), 0.047, 0.047, Mat((255, 110, 20), spec=0.3))                       # orange reflective tip
    s.capsule(V(0, 1.22, 0), V(0, 1.3, 0), 0.047, 0.047, Mat((20, 20, 24)))
    s.ellipsoid(V(0, 1.72, 0), V(0.05, 0.035, 0.05), SNOW)
    return s, (0.4, 2.0), (16, 80)

TREES = {"roadpole": (roadpole, 1), "tree.spruce": (spruce, 3), "tree.pine": (pine, 2), "tree.birch": (birch, 2), "tree.bare": (bare, 2), "bush": (bush, 3), "rock": (rock, 2), "snowdrift": (drift, 2), "fence": (fence, 1), "streetlamp": (lamp, 1)}

def main(only=None):
    for k, (fn, n) in TREES.items():
        if only and k not in only: continue
        for v in range(n):
            sc, (ww, wh), (W, H) = fn(v * 17 + 3)
            # a slight 3/4 downward tilt for low objects so tops read; trees are viewed horizontally
            img = render(sc, W, H, x_range=(-ww / 2, ww / 2), y_range=(0.0, wh), steps=50, light=(-0.5, 0.75, 0.5), ambient=0.34, edge=0.0 if k.startswith("tree") else 0.25)
            rel = f"sprites/nature/{k.replace('.', '_')}_{v}.png"
            img.save(out(rel))
            reg(f"{k}.{v}", rel, "sprite", f"Static billboard: {k} variant {v} (bottom-anchored, snow-covered; world {ww}x{wh} m)", source="generated (tools/gen_world.py, SDF-rendered)", notes=f"size={W}x{H}; keep aspect when replacing")
        print(k, "ok", flush=True)

if __name__ == "__main__":
    main(sys.argv[1:] or None)

# ---------------------------------------------------------------- items (pickups / inventory icons), 64x64, 3/4 view
CARD = Mat((176, 130, 70), noise="cloth", nscale=60, namp=0.1)
PLASTIC_W = Mat((236, 236, 240), noise="cloth", nscale=120, namp=0.03, spec=0.4)
RED_P = Mat((190, 30, 30), spec=0.5)
GREEN_P = Mat((72, 150, 80), spec=0.2)
BRASS_M = Mat((210, 170, 70), spec=0.9)
STEEL_M = Mat((150, 156, 168), spec=0.9)

def it_medkit():
    s = Scene(); s.box(V(0, 0.11, 0), V(0.22, 0.11, 0.1), PLASTIC_W, rnd=0.025)
    s.box(V(0, 0.115, 0.098), V(0.055, 0.075, 0.012), RED_P, rnd=0.004); s.box(V(0, 0.115, 0.098), V(0.075, 0.055, 0.012), RED_P, rnd=0.004)
    s.capsule(V(-0.1, 0.22, 0), V(0.1, 0.22, 0), 0.018, 0.018, STEEL_M); return s
def it_ammo_pistol():
    s = Scene(); s.box(V(0, 0.06, 0), V(0.2, 0.06, 0.1), CARD, rnd=0.012)
    for i in range(6): s.capsule(V(-0.15 + i * 0.06, 0.12, 0.0), V(-0.15 + i * 0.06, 0.2, 0.0), 0.022, 0.016, BRASS_M)
    s.box(V(0, 0.06, 0.1), V(0.12, 0.035, 0.005), Mat((240, 230, 200)), rnd=0.002); return s
def it_ammo_shells():
    s = Scene()
    for i in range(4):
        x = -0.17 + i * 0.115
        s.capsule(V(x, 0.02, 0), V(x, 0.2, 0), 0.05, 0.05, RED_P); s.capsule(V(x, 0.0, 0), V(x, 0.07, 0), 0.052, 0.052, BRASS_M)
    return s
def it_ammo_rifle():
    s = Scene(); s.box(V(0, 0.05, 0), V(0.2, 0.05, 0.09), Mat((70, 90, 70), noise="cloth", nscale=60, namp=0.08), rnd=0.01)
    for i in range(5):
        x = -0.17 + i * 0.085; s.capsule(V(x, 0.1, 0), V(x, 0.22, 0), 0.026, 0.026, BRASS_M); s.capsule(V(x, 0.2, 0), V(x, 0.28, 0), 0.026, 0.01, STEEL_M)
    return s
def it_coffee():
    s = Scene(); s.capsule(V(0, 0.02, 0), V(0, 0.2, 0), 0.1, 0.11, PLASTIC_W)
    s.ellipsoid(V(0, 0.2, 0), V(0.095, 0.02, 0.095), Mat((60, 36, 20), spec=0.8))
    for a in np.linspace(-1.2, 1.2, 6): s.sphere(V(0.12 + 0.06 * np.cos(a), 0.11 + 0.07 * np.sin(a), 0), 0.017, PLASTIC_W)
    return s
def it_sausage():
    s = Scene(); pink = Mat((200, 80, 60), noise="cloth", nscale=40, namp=0.08, spec=0.4); dark = Mat((120, 40, 30))
    for k, z in enumerate((-0.07, 0.07)):
        s.capsule(V(-0.2, 0.06, z), V(0.2, 0.06, z + 0.02), 0.06, 0.06, pink)
        for i in range(5): s.box(V(-0.14 + i * 0.07, 0.115, z), V(0.012, 0.003, 0.05), dark, rnd=0.001)
    return s
def it_bolts():
    s = Scene()
    for i, (x, z) in enumerate(((-0.12, -0.05), (0.1, -0.06), (0.0, 0.1))):
        s.capsule(V(x, 0.02, z), V(x, 0.2, z), 0.032, 0.03, STEEL_M)
        s.box(V(x, 0.2, z), V(0.06, 0.03, 0.06), STEEL_M, rot=rot_y(0.5 * i), rnd=0.01)
    return s
def it_money():
    s = Scene()
    for i in range(4): s.box(V(0.02 * (i % 2), 0.025 + i * 0.03, 0.02 * i), V(0.2, 0.015, 0.1), GREEN_P, rot=rot_y(0.12 * (i - 1.5)), rnd=0.004)
    s.box(V(0, 0.09, 0.0), V(0.05, 0.05, 0.11), Mat((230, 224, 190)), rnd=0.006); return s
def it_wrench():
    s = Scene(); s.capsule(V(-0.2, 0.03, 0), V(0.14, 0.03, 0), 0.04, 0.035, STEEL_M)
    for sy in (-1, 1): s.capsule(V(0.14, 0.03, sy * 0.045), V(0.27, 0.03, sy * 0.06), 0.03, 0.03, STEEL_M)
    s.capsule(V(0.27, 0.03, -0.06), V(0.27, 0.03, 0.06), 0.03, 0.03, STEEL_M); return s
def it_battery():
    s = Scene(); s.box(V(0, 0.1, 0), V(0.2, 0.1, 0.11), Mat((36, 60, 120), noise="cloth", nscale=80, namp=0.04, spec=0.3), rnd=0.015)
    s.box(V(-0.12, 0.215, 0), V(0.03, 0.025, 0.03), STEEL_M, rnd=0.005); s.box(V(0.12, 0.215, 0), V(0.03, 0.025, 0.03), STEEL_M, rnd=0.005)
    s.box(V(0, 0.12, 0.112), V(0.12, 0.04, 0.003), Mat((220, 220, 200)), rnd=0.001); return s
def it_sparkplug():
    s = Scene(); s.capsule(V(0, 0.2, 0), V(0, 0.34, 0), 0.04, 0.035, PLASTIC_W); s.box(V(0, 0.14, 0), V(0.06, 0.04, 0.06), STEEL_M, rot=rot_y(0.5), rnd=0.008)
    s.capsule(V(0, 0.02, 0), V(0, 0.12, 0), 0.028, 0.028, STEEL_M); s.capsule(V(0, 0.0, 0), V(0, 0.03, 0), 0.01, 0.008, Mat((60, 60, 66))); return s
def it_tire():
    s = Scene(); tire = Mat((30, 30, 34), noise="cloth", nscale=90, namp=0.05)
    for i in range(14):
        a = i / 14 * 2 * np.pi; b = (i + 1) / 14 * 2 * np.pi
        s.capsule(V(np.cos(a) * 0.2, 0.1 + np.sin(a) * 0.2, 0), V(np.cos(b) * 0.2, 0.1 + np.sin(b) * 0.2, 0), 0.075, 0.075, tire)
    s.capsule(V(0, 0.1, -0.02), V(0, 0.1, 0.03), 0.11, 0.11, STEEL_M); return s
def it_fuel_can():
    s = Scene(); s.box(V(0, 0.15, 0), V(0.14, 0.15, 0.06), RED_P, rnd=0.02); s.box(V(-0.02, 0.15, 0.062), V(0.07, 0.07, 0.003), Mat((230, 200, 60)), rnd=0.001)
    s.capsule(V(0.1, 0.3, 0), V(0.16, 0.38, 0), 0.025, 0.02, RED_P); s.capsule(V(-0.08, 0.3, 0), V(0.05, 0.3, 0), 0.012, 0.012, RED_P); return s
def it_snow_brush():
    s = Scene(); s.capsule(V(-0.2, 0.05, 0), V(0.14, 0.05, 0), 0.025, 0.025, Mat((60, 100, 200), spec=0.3))
    s.box(V(0.22, 0.05, 0), V(0.07, 0.05, 0.2), Mat((50, 80, 170)), rnd=0.01)
    for i in range(9): s.box(V(0.29, 0.05, -0.18 + i * 0.045), V(0.045, 0.03, 0.012), Mat((220, 220, 228)), rnd=0.003); return s
def it_dry_gas():
    s = Scene(); s.capsule(V(0, 0.02, 0), V(0, 0.26, 0), 0.075, 0.07, Mat((40, 120, 210), spec=0.5)); s.capsule(V(0, 0.26, 0), V(0, 0.34, 0), 0.04, 0.035, PLASTIC_W)
    s.box(V(0, 0.15, 0.07), V(0.05, 0.07, 0.004), Mat((240, 240, 240)), rnd=0.001); return s
def it_briefcase():
    s = Scene(); leather = Mat((92, 52, 30), noise="cloth", nscale=60, namp=0.1, spec=0.3)
    s.box(V(0, 0.14, 0), V(0.26, 0.14, 0.07), leather, rnd=0.025)
    s.capsule(V(-0.08, 0.28, 0), V(-0.08, 0.34, 0), 0.015, 0.015, STEEL_M); s.capsule(V(0.08, 0.28, 0), V(0.08, 0.34, 0), 0.015, 0.015, STEEL_M); s.capsule(V(-0.08, 0.34, 0), V(0.08, 0.34, 0), 0.015, 0.015, STEEL_M)
    for x in (-0.14, 0.14): s.box(V(x, 0.2, 0.072), V(0.03, 0.025, 0.004), BRASS_M, rnd=0.002)
    return s

ITEMS = {"medkit": it_medkit, "ammo_pistol": it_ammo_pistol, "ammo_shells": it_ammo_shells, "ammo_rifle": it_ammo_rifle, "coffee": it_coffee, "sausage": it_sausage, "bolts": it_bolts,
         "money": it_money, "wrench": it_wrench, "battery": it_battery, "sparkplug": it_sparkplug, "tire": it_tire, "fuel_can": it_fuel_can, "snow_brush": it_snow_brush, "dry_gas": it_dry_gas, "key": it_briefcase}

def gen_items():
    for k, fn in ITEMS.items():
        sc = fn().transform(rot_x(0.5))                                                 # tilt toward the viewer so tops are visible
        sc = sc.transform(None, (0, 0.05, 0))
        img = render(sc, 64, 64, x_range=(-0.4, 0.4), y_range=(-0.05, 0.75), steps=40, light=(-0.5, 0.8, 0.6), ambient=0.38, edge=0.06)
        rel = f"sprites/items/{k}.png"; img.save(out(rel))
        reg(f"item.{k}", rel, "sprite", f"Pickup/inventory icon: item.{k} (64x64, 3/4 view)", source="generated (tools/gen_world.py, SDF-rendered)")
    print("items ok")

if __name__ == "__main__" and (len(sys.argv) < 2 or "items" in sys.argv[1:]):
    gen_items()
