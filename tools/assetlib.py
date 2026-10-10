"""Shared helpers for CarOMine placeholder asset generators (CC0, by Crowelian 2026 + Sonnet).
Every generated file is registered into assets/manifest.json via reg()."""
import json, pathlib, random
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

ROOT = pathlib.Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
MANIFEST = ASSETS / "manifest.json"

def _load():
    return json.load(open(MANIFEST)) if MANIFEST.exists() else {}

def reg(id, path, kind, use, source="generated (tools/)", license="CC0", notes=""):
    """Register asset. path is relative to assets/. Merges into manifest.json."""
    m = _load()
    m[id] = {"path": path, "kind": kind, "use": use, "source": source, "license": license, "notes": notes}
    json.dump(dict(sorted(m.items())), open(MANIFEST, "w"), indent=1)

def out(rel):
    p = ASSETS / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    return p

def save_img(img, rel, id, use, notes="", scale=1):
    if scale != 1:
        img = img.resize((img.width * scale, img.height * scale), Image.NEAREST)
    img.save(out(rel))
    reg(id, rel, "sprite" if rel.startswith("sprites") else "texture", use, notes=notes + f" size={img.width}x{img.height}")

def outline(img, col=(10, 12, 18, 255)):
    """1px dark outline around opaque pixels."""
    a = img.split()[3]
    grown = a.filter(ImageFilter.MaxFilter(3))
    base = Image.new("RGBA", img.size, col)
    base.putalpha(grown)
    base.alpha_composite(img)
    return base

def tile_noise(size, octaves=4, seed=0):
    """Tileable value noise 0..1 (wraps on both axes)."""
    rng = np.random.default_rng(seed)
    acc = np.zeros((size, size)); amp = 1.0; tot = 0
    for o in range(octaves):
        n = 4 * 2 ** o
        g = rng.random((n, n))
        # bilinear upsample with wrap
        xs = np.linspace(0, n, size, endpoint=False)
        i0 = xs.astype(int) % n; i1 = (i0 + 1) % n; f = xs - np.floor(xs)
        f = f * f * (3 - 2 * f)
        rows = g[i0][:, :] * (1 - f)[:, None] + g[i1][:, :] * f[:, None]
        up = rows[:, i0] * (1 - f)[None, :] + rows[:, i1] * f[None, :]
        acc += up * amp; tot += amp; amp *= 0.5
    return acc / tot

def rgba(h, a=255):
    h = h.lstrip("#")
    if len(h) == 3: h = "".join(c * 2 for c in h)
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)

class SDraw(ImageDraw.ImageDraw):
    """ImageDraw that tolerates reversed rectangle/ellipse coords."""
    def rectangle(self, xy, *a, **k):
        x0, y0, x1, y1 = xy
        return super().rectangle([min(x0, x1), min(y0, y1), max(x0, x1), max(y0, y1)], *a, **k)
    def ellipse(self, xy, *a, **k):
        x0, y0, x1, y1 = xy
        return super().ellipse([min(x0, x1), min(y0, y1), max(x0, x1), max(y0, y1)], *a, **k)
    def rounded_rectangle(self, xy, *a, **k):
        x0, y0, x1, y1 = xy
        return super().rounded_rectangle([min(x0, x1), min(y0, y1), max(x0, x1), max(y0, y1)], *a, **k)
