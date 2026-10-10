#!/usr/bin/env python3
"""T-034: Doom-style status bar art: steel plate (320x32), red digits, and the player's face portrait with 5 damage tiers
(rendered from the 3D head model). Faces: cols 0 centre,1 look left,2 look right,3 pain,4 grin; rows 0..4 health tiers (100%..0%), row 5: 0 dead, 1 god mode, 2 hurt-badly-grin."""
import numpy as np
from PIL import Image, ImageDraw, ImageFont
from assetlib import *
from sdfrender import *
from rig_humanoid import V, Spec, head as draw_head

BAR_W, BAR_H = 480, 48
# panel layout in native bar pixels (the game uses the same numbers): name -> (x0, x1)
PANELS = {"ammo": (4, 70), "health": (74, 146), "arms": (150, 196), "face": (200, 252), "armor": (256, 328), "eur": (332, 396), "needs": (400, 476)}

def plate():
    W, H = BAR_W, BAR_H
    im = Image.new("RGBA", (W, H), (0, 0, 0, 255)); d = ImageDraw.Draw(im)
    for y in range(H):                                     # brushed steel with a top highlight
        v = int(76 - y * 0.6); d.line([0, y, W, y], fill=(v - 8, v - 4, v + 6, 255))
    rng = np.random.default_rng(3)
    for _ in range(1100):
        x, y = int(rng.integers(0, W)), int(rng.integers(0, H)); d.point((x, y), fill=(int(rng.integers(52, 98)),) * 2 + (int(rng.integers(60, 106)),) + (255,))
    d.line([0, 0, W, 0], fill=(150, 156, 170, 255)); d.line([0, 1, W, 1], fill=(104, 110, 124, 255))
    d.fontmode = "1"
    try: f = ImageFont.load_default(size=8)
    except TypeError: f = ImageFont.load_default()
    labels = {"ammo": "AMMO", "health": "HEALTH", "arms": "ARMS", "armor": "ARMOR", "eur": "EUR", "needs": "FOOD NRG WARM"}
    for name, (x0, x1) in PANELS.items():
        y0, y1 = (2, 45) if name == "face" else (5, 43)
        d.rectangle([x0, y0, x1, y1], fill=(18, 20, 26, 255)); d.line([x0, y0, x1, y0], fill=(8, 8, 12, 255)); d.line([x0, y0, x0, y1], fill=(8, 8, 12, 255))
        d.line([x0, y1, x1, y1], fill=(120, 126, 140, 255)); d.line([x1, y0, x1, y1], fill=(120, 126, 140, 255))
        if name in labels: d.text((x0 + 4, y1 - 10), labels[name], font=f, fill=(190, 150, 70, 255))
    for x in (1, W - 3):
        for y in (4, H - 6): d.rectangle([x, y, x + 1, y + 1], fill=(150, 156, 170, 255))
    return im

def digits():
    """12 glyphs 14x16: 0-9 % -"""
    try: f = ImageFont.load_default(size=15)
    except TypeError: f = ImageFont.load_default()
    sheet = Image.new("RGBA", (14 * 12, 16), (0, 0, 0, 0))
    for i, ch in enumerate("0123456789%-"):
        g = Image.new("RGBA", (14, 16), (0, 0, 0, 0)); gd = ImageDraw.Draw(g); gd.fontmode = "1"
        for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1), (1, 1), (-1, -1)): gd.text((2 + dx, 0 + dy), ch, font=f, fill=(30, 4, 4, 255))
        gd.text((2, 0), ch, font=f, fill=(226, 36, 28, 255))
        px = g.load()
        for y in range(16):
            for x in range(14):
                r, gg, b, a = px[x, y]
                if a and r > 150: px[x, y] = (min(255, r + y * 2), min(255, 36 + y * 7), 28, 255)       # warmer toward the bottom, like Doom's numbers
        sheet.alpha_composite(g, (i * 14, 0))
    return sheet

SK = Mat((228, 176, 148), noise="cloth", nscale=60, namp=0.03)
def face_scene(tier, look=0.0, pain=False, grin=False, dead=False, god=False):
    sp = Spec()
    skin = Mat((228 - 30 * (tier >= 3) - 20 * (tier >= 4), 176 - 30 * (tier >= 3) - 25 * (tier >= 4), 148 - 30 * (tier >= 3) - 25 * (tier >= 4)), noise="cloth", nscale=60, namp=0.03)
    sp.skin = skin; sp.hair = Mat((34, 28, 24), noise="streak", nscale=90, namp=0.12)
    s = Scene(); c = V(0, 1.66, 0.0); R = rot_y(look * 0.35)
    draw_head(s, sp, c, R, face=False)
    # neck + suit collar / shirt
    s.capsule(c + V(0, -0.1, 0), c + V(0, -0.2, 0), 0.05, 0.055, skin)
    suit = Mat((52, 56, 78), noise="cloth", nscale=100, namp=0.05); shirt = Mat((226, 226, 232), noise="cloth", nscale=100, namp=0.03)
    s.ellipsoid(c + V(0, -0.27, -0.01), V(0.2, 0.1, 0.11), suit); s.box(c + V(0, -0.2, 0.06), V(0.045, 0.045, 0.03), shirt, rnd=0.01)
    s.box(c + V(0, -0.24, 0.1), V(0.012, 0.05, 0.01), Mat((176, 30, 36)), rnd=0.004)                                    # tie knot
    # hair: short, parted
    s.ellipsoid(c + R @ V(0, 0.06, -0.015), V(0.106, 0.09, 0.112), sp.hair, rot=R)
    s.ellipsoid(c + R @ V(0.04, 0.095, 0.05), V(0.06, 0.03, 0.05), sp.hair, rot=R)
    # eyes: whites + pupils (look direction), lids for pain / low health
    white = Mat((244, 244, 244)); pup = Mat((24, 22, 22), spec=0.5)
    squint = 0.5 if (pain or tier >= 3) else (0.2 if tier >= 2 else 0.0)
    for sx in (-1, 1):
        e = c + R @ V(sx * 0.04, 0.02, 0.092)
        s.sphere(e, 0.021, white); s.sphere(e + R @ V(look * 0.012, 0, 0.012), 0.0115, pup)
        if squint: s.ellipsoid(e + R @ V(0, 0.016 - squint * 0.012, 0.012), V(0.027, 0.013 + squint * 0.012, 0.016), skin, rot=R)      # upper lid
        brow_y = 0.058 - (0.012 if pain else 0.0)
        s.box(c + R @ V(sx * 0.042, brow_y, 0.096), V(0.03, 0.007, 0.009), sp.hair, rot=R @ rot_z(sx * (-0.35 if (pain or tier >= 3) else (0.18 if grin else 0.0))), rnd=0.004)
        s.sphere(c + R @ V(sx * 0.1, -0.01, -0.01), 0.026, skin)                                                          # ears
    s.ellipsoid(c + R @ V(0, -0.015, 0.108), V(0.019, 0.03, 0.027), skin, rot=R)                                         # nose
    # mouth
    lip = Mat((150, 66, 66)); teeth = Mat((246, 244, 238)); dark = Mat((60, 20, 24))
    if pain:
        s.ellipsoid(c + R @ V(0, -0.072, 0.098), V(0.026, 0.02, 0.012), dark, rot=R); s.box(c + R @ V(0, -0.058, 0.103), V(0.02, 0.005, 0.004), teeth, rot=R, rnd=0.002)
    elif grin:
        for k in range(9):
            t = (k - 4) / 4.0; s.sphere(c + R @ V(t * 0.045, -0.062 - 0.012 * (1 - t * t) * -1 - 0.0, 0.098), 0.0075, lip)
        s.box(c + R @ V(0, -0.069, 0.1), V(0.04, 0.011, 0.006), teeth, rot=R, rnd=0.003)
    else:
        s.box(c + R @ V(0, -0.07, 0.099), V(0.028, 0.0055, 0.006), lip, rot=R, rnd=0.003)
    stub = Mat((160, 118, 104), noise="streak", nscale=140, namp=0.2)
    s.ellipsoid(c + R @ V(0, -0.07, 0.05), V(0.075, 0.04, 0.06), stub, rot=R)                                          # stubble shadow
    # damage: scratches, bruises, blood
    red = Mat((176, 20, 24), spec=0.4); bruise = Mat((120, 70, 120))
    if tier >= 1: s.capsule(c + R @ V(-0.06, 0.07, 0.092), c + R @ V(-0.03, 0.03, 0.105), 0.004, 0.003, red)
    if tier >= 2:
        s.ellipsoid(c + R @ V(0.06, -0.02, 0.085), V(0.03, 0.025, 0.02), bruise, rot=R); s.capsule(c + R @ V(0.05, 0.075, 0.09), c + R @ V(0.07, 0.02, 0.1), 0.0045, 0.0035, red)
    if tier >= 3:
        s.capsule(c + R @ V(-0.02, 0.1, 0.095), c + R @ V(-0.03, -0.03, 0.108), 0.006, 0.005, red)                        # blood from the brow
        s.ellipsoid(c + R @ V(-0.045, 0.012, 0.1), V(0.035, 0.03, 0.02), bruise, rot=R)
        s.capsule(c + R @ V(0.0, -0.085, 0.105), c + R @ V(0.02, -0.11, 0.1), 0.005, 0.004, red)
    if tier >= 4:
        s.ellipsoid(c + R @ V(0.0, 0.0, 0.1), V(0.08, 0.09, 0.012), Mat((150, 20, 22), spec=0.3), rot=R) if False else None
        for k in range(5): s.capsule(c + R @ V(-0.07 + k * 0.035, 0.09, 0.09), c + R @ V(-0.075 + k * 0.035, -0.05 - 0.02 * k, 0.1), 0.0045, 0.003, red)
    if dead:
        for sx in (-1, 1): s.box(c + R @ V(sx * 0.04, 0.022, 0.1), V(0.026, 0.004, 0.008), Mat((20, 18, 18)), rot=R, rnd=0.002)   # closed eyes
        s.ellipsoid(c + R @ V(0, -0.02, 0.1), V(0.1, 0.1, 0.01), Mat((130, 16, 20), spec=0.3), rot=R)
    if god:
        for sx in (-1, 1): s.sphere(c + R @ V(sx * 0.04, 0.02, 0.1), 0.02, Mat((255, 230, 120), emissive=True))
    return s

def main():
    out_img = plate(); out("sprites/ui").mkdir(parents=True, exist_ok=True)
    out_img.save(out("sprites/ui/statusbar.png")); reg("ui.statusbar", "sprites/ui/statusbar.png", "sprite", "Doom-style status bar plate 480x48 (panels: AMMO, HEALTH, ARMS, FACE, ARMOR, EUR, NEEDS - positions in tools/gen_hud.py PANELS and src/combat/statusbar.rs); game scales it x2 nearest", source="generated (tools/gen_hud.py)")
    dg = digits(); dg.save(out("sprites/ui/digits.png")); reg("ui.digits", "sprites/ui/digits.png", "sprite", "Red status-bar digits: 12 glyphs 14x16 in one row: 0-9 % -", source="generated (tools/gen_hud.py)")
    FW, FH = 36, 44
    faces = Image.new("RGBA", (FW * 5, FH * 6), (0, 0, 0, 0))
    def r(sc): return render(sc, FW, FH, x_range=(-0.19, 0.19), y_range=(1.36, 1.8), steps=44, light=(-0.4, 0.55, 0.75), ambient=0.4, edge=0.0)
    for t in range(5):
        for k, kw in enumerate((dict(look=0.0), dict(look=-1.0), dict(look=1.0), dict(pain=True), dict(grin=True))):
            faces.alpha_composite(r(face_scene(t, **kw)), (k * FW, t * FH))
    faces.alpha_composite(r(face_scene(4, dead=True)), (0, 5 * FH)); faces.alpha_composite(r(face_scene(0, god=True)), (FW, 5 * FH)); faces.alpha_composite(r(face_scene(3, grin=True)), (2 * FW, 5 * FH))
    faces.save(out("sprites/ui/faces.png"))
    reg("ui.faces", "sprites/ui/faces.png", "sprite", f"Player face portrait atlas {FW}x{FH} cells, 5 cols (centre, left, right, pain, grin) x 6 rows (health tiers 100..0%, then dead/god/bloody grin)", source="generated (tools/gen_hud.py, SDF-rendered)")
    print("hud ok")

if __name__ == "__main__": main()
