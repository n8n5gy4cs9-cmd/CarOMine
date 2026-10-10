#!/usr/bin/env python3
"""T-031: character sheets, rendered from 3D rigs with tools/sdfrender.py (Doom / ROTT-style shaded pixel art).
Sheet: 12 cols x 8 rows. cols: 0 idle, 1-4 walk, 5-6 attack, 7 pain, 8-11 death. rows: view angle 0,45,..315 (0 = facing camera).
Humans: 72x96 px. Wolf 112x64, moose 144x112. Each sheet records its WORLD size in assets/data/sprites.json so the game sizes quads correctly."""
import json, sys
import numpy as np
from PIL import Image
from assetlib import *
from sdfrender import *
from rig_humanoid import *

COLS, ROWS = 12, 8

# ------------------------------------------------------------------ humans
def goon_spec():
    sp = Spec(coat=Mat((60, 64, 88), noise="cloth", nscale=110, namp=0.05), pants=Mat((54, 58, 80), noise="cloth", nscale=110, namp=0.05),
              hair=Mat((26, 22, 22), noise="streak", nscale=90, namp=0.1, spec=0.3), skin=Mat((228, 182, 156), noise="cloth", nscale=55, namp=0.03))
    def torso_extra(s, sp, chest, R, hip):
        s.box(chest + R @ V(0, 0.07, 0.115), V(0.065, 0.19, 0.02), sp.shirt, rot=R, rnd=0.01)                 # shirt front
        s.box(chest + R @ V(0, 0.03, 0.14), V(0.017, 0.15, 0.012), sp.accent, rot=R, rnd=0.006)                  # red tie
        s.box(chest + R @ V(0.075, 0.1, 0.13), V(0.045, 0.18, 0.012), sp.coat, rot=R @ rot_z(0.18), rnd=0.008)   # lapels
        s.box(chest + R @ V(-0.075, 0.1, 0.13), V(0.045, 0.18, 0.012), sp.coat, rot=R @ rot_z(-0.18), rnd=0.008)
        s.box(hip + V(0, 0.12, 0.0), V(0.18, 0.018, 0.12), Mat((20, 18, 20), spec=0.4), rnd=0.01)                  # belt
    def head_extra(s, sp, c, R):
        s.ellipsoid(c + R @ V(0, 0.05, -0.012), V(0.104, 0.092, 0.112), sp.hair, rot=R)                          # slicked hair
        glass = Mat((10, 10, 14), spec=0.95)
        s.box(c + R @ V(0, 0.022, 0.1), V(0.085, 0.022, 0.012), glass, rot=R, rnd=0.008)                         # sunglasses
        for sx in (-1, 1): s.box(c + R @ V(sx * 0.099, 0.025, 0.04), V(0.004, 0.012, 0.06), glass, rot=R, rnd=0.002)
    sp.torso_extra, sp.head_extra = torso_extra, head_extra
    def carry(s, sp, wr):
        s.box(wr + V(0.0, -0.02, 0.05), V(0.024, 0.045, 0.1), sp.gun, rnd=0.01); s.capsule(wr + V(0, 0.0, 0.1), wr + V(0, -0.01, 0.25), 0.018, 0.018, sp.metal)
    sp.carry_right = carry
    return sp

def hunter_spec():
    sp = Spec(coat=Mat((74, 84, 52), noise="cloth", nscale=70, namp=0.1), pants=Mat((88, 76, 52), noise="cloth", nscale=80, namp=0.08),
              hair=Mat((120, 120, 120), noise="streak", nscale=90, namp=0.15), skin=Mat((214, 160, 128), noise="cloth", nscale=50, namp=0.05),
              boots=Mat((70, 44, 24), spec=0.3), glove=Mat((60, 52, 40)), shoulder=0.225)
    orange = Mat((255, 110, 10), noise="cloth", nscale=80, namp=0.03); refl = Mat((230, 230, 200), spec=0.5)
    sp.belly = 0.02
    def torso_extra(s, sp, chest, R, hip):
        for sx in (-1, 1): s.box(chest + R @ V(sx * 0.1, 0.05, 0.1), V(0.085, 0.22, 0.04), orange, rot=R, rnd=0.015)   # hi-vis vest panels
        s.box(chest + R @ V(0, 0.0, 0.115), V(0.19, 0.012, 0.03), refl, rot=R, rnd=0.004)                        # reflective stripe
        s.box(chest + R @ V(0, -0.12, 0.115), V(0.19, 0.012, 0.03), refl, rot=R, rnd=0.004)
        s.box(hip + V(0, 0.12, 0.0), V(0.18, 0.02, 0.12), Mat((60, 40, 20), spec=0.2), rnd=0.01)
    def head_extra(s, sp, c, R):
        s.ellipsoid(c + R @ V(0, 0.07, -0.005), V(0.108, 0.075, 0.118), orange, rot=R)                          # blaze-orange cap
        s.box(c + R @ V(0, 0.06, 0.115), V(0.085, 0.01, 0.055), orange, rot=R @ rot_x(0.12), rnd=0.006)           # brim
        s.ellipsoid(c + R @ V(0, -0.095, 0.045), V(0.085, 0.07, 0.07), sp.hair, rot=R)                          # grey beard
        s.ellipsoid(c + R @ V(0, -0.062, 0.1), V(0.05, 0.016, 0.014), sp.hair, rot=R)                           # moustache
    sp.torso_extra, sp.head_extra = torso_extra, head_extra
    return sp

def neighbour_spec():
    sp = Spec(coat=Mat((84, 104, 140), noise="cloth", nscale=60, namp=0.1), pants=Mat((110, 112, 120), noise="cloth", nscale=90, namp=0.06),
              hair=Mat((150, 150, 150), noise="streak", nscale=100, namp=0.18), skin=Mat((232, 168, 154), noise="cloth", nscale=50, namp=0.05),
              boots=Mat((130, 126, 120), noise="cloth", nscale=60, namp=0.1), belly=0.07, shoulder=0.22)
    def torso_extra(s, sp, chest, R, hip):
        s.ellipsoid(chest + R @ V(0, 0.02, 0.1), V(0.13, 0.24, 0.07), sp.shirt, rot=R)                          # tank top under the open parka
        s.ellipsoid(hip + R @ V(0, 0.26, 0.15), V(0.14, 0.12, 0.08), sp.shirt, rot=R)                           # belly
        s.capsule(chest + R @ V(0.14, 0.22, 0.08), chest + R @ V(0.2, 0.1, 0.1), 0.05, 0.05, Mat((60, 80, 120), noise="cloth"))  # fur-ish hood trim
    def head_extra(s, sp, c, R):
        for dx in (-1, 1):                                                                                        # messy side hair
            s.ellipsoid(c + R @ V(dx * 0.085, 0.045, -0.02), V(0.035, 0.05, 0.055), sp.hair, rot=R)
        s.ellipsoid(c + R @ V(0, 0.09, -0.04), V(0.07, 0.03, 0.07), sp.hair, rot=R)
        s.ellipsoid(c + R @ V(0, -0.025, 0.115), V(0.024, 0.024, 0.022), Mat((240, 110, 110), spec=0.2), rot=R)   # red nose
        s.ellipsoid(c + R @ V(0, -0.08, 0.06), V(0.075, 0.05, 0.06), Mat((190, 150, 138), noise="streak", nscale=120, namp=0.2), rot=R)  # stubble
    sp.torso_extra, sp.head_extra = torso_extra, head_extra
    return sp

def director_spec():
    sp = Spec(scale=1.0, coat=Mat((22, 22, 28), noise="cloth", nscale=100, namp=0.04, spec=0.1), pants=Mat((24, 24, 30), noise="cloth", nscale=100, namp=0.04),
              hair=Mat((235, 235, 240), noise="streak", nscale=100, namp=0.12, spec=0.3), skin=Mat((236, 200, 180), noise="cloth", nscale=55, namp=0.03),
              boots=Mat((14, 12, 14), spec=0.7), gun=Mat((205, 168, 70), spec=0.95), metal=Mat((225, 228, 235), spec=0.95))
    gold = Mat((230, 190, 60), spec=0.95)
    def torso_extra(s, sp, chest, R, hip):
        s.box(chest + R @ V(0, 0.08, 0.117), V(0.06, 0.19, 0.02), sp.shirt, rot=R, rnd=0.01)
        s.box(chest + R @ V(0, 0.03, 0.14), V(0.015, 0.14, 0.012), gold, rot=R, rnd=0.006)                       # gold tie
        # long coat: skirt + tails to the knees, high collar
        s.ellipsoid(hip + V(0, -0.2, 0), V(0.2, 0.34, 0.14), sp.coat)
        s.box(chest + R @ V(0, 0.26, -0.02), V(0.15, 0.05, 0.1), sp.coat, rot=R, rnd=0.03)
    def head_extra(s, sp, c, R):
        s.ellipsoid(c + R @ V(0, 0.055, -0.02), V(0.108, 0.09, 0.115), sp.hair, rot=R)
        rim = Mat((220, 190, 90), spec=0.9)
        for sx in (-1, 1): s.sphere(c + R @ V(sx * 0.04, 0.022, 0.108), 0.03, rim)                                  # round gold glasses (ring)
        s.sphere(c + R @ V(-0.04, 0.022, 0.112), 0.022, Mat((20, 20, 24), spec=0.9))                                # eyepatch
    sp.torso_extra, sp.head_extra = torso_extra, head_extra
    def carry(s, sp, wr): s.capsule(wr + V(0, -0.3, 0.02), wr + V(0, 0.15, 0.02), 0.014, 0.014, gold); s.sphere(wr + V(0, 0.16, 0.02), 0.03, gold)   # cane
    sp.carry_right = carry
    return sp

HUMANS = {
    "goon": dict(spec=goon_spec, idle="swing", walk="swing", attack="aim_pistol", world=(1.5, 2.0), px=(72, 96)),
    "hunter": dict(spec=hunter_spec, idle="rifle", walk="rifle", attack="rifle", world=(1.5, 2.0), px=(72, 96)),
    "neighbour": dict(spec=neighbour_spec, idle="bottle", walk="bottle", attack="fists", world=(1.5, 2.0), px=(72, 96), sway=0.03, lean=0.06),
    "director": dict(spec=director_spec, idle="swing", walk="swing", attack="aim_pistol", world=(1.8, 2.4), px=(84, 112), scale=1.18),
}

def human_pose(cfg, frame_idx):
    """frame_idx 0..11 -> pose dict."""
    sway = cfg.get("sway", 0.0); lean = cfg.get("lean", 0.0)
    if frame_idx == 0: return dict(arms=cfg["idle"], amp=0.0, bob=0.0, lean=lean * 0.3)
    if 1 <= frame_idx <= 4:
        k = frame_idx - 1; ph = k * np.pi / 2
        return dict(arms=cfg["walk"], phase=ph, amp=0.5, bob=0.025, sway=sway * np.sin(ph), lean=lean)
    if frame_idx in (5, 6):
        fire = frame_idx == 6; arms = cfg["attack"]
        return dict(arms=arms, fire=fire and arms != "fists", punch=1.0 if fire else 0.0, amp=0.0, lean=0.04 if fire else 0.0)
    if frame_idx == 7: return dict(arms="swing", pain=1.0, amp=0.0)
    d = (frame_idx - 7) / 4.0
    return dict(arms="swing", pain=1.0 - 0.5 * d, death=d, amp=0.0)

def render_human(name, cfg):
    W, H = cfg["px"]; ww, wh = cfg["world"]
    sp = cfg["spec"](); sp.scale = cfg.get("scale", 1.0)
    sheet = Image.new("RGBA", (W * COLS, H * ROWS), (0, 0, 0, 0))
    for fi in range(COLS):
        scene = build(sp, human_pose(cfg, fi))
        if cfg.get("scale", 1.0) != 1.0: scene = scene.transform(np.eye(3) * 1.0)
        for r in range(ROWS):
            rel = r * np.pi / 4
            sc = scene.transform(rot_y(-rel))
            img = render(sc, W, H, x_range=(-ww / 2, ww / 2), y_range=(0.0, wh))
            sheet.alpha_composite(img, (fi * W, r * H))
    return sheet

def meta_for(name, W, H, world):
    return dict(frame=[W, H], cols=COLS, rows=ROWS, angles=8, anims=dict(idle=[0], walk=[1, 2, 3, 4], attack=[5, 6], pain=[7], die=[8, 9, 10, 11]), anchor="bottom", world=list(world))

def write_sheet(name, sheet, W, H, world, use):
    rel = f"sprites/enemies/{name}.png"
    sheet.save(out(rel))
    reg(f"enemy.{name}", rel, "sprite", use, source="generated (tools/gen_chars.py, SDF-rendered)", notes=f"{COLS} cols x {ROWS} angle rows, frame {W}x{H}px, world {world[0]}x{world[1]} m. cols: 0 idle,1-4 walk,5-6 attack,7 pain,8-11 death; rows: view angle 0=facing camera, +45deg each. Replace keeping the grid + update assets/data/sprites.json")
    p = out("data/sprites.json"); cur = json.load(open(p)) if p.exists() else {}
    cur[f"enemy.{name}"] = meta_for(name, W, H, world); json.dump(cur, open(p, "w"), indent=1)

def main(only=None):
    from rig_quad import QUADS, render_quad
    for name, cfg in HUMANS.items():
        if only and name not in only: continue
        W, H = cfg["px"]; sheet = render_human(name, cfg)
        write_sheet(name, sheet, W, H, cfg["world"], f"Enemy '{name}' billboard sheet (3D-rendered pixel art)"); print(name, "ok", flush=True)
    for name, cfg in QUADS.items():
        if only and name not in only: continue
        W, H = cfg["px"]; sheet = render_quad(name, cfg, COLS, ROWS)
        write_sheet(name, sheet, W, H, cfg["world"], f"Enemy '{name}' billboard sheet (3D-rendered pixel art)"); print(name, "ok", flush=True)

if __name__ == "__main__":
    main(sys.argv[1:] or None)
