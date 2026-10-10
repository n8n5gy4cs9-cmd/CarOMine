"""Tiny software SDF ray-marcher that produces Doom / Rise-of-the-Triad style pixel sprites.
Scene = list of primitives (sphere / capsule / box / ellipsoid) with materials. Orthographic or perspective camera looking down -Z.
Shading: lambert + AO from the SDF, quantised to a per-material colour ramp with ordered dithering; depth-edge outlines.
All numpy; ~0.1 s per 72x96 frame."""
import numpy as np
from PIL import Image

BAYER = np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]], float) / 16.0

def ramp(base, n=5, warm=0.07):
    """Colour ramp dark->light: shadows shift towards blue/purple, highlights towards warm yellow."""
    b = np.array(base[:3], float) / 255.0
    out = []
    for i in range(n):
        t = i / (n - 1)                       # 0 dark .. 1 light
        lum = 0.36 + 0.9 * t
        c = b * lum
        shadow = (1 - t) ** 1.5
        c = c * (1 - 0.35 * shadow) + np.array([0.10, 0.12, 0.30]) * 0.35 * shadow * (0.4 + b.mean())
        c = c + np.array([1.0, 0.85, 0.5]) * warm * t ** 3
        out.append(np.clip(c, 0, 1))
    return (np.array(out) * 255).astype(np.uint8)

def shift(a, dy, dx, fill=0):
    out = np.full_like(a, fill)
    h, w = a.shape[:2]
    ys, yd = (slice(dy, h), slice(0, h - dy)) if dy >= 0 else (slice(0, h + dy), slice(-dy, h))
    xs, xd = (slice(dx, w), slice(0, w - dx)) if dx >= 0 else (slice(0, w + dx), slice(-dx, w))
    out[ys, xs] = a[yd, xd]
    return out

class Mat:
    def __init__(self, base, noise=None, nscale=40.0, namp=0.12, spec=0.0, emissive=False, n=5):
        self.base, self.noise, self.nscale, self.namp, self.spec, self.emissive = base, noise, nscale, namp, spec, emissive
        self.ramp = ramp(base, n)

def hash3(p, seed=0.0):
    x = np.sin(p[:, 0] * 127.1 + p[:, 1] * 311.7 + p[:, 2] * 74.7 + seed) * 43758.5453
    return x - np.floor(x)

def vnoise(p, scale, seed=0.0):
    q = p * scale; i = np.floor(q); f = q - i; f = f * f * (3 - 2 * f)
    def h(dx, dy, dz): return hash3(i + np.array([dx, dy, dz]), seed)
    c000, c100, c010, c110 = h(0, 0, 0), h(1, 0, 0), h(0, 1, 0), h(1, 1, 0)
    c001, c101, c011, c111 = h(0, 0, 1), h(1, 0, 1), h(0, 1, 1), h(1, 1, 1)
    fx, fy, fz = f[:, 0], f[:, 1], f[:, 2]
    x00 = c000 + (c100 - c000) * fx; x10 = c010 + (c110 - c010) * fx; x01 = c001 + (c101 - c001) * fx; x11 = c011 + (c111 - c011) * fx
    y0 = x00 + (x10 - x00) * fy; y1 = x01 + (x11 - x01) * fy
    return y0 + (y1 - y0) * fz

class Scene:
    def __init__(self): self.prims = []
    def sphere(self, c, r, m): self.prims.append(("s", np.array(c, float), float(r), m)); return self
    def capsule(self, a, b, r1, r2, m): self.prims.append(("c", np.array(a, float), np.array(b, float), float(r1), float(r2), m)); return self
    def box(self, c, half, m, rot=None, rnd=0.01): self.prims.append(("b", np.array(c, float), np.array(half, float), np.eye(3) if rot is None else np.array(rot, float), float(rnd), m)); return self
    def ellipsoid(self, c, radii, m, rot=None): self.prims.append(("e", np.array(c, float), np.array(radii, float), np.eye(3) if rot is None else np.array(rot, float), m)); return self
    def transform(self, R=None, t=(0, 0, 0)):
        """Return a new scene with every primitive rotated by R (3x3 about origin) then translated."""
        R = np.eye(3) if R is None else np.array(R, float); t = np.array(t, float); s = Scene()
        for p in self.prims:
            k = p[0]
            if k == "s": s.prims.append(("s", R @ p[1] + t, p[2], p[3]))
            elif k == "c": s.prims.append(("c", R @ p[1] + t, R @ p[2] + t, p[3], p[4], p[5]))
            elif k == "b": s.prims.append(("b", R @ p[1] + t, p[2], R @ p[3], p[4], p[5]))
            else: s.prims.append(("e", R @ p[1] + t, p[2], R @ p[3], p[4]))
        return s
    def scaled(self, k):
        """Uniformly scale everything (positions AND radii) about the origin."""
        s = Scene()
        for p in self.prims:
            kd = p[0]
            if kd == "s": s.prims.append(("s", p[1] * k, p[2] * k, p[3]))
            elif kd == "c": s.prims.append(("c", p[1] * k, p[2] * k, p[3] * k, p[4] * k, p[5]))
            elif kd == "b": s.prims.append(("b", p[1] * k, p[2] * k, p[3], p[4] * k, p[5]))
            else: s.prims.append(("e", p[1] * k, p[2] * k, p[3], p[4]))
        return s
    def merge(self, other): self.prims += other.prims; return self

def rot_y(a): c, s = np.cos(a), np.sin(a); return np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])
def rot_x(a): c, s = np.cos(a), np.sin(a); return np.array([[1, 0, 0], [0, c, -s], [0, s, c]])
def rot_z(a): c, s = np.cos(a), np.sin(a); return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])

def _dist(prim, P):
    k = prim[0]
    if k == "s": return np.linalg.norm(P - prim[1], axis=1) - prim[2]
    if k == "c":
        a, b, r1, r2 = prim[1], prim[2], prim[3], prim[4]
        ba = b - a; pa = P - a
        h = np.clip((pa @ ba) / max(ba @ ba, 1e-9), 0, 1)
        return np.linalg.norm(pa - h[:, None] * ba, axis=1) - (r1 + (r2 - r1) * h)
    if k == "b":
        c, half, R, rnd = prim[1], prim[2], prim[3], prim[4]
        q = np.abs((P - c) @ R) - half + rnd
        return np.linalg.norm(np.maximum(q, 0), axis=1) + np.minimum(q.max(axis=1), 0) - rnd
    c, rad, R = prim[1], prim[2], prim[3]
    q = ((P - c) @ R) / rad
    k0 = np.linalg.norm(q, axis=1); k1 = np.linalg.norm(q / rad, axis=1)
    return k0 * (k0 - 1) / np.maximum(k1, 1e-9)

def scene_sdf(scene, P):
    best = np.full(len(P), 1e9); idx = np.zeros(len(P), int)
    for i, pr in enumerate(scene.prims):
        d = _dist(pr, P); m = d < best; best = np.where(m, d, best); idx = np.where(m, i, idx)
    return best, idx

def material_of(pr): return pr[-1]

LIGHT = np.array([-0.55, 0.65, 0.55]); LIGHT = LIGHT / np.linalg.norm(LIGHT)

def render(scene, w=72, h=96, x_range=(-0.75, 0.75), y_range=(0.0, 2.0), steps=44, outline=(8, 6, 14), light=None, ambient=0.28, perspective=None, dither=0.55, edge=0.30):
    """Returns (RGBA image). Orthographic by default; perspective=(eye_z, focal) for first-person weapon renders."""
    lt = LIGHT if light is None else np.array(light) / np.linalg.norm(light)
    xs = np.linspace(x_range[0], x_range[1], w, endpoint=False) + (x_range[1] - x_range[0]) / w / 2
    ys = np.linspace(y_range[1], y_range[0], h, endpoint=False) - (y_range[1] - y_range[0]) / h / 2
    X, Y = np.meshgrid(xs, ys)
    X = X.ravel(); Y = Y.ravel(); N = len(X)
    if perspective is None:
        O = np.stack([X, Y, np.full(N, 6.0)], 1); D = np.tile([0.0, 0.0, -1.0], (N, 1))
    else:
        ez, f = perspective
        O = np.tile([0.0, 0.0, ez], (N, 1)); D = np.stack([X / f, Y / f, np.full(N, -1.0)], 1); D /= np.linalg.norm(D, axis=1, keepdims=True)
    t = np.zeros(N); hit = np.zeros(N, bool); alive = np.ones(N, bool)
    for _ in range(steps):
        idx = np.where(alive)[0]
        if len(idx) == 0: break
        P = O[idx] + D[idx] * t[idx, None]
        d, _ = scene_sdf(scene, P)
        done = d < 0.004
        hit[idx[done]] = True
        t[idx] += np.maximum(d, 0.002)
        alive[idx[done]] = False
        alive[idx[t[idx] > 12.0]] = False
    img = np.zeros((N, 4), np.uint8); depth = np.full(N, 1e9)
    hi = np.where(hit)[0]
    if len(hi):
        P = O[hi] + D[hi] * t[hi, None]; depth[hi] = t[hi]
        e = 0.006
        def sd(Q): return scene_sdf(scene, Q)[0]
        n = np.stack([sd(P + [e, 0, 0]) - sd(P - [e, 0, 0]), sd(P + [0, e, 0]) - sd(P - [0, e, 0]), sd(P + [0, 0, e]) - sd(P - [0, 0, e])], 1)
        n /= np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-9)
        dd, mi = scene_sdf(scene, P)
        diff = np.clip(n @ lt, 0, 1)
        ao = np.clip(sd(P + n * 0.07) / 0.07, 0, 1) * 0.5 + np.clip(sd(P + n * 0.18) / 0.18, 0, 1) * 0.5
        v = ambient + (1 - ambient) * diff
        v *= 0.55 + 0.45 * ao
        mats = [material_of(p) for p in scene.prims]
        bay = BAYER[(hi // w) % 4, (hi % w) % 4]
        out = np.zeros((len(hi), 3), np.uint8)
        for k in np.unique(mi):
            sel = np.where(mi == k)[0]; m = mats[k]
            vv = v[sel].copy()
            if m.noise:
                nz = vnoise(P[sel], m.nscale, seed=float(k)) if m.noise != "streak" else vnoise(P[sel] * np.array([1.0, 0.25, 1.0]), m.nscale)
                vv += (nz - 0.5) * m.namp * 2
            if m.spec:
                hv = lt + np.array([0, 0, 1.0]); hv /= np.linalg.norm(hv)
                vv += m.spec * np.clip(n[sel] @ hv, 0, 1) ** 24
            nr = len(m.ramp)
            if m.emissive: idxr = np.full(len(sel), nr - 1)
            else: idxr = np.clip(np.floor(vv * (nr - 1) + (bay[sel] - 0.5) * dither + 0.5), 0, nr - 1).astype(int)
            out[sel] = m.ramp[idxr]
        img[hi, :3] = out; img[hi, 3] = 255
    img = img.reshape(h, w, 4); depth = depth.reshape(h, w)
    # internal edges from depth discontinuities (separates limbs from torso like hand-painted sprites)
    if edge:
        dz = np.zeros((h, w), bool)
        for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)):
            sh = shift(depth, dy, dx, 1e9); dz |= (np.abs(depth - sh) > edge) & (depth < 1e8) & (sh < depth - edge)
        dark = (img[..., :3] * 0.42).astype(np.uint8)
        img[..., :3] = np.where(dz[..., None], dark, img[..., :3])
    im = Image.fromarray(img, "RGBA")
    # 1px outside outline
    a = np.array(im)[..., 3] > 0
    grown = a.copy()
    for dy, dx in ((0, 1), (1, 0), (0, -1), (-1, 0)): grown |= shift(a, dy, dx, False)
    ol = np.zeros((h, w, 4), np.uint8); ol[grown & ~a] = (*outline, 255)
    base = Image.fromarray(ol, "RGBA"); base.alpha_composite(im)
    return base
