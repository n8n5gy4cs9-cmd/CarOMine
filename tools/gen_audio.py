#!/usr/bin/env python3
"""T-027/T-028: placeholder SFX + music, synthesized (CC0). 22.05kHz mono 16-bit WAV."""
import wave, numpy as np
from assetlib import *
SR = 22050
rng = np.random.default_rng(7)

def t(sec): return np.arange(int(SR * sec)) / SR
def env(n, a=0.005, d=0.1, s=0.0, r=0.1):
    e = np.ones(n); na, nr = int(a * SR), int(r * SR)
    e[:na] = np.linspace(0, 1, max(na, 1)); e[-nr:] = np.minimum(e[-nr:], np.linspace(1, 0, max(nr, 1)))
    return e
def decay(sec, k=8): return np.exp(-k * t(sec) / sec)
def noise(sec): return rng.uniform(-1, 1, int(SR * sec))
def lp(x, a=0.1):
    y = np.zeros_like(x); acc = 0
    for i, v in enumerate(x): acc += a * (v - acc); y[i] = acc
    return y
def lpf(x, a):  # vectorised-ish one pole via cumulative filter (fast enough for short sounds)
    from numpy.lib.stride_tricks import sliding_window_view
    k = max(1, int(1 / a)); return np.convolve(x, np.ones(k) / k, mode="same")
def sq(f, sec, duty=.5): return np.where((t(sec) * f) % 1 < duty, 1.0, -1.0)
def tri(f, sec): return 2 * np.abs(2 * ((t(sec) * f) % 1) - 1) - 1
def saw(f, sec): return 2 * ((t(sec) * f) % 1) - 1
def sine(f, sec): return np.sin(2 * np.pi * f * t(sec))

def write(x, rel, id, use, loop=False):
    x = np.asarray(x, float); x = x / (np.max(np.abs(x)) + 1e-9) * 0.8
    p = out(rel)
    with wave.open(str(p), "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR); w.writeframes((x * 32767).astype(np.int16).tobytes())
    reg(id, rel, "music" if rel.startswith("music") else "sfx", use, notes=f"{len(x)/SR:.2f}s {'LOOP' if loop else 'one-shot'} wav mono {SR}Hz")

def engine(rpm_hz, sec=2.0):
    f = rpm_hz; x = sum(saw(f * k, sec) / k for k in (1, 2, 3)) * (1 + 0.3 * sine(f / 2, sec)) + 0.3 * lpf(noise(sec), 0.2)
    return lpf(x, 0.3)

def loopfade(x, n=1500):
    x = x.copy(); x[:n] = x[:n] * np.linspace(0, 1, n) + x[-n:] * np.linspace(1, 0, n) * 0; return x

def sfx():
    write(engine(38), "sfx/engine_idle.wav", "sfx.engine_idle", "Car engine idle loop (pitch-shifted by RPM in code)", True)
    write(engine(90), "sfx/engine_high.wav", "sfx.engine_high", "Car engine high-rev loop", True)
    st = np.concatenate([engine(15 + 25 * i / 6, 0.15) for i in range(6)] + [engine(40, 0.8) * np.linspace(1, 0.6, int(SR * .8))])
    write(st, "sfx/engine_start.wav", "sfx.engine_start", "Engine cranking and catching")
    write(np.concatenate([engine(35, .5) * np.linspace(1, 0, int(SR * .5))]), "sfx/engine_stall.wav", "sfx.engine_stall", "Engine stalls/dies")
    c = np.concatenate([(sq(9, .12) * decay(.12, 5)), np.zeros(int(SR * .1)), (sq(9, .12) * decay(.12, 5))])
    write(c, "sfx/starter_click.wav", "sfx.starter_click", "Dead-battery clicking")
    write(lpf(noise(2), 0.25) * (0.5 + 0.5 * sine(0.5, 2)), "sfx/tire_snow.wav", "sfx.tire_snow", "Tyres on snow loop (volume by speed)", True)
    write(lpf(noise(1.5), 0.03) * (0.6 + 0.4 * sine(0.3, 1.5)), "sfx/wind.wav", "sfx.wind", "Ambient winter wind loop", True)
    sw = np.concatenate([lpf(noise(.35), .1) * np.sin(np.linspace(0, np.pi, int(SR * .35))), np.zeros(int(SR * .1)), lpf(noise(.35), .1) * np.sin(np.linspace(0, np.pi, int(SR * .35)))])
    write(sw, "sfx/wiper.wav", "sfx.wiper", "Windshield wiper swipe")
    write(lpf(noise(1.2), .15) * decay(1.2, 4) + 0.7 * sine(60, 1.2) * decay(1.2, 8) + 0.3 * lpf(noise(1.2), 0.5) * (rng.random(int(SR * 1.2)) > .9), "sfx/crash.wav", "sfx.crash", "Car crash")
    write(sine(440, .35) * np.where(t(.35) < .3, 1, 0) + sine(554, .35) * .6, "sfx/horn.wav", "sfx.horn", "Car horn")
    write(lpf(noise(.15), .5) * decay(.15, 10) + sine(120, .15) * decay(.15, 15) * .5, "sfx/door.wav", "sfx.door", "Door/trunk thud")
    write(lpf(noise(.25), .4) * decay(.25, 12) + sine(180, .25) * decay(.25, 20), "sfx/pistol.wav", "sfx.pistol", "Pistol shot")
    write(lpf(noise(.6), .15) * decay(.6, 7) + sine(70, .6) * decay(.6, 10), "sfx/shotgun.wav", "sfx.shotgun", "Shotgun blast")
    write(lpf(noise(.12), .5) * decay(.12, 14) + sine(220, .12) * decay(.12, 25), "sfx/smg.wav", "sfx.smg", "SMG single shot (repeat for burst)")
    write(lpf(noise(.7), .25) * decay(.7, 6) + sine(110, .7) * decay(.7, 12) * .8, "sfx/rifle.wav", "sfx.rifle", "Hunting rifle shot")
    rl = np.concatenate([sq(900, .03) * .8, np.zeros(int(SR * .15)), sq(500, .05), np.zeros(int(SR * .1)), sq(1300, .03)])
    write(rl, "sfx/reload.wav", "sfx.reload", "Reload clicks")
    write(lpf(noise(.12), .3) * decay(.12, 10), "sfx/step_snow.wav", "sfx.step_snow", "Footstep in snow")
    write(lpf(noise(.2), .3) * decay(.2, 9) + sine(90, .2) * decay(.2, 12), "sfx/hit_flesh.wav", "sfx.hit", "Bullet hits enemy")
    write(saw(200, .6) * np.linspace(1, .1, int(SR * .6)) * (1 + .2 * sine(30, .6)) * decay(.6, 3), "sfx/enemy_die.wav", "sfx.enemy_die", "Human enemy death grunt")
    write(lpf(saw(90, .8) * (1 + .6 * sine(35, .8)) + .4 * noise(.8), .2) * env(int(SR * .8), .1, 0, 0, .2), "sfx/wolf_growl.wav", "sfx.wolf_growl", "Wolf growl")
    write(lpf(saw(60, 1.2) * (1 + .3 * sine(4, 1.2)), .3) * env(int(SR * 1.2), .2, 0, 0, .4), "sfx/moose_call.wav", "sfx.moose_call", "Moose bellow")
    write(np.concatenate([sine(660, .06), sine(990, .1)]) * np.concatenate([decay(.06, 4), decay(.1, 6)]), "sfx/pickup.wav", "sfx.pickup", "Item pickup chime")
    write(sine(800, .05) * decay(.05, 8), "sfx/ui_click.wav", "sfx.ui_click", "UI click")
    write(np.concatenate([sine(523, .08), sine(659, .08), sine(784, .15)]), "sfx/ui_confirm.wav", "sfx.ui_confirm", "UI confirm")
    write(lpf(noise(1.5), .08) * env(int(SR * 1.5), .3, 0, 0, .6), "sfx/sauna_hiss.wav", "sfx.sauna_hiss", "Sauna water on stones")
    write(lpf(noise(1.2), .05) * (0.5 + 0.5 * sine(7, 1.2)) * env(int(SR * 1.2), .1, 0, 0, .2), "sfx/coffee_pour.wav", "sfx.coffee_pour", "Pour coffee")
    write(sine(80, .8) * decay(.8, 5) + lpf(noise(.8), .3) * decay(.8, 9), "sfx/wrench.wav", "sfx.wrench", "Wrench tighten (ratchet clicks)")
    write(np.concatenate([sq(1200 + 100 * (i % 3), .02) * 0.8 if i % 2 == 0 else np.zeros(int(SR * .02)) for i in range(14)]), "sfx/ratchet.wav", "sfx.ratchet", "Ratchet clicks")

# ---- music: tiny sequencer. notes as semitones above A (minor), 0 = rest = None
def freq(n): return 220 * 2 ** (n / 12)
def track(bpm, bars, melody, bass, lead="sq", seed=0, drums=True):
    beat = 60 / bpm; step = beat / 2; n_steps = bars * 8
    out_ = np.zeros(int(SR * step * n_steps) + SR)
    for i in range(n_steps):
        s0 = int(i * step * SR)
        m = melody[i % len(melody)]; b = bass[(i // 2) % len(bass)]
        if m is not None:
            n = int(step * SR * .95); tt = np.arange(n) / SR
            w = (np.where((tt * freq(m) * 2) % 1 < .5, 1, -1) if lead == "sq" else np.sin(2 * np.pi * freq(m) * 2 * tt)) * 0.25 * np.exp(-3 * tt / step)
            out_[s0:s0 + n] += w
        if i % 2 == 0 and b is not None:
            n = int(step * 2 * SR * .9); tt = np.arange(n) / SR
            out_[s0:s0 + n] += (2 * np.abs(2 * ((tt * freq(b - 12)) % 1) - 1) - 1) * 0.35 * np.exp(-1.5 * tt / (step * 2))
        if drums and i % 4 == 0:
            n = int(.12 * SR); tt = np.arange(n) / SR; out_[s0:s0 + n] += np.sin(2 * np.pi * (120 - 400 * tt) * tt) * .5 * np.exp(-25 * tt)
        if drums and i % 4 == 2:
            n = int(.08 * SR); out_[s0:s0 + n] += rng.uniform(-1, 1, n) * .2 * np.exp(-30 * np.arange(n) / SR)
    return out_[:int(SR * step * n_steps)]

def music():
    # A minor: A=0 B=2 C=3 D=5 E=7 F=8 G=10
    write(track(70, 16, [7, None, 3, None, 5, 3, 2, None, 0, None, 3, None, 7, 5, 3, None], [0, 8, 3, 10], "sine", drums=False), "music/menu.wav", "music.menu", "Main menu theme (melancholic Finnish minor, slow)", True)
    write(track(96, 16, [0, 3, 7, 3, 5, 7, 8, 7, 5, 3, 0, 3, 2, 3, 5, None], [0, 0, 8, 10], "sq"), "music/drive.wav", "music.drive", "Driving music (mid-tempo)", True)
    write(track(60, 16, [None, 7, None, None, 3, None, None, 2, None, 0, None, None, 5, None, None, None], [0, 5, 3, 7], "sine", drums=False), "music/night.wav", "music.night", "Night / quiet exploration ambience", True)
    write(track(150, 16, [0, 0, 3, 0, 7, 0, 8, 7, 0, 0, 3, 0, 10, 8, 7, 3], [0, 0, 10, 8], "sq"), "music/combat.wav", "music.combat", "Combat music (fast)", True)

if __name__ == "__main__":
    sfx(); music(); print("audio ok")
