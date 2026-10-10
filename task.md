# CarOMine - TASKS

> Generated from PRD.json by tools/tasks.py - do not hand-edit. **33/69 done.**
> Legend: `[x]` done, `[~]` in progress, `[ ]` not started.

## NEXT UP

- [ ] **T-041** Snow terrain material + footprints/tire tracks decals
- [~] **T-042** Billboard system (Build-engine style: y-axis billboards, 8-angle selection, distance fade, batching/instancing)
- [~] **T-044** Roads (gravel/asphalt spline), junctions, bridges, a frozen lake, a river
- [~] **T-045** Nurmijarvi village: houses (2D/3D hybrid), home cabin + garage, K-market, gas station, church, sauna
- [~] **T-046** Day/night cycle, sun/moon, aurora borealis at night, fog, sky

## P0 Planning (1/1)

- [x] **T-001** Write PRD.md, PRD.json, PLAN.md, ARCHITECTURE.md, task.md, assets.md, CONTINUE.md  
  _note: Planning docs written_

## P1 Foundation (Rust + Bevy) (6/6)

- [x] **T-010** Cargo project, Bevy app skeleton, game states (Boot/Menu/Playing/Paused), window + Metal on M1  
  _note: Bevy 0.20 window, GameState, placeholder scene; cargo check OK_
- [x] **T-011** Asset manifest loader: assets/manifest.json -> every asset id resolves to a file path (easy replace)  
  _note: src/manifest.rs, src/settings.rs, .github/workflows/ci.yml_
- [x] **T-012** Config + keybinds (RON/JSON), settings menu stub  
  _note: src/manifest.rs, src/settings.rs, .github/workflows/ci.yml_
- [x] **T-013** GitHub Actions CI: cargo check/clippy/test on macOS, Windows, Linux  
  _note: src/manifest.rs, src/settings.rs, .github/workflows/ci.yml_
- [x] **T-014** Try cloning reference repos (RoR, eduke32, NBlood, shadow-warrior) into ref/ (gitignored) and write docs/REFERENCE_NOTES.md  
  _note: cloned RoR, shadow-warrior, NBlood; eduke32 blocked (403) -> NBlood covers; docs/REFERENCE_NOTES.md_
- [x] **T-011b** Rust side: load assets/manifest.json into AssetManifest resource (id->path)  
  _note: src/manifest.rs, src/settings.rs, .github/workflows/ci.yml_

## P2 Asset generation (placeholders) (11/11)

- [x] **T-020** tools/gen_sprites.py: enemies (8-angle billboards, walk/attack/die), 5 enemy types  
  _note: tools/gen_enemies.py: goon,hunter,neighbour,wolf,moose sheets 10x8 grid; sprites.json_
- [x] **T-021** gen: trees (spruce/pine/birch/bare), bushes, rocks, snow drifts, fences  
  _note: tools/gen_sprites.py_
- [x] **T-022** gen: items/pickups (ammo, medkit, coffee, sausage, bolts, parts icons, money)  
  _note: tools/gen_sprites.py_
- [x] **T-023** gen: weapon first-person sprites (fists, pistol, shotgun, SMG, hunting rifle) + muzzle flash  
  _note: tools/gen_sprites.py_
- [x] **T-024** gen: tileable textures (snow, packed snow, ice, road, gravel, wood, brick, metal, glass, rust)  
  _note: tools/gen_sprites.py_
- [x] **T-025** gen: house/building facade sprites + 2D/3D hybrid billboards (cabin, red barn, sauna, shop)  
  _note: tools/gen_sprites.py_
- [x] **T-026** gen: HUD/UI sprites, fonts bitmap, crosshair, windshield snow/frost overlay frames  
  _note: tools/gen_sprites.py_
- [x] **T-027** tools/gen_audio.py: placeholder SFX (engine loop, gunshots, steps, wipers, crash, UI) as WAV/OGG
- [x] **T-028** gen: placeholder music (menu, drive, night, combat) via chiptune synth
- [x] **T-029** Placeholder 3D models: low-poly car body+wheels+parts (procedural OBJ/glTF), house, tree trunk
- [x] **T-030** Auto-generate assets.md (clean table on top + detailed section) from manifest.json  
  _note: assets.md generated: python3 tools/gen_assets_md.py_

## P3 World (2/9)

- [x] **T-040** Heightmap terrain chunks (noise, flat Finnish countryside, gentle hills), LOD + streaming, ~8x8 km  
  _note: world/terrain.rs: streamed 64m chunks, noise hills, roads, lake; verified via headless screenshot_
- [ ] **T-041** Snow terrain material + footprints/tire tracks decals
- [~] **T-042** Billboard system (Build-engine style: y-axis billboards, 8-angle selection, distance fade, batching/instancing)  
  _note: face-camera + 8-angle selection done (billboard.rs); GPU instancing/batching perf pass still TODO (T-160)_
- [x] **T-043** Forest/bush scatter (spruce/pine/birch), density map, fields, clearings  
  _note: world/scatter.rs: deterministic forest/bush/rock scatter with collision obstacles_
- [~] **T-044** Roads (gravel/asphalt spline), junctions, bridges, a frozen lake, a river  
  _note: road = vertex-colour packed snow + spur to cabin. TODO: spline meshes, junctions, bridge, markers/poles, river_
- [~] **T-045** Nurmijarvi village: houses (2D/3D hybrid), home cabin + garage, K-market, gas station, church, sauna  
  _note: world/village.rs: cabin, sauna, open garage shed, K-market, 6 houses, glowing night windows, collision. TODO: church w/ steeple, gas station, interiors, far-LOD facade billboards, home furniture_
- [~] **T-046** Day/night cycle, sun/moon, aurora borealis at night, fog, sky  
  _note: sky.rs: day/night, sun, moon, fog verified. TODO: aurora, stars, bloom_
- [~] **T-047** Snowfall particles + weather states (clear, snow, blizzard)  
  _note: weather.rs: 2500-flake snowfall. TODO: weather states (blizzard etc.)_
- [~] **T-048** Lighting: headlights, streetlamps, window lights, fire/sauna glow, good-looking night (bloom, tonemapping)  
  _note: headlight spotlights + moon + ambient done. TODO: streetlamps, window glow, bloom_

## P4 Player + FPS (4/7)

- [x] **T-060** First-person controller (walk/run/crouch/jump, slippery-ice option), collision vs terrain/buildings  
  _note: player.rs: walk/run/crouch/jump, ice inertia, tree collision. (no weapons yet)_
- [x] **T-061** Weapon system: hitscan + projectile, ammo, reload, recoil, 5 weapons  
  _note: combat/weapons.rs: 5 weapons (fists/pistol/shotgun/SMG/rifle), hitscan+pellets, reload, headshots, trees block shots; keys 1-5, LMB, R_
- [x] **T-062** Enemy AI: billboard sprites, states (idle/patrol/chase/attack/flee/dead), line of sight, door/obstacle handling  
  _note: combat/enemy.rs: billboard 8-angle sprites, Idle/Chase/Attack/Dead AI, melee+ranged_
- [x] **T-063** Enemy roster: ex-agency goons, rogue hunters, wolves, angry moose, drunk neighbour  
  _note: 5 enemy defs: goon, hunter, neighbour, wolf, moose (placed at fixed spots; wider spawning in T-066)_
- [~] **T-064** Pickups, health, armor, items, interaction raycast (E key)  
  _note: pickups.rs: medkit/ammo/money/coffee/sausage auto-collect + enemy drops. TODO: E-interact items, armor, inventory_
- [~] **T-065** Damage/hit feedback, gibs/blood (snow-red), enemy drops  
  _note: hit flash + hurt overlay + corpses. TODO: blood/snow splats, gibs_
- [ ] **T-066** Agency missions: 'side jobs' for money, combat arenas in the countryside

## P5 Vehicle simulation (3/9)

- [x] **T-080** Raycast vehicle: chassis, 4 wheels, suspension, steering, drivetrain (RWD/FWD), gearbox  
  _note: CUSTOM bicycle model (no physics crate): vehicle/physics.rs, auto gearbox, terrain follow, tree collisions + Crash message_
- [x] **T-081** Tire/snow friction model: surface-dependent grip, summer/winter/studded tires, slide, ice, snowbank drag  
  _note: friction circle; spec.rs mu(surface,tyre): summer/winter/studded x snow/packed/ice/gravel_
- [x] **T-082** Engine model: torque curve, temperature, oil, fuel, battery, starter, choke  
  _note: engine temp, battery, fuel, starter attempts, torque curve, belt, alternator (build/sim.rs)_
- [~] **T-083** Enter/exit car, steering wheel view, interior cam, mirrors, handbrake  
  _note: enter/exit with E + driver cam done. TODO: chase/exterior cam, mirrors, steering wheel model_
- [~] **T-084** Crash damage: deformable-look (mesh vertex displacement + swap states), per-part HP, broken parts drop  
  _note: part damage on crash done. TODO: visible body deformation/mesh swap, smoke, broken parts drop_
- [~] **T-085** Windshield snow accumulation (Toyota Celica / Amiga style), wipers, defrost, washer fluid  
  _note: windshield.rs: snow accumulation, wipers 2 speeds, frost, night tint verified. TODO: swept-arc partial clearing, washer fluid, defrost key_
- [~] **T-086** Lights: headlights/high-beam/fog, brake lights, interior light; battery drain  
  _note: headlights toggle L. TODO: brake lights, interior light, battery drain_
- [ ] **T-087** Car audio: engine pitch by RPM, tires on snow, wipers, crash, radio
- [ ] **T-088** RoR truck-file (.truck/.car) importer to load RoR cars as placeholder vehicles (parse nodes/beams -> simplified chassis)

## P6 Car building + repair (4/9)

- [~] **T-100** Part database (JSON): engine, gearbox, brakes, tires, battery, etc. each swappable, with condition + stats  
  _note: src/build/mod.rs: 18 slots w/ condition, inventory, stock parts. TODO: move data to assets/data/parts.json (data-driven), more parts (exhaust, suspension, steering)_
- [~] **T-101** Garage mode: lift car, remove/install parts, bolt tightening (simplified), tool wheel (wrench, jack)  
  _note: src/build/workshop.rs: T near car -> Parts/Fluids/Manual menu: install/remove/fix/buy; heavy parts need garage zone. TODO: 3D lift/jack, tool wheel, visual part models in engine bay_
- [x] **T-102** Malfunction system: realistic faults (dead battery, fouled plugs, flat tire, clogged fuel filter, blown fuse, leaking coolant, worn brakes, frozen fuel line)  
  _note: build/sim.rs: dead battery, weak battery, fouled plugs, clogged filter, frozen fuel line, overheating+coolant leak, snapped belt, bad alternator, blown fuse, flat tyre, worn brakes/wipers/tyres; verified via scripted headless runs_
- [x] **T-103** Diagnosis hints: symptoms -> cause (sounds, smoke, dashboard lights), in-game manual 'Korjaamo-opas'  
  _note: start-up symptom toasts + in-game Korjaamo-opas manual page_
- [x] **T-104** Easy-repair flow after crash (simple repair minigames, cheaper than MSC)  
  _note: crash -> per-part damage (hood/bumper/radiator/engine/doors/glass/fuse/tyre), fixes: [F] hammer/patch, replace, change wheel_
- [x] **T-105** Starting project car: old Finnish-style winter wreck delivered in boxes; goal = build + pass inspection (katsastus)  
  _note: starter shell + boxes inventory (engine, gearbox, battery, plugs, filter, radiator, alternator, belt); goal: install, fill fluids, start_
- [~] **T-106** Upgrades: winter tires, studs, block heater (moottorinlampo), better lights, snow chains, engine tuning  
  _note: buy winter(180)/studded(260) tyres done. TODO: block heater, chains, lights upgrade, engine tuning_
- [ ] **T-107** Physical shops (K-market, auto parts) with buy UI + mailbox delivery; currently buy anywhere in workshop
- [ ] **T-108** Katsastus (inspection) mini-flow: checks brakes, lights, glass, body, emissions

## P7 Game loop + life sim (0/7)

- [ ] **T-120** Needs: hunger, thirst/coffee, warmth/cold, sleep, sauna; simple and forgiving
- [ ] **T-121** Economy: unemployment benefit, odd jobs, agency gigs, shops (K-market, auto parts, gas)
- [ ] **T-122** Home cabin: bed, sauna, stove (wood), fridge, TV, mailbox, phone
- [ ] **T-123** Story: premise, intro cutscene, goals (build car -> katsastus -> drive to Helsinki for final mission)
- [ ] **T-124** Save/load system (JSON), autosave, multiple slots
- [ ] **T-125** Finnish humour/flavour: NPC dialogue (Finnish+English), radio chatter, easter eggs
- [ ] **T-126** Localization (EN default, FI)

## P8 UI + audio integration (2/6)

- [~] **T-140** HUD: health, ammo, needs, speed, tacho, fuel, temp, dash lights  
  _note: combat/hud.rs: hp/armor/money/ammo/speed/gear/rpm/fuel/clock/snow%. TODO: proper gauges, needs bars_
- [ ] **T-141** Main menu, pause, settings, credits (by Crowelian 2026 + Sonnet)
- [ ] **T-142** Audio engine integration: spatial SFX, music state machine, volume sliders
- [ ] **T-143** Inventory + tool wheel UI
- [x] **T-144** Developer console (backtick or F1): pauses game, command history (up/down), help  
  _note: src/console.rs_
- [x] **T-145** Cheat codes + dev commands: god/fly/speed/give/ammo/fuel/spawn/killall/tp/time/snow/ws/car bring|goto|tyres/fov; iddqd idkfa idclip dnstuff  
  _note: src/console.rs exec_cmds; type help in game_

## P9 Polish + release (0/4)

- [ ] **T-160** Performance pass (M1 Max 60fps+, instancing, culling)
- [ ] **T-161** macOS .app bundle (cargo-bundle / scripts), Apple Silicon universal build
- [ ] **T-162** Windows + Linux builds, CI release artifacts
- [ ] **T-163** Playtest balance pass, bug list, README screenshots
