#!/usr/bin/env python3
"""T-029: placeholder 3D models (.glb). Units: metres, -Z forward, +Y up. Car origin = centre of wheelbase on ground."""
from assetlib import *
from glb import Mesh

def car_body(name="wagon", color=(150, 30, 30)):
    m = Mesh(); body = m.mat("paint", color); glass = m.mat("glass", (90, 120, 150, 255)); dark = m.mat("trim", (20, 20, 24))
    m.box(body, 0, 0.62, 0, 1.75, 0.55, 4.5)              # lower body
    m.box(glass, 0, 1.12, 0.2, 1.6, 0.5, 2.4)             # cabin glass
    m.box(body, 0, 1.40, 0.2, 1.62, 0.06, 2.3)             # roof
    m.box(dark, 0, 0.35, 0, 1.7, 0.15, 4.4)               # underside
    m.box(m.mat("lamp", (255, 240, 180)), -0.6, 0.72, -2.26, 0.3, 0.15, 0.05); m.box("lamp", 0.6, 0.72, -2.26, 0.3, 0.15, 0.05)
    m.box(m.mat("tail", (200, 20, 20)), -0.65, 0.72, 2.26, 0.3, 0.15, 0.05); m.box("tail", 0.65, 0.72, 2.26, 0.3, 0.15, 0.05)
    return m

def wheel(winter=True):
    m = Mesh(); t = m.mat("tyre", (20, 20, 22)); r = m.mat("rim", (150, 155, 165))
    m.cyl(t, 0, 0, 0, 0.32, 0.2, "x", 20); m.cyl(r, 0, 0, 0, 0.2, 0.22, "x", 12); return m

def part(name, dims, color):
    m = Mesh(); k = m.mat(name, color); m.box(k, 0, 0, 0, *dims); return m

def house():
    m = Mesh(); w = m.mat("wall", (140, 40, 32)); r = m.mat("roof", (50, 50, 58)); s = m.mat("snow", (240, 245, 255)); wn = m.mat("window", (255, 220, 130))
    m.box(w, 0, 1.4, 0, 6, 2.8, 5); m.prism_roof(r, 0, 2.8, 0, 6.6, 5.6, 1.6); m.prism_roof(s, 0, 2.95, 0, 6.6, 5.6, 1.5)
    m.box(wn, -1.8, 1.5, -2.52, 0.9, 0.9, 0.05); m.box(wn, 1.8, 1.5, -2.52, 0.9, 0.9, 0.05); m.box(m.mat("door", (70, 50, 32)), 0, 0.95, -2.52, 1.0, 1.9, 0.05)
    return m

def main():
    G = "models"
    items = [
        ("car.body", "car_body.glb", car_body(), "Player car body (wagon), paint/glass/trim materials; replace with any glb, origin at wheelbase centre"),
        ("car.wheel", "car_wheel.glb", wheel(), "Wheel+tyre, axis X, radius 0.32m"),
        ("part.engine", "part_engine.glb", part("engine", (0.6, 0.55, 0.7), (80, 85, 95)), "Engine block placeholder"),
        ("part.gearbox", "part_gearbox.glb", part("gearbox", (0.35, 0.35, 0.6), (95, 100, 110)), "Gearbox placeholder"),
        ("part.battery", "part_battery.glb", part("battery", (0.3, 0.2, 0.2), (30, 50, 110)), "Battery placeholder"),
        ("part.radiator", "part_radiator.glb", part("radiator", (0.6, 0.5, 0.08), (150, 150, 155)), "Radiator placeholder"),
        ("part.fuel_tank", "part_fuel_tank.glb", part("tank", (0.7, 0.25, 0.45), (120, 120, 125)), "Fuel tank placeholder"),
        ("part.seat", "part_seat.glb", part("seat", (0.5, 0.9, 0.5), (60, 60, 70)), "Seat placeholder"),
        ("part.hood", "part_hood.glb", part("paint", (1.6, 0.06, 1.2), (150, 30, 30)), "Hood panel placeholder"),
        ("part.door", "part_door.glb", part("paint", (0.06, 0.8, 1.0), (150, 30, 30)), "Door panel placeholder"),
        ("part.bumper", "part_bumper.glb", part("bumper", (1.7, 0.2, 0.15), (30, 30, 34)), "Bumper placeholder"),
        ("building.house3d", "house.glb", house(), "Low-poly house for 2D/3D hybrid near LOD"),
    ]
    for id, f, mesh, use in items:
        mesh.write(out(f"{G}/{f}")); reg(id, f"{G}/{f}", "model", use, notes="glb, metres, +Y up, -Z forward")
    print("models ok")

if __name__ == "__main__": main()
