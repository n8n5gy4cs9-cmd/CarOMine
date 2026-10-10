#!/usr/bin/env python3
"""T-020: enemy billboard sheets. Layout: 10 cols x 8 rows (rows = 8 view angles, 0=facing camera, going CCW).
cols: 0 idle, 1-4 walk, 5-6 attack, 7-9 death. Frame 48x64. Also writes assets/data/sprites.json entries."""
import math, json
from PIL import Image, ImageDraw
from assetlib import *

W, H = 48, 64
COLS, ROWS = 10, 8

def biped(d, th, frame, P):
    """th: view angle rad (0 front, pi back). frame: ('idle'|'walk'|'attack'|'die', i)"""
    kind, i = frame
    c, s = math.cos(th), math.sin(th)       # c>0 faces camera
    cx = 24; ground = 62
    ph = i * math.pi / 2
    if kind == "die":
        # collapse: rotate-ish by drawing lying body
        t = i / 2.0
        if i == 0:   # stagger
            return biped(d, th, ("idle", 0), P)
        y = ground - 8 + (2 if i == 1 else 4)
        d.ellipse([cx - 18, y - 8, cx + 18, y + 2], fill=P["body"])
        d.ellipse([cx + 12, y - 10, cx + 22, y], fill=P["skin"])
        d.rectangle([cx - 20, y - 3, cx - 8, y + 1], fill=P["legs"])
        if i == 2:
            d.ellipse([cx - 10, y - 2, cx + 14, y + 4], fill=(150, 20, 25, 255))
        return
    swing = math.sin(ph) * 5 if kind == "walk" else 0
    bob = abs(math.sin(ph)) * 1.5 if kind == "walk" else 0
    w = 5 + 5 * abs(c)           # torso half-ish width factor
    # legs
    for sgn in (-1, 1):
        lx = cx + sgn * 3 * abs(c) + s * sgn * swing * 0.9
        lift = (swing * sgn * 0.5 * abs(c)) if kind == "walk" else 0
        d.rectangle([lx - 2, 42 - bob, lx + 2, ground - max(0, lift)], fill=P["legs"])
        d.rectangle([lx - 3, ground - 3 - max(0, lift), lx + 3 + s * 2, ground - max(0, lift)], fill=P["boots"])
    # torso
    top, bot = 24 - bob, 44 - bob
    d.rounded_rectangle([cx - w, top, cx + w, bot], 3, fill=P["body"])
    if c > 0.3 and P.get("tie"):
        d.rectangle([cx - 1, top + 1, cx + 1, top + 12], fill=P["tie"])
    if P.get("vest") and abs(c) > 0.3:
        d.rectangle([cx - w + 1, top + 2, cx + w - 1, bot - 4], fill=P["vest"])
    # head
    hy = 16 - bob
    d.ellipse([cx - 6, hy - 6, cx + 6, hy + 6], fill=P["skin"] if c > -0.2 else P["hair"])
    if c <= -0.2:
        pass
    else:
        # hair cap
        d.pieslice([cx - 6, hy - 7, cx + 6, hy + 5], 180, 360, fill=P["hair"])
        ex = s * 3
        if abs(s) < 0.9:
            for e in (-2, 2):
                col = P.get("eyes", (20, 20, 20, 255))
                d.rectangle([cx + e + ex - 1, hy - 1, cx + e + ex + 1, hy + (0 if not P.get("shades") else 1)], fill=col)
        if abs(s) > 0.4:
            d.rectangle([cx + int(math.copysign(7, s)) - 1, hy, cx + int(math.copysign(7, s)) + 1, hy + 2], fill=P["skin"])
    if P.get("hat"):
        d.rectangle([cx - 7, hy - 8, cx + 7, hy - 3], fill=P["hat"])
        d.rectangle([cx - 7 + int(4 * s), hy - 4, cx + 7 + int(4 * s), hy - 2], fill=P["hat"])
    # arms
    attack = kind == "attack"
    for sgn in (-1, 1):
        ax = cx + sgn * (w + 1) * (1 if abs(c) > 0.2 else 0.3)
        d.rectangle([ax - 2, top + 1, ax + 2, top + 14], fill=P["body"])
        d.rectangle([ax - 2, top + 13, ax + 2, top + 16], fill=P["skin"])
    # weapon
    gun = P.get("gun")
    if gun:
        gy = top + (6 if attack else 14)
        if c > 0.6:       # pointing at camera
            d.rectangle([cx - 2, gy - 1, cx + 2, gy + 2], fill=(25, 25, 30, 255))
            if attack: d.ellipse([cx - 4, gy - 3, cx + 4, gy + 5], fill=(255, 220, 90, 255))
        elif c > -0.6:
            L = int(gun * abs(s)); sg = 1 if s >= 0 else -1
            d.rectangle([min(cx, cx + sg * L), gy - 1, max(cx, cx + sg * L), gy + 2], fill=(25, 25, 30, 255))
            if attack: d.ellipse([cx + sg * L - 3, gy - 4, cx + sg * L + 5, gy + 4], fill=(255, 220, 90, 255))
    if P.get("bottle") and not attack:
        d.rectangle([cx + 6, top + 8, cx + 8, top + 16], fill=(50, 120, 60, 255))
    if kind == "attack" and not gun and i >= 0:
        # melee swing: fist forward
        d.ellipse([cx - 4, top + 6, cx + 4, top + 14], fill=P["skin"])

def quad(d, th, frame, P):
    kind, i = frame
    c, s = math.cos(th), math.sin(th)
    cx = 24; ground = 60
    big = P["size"]
    ph = i * math.pi / 2
    swing = math.sin(ph) * 3 if kind == "walk" else 0
    if kind == "die":
        y = ground - 6 - (1 if i else 0)
        d.ellipse([cx - 18 * big, y - 8 * big, cx + 18 * big, y + 2], fill=P["body"])
        d.ellipse([cx + 12 * big, y - 6, cx + 22 * big, y + 2], fill=P["body"])
        return
    L = (14 * abs(s) + 7 * abs(c)) * big     # body half-length on screen
    sg = 1 if s >= 0 else -1
    by = ground - 16 * big
    d.ellipse([cx - L, by - 9 * big, cx + L, by + 7 * big], fill=P["body"])
    # legs
    for k, off in enumerate((-0.8, -0.3, 0.3, 0.8)):
        lx = cx + off * L * abs(s) + (k % 2) * 3 * abs(c) - 1.5 * abs(c)
        sw = swing * (1 if k % 2 else -1) * abs(s)
        d.rectangle([lx - 2 + sw, by + 3, lx + 2 + sw, ground], fill=P["legs"])
    # head
    if c > 0.4:
        hx, hy = cx, by - 4 * big
        d.ellipse([hx - 6 * big, hy - 5 * big, hx + 6 * big, hy + 6 * big], fill=P["body"])
        d.ellipse([hx - 3, hy + 2, hx + 3, hy + 7], fill=P["snout"])
        d.rectangle([hx - 4, hy - 2, hx - 2, hy], fill=P["eyes"]); d.rectangle([hx + 2, hy - 2, hx + 4, hy], fill=P["eyes"])
    elif c < -0.4:
        d.rectangle([cx + 6 * sg, by - 4, cx + 8 * sg, by + 6], fill=P["legs"])   # tail
    else:
        hx = cx + sg * (L + 2 * big); hy = by - 8 * big
        d.ellipse([hx - 6 * big, hy - 5 * big, hx + 6 * big, hy + 5 * big], fill=P["body"])
        d.ellipse([hx + sg * 4 - 3, hy, hx + sg * 4 + 3, hy + 5], fill=P["snout"])
        d.rectangle([hx - 1, hy - 2, hx + 1, hy], fill=P["eyes"])
        d.rectangle([cx - sg * L - 1, by - 2, cx - sg * L + 1, by + 4], fill=P["legs"])
    if kind == "attack":
        d.polygon([(cx, by - 6 * big), (cx + sg * 3, by - 12 * big), (cx + 6 * sg, by - 6 * big)], fill=(240, 240, 240, 255))
    if P.get("antlers"):
        hx = cx + (sg * (L + 2 * big) if abs(c) < 0.4 else 0); hy = by - 8 * big if abs(c) < 0.4 else by - 4 * big
        for sgn in (-1, 1):
            d.line([hx, hy - 4, hx + sgn * 10, hy - 12, hx + sgn * 8, hy - 18], fill=(200, 190, 150, 255), width=2)
            d.line([hx + sgn * 10, hy - 12, hx + sgn * 15, hy - 14], fill=(200, 190, 150, 255), width=2)

TYPES = {
 "goon": dict(fn=biped, P=dict(body=rgba("1b1b22"), legs=rgba("14141a"), boots=rgba("0a0a0a"), skin=rgba("e0b090"), hair=rgba("222"), tie=rgba("a01020"), shades=True, eyes=rgba("050505"), gun=10)),
 "hunter": dict(fn=biped, P=dict(body=rgba("3d4a2a"), legs=rgba("4a4030"), boots=rgba("2a1a10"), skin=rgba("d8a888"), hair=rgba("6a4a2a"), vest=rgba("ee6a10"), hat=rgba("ee6a10"), gun=16)),
 "neighbour": dict(fn=biped, P=dict(body=rgba("5a6a8a"), legs=rgba("3a3a4a"), boots=rgba("202020"), skin=rgba("e8a0a0"), hair=rgba("999"), bottle=True)),
 "wolf": dict(fn=quad, P=dict(body=rgba("6b6b72"), legs=rgba("505058"), snout=rgba("30303a"), eyes=rgba("f0d020"), size=0.8)),
 "moose": dict(fn=quad, P=dict(body=rgba("4a3220"), legs=rgba("2e2014"), snout=rgba("30241a"), eyes=rgba("101010"), size=1.1, antlers=True)),
}
FRAMES = [("idle", 0)] + [("walk", i) for i in range(4)] + [("attack", 0), ("attack", 1)] + [("die", i) for i in range(3)]

def main():
    meta = {}
    for name, t in TYPES.items():
        sheet = Image.new("RGBA", (W * COLS, H * ROWS), (0, 0, 0, 0))
        for r in range(ROWS):
            th = r * math.pi / 4
            for ci, fr in enumerate(FRAMES):
                im = Image.new("RGBA", (W, H), (0, 0, 0, 0))
                t["fn"](SDraw(im), th, fr, t["P"])
                sheet.alpha_composite(outline(im), (ci * W, r * H))
        rel = f"sprites/enemies/{name}.png"
        sheet.save(out(rel))
        reg(f"enemy.{name}", rel, "sprite", f"Enemy '{name}' billboard sheet (10 cols x 8 angle rows, frame {W}x{H})",
            notes="cols: 0 idle,1-4 walk,5-6 attack,7-9 death; rows: view angle 0=facing camera, +45deg each. Replace keeping the grid.")
        meta[f"enemy.{name}"] = dict(frame=[W, H], cols=COLS, rows=ROWS, angles=8,
            anims=dict(idle=[0], walk=[1, 2, 3, 4], attack=[5, 6], die=[7, 8, 9]), anchor="bottom")
    p = out("data/sprites.json")
    cur = json.load(open(p)) if p.exists() else {}
    cur.update(meta); json.dump(cur, open(p, "w"), indent=1)
    print("enemies ok")

if __name__ == "__main__":
    main()
