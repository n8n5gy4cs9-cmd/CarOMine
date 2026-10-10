# CONTINUE.md - handoff for any AI / human (read this FIRST)

## How to resume after a token/5h limit
1. `git pull origin main`
2. Read this file, then `task.md` ("NEXT UP" at top), then the relevant section of `PLAN.md` / `ARCHITECTURE.md`.
3. `python3 tools/tasks.py next` prints the next task.
4. Do it. Then: `python3 tools/tasks.py done T-xxx "short note"` -> `git add -A && git commit -m "T-xxx: ..." && git push origin main`.
5. Update the STATE block below if something is half-done.

## Rules
- PRD.json = source of truth for task status. Never hand-edit task.md.
- Assets only via `assets/manifest.json`; regenerate `assets.md` with `python3 tools/gen_assets_md.py` (once it exists).
- Commit to `main` after EVERY task. Do not copy proprietary Duke/SW/Blood game data.
- Author credit: Crowelian 2026 + Sonnet.

## STATE (update me!)
- **Playable vertical slice is DONE and verified by headless screenshots/logs.** See task.md for the full list (~70% of PRD tasks done/partial).
- Built: terrain/forest/farms/village/cabin, day-night + weather + aurora + fog, FPS (5 weapons, 6 enemy types + boss, flashlight), car (physics, faults, workshop, dash, grime), needs/sauna/shops/gigs, story (intro -> katsastus -> HQ boss -> ending), save/load, menus, EN/FI strings, console + cheats, SDF-rendered pixel art.
- **ART PIPELINE** (important): sprites are *rendered from 3D* by `tools/sdfrender.py` (SDF ray-marcher: ramp shading, dither, AO, depth outlines). Characters `tools/gen_chars.py` (+`rig_humanoid.py`, `rig_quad.py`), weapons/hands `tools/gen_weapons.py`, trees/items `tools/gen_world.py`, car/wheel/steering meshes `tools/gen_car.py` (marching cubes -> .glb with baked grime vertex colours), dials/warning icons `tools/gen_dash.py`. `sh tools/gen_all.sh` regenerates everything (~6 min). Needs: pillow numpy scikit-image.
- Sprite sheets declare their world size/grid in assets/data/sprites.json (enemy.rs reads it). Enemy sheet = 12 cols (idle, walk x4, attack x2, pain, death x4) x 8 angle rows. Weapon sheet = 8 frames 192x144 (idle, fire, recoil, reload x4; fists: right/left punches).
- Release build check was started (target dir /tmp/claude-0/target-rel); result unknown - run `cargo build --release` and fix anything LTO-specific.
- Next ideas (see task.md todo): SSAO/colour grading, wind gusts, mailbox, interiors, RoR importer (T-088), perf pass (T-160), rebinding UI/gamepad, localisation of the rest, more art polish (T-032 hand detail, T-034 Doom-style status bar + face), dynamic dirt, turn signals.
- Environment notes: Linux cloud sandbox; lavapipe (software Vulkan) gives screenshots via xvfb. Python3 + Pillow + numpy + scikit-image for generators.
- Toolchain: Bevy **0.20** needs rustc >= 1.97.1 (`rustup update stable`). Linux build deps: libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev. First cargo check ~3 min.
- Bevy 0.20 API gotchas learned: `DirectionalLight.shadow_maps_enabled` (not shadows_enabled). Check `~/.cargo/registry/src/*/bevy_*-0.20*/` source when an API differs from older docs.
- Helper: `tools/commit.sh T-xxx "msg"` commits + pushes to main.
