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
- Done: P1, P2 (assets), world core, player walk, vehicle sim + windshield (all verified by headless screenshots).
- Next: T-045 village+cabin buildings, T-044 road visuals, T-066 missions/spawn director, P6 parts/faults (T-100..), T-141 menus, audio wiring (bevy `wav` feature, T-142). Weapons: keys 1-5, LMB fire, R reload. Dev: CAROMINE_LINEUP=1 spawns all enemies in front of player.
- CONSOLE: press backtick or F1 in game (pauses). `help` lists commands; cheats iddqd idkfa idclip dnstuff; also god/fly/give/spawn/tp/time/snow/car bring. Add new dev commands in src/console.rs exec_cmds + HELP text. Dev env: CAROMINE_CMDS="a;b" CAROMINE_CONSOLE_OPEN=1.
- BUILD/REPAIR: src/build/{mod,sim,workshop}.rs. T near the car opens workshop (Tab pages). Start state = shell + boxes. Console: `build all|shell`, `parts`, `repair`, `fault <name>`. Dev: CAROMINE_RAW=1 CAROMINE_DRIVE=0,0 CAROMINE_IGNITE=60,100 CAROMINE_CMDS=... prints TOAST lines to log = scripted fault tests.
- Run game: `cargo run --release` (use `--release`; debug build is slow). Controls: WASD, Shift run, Ctrl crouch, Space jump, E enter/exit car, I ignition, W/S gas/brake-reverse, A/D steer, Space handbrake, L lights, V wipers.
- Headless screenshot (needs `apt-get install mesa-vulkan-drivers xvfb libxkbcommon-x11-0`):
  `CAROMINE_AUTOSTART=1 CAROMINE_SHOT=/tmp/s.png CAROMINE_SHOT_FRAMES=150 CAROMINE_HOUR=11 CAROMINE_POS=55,28,135,-8 xvfb-run -a -s "-screen 0 1280x720x24" env WGPU_BACKEND=vulkan ./target/debug/caromine`
  Also CAROMINE_DRIVE=throttle,steer  CAROMINE_WS=0..1  CAROMINE_WIPERS=0..2  CAROMINE_SNOW=0..1 (see src/dev.rs).
- Bevy 0.20 renames learned: SceneRoot->WorldAssetRoot, Events->Messages (MessageWriter, add_message), CursorOptions component, FontSize::Px, load_builder().with_settings().load(), DirectionalLight.shadow_maps_enabled, GlobalAmbientLight.
- Known: car placeholder interior is crude (roof/hood slabs); enemies not spawned yet; no audio playback yet.
- Toolchain: Bevy **0.20** needs rustc >= 1.97.1 (`rustup update stable`). Linux build deps: libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev. First cargo check ~3 min.
- Bevy 0.20 API gotchas learned: `DirectionalLight.shadow_maps_enabled` (not shadows_enabled). Check `~/.cargo/registry/src/*/bevy_*-0.20*/` source when an API differs from older docs.
- Helper: `tools/commit.sh T-xxx "msg"` commits + pushes to main.
