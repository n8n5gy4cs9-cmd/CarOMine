# CarOMine

Finnish winter countryside car-building survival + retro FPS. You're an unemployed secret agent in Nurmijärvi: rebuild your old car from a pile of boxes, survive the cold, shoot whoever turns up, pass the *katsastus*, and drive to Helsinki.

**By Crowelian 2026 + Sonnet.** Rust + [Bevy](https://bevy.org) 0.20. Primary target: macOS Apple Silicon (Metal). Windows/Linux builds work from the same code.

Inspired by My Summer Car / My Winter Car / Mökkisimulaattori, with a Build-engine (Duke3D / Shadow Warrior / Blood) look: **2D sprite billboards** for enemies, trees, bushes and items; **3D** cars, terrain and houses.

## Run
```
rustup update stable          # Bevy 0.20 needs rustc >= 1.97.1
cargo run --release           # first build takes a while
```
Linux build deps: `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`.
macOS app bundle: `sh packaging/macos_bundle.sh` (see `packaging/`).

## Controls
| Key | Action |
|---|---|
| WASD / Shift / Ctrl / Space | move / run / crouch / jump |
| Mouse, LMB, R, 1-5 | look, fire, reload, weapons (fists, pistol, shotgun, SMG, rifle) |
| F / X | flashlight on-off / mount it on the weapon |
| E | use doors (cabin, sauna, shops, inspection) / get in or out of the car |
| T | workshop (stand next to your car) |
| Car: W/S, A/D, Space, I, L, V, H, C | gas/brake-reverse, steer, handbrake, ignition, lights, wipers, horn, chase cam |
| `` ` `` or F1 | developer console (type `help`; cheats: `iddqd idkfa idclip dnstuff`) |
| Esc | pause menu (save slots, load, settings) |

## The loop
1. The car is a shell; engine, gearbox, battery, plugs... are in boxes. Install them in the **workshop** (heavy parts at the garage shed west of the cabin).
2. Real faults, real fixes: dead battery, fouled plugs, clogged filter, frozen fuel line, overheating, snapped belt, flat tyre, blown fuse. Symptoms show as messages; the in-game **Korjaamo-opas** manual explains them.
3. Winter: snow piles up on the windshield (run the wipers), summer tyres are lethal on ice, nights are dark (headlights, flashlight, moon, aurora).
4. Live: hunger, energy, warmth. Sleep, coffee, makkara, firewood, **sauna**, Kela benefit every morning, agency gigs by phone, K-market and auto parts shop.
5. Pass the katsastus → drive south to Helsinki → The Director.

## Project docs (for humans and AIs)
[PRD.md](PRD.md) · [PLAN.md](PLAN.md) · [ARCHITECTURE.md](ARCHITECTURE.md) · [task.md](task.md) (generated from [PRD.json](PRD.json)) · [assets.md](assets.md) (every asset, how to replace it) · [CONTINUE.md](CONTINUE.md) (resume here) · [docs/REFERENCE_NOTES.md](docs/REFERENCE_NOTES.md)

## Assets
All art/audio/models are **generated placeholders** (CC0) made by `tools/gen_*.py` (needs Python 3 + Pillow + numpy): `sh tools/gen_all.sh`. Everything is referenced through `assets/manifest.json`; replace any file and keep the same name/grid. See [assets.md](assets.md).

## Dev tools
`CAROMINE_AUTOSTART=1 CAROMINE_SHOT=out.png ...` renders headlessly (xvfb + lavapipe) for screenshots and scripted tests; see `src/dev.rs` and `CONTINUE.md`.

## License
GPL-3.0-or-later (see [LICENSE](LICENSE)); reference-project notes in PRD.md §8. Game data of Duke3D / Shadow Warrior / Blood is not included and not free; all shipped assets are generated.
