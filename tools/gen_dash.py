#!/usr/bin/env python3
"""Dashboard art: analog dial faces, needles and warning-light icons (pixel art, drawn at low res)."""
import math
import numpy as np
from PIL import Image, ImageDraw, ImageFont
from assetlib import *

def font(sz):
    try: return ImageFont.load_default(size=sz)
    except TypeError: return ImageFont.load_default()

def pol(cx, cy, r, deg):
    """deg measured clockwise from 'up'."""
    a = math.radians(deg); return (cx + r * math.sin(a), cy - r * math.cos(a))

def dial(S, lo, hi, sweep, major, minor, labels, redzone=None, title="", sub="", start=-135.0, num_scale=1.0, yellow=None):
    im = Image.new("RGBA", (S, S), (0, 0, 0, 0)); d = ImageDraw.Draw(im); c = S / 2
    # bezel (chrome ring with a lit upper-left edge) + face
    for r in range(int(c), int(c * 0.88), -1):
        t = (c - r) / (c * 0.12); v = int(70 + 120 * (1 - abs(t - 0.5) * 2) ** 1.5)
        d.ellipse([c - r, c - r, c + r, c + r], fill=(v, v + 4, v + 12, 255))
    d.ellipse([c - c * 0.88, c - c * 0.88, c + c * 0.88, c + c * 0.88], fill=(16, 17, 20, 255))
    for k in range(60):                                    # subtle glow towards the rim (backlight)
        r = c * 0.88 * (1 - k / 90)
        sh = int(8 + 22 * (k / 60) ** 2)
        d.ellipse([c - r, c - r, c + r, c + r], outline=(sh + 6, sh + 4, sh, 255))
    d.ellipse([c - c * 0.84, c - c * 0.84, c + c * 0.84, c + c * 0.84], fill=(20, 21, 25, 255))
    def ang(v): return start + sweep * (v - lo) / (hi - lo)
    if redzone:
        a0, a1 = ang(redzone[0]), ang(redzone[1])
        for r in (c * 0.80, c * 0.79, c * 0.78, c * 0.77):
            pts = [pol(c, c, r, a0 + (a1 - a0) * i / 40) for i in range(41)]; d.line(pts, fill=(200, 24, 24, 255), width=2)
    if yellow:
        a0, a1 = ang(yellow[0]), ang(yellow[1]); pts = [pol(c, c, c * 0.79, a0 + (a1 - a0) * i / 40) for i in range(41)]; d.line(pts, fill=(220, 170, 30, 255), width=2)
    v = lo
    while v <= hi + 1e-6:
        d.line([pol(c, c, c * 0.84, ang(v)), pol(c, c, c * 0.68, ang(v))], fill=(235, 232, 220, 255), width=max(1, S // 70)); v += major
    v = lo
    while v <= hi + 1e-6:
        d.line([pol(c, c, c * 0.84, ang(v)), pol(c, c, c * 0.76, ang(v))], fill=(150, 148, 140, 255), width=1); v += minor
    f = font(max(7, int(S * 0.095 * num_scale)))
    for v, txt in labels:
        x, y = pol(c, c, c * 0.54, ang(v)); w = d.textlength(txt, font=f); d.text((x - w / 2, y - S * 0.045 * num_scale), txt, font=f, fill=(238, 232, 214, 255))
    if title:
        ft = font(max(6, int(S * 0.075))); w = d.textlength(title, font=ft); d.text((c - w / 2, c + c * 0.34), title, font=ft, fill=(255, 190, 90, 255))
    if sub:
        ft = font(max(6, int(S * 0.065))); w = d.textlength(sub, font=ft); d.text((c - w / 2, c + c * 0.5), sub, font=ft, fill=(170, 170, 160, 255))
    # glass glint
    g = Image.new("RGBA", (S, S), (0, 0, 0, 0)); gd = ImageDraw.Draw(g)
    gd.pieslice([c - c * 0.8, c - c * 0.8, c + c * 0.8, c + c * 0.8], 200, 260, fill=(255, 255, 255, 22)); im.alpha_composite(g)
    return im

def needle(S, col=(235, 60, 30), length=0.74, w=0.045):
    im = Image.new("RGBA", (S, S), (0, 0, 0, 0)); d = ImageDraw.Draw(im); c = S / 2
    d.polygon([(c - S * w, c), (c + S * w, c), (c + S * w * 0.2, c - c * length), (c - S * w * 0.2, c - c * length)], fill=col + (255,))
    d.polygon([(c - S * w * 0.6, c), (c, c + c * 0.2), (c + S * w * 0.6, c)], fill=col + (255,))                       # short tail
    d.polygon([(c - S * w * 0.5, c), (c - S * w * 0.15, c - c * length * 0.95), (c, c - c * length * 0.95), (c, c)], fill=tuple(min(255, int(x * 1.25)) for x in col) + (255,))  # highlight
    d.ellipse([c - S * 0.075, c - S * 0.075, c + S * 0.075, c + S * 0.075], fill=(40, 40, 46, 255), outline=(150, 150, 160, 255))
    return im

def icon(i, S=32):
    """White silhouettes (the game tints them: dim when off, red/amber/green/blue when on)."""
    im = Image.new("RGBA", (S, S), (0, 0, 0, 0)); d = ImageDraw.Draw(im); W = (255, 255, 255, 255)
    c = S // 2
    if i == 0:   # battery
        d.rectangle([4, 10, 27, 24], outline=W, width=2); d.rectangle([8, 7, 12, 10], fill=W); d.rectangle([19, 7, 23, 10], fill=W)
        d.line([8, 17, 13, 17], fill=W, width=2); d.line([19, 17, 24, 17], fill=W, width=2); d.line([21.5, 14.5, 21.5, 19.5], fill=W, width=2)
    elif i == 1: # oil can
        d.polygon([(6, 14), (20, 14), (20, 24), (6, 24)], outline=W, fill=None); d.line([6, 14, 6, 24, 20, 24, 20, 14, 6, 14], fill=W, width=2)
        d.line([20, 16, 27, 11], fill=W, width=2); d.line([9, 14, 9, 10, 15, 10], fill=W, width=2); d.ellipse([26, 18, 29, 24], fill=W)
    elif i == 2: # engine block
        d.rectangle([7, 12, 24, 23], outline=W, width=2); d.rectangle([11, 8, 20, 12], fill=W); d.line([4, 15, 7, 15], fill=W, width=2); d.line([24, 15, 28, 15], fill=W, width=2)
        d.line([4, 22, 7, 22], fill=W, width=2); d.line([24, 22, 28, 22], fill=W, width=2); d.line([15, 5, 15, 8], fill=W, width=2)
    elif i == 3: # brake / handbrake: (!)
        d.ellipse([8, 8, 24, 24], outline=W, width=2); d.line([16, 12, 16, 18], fill=W, width=2); d.rectangle([15, 20, 17, 21], fill=W)
        d.arc([3, 5, 29, 27], 110, 250, fill=W, width=2); d.arc([3, 5, 29, 27], 290, 70, fill=W, width=2)
    elif i == 4: # coolant temperature: thermometer in waves
        d.rectangle([7, 5, 10, 18], outline=W, width=1); d.ellipse([5, 17, 12, 24], fill=W)
        for y in (13, 19, 25): d.line([14, y, 17, y - 2, 20, y, 23, y - 2, 26, y], fill=W, width=2)
    elif i == 5: # fuel pump
        d.rectangle([7, 7, 19, 24], outline=W, width=2); d.rectangle([9, 9, 17, 14], fill=W); d.line([19, 12, 24, 12, 25, 22], fill=W, width=2); d.line([4, 25, 22, 25], fill=W, width=2)
    elif i == 6: # headlights
        d.pieslice([5, 8, 17, 24], 90, 270, outline=W, width=2); d.line([5, 16, 11, 16], fill=None)
        for y in (10, 15, 20): d.line([19, y, 28, y], fill=W, width=2)
    elif i == 7: # ice / snowflake
        for a in (0, 60, 120):
            x, y = 12 * math.cos(math.radians(a)), 12 * math.sin(math.radians(a)); d.line([c - x, c - y, c + x, c + y], fill=W, width=2)
            bx, by = 7 * math.cos(math.radians(a)), 7 * math.sin(math.radians(a))
            for s in (-1, 1): d.line([c + bx, c + by, c + bx + s * 3 * math.cos(math.radians(a + 90)) , c + by + s * 3 * math.sin(math.radians(a + 90))], fill=W, width=1)
    elif i == 8: # wipers
        d.arc([2, 6, 30, 34], 200, 340, fill=W, width=2); d.line([c, 26, 26, 12], fill=W, width=2); d.line([c, 26, 8, 12], fill=W, width=2)
    return im

def main():
    S = 128
    imgs = {}
    imgs["dash_tacho"] = dial(S, 0, 7, 270, 1, 0.5, [(i, str(i)) for i in range(0, 8)], redzone=(5.6, 7), title="x1000", sub="r/min")
    imgs["dash_speedo"] = dial(S, 0, 180, 270, 20, 10, [(v, str(v)) for v in range(0, 181, 20)], title="km/h", sub="", num_scale=0.9)
    sm = 80
    imgs["dash_fuel"] = dial(sm, 0, 1, 90, 0.25, 0.125, [(0, "E"), (1, "F")], redzone=(0, 0.12), title="FUEL", start=-45.0, num_scale=1.4)
    imgs["dash_temp"] = dial(sm, 40, 120, 90, 20, 10, [(40, "C"), (120, "H")], redzone=(105, 120), title="TEMP", start=-45.0, num_scale=1.4)
    imgs["dash_oil"] = dial(sm, 0, 1, 90, 0.5, 0.25, [(0, "0"), (1, "OIL")], redzone=(0, 0.2), title="OIL", start=-45.0, num_scale=1.2)
    imgs["dash_volt"] = dial(sm, 10, 16, 90, 2, 1, [(10, "10"), (16, "16")], redzone=(10, 11.5), title="VOLT", start=-45.0, num_scale=1.2)
    for k, im in imgs.items():
        rel = f"sprites/ui/{k}.png"; im.save(out(rel)); reg(f"ui.{k}", rel, "sprite", f"Dashboard dial face '{k}' ({im.width}px; game shows it scaled up, nearest)", source="generated (tools/gen_dash.py)")
    for nm, S2, col in (("dash_needle", 128, (235, 70, 30)), ("dash_needle_small", 80, (250, 170, 40))):
        im = needle(S2, col, 0.78 if S2 == 128 else 0.7); rel = f"sprites/ui/{nm}.png"; im.save(out(rel))
        reg(f"ui.{nm}", rel, "sprite", f"Dial needle {S2}px: points UP, rotated about the image centre by the game", source="generated (tools/gen_dash.py)")
    sheet = Image.new("RGBA", (32 * 9, 32), (0, 0, 0, 0))
    for i in range(9): sheet.alpha_composite(icon(i), (i * 32, 0))
    rel = "sprites/ui/dash_icons.png"; sheet.save(out(rel))
    reg("ui.dash_icons", rel, "sprite", "Warning-light icon strip: 9 white 32x32 silhouettes (battery, oil, engine, brake, temp, fuel, lights, ice, wipers); tinted by the game",
        source="generated (tools/gen_dash.py)", notes="order: 0 battery,1 oil,2 check engine,3 brake,4 coolant temp,5 fuel,6 lights,7 ice,8 wipers")
    print("dash ok")

if __name__ == "__main__": main()
