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
- Last completed: P1 + P2 (T-010..T-030, T-014). All assets generated (see assets.md).
- **WIP, UNVERIFIED (committed uncompiled!)**: src/world/*, src/player.rs, src/vehicle/*, src/dev.rs. Next exact step: `cargo build` and fix compile errors (Bevy 0.20 API drift: check ~/.cargo/registry/src/*/bevy_*-0.20*), then run headless screenshot:
  `CAROMINE_AUTOSTART=1 CAROMINE_SHOT=/tmp/s.png CAROMINE_HOUR=11 CAROMINE_POS=40,20,200,-5 xvfb-run -a env WGPU_BACKEND=vulkan ./target/debug/caromine` (needs `apt-get install mesa-vulkan-drivers xvfb`; first debug build ~10+ min).
  After it works: mark T-040,T-042,T-043,T-046,T-047,T-060,T-080,T-081,T-083,T-085,T-086 as done/partial in PRD.json via tools/tasks.py.
- Environment notes: Linux cloud sandbox; lavapipe gives software Vulkan for screenshots. Python3 + Pillow + numpy for asset generators.
- Toolchain: Bevy **0.20** needs rustc >= 1.97.1 (`rustup update stable`). Linux build deps: libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev. First cargo check ~3 min.
- Bevy 0.20 API gotchas learned: `DirectionalLight.shadow_maps_enabled` (not shadows_enabled). Check `~/.cargo/registry/src/*/bevy_*-0.20*/` source when an API differs from older docs.
- Helper: `tools/commit.sh T-xxx "msg"` commits + pushes to main.
- Known issues: none yet.
