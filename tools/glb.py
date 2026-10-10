"""Minimal glTF binary writer for procedural placeholder meshes (no deps beyond numpy)."""
import json, struct, math
import numpy as np

class Mesh:
    def __init__(self): self.groups = {}   # material name -> (color, tris list)
    def mat(self, name, color):
        if name not in self.groups: self.groups[name] = [color, [], [], []]
        return name
    def tri(self, m, a, b, c, na=None, nb=None, nc=None, ca=None, cb=None, cc=None):
        g = self.groups[m]; g[1].append((a, b, c)); g[2].append(None if na is None else (na, nb, nc)); g[3].append(None if ca is None else (ca, cb, cc))
    def mesh(self, m, verts, faces, normals):
        """Add an indexed mesh with smooth per-vertex normals to material m."""
        for f in faces: self.tri(m, verts[f[0]], verts[f[1]], verts[f[2]], normals[f[0]], normals[f[1]], normals[f[2]])
    def quad(self, m, a, b, c, d): self.tri(m, a, b, c); self.tri(m, a, c, d)
    def box(self, m, cx, cy, cz, sx, sy, sz):
        x0, x1, y0, y1, z0, z1 = cx - sx / 2, cx + sx / 2, cy - sy / 2, cy + sy / 2, cz - sz / 2, cz + sz / 2
        v = lambda x, y, z: (x, y, z)
        for q in [((x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)), ((x1, y0, z0), (x0, y0, z0), (x0, y1, z0), (x1, y1, z0)),
                  ((x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0)), ((x1, y0, z1), (x1, y0, z0), (x1, y1, z0), (x1, y1, z1)),
                  ((x0, y1, z1), (x1, y1, z1), (x1, y1, z0), (x0, y1, z0)), ((x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1))]:
            self.quad(m, *q)
    def cyl(self, m, cx, cy, cz, r, length, axis="x", seg=16):
        def P(a, u):  # u along axis
            ca, sa = math.cos(a) * r, math.sin(a) * r
            return {"x": (cx + u, cy + ca, cz + sa), "y": (cx + ca, cy + u, cz + sa), "z": (cx + ca, cy + sa, cz + u)}[axis]
        h = length / 2
        for i in range(seg):
            a0, a1 = 2 * math.pi * i / seg, 2 * math.pi * (i + 1) / seg
            self.quad(m, P(a0, -h), P(a1, -h), P(a1, h), P(a0, h))
            c0, c1 = P(0, -h), P(0, h); c0 = {"x": (cx - h, cy, cz), "y": (cx, cy - h, cz), "z": (cx, cy, cz - h)}[axis]; c1 = {"x": (cx + h, cy, cz), "y": (cx, cy + h, cz), "z": (cx, cy, cz + h)}[axis]
            self.tri(m, c0, P(a1, -h), P(a0, -h)); self.tri(m, c1, P(a0, h), P(a1, h))
    def prism_roof(self, m, cx, y, cz, sx, sz, h):
        x0, x1, z0, z1 = cx - sx / 2, cx + sx / 2, cz - sz / 2, cz + sz / 2
        r0, r1 = (cx, y + h, z0), (cx, y + h, z1)
        self.quad(m, (x0, y, z1), (x1, y, z1), r1, r1) ; self.quad(m, (x0, y, z0), (x0, y, z1), r1, r0)
        self.quad(m, (x1, y, z1), (x1, y, z0), r0, r1); self.tri(m, (x1, y, z0), (x0, y, z0), r0); self.tri(m, (x0, y, z1), (x1, y, z1), r1)

    def write(self, path):
        bin_ = bytearray(); views, accs, prims, mats = [], [], [], []
        for name, grp in self.groups.items():
            color, tris = grp[0], grp[1]
            norms = grp[2] if len(grp) > 2 else [None] * len(tris)
            cols = grp[3] if len(grp) > 3 else [None] * len(tris)
            has_col = len(cols) == len(tris) and all(x is not None for x in cols) and len(cols) > 0
            if not tris: continue
            pos = np.array(tris, np.float32).reshape(-1, 3)
            tr = pos.reshape(-1, 3, 3); n = np.cross(tr[:, 1] - tr[:, 0], tr[:, 2] - tr[:, 0]); n /= (np.linalg.norm(n, axis=1, keepdims=True) + 1e-9)
            nor = np.repeat(n, 3, axis=0).astype(np.float32)
            if all(x is not None for x in norms) and len(norms) == len(tris): nor = np.array(norms, np.float32).reshape(-1, 3)
            idx = np.arange(len(pos), dtype=np.uint32)
            if all(x is not None for x in norms) and len(norms) == len(tris):
                # smooth mesh: weld identical (position, normal) vertices -> indexed geometry (much smaller files)
                vc = np.array(cols, np.float32).reshape(-1, 4) if has_col else np.zeros((len(pos), 0), np.float32)
                key = np.round(np.concatenate([pos, nor, vc], 1) * 2000).astype(np.int64)
                _, first, inv = np.unique(key, axis=0, return_index=True, return_inverse=True)
                pos, nor, idx = pos[first], nor[first], inv.reshape(-1).astype(np.uint32)
                if has_col: vc = vc[first]
            else: has_col = False
            def add(arr, target, typ, ctype, cnt, mn=None, mx=None):
                while len(bin_) % 4: bin_.append(0)
                off = len(bin_); bin_.extend(arr.tobytes())
                views.append({"buffer": 0, "byteOffset": off, "byteLength": arr.nbytes, "target": target})
                a = {"bufferView": len(views) - 1, "componentType": ctype, "count": cnt, "type": typ}
                if mn is not None: a["min"], a["max"] = mn, mx
                accs.append(a); return len(accs) - 1
            ap = add(pos, 34962, "VEC3", 5126, len(pos), pos.min(0).tolist(), pos.max(0).tolist())
            an = add(nor, 34962, "VEC3", 5126, len(nor)); ai = add(idx, 34963, "SCALAR", 5125, len(idx))
            attrs = {"POSITION": ap, "NORMAL": an}
            if has_col:
                cb = np.clip(np.round(vc * 255), 0, 255).astype(np.uint8)
                ci = add(cb, 34962, "VEC4", 5121, len(cb)); accs[ci]["normalized"] = True; attrs["COLOR_0"] = ci
            alpha = color[3] / 255 if len(color) > 3 else 1.0
            md = {"name": name, "pbrMetallicRoughness": {"baseColorFactor": [c / 255 for c in color[:3]] + [alpha], "metallicFactor": 0.1, "roughnessFactor": 0.8}}
            if alpha < 1.0: md["alphaMode"] = "BLEND"
            if name in ("lamp", "tail") : md["emissiveFactor"] = [c / 255 * 0.8 for c in color[:3]]
            mats.append(md)
            prims.append({"attributes": attrs, "indices": ai, "material": len(mats) - 1})
        g = {"asset": {"version": "2.0", "generator": "CarOMine tools/glb.py"}, "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
             "meshes": [{"primitives": prims}], "materials": mats, "buffers": [{"byteLength": len(bin_)}], "bufferViews": views, "accessors": accs}
        js = json.dumps(g).encode(); js += b" " * (-len(js) % 4)
        while len(bin_) % 4: bin_.append(0)
        data = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(bin_)) + struct.pack("<II", len(js), 0x4E4F534A) + js + struct.pack("<II", len(bin_), 0x004E4942) + bytes(bin_)
        open(path, "wb").write(data)
