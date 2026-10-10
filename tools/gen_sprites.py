#!/usr/bin/env python3
"""T-021..T-026: nature, items, weapons, textures, buildings, UI placeholders."""
import math, random, json
import numpy as np
from PIL import Image, ImageDraw, ImageFont
from assetlib import *

SNOW = rgba("f4f8ff"); SNOW2 = rgba("c9d6ea")

def spruce(seed, w=64, h=112):
    r = random.Random(seed); im = Image.new("RGBA", (w, h), (0, 0, 0, 0)); d = SDraw(im)
    cx = w // 2
    d.rectangle([cx - 3, h - 14, cx + 3, h], fill=rgba("4a3220"))
    layers = 6 + r.randint(0, 1)
    for i in range(layers):
        y1 = h - 14 - i * (h - 26) // layers; hw = (w // 2 - 2) * (1 - i / (layers + 0.6))
        green = rgba("1f4a2c") if i % 2 else rgba("27593a")
        d.polygon([(cx - hw, y1), (cx + hw, y1), (cx, y1 - 30 + i * 2)], fill=green)
        d.polygon([(cx - hw * .8, y1 - 3), (cx + hw * .8, y1 - 3), (cx, y1 - 28 + i * 2)], fill=None)
        # snow on top edge
        d.polygon([(cx - hw * .55, y1 - 9 + i), (cx + hw * .55, y1 - 9 + i), (cx, y1 - 30 + i * 2)], fill=SNOW)
    return outline(im)

def pine(seed, w=64, h=112):
    r = random.Random(seed); im = Image.new("RGBA", (w, h), (0, 0, 0, 0)); d = SDraw(im); cx = w // 2
    d.rectangle([cx - 2, 28, cx + 2, h], fill=rgba("8a5a30"))
    for (ox, oy, rr) in [(0, 20, 16), (-12, 34, 11), (12, 30, 12), (-4, 8, 11)]:
        d.ellipse([cx + ox - rr, oy - rr * .6, cx + ox + rr, oy + rr * .6], fill=rgba("2c5a30"))
        d.ellipse([cx + ox - rr * .8, oy - rr * .6, cx + ox + rr * .8, oy], fill=SNOW)
    return outline(im)

def birch(seed, w=64, h=112):
    r = random.Random(seed); im = Image.new("RGBA", (w, h), (0, 0, 0, 0)); d = SDraw(im); cx = w // 2
    d.rectangle([cx - 3, 30, cx + 3, h], fill=rgba("e8e8e0"))
    for _ in range(9):
        y = r.randint(34, h - 4); d.rectangle([cx - 3, y, cx - 3 + r.randint(2, 5), y + 1], fill=rgba("222"))
    for _ in range(9):
        a = r.uniform(-1, 1); L = r.randint(14, 26); y = r.randint(30, 60)
        d.line([cx, y, cx + a * L, y - L * .8], fill=rgba("6a5a4a"), width=1)
        d.ellipse([cx + a * L - 3, y - L * .8 - 2, cx + a * L + 3, y - L * .8 + 2], fill=SNOW)
    return outline(im)

def bare(seed, w=64, h=96):
    r = random.Random(seed); im = Image.new("RGBA", (w, h), (0, 0, 0, 0)); d = SDraw(im); cx = w // 2
    d.rectangle([cx - 3, 40, cx + 3, h], fill=rgba("3a2c22"))
    for _ in range(12):
        y = r.randint(20, 56); a = r.uniform(-1, 1); L = r.randint(12, 26)
        d.line([cx, y + 12, cx + a * L, y - L * .6], fill=rgba("3a2c22"), width=2)
        d.ellipse([cx + a * L - 2, y - L * .6 - 1, cx + a * L + 3, y - L * .6 + 2], fill=SNOW)
    return outline(im)

def bush(seed, w=40, h=28, berries=False):
    r = random.Random(seed); im = Image.new("RGBA", (w, h), (0, 0, 0, 0)); d = SDraw(im)
    for _ in range(5):
        x = r.randint(6, w - 6); y = r.randint(10, h - 5); rr = r.randint(5, 8)
        d.ellipse([x - rr, y - rr, x + rr, y + rr], fill=rgba("2d4a2a"))
        d.ellipse([x - rr * .8, y - rr, x + rr * .8, y - rr * .1], fill=SNOW)
    if berries:
        for _ in range(6): d.point((r.randint(8, w - 8), r.randint(10, h - 6)), fill=rgba("c01830"))
    return outline(im)

def rock(seed, w=40, h=28):
    r = random.Random(seed); im = Image.new("RGBA", (w, h), (0, 0, 0, 0)); d = SDraw(im)
    pts = [(2, h - 2), (6, 10), (16, 3), (28, 6), (w - 3, 14), (w - 2, h - 2)]
    d.polygon(pts, fill=rgba("6e7078")); d.polygon([(6, 10), (16, 3), (28, 6), (w - 6, 12), (20, 14)], fill=SNOW)
    return outline(im)

def drift(seed, w=64, h=20):
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0)); d = SDraw(im)
    d.ellipse([0, 4, w - 1, h + 8], fill=SNOW); d.ellipse([8, 2, w - 10, h], fill=rgba("ffffff"))
    return outline(im, SNOW2)

def fence(seed, w=48, h=32):
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0)); d = SDraw(im)
    for x in (4, 22, 40): d.rectangle([x, 6, x + 3, h - 1], fill=rgba("7a5a3a")); d.rectangle([x, 4, x + 3, 6], fill=SNOW)
    for y in (12, 22): d.rectangle([0, y, w - 1, y + 2], fill=rgba("8a6a46"))
    return outline(im)

def streetlamp(seed, w=24, h=80):
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0)); d = SDraw(im)
    d.rectangle([11, 10, 13, h], fill=rgba("3a3a42")); d.rectangle([6, 6, 18, 12], fill=rgba("555560")); d.ellipse([8, 10, 16, 16], fill=rgba("ffe9a0"))
    return outline(im)

NATURE = {
 "tree.spruce": (spruce, 3), "tree.pine": (pine, 2), "tree.birch": (birch, 2), "tree.bare": (bare, 2),
 "bush": (bush, 3), "rock": (rock, 2), "snowdrift": (drift, 2), "fence": (fence, 1), "streetlamp": (streetlamp, 1),
}

def gen_nature():
    for k, (fn, n) in NATURE.items():
        for v in range(n):
            im = fn(v * 17 + 3)
            rel = f"sprites/nature/{k.replace('.', '_')}_{v}.png"
            save_img(im, rel, f"{k}.{v}", f"Static billboard: {k} variant {v} (bottom-anchored, snow-covered)", scale=2)

def icon(draw_fn):
    im = Image.new("RGBA", (32, 32), (0, 0, 0, 0)); draw_fn(SDraw(im)); return outline(im)

ITEMS = {
 "item.medkit": lambda d: (d.rectangle([5, 9, 26, 24], fill=rgba("f2f2f2")), d.rectangle([14, 11, 17, 22], fill=rgba("d01020")), d.rectangle([8, 15, 23, 18], fill=rgba("d01020"))),
 "item.ammo_pistol": lambda d: [d.rectangle([6 + i * 5, 10, 9 + i * 5, 24], fill=rgba("d8b040")) or d.rectangle([6 + i * 5, 8, 9 + i * 5, 10], fill=rgba("a06030")) for i in range(4)],
 "item.ammo_shells": lambda d: [d.rectangle([6 + i * 6, 8, 11 + i * 6, 24], fill=rgba("c02818")) or d.rectangle([6 + i * 6, 20, 11 + i * 6, 24], fill=rgba("d8b040")) for i in range(4)],
 "item.ammo_rifle": lambda d: [d.rectangle([7 + i * 6, 6, 9 + i * 6, 26], fill=rgba("b8a060")) for i in range(4)],
 "item.coffee": lambda d: (d.rectangle([8, 10, 22, 26], fill=rgba("f0f0f0")), d.ellipse([20, 13, 29, 22], outline=rgba("f0f0f0"), width=2), d.rectangle([9, 11, 21, 14], fill=rgba("5a3418"))),
 "item.sausage": lambda d: (d.rounded_rectangle([4, 12, 28, 22], 5, fill=rgba("c04a30")), d.line([8, 14, 24, 14], fill=rgba("e08060"))),
 "item.bolts": lambda d: [d.polygon([(8 + i * 8, 10), (14 + i * 8, 10), (17 + i * 8, 15), (14 + i * 8, 20), (8 + i * 8, 20), (5 + i * 8, 15)], fill=rgba("9aa0aa")) for i in range(3)],
 "item.money": lambda d: (d.rectangle([4, 10, 28, 23], fill=rgba("6aa860")), d.ellipse([12, 12, 20, 21], outline=rgba("2a5a28"), width=2)),
 "item.wrench": lambda d: (d.line([6, 26, 22, 10], fill=rgba("aab0ba"), width=4), d.ellipse([18, 3, 28, 13], fill=rgba("aab0ba")), d.ellipse([21, 6, 25, 10], fill=(0, 0, 0, 0))),
 "item.battery": lambda d: (d.rectangle([4, 10, 28, 26], fill=rgba("2a3a6a")), d.rectangle([7, 7, 11, 10], fill=rgba("c0c0c0")), d.rectangle([21, 7, 25, 10], fill=rgba("c0c0c0")), d.text((12, 14), "+", fill=rgba("ffffff"))),
 "item.sparkplug": lambda d: (d.rectangle([13, 4, 18, 10], fill=rgba("f0f0f0")), d.rectangle([12, 10, 19, 20], fill=rgba("9aa0aa")), d.rectangle([15, 20, 16, 28], fill=rgba("606060"))),
 "item.tire": lambda d: (d.ellipse([3, 3, 29, 29], fill=rgba("222226")), d.ellipse([11, 11, 21, 21], fill=rgba("888c96"))),
 "item.fuel_can": lambda d: (d.rectangle([7, 8, 25, 28], fill=rgba("c02818")), d.rectangle([20, 4, 25, 8], fill=rgba("c02818")), d.rectangle([11, 13, 21, 20], fill=rgba("e8c030"))),
 "item.snow_brush": lambda d: (d.rectangle([14, 4, 17, 22], fill=rgba("7a5a3a")), d.rectangle([6, 22, 25, 28], fill=rgba("3a6ad0"))),
 "item.dry_gas": lambda d: (d.rectangle([10, 6, 21, 28], fill=rgba("2a8ad0")), d.rectangle([13, 3, 18, 6], fill=rgba("f0f0f0"))),
 "item.key": lambda d: (d.ellipse([4, 8, 14, 18], fill=rgba("e0c040")), d.rectangle([13, 12, 28, 14], fill=rgba("e0c040")), d.rectangle([24, 14, 26, 18], fill=rgba("e0c040"))),
}
def gen_items():
    for k, f in ITEMS.items():
        save_img(icon(f), f"sprites/items/{k.split('.')[1]}.png", k, f"Pickup/inventory icon: {k} (32x32 x2)", scale=2)

def weapon(kind, frame):
    W_, H_ = 160, 128
    im = Image.new("RGBA", (W_, H_), (0, 0, 0, 0)); d = SDraw(im)
    cx = 90; kick = 8 if frame == 1 else (14 if frame == 2 else 0)
    hand = rgba("e0b090"); sleeve = rgba("1b1b22")
    def arm(x, y):
        d.polygon([(x - 18, H_), (x + 18, H_), (x + 10, y + 22), (x - 10, y + 22)], fill=sleeve)
        d.ellipse([x - 12, y, x + 12, y + 28], fill=hand)
    if kind == "fists":
        arm(60 + (20 if frame == 1 else 0), 70 - (30 if frame == 1 else 0) + kick); arm(120, 80 + kick)
    elif kind == "pistol":
        arm(cx, 80 + kick); d.rectangle([cx - 6, 40 + kick, cx + 6, 82 + kick], fill=rgba("2a2a30")); d.rectangle([cx - 4, 36 + kick, cx + 4, 44 + kick], fill=rgba("444450"))
    elif kind == "shotgun":
        arm(cx + 18, 86 + kick); arm(cx - 8, 100); d.rectangle([cx - 5, 20 + kick, cx + 5, 100 + kick], fill=rgba("2a2a30")); d.rectangle([cx - 5, 60 + kick, cx + 5, 100], fill=rgba("7a4a24"))
    elif kind == "smg":
        arm(cx + 10, 90 + kick); d.rectangle([cx - 8, 30 + kick, cx + 8, 100 + kick], fill=rgba("2a2a30")); d.rectangle([cx - 3, 70, cx + 3, 118], fill=rgba("444450"))
    elif kind == "rifle":
        arm(cx + 20, 86 + kick); arm(cx - 6, 100); d.rectangle([cx - 4, 10 + kick, cx + 4, 110 + kick], fill=rgba("3a3a40")); d.rectangle([cx - 6, 64 + kick, cx + 6, 118], fill=rgba("8a5a2a")); d.rectangle([cx - 5, 40 + kick, cx + 5, 52 + kick], fill=rgba("1a1a1a"))
    if frame == 1 and kind != "fists":
        top = {"pistol": 36, "shotgun": 20, "smg": 30, "rifle": 10}[kind] + kick
        d.ellipse([cx - 22, top - 26, cx + 22, top + 12], fill=rgba("fff3a0")); d.ellipse([cx - 12, top - 16, cx + 12, top + 4], fill=rgba("ffffff"))
    return outline(im)

def gen_weapons():
    meta = {}
    for k in ["fists", "pistol", "shotgun", "smg", "rifle"]:
        sheet = Image.new("RGBA", (160 * 3, 128), (0, 0, 0, 0))
        for f in range(3): sheet.alpha_composite(weapon(k, f), (f * 160, 0))
        rel = f"sprites/weapons/{k}.png"
        save_img(sheet, rel, f"weapon.{k}", f"First-person weapon sheet '{k}': 3 frames 160x128 (idle, fire, recoil)")
        meta[f"weapon.{k}"] = dict(frame=[160, 128], cols=3, rows=1, anims=dict(idle=[0], fire=[1, 2]))
    p = out("data/sprites.json"); cur = json.load(open(p)); cur.update(meta); json.dump(cur, open(p, "w"), indent=1)

# ---- textures
def tex_from(n, cols, size=128):
    lo, hi = [np.array(c[:3], float) for c in cols]
    a = (lo[None, None] * (1 - n[..., None]) + hi[None, None] * n[..., None]).astype(np.uint8)
    return Image.fromarray(a, "RGB").convert("RGBA")

def gen_textures():
    S = 128
    T = {}
    n = tile_noise(S, 5, 1); T["snow"] = tex_from(n, [rgba("dfe8f6"), rgba("fbfdff")])
    n = tile_noise(S, 5, 2); t = tex_from(n, [rgba("b8c4d6"), rgba("e0e8f4")]); T["snow_packed"] = t
    n = tile_noise(S, 3, 3); T["ice"] = tex_from(n, [rgba("8ab6d6"), rgba("c8e4f6")])
    n = tile_noise(S, 6, 4); n2 = (np.random.default_rng(1).random((S, S)) > .97) * .3
    T["asphalt"] = tex_from(np.clip(n * .5 + n2, 0, 1), [rgba("2c2d32"), rgba("55565c")])
    n = tile_noise(S, 6, 5); T["gravel"] = tex_from(n, [rgba("6a6560"), rgba("a8a298")])
    n = tile_noise(S, 4, 6); n = n + 0.15 * np.sin(np.arange(S)[None, :] * 0.8); T["wood"] = tex_from(np.clip(n, 0, 1), [rgba("6a4a2a"), rgba("9a7040")])
    T["wood_red"] = tex_from(tile_noise(S, 4, 7), [rgba("8a2a20"), rgba("b04a38")])  # classic Finnish falu-red
    n = tile_noise(S, 4, 8); T["brick"] = tex_from(n, [rgba("7a3a2a"), rgba("a05a40")])
    n = tile_noise(S, 3, 9); T["metal"] = tex_from(n, [rgba("7a808a"), rgba("b0b6c0")])
    n = tile_noise(S, 5, 10); T["rust"] = tex_from(n, [rgba("5a2a14"), rgba("b0602a")])
    T["glass"] = tex_from(tile_noise(S, 2, 11), [rgba("6a8aa8"), rgba("a8c8e0")])
    n = tile_noise(S, 5, 12); T["dirt_frozen"] = tex_from(n, [rgba("3a2e26"), rgba("6a5a4a")])
    T["roof_dark"] = tex_from(tile_noise(S, 4, 13), [rgba("2a2a30"), rgba("4a4a52")])
    T["floor_wood"] = tex_from(tile_noise(S, 4, 14), [rgba("8a6a40"), rgba("b89058")])
    for k, im in T.items():
        save_img(im, f"textures/{k}.png", f"tex.{k}", f"Tileable 128x128 texture: {k}", notes="seamless; keep tileable")
    # bake brick lines + planks
    return

def house(kind, lit=False):
    W_, H_ = 128, 96
    im = Image.new("RGBA", (W_, H_), (0, 0, 0, 0)); d = SDraw(im)
    walls = {"cabin": rgba("8a2a20"), "barn": rgba("a03024"), "sauna": rgba("6a4a2a"), "shop": rgba("d8d0b8"), "house": rgba("e0c878"), "church": rgba("f0f0e8")}[kind]
    d.rectangle([8, 36, W_ - 8, H_ - 2], fill=walls)
    d.polygon([(2, 38), (W_ - 2, 38), (W_ - 18, 8), (18, 8)], fill=rgba("3a3a42"))
    d.polygon([(6, 36), (W_ - 6, 36), (W_ - 20, 12), (20, 12)], fill=SNOW)
    d.rectangle([54, 56, 74, H_ - 2], fill=rgba("4a3220"))
    glow = rgba("ffd878") if lit else rgba("5a7a9a")
    for x in (18, 92):
        d.rectangle([x, 50, x + 18, 70], fill=glow); d.line([x + 9, 50, x + 9, 70], fill=rgba("f0f0f0")); d.line([x, 60, x + 18, 60], fill=rgba("f0f0f0"))
    if kind == "church":
        d.rectangle([54, 0, 74, 38], fill=walls); d.polygon([(52, 0), (76, 0), (64, -2)], fill=rgba("3a3a42"))
    if kind == "sauna":
        d.rectangle([90, 4, 98, 20], fill=rgba("555560")); d.ellipse([88, -2, 102, 6], fill=rgba("d0d0d8"))
    return outline(im)

def gen_buildings():
    for k in ["cabin", "barn", "sauna", "shop", "house", "church"]:
        for lit in (False, True):
            save_img(house(k, lit), f"sprites/buildings/{k}{'_lit' if lit else ''}.png", f"building.{k}{'_lit' if lit else ''}",
                     f"Building facade billboard '{k}' ({'night/lit windows' if lit else 'day'}) 128x96 x2; used as 2D/3D hybrid far-LOD + wall decal", scale=2)

def gen_ui():
    # crosshair
    im = Image.new("RGBA", (32, 32), (0, 0, 0, 0)); d = SDraw(im)
    for a in [(15, 4, 16, 11), (15, 21, 16, 28), (4, 15, 11, 16), (21, 15, 28, 16)]: d.rectangle(a, fill=rgba("ffffff"))
    save_img(im, "sprites/ui/crosshair.png", "ui.crosshair", "Crosshair 32x32")
    # HUD bar frame
    im = Image.new("RGBA", (256, 24), (0, 0, 0, 0)); d = SDraw(im)
    d.rectangle([0, 0, 255, 23], fill=rgba("14141c", 200), outline=rgba("ffffff")); save_img(im, "sprites/ui/bar_frame.png", "ui.bar_frame", "HUD bar frame 256x24 (health/needs/fuel)")
    # windshield overlays: 5 coverage levels, 512x256
    S = 256
    for lvl in range(1, 6):
        rng = np.random.default_rng(lvl)
        cov = lvl / 5.0 * 0.7
        n = tile_noise(S, 5, 40 + lvl)
        n2 = np.array(Image.fromarray((n * 255).astype(np.uint8)).resize((512, 256), Image.BILINEAR)) / 255.0
        # snow clumps stronger at the bottom
        yy = np.linspace(0, 1, 256)[:, None]
        thresh = 1.0 - cov * (0.55 + 0.7 * yy)
        a = np.clip((n2 - thresh) * 8, 0, 1)
        flakes = (rng.random((256, 512)) < cov * 0.04)
        a = np.maximum(a, flakes * 0.9)
        rgbaarr = np.zeros((256, 512, 4), np.uint8); rgbaarr[..., :3] = [238, 244, 255]; rgbaarr[..., 3] = (a * 215).astype(np.uint8)
        save_img(Image.fromarray(rgbaarr, "RGBA"), f"sprites/ui/windshield_snow_{lvl}.png", f"ui.windshield_snow_{lvl}", f"Windshield snow overlay level {lvl}/5 (512x256, blended over view; wipers clear a sweep). Amiga-Celica style")
    # frost overlay
    n = tile_noise(256, 6, 55); n2 = np.array(Image.fromarray((n * 255).astype(np.uint8)).resize((512, 256), Image.BILINEAR)) / 255.0
    ed = np.linspace(-1, 1, 512)[None, :] ** 2 + np.linspace(-1, 1, 256)[:, None] ** 2
    a = np.clip((ed - 0.35) * 0.9 + (n2 - 0.5) * 0.7, 0, 1)
    arr = np.zeros((256, 512, 4), np.uint8); arr[..., :3] = [200, 225, 245]; arr[..., 3] = (a * 200).astype(np.uint8)
    save_img(Image.fromarray(arr, "RGBA"), "sprites/ui/windshield_frost.png", "ui.windshield_frost", "Windshield frost overlay (edges frosted), cleared by defrost")
    # wiper
    im = Image.new("RGBA", (128, 8), (0, 0, 0, 0)); SDraw(im).rectangle([0, 2, 127, 5], fill=rgba("101014")); save_img(im, "sprites/ui/wiper.png", "ui.wiper", "Wiper blade 128x8")
    # bitmap font
    try: f = ImageFont.load_default(size=14)
    except TypeError: f = ImageFont.load_default()
    chars = "".join(chr(c) for c in range(32, 128))
    im = Image.new("RGBA", (16 * 12, 6 * 18), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
    for i, ch in enumerate(chars): d.text(((i % 16) * 12 + 1, (i // 16) * 18), ch, font=f, fill=(255, 255, 255, 255))
    save_img(im, "sprites/ui/font_atlas.png", "ui.font", "Bitmap font atlas ASCII 32..127, 16x6 grid of 12x18 cells (fallback; real TTF can replace)")

if __name__ == "__main__":
    gen_textures(); gen_buildings(); gen_ui(); print("sprites ok")

def gen_fx():
    """Soft puff sprite for smoke / breath / snow spray."""
    S = 64
    y, x = np.mgrid[0:S, 0:S]; r = np.hypot(x - S / 2 + .5, y - S / 2 + .5) / (S / 2)
    n = tile_noise(S, 4, 77)
    a = np.clip(1 - r, 0, 1) ** 1.6 * (0.7 + 0.5 * n)
    arr = np.zeros((S, S, 4), np.uint8); arr[..., :3] = 255; arr[..., 3] = (np.clip(a, 0, 1) * 255).astype(np.uint8)
    save_img(Image.fromarray(arr, "RGBA"), "sprites/fx/puff.png", "fx.puff", "Soft puff 64x64 (breath vapour, exhaust, steam, snow spray, chimney smoke)")
    # soft disc for sun / moon
    S = 128; y, x = np.mgrid[0:S, 0:S]; r = np.hypot(x - S / 2 + .5, y - S / 2 + .5) / (S / 2)
    a = np.clip((1 - r) * 6, 0, 1); glow = np.clip(1 - r, 0, 1) ** 2 * 0.5
    arr = np.zeros((S, S, 4), np.uint8); arr[..., :3] = 255; arr[..., 3] = (np.clip(a + glow, 0, 1) * 255).astype(np.uint8)
    save_img(Image.fromarray(arr, "RGBA"), "sprites/fx/disc.png", "fx.disc", "Sun/moon disc with glow 128x128")

if __name__ == "__main__":
    gen_fx()
