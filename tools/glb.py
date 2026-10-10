"""Minimal glTF binary writer for procedural placeholder meshes (no deps beyond numpy)."""
import json, struct, math
import numpy as np

class Mesh:
    def __init__(self): self.groups = {}   # material name -> (color, tris list)
    def mat(self, name, color): self.groups.setdefault(name, [color, []]); return name
    def tri(self, m, a, b, c): self.groups[m][1].append((a, b, c))
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
        for name, (color, tris) in self.groups.items():
            if not tris: continue
            pos = np.array(tris, np.float32).reshape(-1, 3)
            tr = pos.reshape(-1, 3, 3); n = np.cross(tr[:, 1] - tr[:, 0], tr[:, 2] - tr[:, 0]); n /= (np.linalg.norm(n, axis=1, keepdims=True) + 1e-9)
            nor = np.repeat(n, 3, axis=0).astype(np.float32)
            idx = np.arange(len(pos), dtype=np.uint32)
            def add(arr, target, typ, ctype, cnt, mn=None, mx=None):
                while len(bin_) % 4: bin_.append(0)
                off = len(bin_); bin_.extend(arr.tobytes())
                views.append({"buffer": 0, "byteOffset": off, "byteLength": arr.nbytes, "target": target})
                a = {"bufferView": len(views) - 1, "componentType": ctype, "count": cnt, "type": typ}
                if mn is not None: a["min"], a["max"] = mn, mx
                accs.append(a); return len(accs) - 1
            ap = add(pos, 34962, "VEC3", 5126, len(pos), pos.min(0).tolist(), pos.max(0).tolist())
            an = add(nor, 34962, "VEC3", 5126, len(nor)); ai = add(idx, 34963, "SCALAR", 5125, len(idx))
            mats.append({"name": name, "pbrMetallicRoughness": {"baseColorFactor": [c / 255 for c in color[:3]] + [color[3] / 255 if len(color) > 3 else 1.0], "metallicFactor": 0.1, "roughnessFactor": 0.8}})
            prims.append({"attributes": {"POSITION": ap, "NORMAL": an}, "indices": ai, "material": len(mats) - 1})
        g = {"asset": {"version": "2.0", "generator": "CarOMine tools/glb.py"}, "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
             "meshes": [{"primitives": prims}], "materials": mats, "buffers": [{"byteLength": len(bin_)}], "bufferViews": views, "accessors": accs}
        js = json.dumps(g).encode(); js += b" " * (-len(js) % 4)
        while len(bin_) % 4: bin_.append(0)
        data = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(bin_)) + struct.pack("<II", len(js), 0x4E4F534A) + js + struct.pack("<II", len(bin_), 0x004E4942) + bytes(bin_)
        open(path, "wb").write(data)
