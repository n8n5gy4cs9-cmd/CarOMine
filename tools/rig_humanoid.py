"""Articulated humanoid for sdfrender. Character space: +Y up, faces +Z (toward the camera at view angle 0), char's LEFT = +X."""
import numpy as np
from sdfrender import *

def V(*a): return np.array(a, float)

def ik2(shoulder, target, l1, l2, pole):
    """Two-bone IK -> elbow/knee position. pole = direction the joint bends toward."""
    d = target - shoulder; L = np.linalg.norm(d); L = np.clip(L, 1e-4, l1 + l2 - 1e-4); dn = d / max(np.linalg.norm(d), 1e-9)
    a = (l1 * l1 - l2 * l2 + L * L) / (2 * L); h = np.sqrt(max(l1 * l1 - a * a, 0))
    p = pole - dn * (pole @ dn); p = p / max(np.linalg.norm(p), 1e-9)
    return shoulder + dn * a + p * h

class Spec:
    """Everything that makes one human look different from another."""
    def __init__(self, **k):
        self.scale = 1.0; self.belly = 0.0; self.shoulder = 0.21; self.hip = 0.10
        self.skin = Mat((226, 176, 146), noise="cloth", nscale=55, namp=0.035)
        self.hair = Mat((30, 26, 24), noise="streak", nscale=80, namp=0.12)
        self.coat = Mat((36, 38, 54), noise="cloth", nscale=100, namp=0.05)
        self.pants = Mat((30, 32, 46), noise="cloth", nscale=100, namp=0.05)
        self.shirt = Mat((225, 225, 230), noise="cloth", nscale=100, namp=0.04)
        self.boots = Mat((22, 20, 22), spec=0.5)
        self.accent = Mat((170, 20, 30)); self.glove = self.skin
        self.gun = Mat((38, 40, 46), spec=0.7); self.wood = Mat((110, 70, 36), noise="streak", nscale=45, namp=0.1, spec=0.15)
        self.metal = Mat((150, 156, 168), spec=0.9)
        self.__dict__.update(k)

def torso(s, sp, hipc, chest_lean=0.0, belly=0.0):
    """pelvis + abdomen + chest + shoulders + neck; returns (shoulderL, shoulderR, neck, headc)."""
    sc = sp.scale
    R = rot_x(chest_lean)
    pelvis = hipc + V(0, 0.0, 0)
    s.ellipsoid(pelvis, V(0.175, 0.13, 0.115) * [1, 1, 1], sp.pants if sp.coat is not None else sp.pants)
    chest_c = pelvis + R @ V(0, 0.36, 0.0)
    s.ellipsoid(chest_c, V(0.205 + sp.belly * 0.02, 0.27, 0.125 + sp.belly), sp.coat, rot=R)
    if sp.belly > 0: s.ellipsoid(pelvis + R @ V(0, 0.2, 0.06 + sp.belly * 0.5), V(0.17, 0.17, 0.12 + sp.belly), sp.coat, rot=R)
    # coat/jacket skirt covering the pelvis
    s.ellipsoid(pelvis + V(0, 0.02, 0), V(0.19, 0.15, 0.125), sp.coat)
    neck_b = chest_c + R @ V(0, 0.27, 0.0)
    s.capsule(neck_b, neck_b + R @ V(0, 0.09, 0.015), 0.05, 0.045, sp.skin)
    shl = chest_c + R @ V(sp.shoulder, 0.2, 0); shr = chest_c + R @ V(-sp.shoulder, 0.2, 0)
    s.sphere(shl, 0.075, sp.coat); s.sphere(shr, 0.075, sp.coat)           # shoulder caps
    return shl, shr, neck_b + R @ V(0, 0.1, 0.02), chest_c, R

def head(s, sp, c, R_head, face=True, extra=None):
    c = np.array(c)
    s.ellipsoid(c, V(0.098, 0.122, 0.108), sp.skin, rot=R_head)
    s.ellipsoid(c + R_head @ V(0, -0.075, 0.02), V(0.085, 0.07, 0.085), sp.skin, rot=R_head)         # jaw
    if face:
        eye = Mat((245, 245, 245)); pup = Mat((20, 18, 18), spec=0.6)
        for sx in (-1, 1):
            s.sphere(c + R_head @ V(sx * 0.04, 0.02, 0.092), 0.018, eye); s.sphere(c + R_head @ V(sx * 0.04, 0.02, 0.105), 0.0105, pup)
            s.sphere(c + R_head @ V(sx * 0.098, -0.01, -0.01), 0.026, sp.skin)                           # ears
            s.box(c + R_head @ V(sx * 0.042, 0.052, 0.098), V(0.026, 0.006, 0.008), sp.hair, rot=R_head, rnd=0.004)  # brows
        s.ellipsoid(c + R_head @ V(0, -0.015, 0.108), V(0.018, 0.03, 0.026), sp.skin, rot=R_head)       # nose
        s.box(c + R_head @ V(0, -0.07, 0.098), V(0.026, 0.005, 0.006), Mat((120, 50, 50)), rot=R_head, rnd=0.003)   # mouth

def leg(s, sp, hip, ankle, sign, foot_dir=V(0, 0, 1)):
    l1, l2 = 0.44 * sp.scale, 0.44 * sp.scale
    knee = ik2(hip, ankle, l1, l2, V(0, 0.0, 1.0))
    s.capsule(hip, knee, 0.088, 0.066, sp.pants); s.capsule(knee, ankle, 0.066, 0.052, sp.pants)
    toe = ankle + foot_dir * 0.17 + V(0, -0.045, 0)
    s.capsule(ankle + V(0, 0.0, -0.02), toe, 0.052, 0.046, sp.boots)
    s.box((ankle + toe) / 2 + V(0, -0.03, 0), V(0.05, 0.026, 0.105), sp.boots, rnd=0.015)

def arm(s, sp, shoulder, wrist, pole, sleeve=None, hand=None):
    l1, l2 = 0.29 * sp.scale, 0.27 * sp.scale
    elbow = ik2(shoulder, wrist, l1, l2, pole)
    sl = sleeve or sp.coat
    s.capsule(shoulder, elbow, 0.062, 0.05, sl); s.capsule(elbow, wrist, 0.05, 0.04, sl)
    s.sphere(wrist, 0.047, hand or sp.glove)
    return elbow

def walk_pose(sp, phase, amp=0.5, bob=0.02, sway=0.0, arms="swing", lean=0.0):
    """Return dict of joint targets for a walk phase."""
    out = {}
    for side, sx in (("L", 1), ("R", -1)):
        ph = phase + (0 if side == "L" else np.pi)
        a = amp * np.sin(ph)                       # forward/back swing of the leg (positive = forward)
        lift = max(0.0, np.cos(ph)) * 0.10 * (amp / 0.5)    # foot lift on the recovery stroke
        ankle = V(sx * sp.hip + sway, 0.075 + lift, 0.0) + V(0, 0, 0.9 * sp.scale * np.sin(a) * 0.5)
        ankle[1] += (1 - np.cos(a)) * 0.0
        out["ankle" + side] = ankle
        out["armswing" + side] = -np.sin(ph) * 0.35
    out["hipy"] = 0.93 * sp.scale - bob * abs(np.sin(phase)) - 0.02 * (amp / 0.5)
    return out

def build(sp, pose):
    """pose keys: phase, amp, bob, sway, lean, arms ('swing'|'aim_pistol'|'rifle'|'bottle'|'fists'|'down'), fire(bool), pain, death(0..1), head_turn"""
    s = Scene()
    ph = pose.get("phase", 0.0); amp = pose.get("amp", 0.0)
    wp = walk_pose(sp, ph, amp, pose.get("bob", 0.0), pose.get("sway", 0.0))
    hipc = V(pose.get("sway", 0.0), wp["hipy"], 0.0)
    lean = pose.get("lean", 0.0) + pose.get("pain", 0.0) * -0.25
    shl, shr, neck, chest, R = torso(s, sp, hipc, lean, sp.belly)
    hr = rot_y(pose.get("head_turn", 0.0)) @ rot_x(pose.get("head_tilt", 0.0) + pose.get("pain", 0.0) * 0.3)
    hc = neck + R @ V(0, 0.06, 0.0)
    head(s, sp, hc, hr)
    if hasattr(sp, "head_extra") and sp.head_extra: sp.head_extra(s, sp, hc, hr)
    if hasattr(sp, "torso_extra") and sp.torso_extra: sp.torso_extra(s, sp, chest, R, hipc)
    for side, sx in (("L", 1), ("R", -1)):
        hip = hipc + V(sx * sp.hip, -0.02, 0)
        leg(s, sp, hip, wp["ankle" + side], sx)
    # arms
    mode = pose.get("arms", "swing")
    fire = pose.get("fire", False)
    sleeve = sp.coat
    def swing_target(sh, side, pain=0.0):
        sw = wp["armswing" + side] * (1.0 if amp > 0 else 0.0)
        return sh + V(0.04 * (1 if side == "L" else -1) + pain * 0.12 * (1 if side == "L" else -1), -0.50 * np.cos(sw * 0.7), 0.50 * np.sin(sw) + 0.05)
    pole_out_L, pole_out_R = V(0.8, -0.3, -0.5), V(-0.8, -0.3, -0.5)
    gun_tip = None
    if mode in ("aim_pistol", "rifle"):
        if mode == "aim_pistol":
            # right arm out toward the camera, left arm tucked / supporting
            wr = shr + V(0.06, -0.12 + (0.07 if fire else 0.0), 0.50); arm(s, sp, shr, wr, V(-0.4, -0.7, -0.1))
            wl = shl + V(-0.20, -0.18, 0.40); arm(s, sp, shl, wl, V(0.5, -0.7, -0.2))
            g = sp.gun
            s.box(wr + V(0, 0.015, 0.06), V(0.026, 0.05, 0.11), g, rnd=0.012)
            s.box(wr + V(0, -0.045, -0.02), V(0.022, 0.06, 0.03), g, rnd=0.01)
            s.capsule(wr + V(0, 0.03, 0.12), wr + V(0, 0.03, 0.30), 0.02, 0.02, sp.metal)       # suppressor / barrel
            gun_tip = wr + V(0, 0.03, 0.31)
        else:
            # rifle held in both hands across the body; muzzle toward the camera-left when aiming
            wr = shr + V(0.08, -0.22 + (0.05 if fire else 0.0), 0.34); arm(s, sp, shr, wr, V(-0.5, -0.8, -0.2))
            wl = shl + V(-0.28, -0.24, 0.62); arm(s, sp, shl, wl, V(0.6, -0.7, -0.2))
            butt = wr + V(0.02, 0.0, -0.20); tip = wr + V(-0.04, 0.04, 0.95)
            s.capsule(butt, wr + V(0, 0.0, 0.1), 0.045, 0.036, sp.wood)
            s.capsule(wr + V(0, 0.01, 0.05), tip, 0.022, 0.016, sp.gun)
            s.box(wr + V(0, 0.05, 0.28), V(0.016, 0.022, 0.07), sp.gun, rnd=0.008)          # scope
            s.capsule(wr + V(0, 0.0, 0.2), wl + V(0, -0.02, 0.0), 0.032, 0.03, sp.wood)
            gun_tip = tip
    elif mode == "bottle":
        wr = shr + V(0.06, -0.3, 0.38); arm(s, sp, shr, wr, V(-0.6, -0.5, -0.4))
        bottle = Mat((40, 120, 60), spec=0.9); label = Mat((220, 200, 150))
        s.capsule(wr + V(0, -0.05, 0), wr + V(0, 0.13, 0), 0.036, 0.034, bottle); s.capsule(wr + V(0, 0.12, 0), wr + V(0, 0.22, 0), 0.034, 0.014, bottle)
        s.box(wr + V(0, 0.04, 0.034), V(0.026, 0.035, 0.004), label, rnd=0.002)
        wl = swing_target(shl, "L"); arm(s, sp, shl, wl, pole_out_L)
    elif mode == "fists":
        t = pose.get("punch", 0.0)
        wr = shr + V(0.08, -0.12, 0.28 + 0.34 * t); arm(s, sp, shr, wr, V(-0.4, -0.6, -0.1))
        wl = shl + V(-0.12, -0.14, 0.22); arm(s, sp, shl, wl, V(0.4, -0.6, -0.2))
    else:
        pain = pose.get("pain", 0.0)
        wl = swing_target(shl, "L", pain); arm(s, sp, shl, wl, pole_out_L)
        wr = swing_target(shr, "R", pain); arm(s, sp, shr, wr, pole_out_R)
        if getattr(sp, "carry_right", None): sp.carry_right(s, sp, wr)
    if fire and gun_tip is not None:
        fl = Mat((255, 230, 120), emissive=True)
        s.sphere(gun_tip + V(0, 0, 0.07), 0.07, fl); s.sphere(gun_tip + V(0, 0, 0.14), 0.04, Mat((255, 255, 220), emissive=True))
    # death / fall: rotate whole body about the feet axis
    d = pose.get("death", 0.0)
    if d > 0:
        ang = d * (np.pi / 2 - 0.12)
        s = s.transform(rot_x(-ang))                       # fall backwards (head toward -z)
        # sprawl: lower everything so it lies on the ground (y>=0): lift by body thickness
        s = s.transform(None, (0, 0.17 * d + 0.02, 0.85 * d))      # lift onto the ground and centre the long lying body on the frame
        s = s.scaled(1.0 - 0.2 * d)                          # lying bodies are long: shrink so they fit the frame
    return s
