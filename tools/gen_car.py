#!/usr/bin/env python3
"""T-029b: smooth car + wheel meshes from signed-distance fields (marching cubes) -> assets/models/car_body.glb, car_wheel.glb.
Coordinates: metres, +Y up, -Z forward, origin at the middle of the wheelbase on the ground. Wheels are separate (car.wheel)."""
import sys
import numpy as np
from skimage import measure
from assetlib import *
from glb import Mesh

G = 0.03

def grid(x0, x1, y0, y1, z0, z1, g):
    xs = np.arange(x0, x1 + g, g, dtype=np.float32); ys = np.arange(y0, y1 + g, g, dtype=np.float32); zs = np.arange(z0, z1 + g, g, dtype=np.float32)
    X, Y, Z = np.meshgrid(xs, ys, zs, indexing="ij"); return (xs, ys, zs), X, Y, Z

def smin(a, b, k):
    h = np.clip(0.5 + 0.5 * (b - a) / k, 0, 1); return b * (1 - h) + a * h - k * h * (1 - h)

class F:
    def __init__(self, X, Y, Z): self.X, self.Y, self.Z = X, Y, Z
    def rbox(self, c, h, r=0.0, rx=0.0):
        y = self.Y - c[1]; z = self.Z - c[2]
        if rx: cs, sn = np.cos(rx), np.sin(rx); y, z = y * cs - z * sn, y * sn + z * cs
        q0 = np.abs(self.X - c[0]) - h[0] + r; q1 = np.abs(y) - h[1] + r; q2 = np.abs(z) - h[2] + r
        return np.sqrt(np.maximum(q0, 0) ** 2 + np.maximum(q1, 0) ** 2 + np.maximum(q2, 0) ** 2) + np.minimum(np.maximum(q0, np.maximum(q1, q2)), 0) - r
    def sphere(self, c, r): return np.sqrt((self.X - c[0]) ** 2 + (self.Y - c[1]) ** 2 + (self.Z - c[2]) ** 2) - r
    def capsule(self, a, b, r):
        a = np.array(a, np.float32); b = np.array(b, np.float32); ba = b - a
        px, py, pz = self.X - a[0], self.Y - a[1], self.Z - a[2]
        h = np.clip((px * ba[0] + py * ba[1] + pz * ba[2]) / float(ba @ ba), 0, 1)
        return np.sqrt((px - h * ba[0]) ** 2 + (py - h * ba[1]) ** 2 + (pz - h * ba[2]) ** 2) - r
    def cyl_x(self, cy, cz, r, x0, x1):
        dx = np.abs(self.X - (x0 + x1) / 2) - (x1 - x0) / 2; dr = np.sqrt((self.Y - cy) ** 2 + (self.Z - cz) ** 2) - r
        return np.sqrt(np.maximum(dx, 0) ** 2 + np.maximum(dr, 0) ** 2) + np.minimum(np.maximum(dx, dr), 0)
    def torus(self, c, R, r, tilt):
        y = self.Y - c[1]; z = self.Z - c[2]; cs, sn = np.cos(tilt), np.sin(tilt); y2, z2 = y * cs - z * sn, y * sn + z * cs
        x = self.X - c[0]; return np.sqrt((np.sqrt(x * x + y2 * y2) - R) ** 2 + z2 * z2) - r

def mesh_from_field(fl, origin, g):
    """Marching cubes of one SDF field -> (verts, faces, smooth outward normals) or None if empty."""
    if not (fl.min() < 0 < fl.max()): return None
    verts, faces, _n, _v = measure.marching_cubes(fl, 0.0, spacing=(g, g, g))
    verts = (verts + origin).astype(np.float32)
    gx, gy, gz = np.gradient(fl, g)
    idx = np.clip(np.round((verts - origin) / g).astype(int), 0, np.array(fl.shape) - 1)
    nrm = np.stack([gx[idx[:, 0], idx[:, 1], idx[:, 2]], gy[idx[:, 0], idx[:, 1], idx[:, 2]], gz[idx[:, 0], idx[:, 1], idx[:, 2]]], 1)
    nrm /= np.maximum(np.linalg.norm(nrm, axis=1, keepdims=True), 1e-9)
    tri = verts[faces]; gn = np.cross(tri[:, 1] - tri[:, 0], tri[:, 2] - tri[:, 0]); an = nrm[faces].sum(1)
    flip = (gn * an).sum(1) < 0; faces = faces.copy(); faces[flip] = faces[flip][:, ::-1]
    return verts, faces, nrm

def hashn(p, seed=0.0):
    x = np.sin(p[:, 0] * 127.1 + p[:, 1] * 311.7 + p[:, 2] * 74.7 + seed) * 43758.5453
    return x - np.floor(x)

def vnoise3(p, scale, seed=0.0):
    q = p * scale; i = np.floor(q); f = q - i; f = f * f * (3 - 2 * f)
    def h(a, b, c): return hashn(i + np.array([a, b, c]), seed)
    x00 = h(0, 0, 0) + (h(1, 0, 0) - h(0, 0, 0)) * f[:, 0]; x10 = h(0, 1, 0) + (h(1, 1, 0) - h(0, 1, 0)) * f[:, 0]
    x01 = h(0, 0, 1) + (h(1, 0, 1) - h(0, 0, 1)) * f[:, 0]; x11 = h(0, 1, 1) + (h(1, 1, 1) - h(0, 1, 1)) * f[:, 0]
    y0 = x00 + (x10 - x00) * f[:, 1]; y1 = x01 + (x11 - x01) * f[:, 1]
    return y0 + (y1 - y0) * f[:, 2]

def fbm3(p, scale, seed=0.0, o=3):
    a, tot, acc = 1.0, 0.0, 0.0
    for k in range(o): acc += vnoise3(p, scale * 2 ** k, seed + k * 13) * a; tot += a; a *= 0.5
    return acc / tot

def grime_colors(kind, verts, nrm):
    """Baked per-vertex dirt: road slush up the lower body + arches + rear, rust at sills/arch lips, grime streaks, paint chips. Returns RGBA float colours."""
    y, z = verts[:, 1], verts[:, 2]; x = verts[:, 0]
    # distance (in the yz plane) to the nearest wheel centre
    dw = np.minimum(np.hypot(y - 0.34, z + 1.35), np.hypot(y - 0.34, z - 1.35))
    low = np.clip(1.25 - y * 1.35, 0, 1)                                   # dirtier near the ground
    arch = np.clip(1.0 - (dw - 0.42) / 0.5, 0, 1) ** 1.3                    # splash zone around the wheels
    rear = np.clip((z - 0.2) / 2.0, 0, 1) * 0.25                            # slush kicked up behind
    blot = fbm3(verts, 3.2, 1.0); speck = vnoise3(verts, 42.0, 5.0); streak = vnoise3(verts * np.array([5.0, 0.7, 5.0]), 1.0, 9.0)
    dirt = np.clip(low * 0.75 + arch * 0.6 + rear + (blot - 0.5) * 0.5 + (streak - 0.5) * 0.28 * (nrm[:, 1] < 0.5), 0, 1)
    dirt = np.clip(dirt * (0.85 + 0.3 * speck), 0, 1)
    base = {"paint": np.array([0.58, 0.11, 0.12]), "trim": np.array([0.13, 0.13, 0.15]), "glass": np.array([0.42, 0.58, 0.68]), "chrome": np.array([0.74, 0.76, 0.8]),
            "tyre": np.array([0.08, 0.08, 0.09]), "rim": np.array([0.66, 0.68, 0.72]), "dark": np.array([0.2, 0.19, 0.2]), "seat": np.array([0.45, 0.4, 0.34])}.get(kind, np.array([0.6, 0.6, 0.6]))
    grime = np.array([0.30, 0.25, 0.20]) * (0.8 + 0.4 * blot[:, None]) if False else None
    mud = np.array([0.32, 0.27, 0.22])[None, :] * (0.75 + 0.5 * blot[:, None])
    col = base[None, :] * (0.92 + 0.16 * vnoise3(verts, 9.0, 3.0))[:, None]          # faded, uneven old paint
    if kind in ("paint", "trim", "glass", "chrome", "rim", "tyre"):
        amt = dirt * (0.95 if kind != "glass" else 0.7)
        if kind == "chrome": amt = np.clip(amt * 1.2, 0, 1)
        col = col * (1 - amt[:, None]) + mud * amt[:, None]
    if kind == "paint":
        rust_mask = (np.clip((0.62 - y) * 3.0, 0, 1) * 0.6 + arch * 0.45) * (fbm3(verts, 11.0, 4.0) > 0.52) * (vnoise3(verts, 25.0, 6.0) > 0.45)
        rust = np.array([0.45, 0.22, 0.10]) * (0.8 + 0.4 * speck[:, None])
        col = col * (1 - rust_mask[:, None]) + rust * rust_mask[:, None]
        chip = (speck > 0.93) & (np.abs(nrm[:, 1]) < 0.9)                                  # stone chips down to grey primer
        col[chip] = np.array([0.46, 0.46, 0.5])
        scr = (vnoise3(verts * np.array([1.0, 6.0, 0.4]), 22.0, 8.0) > 0.86)               # long horizontal scratches
        col[scr] = col[scr] * 0.6 + 0.4 * np.array([0.65, 0.62, 0.6])
        fade = np.clip((y - 1.25) * 3.0, 0, 1) * (blot > 0.5)                                # sun-bleached roof
        col = col * (1 - 0.18 * fade[:, None]) + 0.18 * fade[:, None] * np.array([0.62, 0.35, 0.35])
    a = np.ones(len(verts)); 
    if kind == "glass": a = np.clip(0.27 + 0.55 * dirt, 0, 0.92)                                # dirty glass is more opaque
    return np.concatenate([np.clip(col, 0, 1), a[:, None]], 1).astype(np.float32)

WIND = {}

def fit_plane(P, N):
    """Least-squares plane through glass vertices -> centre, outward normal (up/front), in-plane axes and extents (metres)."""
    c = P.mean(0); _, _, vt = np.linalg.svd(P - c, full_matrices=False); n = vt[2]
    if n @ np.array([0.0, 0.8, -0.6]) < 0: n = -n
    u = np.array([1.0, 0, 0]) - n * n[0]; u /= np.linalg.norm(u); v = np.cross(u, n)          # u = right, v = up the slope
    pu = (P - c) @ u; pv = (P - c) @ v
    u0, u1 = np.percentile(pu, [1, 99]); v0, v1 = np.percentile(pv, [1, 99])
    c2 = c + u * (u0 + u1) / 2 + v * (v0 + v1) / 2
    return dict(center=[round(float(x), 4) for x in c2], normal=[round(float(x), 4) for x in n], u=[round(float(x), 4) for x in u], v=[round(float(x), 4) for x in v],
                width=round(float(u1 - u0), 3), height=round(float(v1 - v0), 3))

def make_car(paint=(150, 28, 30)):
    (xs, ys, zs), X, Y, Z = grid(-1.12, 1.12, 0.20, 1.62, -2.5, 2.5, G)
    f = F(X, Y, Z)
    lower = f.rbox((0, 0.64, 0.0), (0.875, 0.30, 2.24), 0.20)
    cabin = f.rbox((0, 1.17, 0.3), (0.74, 0.27, 1.05), 0.16)
    fslope = f.rbox((0, 1.1, -0.85), (0.72, 0.26, 0.42), 0.14, rx=0.62)
    rslope = f.rbox((0, 1.1, 1.55), (0.70, 0.26, 0.36), 0.14, rx=-0.55)
    outer = smin(lower, smin(cabin, smin(fslope, rslope, 0.1), 0.1), 0.12)
    for z in (-1.35, 1.35):
        for sx in (-1, 1):
            arch = f.cyl_x(0.34, z, 0.45, 0.58 if sx > 0 else -1.2, 1.2 if sx > 0 else -0.58)
            outer = np.maximum(outer, -arch)
    t = 0.07
    shell = np.maximum(outer, -outer - t)
    # window openings (union of boxes)
    win = np.minimum.reduce([
        f.rbox((0, 1.17, -0.2), (1.0, 0.17, 0.46), 0.03),              # front side windows
        f.rbox((0, 1.17, 0.72), (1.0, 0.16, 0.30), 0.03),               # rear side windows
        f.rbox((0, 1.08, -0.95), (0.68, 0.24, 0.5), 0.03, rx=0.62),     # windshield
        f.rbox((0, 1.08, 1.66), (0.60, 0.24, 0.42), 0.03, rx=-0.55),     # rear window
    ])
    paintf = np.maximum(shell, -win)
    glass = np.maximum(np.maximum(outer, -outer - 0.045), win)        # thin pane at the outer surface of each window opening
    # interior
    floor = f.rbox((0, 0.42, 0.15), (0.80, 0.02, 1.35), 0.01)
    dash = smin(f.rbox((0, 0.98, -0.7), (0.72, 0.09, 0.2), 0.05), f.rbox((-0.38, 1.07, -0.72), (0.3, 0.06, 0.1), 0.04), 0.05)
    column = f.capsule((-0.38, 0.99, -0.52), (-0.38, 0.9, -0.66), 0.035)
    interior = np.minimum.reduce([floor, dash, column, f.capsule((0, 0.5, -0.2), (0, 0.78, -0.24), 0.014), f.rbox((0, 0.6, 0.0), (0.12, 0.15, 0.45), 0.05)])   # + console tunnel
    seats = None
    for sx in (-0.38, 0.38):
        s_ = np.minimum.reduce([f.rbox((sx, 0.62, 0.24), (0.22, 0.08, 0.26), 0.05), f.rbox((sx, 0.93, 0.54), (0.22, 0.3, 0.06), 0.05, rx=-0.18), f.rbox((sx, 1.27, 0.6), (0.1, 0.07, 0.05), 0.03)])
        seats = s_ if seats is None else np.minimum(seats, s_)
    seats = np.minimum.reduce([seats, f.rbox((0, 0.62, 1.0), (0.7, 0.08, 0.22), 0.05), f.rbox((0, 0.95, 1.22), (0.7, 0.28, 0.05), 0.04, rx=0.1)])
    # exterior trims & lamps
    trim = np.minimum.reduce([f.rbox((0, 0.48, -2.2), (0.9, 0.09, 0.09), 0.05), f.rbox((0, 0.48, 2.2), (0.9, 0.09, 0.09), 0.05), f.rbox((0, 0.7, -2.22), (0.44, 0.07, 0.05), 0.02),
                              f.rbox((0.96, 1.08, -0.5), (0.07, 0.055, 0.1), 0.03), f.rbox((-0.96, 1.08, -0.5), (0.07, 0.055, 0.1), 0.03),
                              f.capsule((0.9, 1.02, -0.5), (0.8, 1.04, -0.5), 0.012), f.capsule((-0.9, 1.02, -0.5), (-0.8, 1.04, -0.5), 0.012)])
    chrome = np.minimum.reduce([f.capsule((0.45, 0.4, 2.0), (0.45, 0.4, 2.38), 0.04), f.rbox((0.89, 0.98, 0.08), (0.012, 0.012, 0.07), 0.005), f.rbox((-0.89, 0.98, 0.08), (0.012, 0.012, 0.07), 0.005),
                                f.rbox((0, 0.78, -2.26), (0.1, 0.03, 0.01), 0.004)])
    lamp = np.minimum(f.sphere((0.62, 0.8, -2.17), 0.1), f.sphere((-0.62, 0.8, -2.17), 0.1))
    tail = np.minimum(f.rbox((0.66, 0.82, 2.21), (0.16, 0.06, 0.04), 0.03), f.rbox((-0.66, 0.82, 2.21), (0.16, 0.06, 0.04), 0.03))
    fields = [("paint", paintf, (255, 255, 255)), ("glass", glass, (255, 255, 255, 150)), ("trim", trim, (255, 255, 255)), ("chrome", chrome, (255, 255, 255)), ("lamp", lamp, (255, 244, 200)),
              ("tail", tail, (190, 20, 20)), ("dark", interior, (255, 255, 255)), ("seat", seats, (255, 255, 255))]
    origin = np.array([xs[0], ys[0], zs[0]], np.float32)
    m = Mesh()
    nfaces = 0
    for n, fl, c in fields: m.mat(n, c)
    m.mat("trim", (255, 255, 255))
    for n, fl, c in fields:
        r = mesh_from_field(fl, origin, G)
        if r is None: continue
        verts, faces, nrm = r
        if n == "glass":
            sel = (verts[:, 2] < -0.7) & (verts[:, 1] > 0.95) & (np.abs(verts[:, 0]) < 0.74) & (nrm[:, 2] < -0.25) & (nrm[:, 1] > 0.3)
            if sel.sum() > 50: WIND.update(fit_plane(verts[sel], nrm[sel]))
        cen = verts[faces].mean(1)
        plain = n in ("lamp", "tail")                                     # lamps keep their emissive colour (no grime)
        col_main = None if plain else grime_colors(n, verts, nrm)
        col_trim = grime_colors("trim", verts, nrm) if n == "paint" else None
        for fi in range(len(faces)):
            a, b, cc = faces[fi]
            if n == "paint" and cen[fi, 1] < 0.44: tgt, cols = "trim", col_trim        # sills / underside / arch liners = black plastic
            else: tgt, cols = n, col_main
            if cols is None: m.tri(tgt, verts[a], verts[b], verts[cc], nrm[a], nrm[b], nrm[cc])
            else: m.tri(tgt, verts[a], verts[b], verts[cc], nrm[a], nrm[b], nrm[cc], cols[a], cols[b], cols[cc])
        nfaces += len(faces)
    return m, nfaces


def make_wheel():
    (xs, ys, zs), X, Y, Z = grid(-0.16, 0.16, -0.36, 0.36, -0.36, 0.36, 0.012)
    f = F(X, Y, Z)
    r = np.sqrt(Y ** 2 + Z ** 2); ang = np.arctan2(Z, Y)
    tire = np.maximum(np.sqrt(np.maximum(np.abs(X) - 0.075, 0) ** 2 + np.maximum(r - 0.32, 0) ** 2) - 0.0 + np.minimum(np.maximum(np.abs(X) - 0.075, r - 0.32), 0), -(r - 0.215))
    tread = 0.004 * np.sign(np.sin(ang * 22)) * (np.abs(X) < 0.07)
    tire = tire + tread * (r > 0.3)
    rim = np.maximum(np.maximum(np.abs(X - 0.03) - 0.04, r - 0.222), -(r - 0.19) * (np.abs(ang * 1.0) > 9))      # dish
    rim = np.maximum(np.maximum(np.abs(X - 0.045) - 0.025, r - 0.225), -(f.cyl_x(0, 0, 0.05, -1, 1)))
    holes = np.minimum.reduce([np.sqrt(X * X + (Y - 0.12 * np.cos(a)) ** 2 + (Z - 0.12 * np.sin(a)) ** 2) - 0.032 for a in np.linspace(0, 2 * np.pi, 6, endpoint=False)])
    rim = np.maximum(rim, -holes)
    hub = f.cyl_x(0, 0, 0.045, 0.02, 0.075)
    fields = [("tyre", tire, (255, 255, 255)), ("rim", np.minimum(rim, hub), (255, 255, 255))]
    origin = np.array([xs[0], ys[0], zs[0]], np.float32)
    m = Mesh(); [m.mat(n, c) for n, _, c in fields]; nf = 0
    for n, fl, c in fields:
        r = mesh_from_field(fl, origin, 0.012)
        if r is None: continue
        verts, faces, nrm = r
        cols = grime_colors(n, np.stack([verts[:, 0] * 0 + 0.0, verts[:, 1] * 0 + 0.6 + 0.0, verts[:, 2] * 0], 1) * 0 + verts * np.array([1, 1, 1]) + np.array([0, 0, 0]) if False else verts, nrm)
        for a, b, cc in faces: m.tri(n, verts[a], verts[b], verts[cc], nrm[a], nrm[b], nrm[cc], cols[a], cols[b], cols[cc])
        nf += len(faces)
    return m, nf


def make_steering():
    (xs, ys, zs), X, Y, Z = grid(-0.24, 0.24, -0.24, 0.24, -0.07, 0.07, 0.006)
    f = F(X, Y, Z)
    rim = f.torus((0, 0, 0), 0.19, 0.021, 0.0)
    spokes = np.minimum.reduce([f.capsule((0, 0, 0), (0.19 * np.cos(a), 0.19 * np.sin(a), 0.0), 0.014) for a in (np.pi, 0.0, -np.pi / 2)])
    hub = f.rbox((0, 0, 0.012), (0.05, 0.05, 0.03), 0.025)
    fields = [("leather", np.minimum(rim, np.minimum(spokes, hub)), (255, 255, 255))]
    origin = np.array([xs[0], ys[0], zs[0]], np.float32)
    m = Mesh(); m.mat("leather", (255, 255, 255)); nf = 0
    r = mesh_from_field(fields[0][1], origin, 0.006)
    verts, faces, nrm = r
    # worn leather: darker/greyer where hands grip (sides of the rim), lighter scuffs
    ang = np.arctan2(verts[:, 1], verts[:, 0]); rr = np.hypot(verts[:, 0], verts[:, 1])
    wear = np.clip(np.abs(np.cos(ang)) * (rr > 0.15), 0, 1)
    base = np.array([0.11, 0.10, 0.10]) * (0.9 + 0.2 * vnoise3(verts, 60.0, 2.0))[:, None]
    col = base * (1 - wear[:, None] * 0.0) + wear[:, None] * np.array([0.12, 0.1, 0.09]) * vnoise3(verts, 140.0, 4.0)[:, None] * 0.8
    col[(rr < 0.07)] = np.array([0.08, 0.08, 0.09]); col[(vnoise3(verts, 90.0, 7.0) > 0.9)] = np.array([0.3, 0.28, 0.26])
    cols = np.concatenate([col, np.ones((len(verts), 1))], 1).astype(np.float32)
    for a, b, cc in faces: m.tri("leather", verts[a], verts[b], verts[cc], nrm[a], nrm[b], nrm[cc], cols[a], cols[b], cols[cc])
    return m, len(faces)

def main():
    car, nf = make_car(); car.write(out("models/car_body.glb")); print("car faces", nf)
    import json
    json.dump({"windshield": WIND, "driver_eye": [-0.38, 1.27, 0.0]}, open(out("data/car.json"), "w"), indent=1); print("windshield", WIND)
    reg("data.car", "data/car.json", "data", "Car glass geometry measured from the mesh (windshield plane for the 3D snow/wipers) + driver eye position", source="generated (tools/gen_car.py)", notes="replace together with car.body")
    reg("car.body", "models/car_body.glb", "model", "Player car body (smooth SDF mesh: paint, glass, trim, chrome, lamps, interior with dashboard/wheel/seats)", source="generated (tools/gen_car.py, marching cubes)", notes="origin mid-wheelbase on ground, -Z forward, wheels separate; replace with any glb (keep driver eye ~(-0.38,1.12,-0.15))")
    w, nf2 = make_wheel(); w.write(out("models/car_wheel.glb")); print("wheel faces", nf2)
    sw, nf3 = make_steering(); sw.write(out("models/car_steering.glb")); print("steering faces", nf3)
    reg("car.steering", "models/car_steering.glb", "model", "Steering wheel (disc in the XY plane, hub at origin; game spins it about its Z axis with the steering input)", source="generated (tools/gen_car.py, marching cubes)")
    reg("car.wheel", "models/car_wheel.glb", "model", "Wheel + tyre with tread and 6-hole rim (axis X, radius 0.32 m)", source="generated (tools/gen_car.py, marching cubes)")

if __name__ == "__main__":
    main()
