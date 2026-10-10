# CarOMine - TASKS

> Generated from PRD.json by tools/tasks.py - do not hand-edit. **13/65 done.**
> Legend: `[x]` done, `[~]` in progress, `[ ]` not started.

## NEXT UP

- [~] **T-011** Asset manifest loader: assets/manifest.json -> every asset id resolves to a file path (easy replace)
- [ ] **T-012** Config + keybinds (RON/JSON), settings menu stub
- [ ] **T-013** GitHub Actions CI: cargo check/clippy/test on macOS, Windows, Linux
- [ ] **T-014** Try cloning reference repos (RoR, eduke32, NBlood, shadow-warrior) into ref/ (gitignored) and write docs/REFERENCE_NOTES.md
- [ ] **T-011b** Rust side: load assets/manifest.json into AssetManifest resource (id->path)

## P0 Planning (1/1)

- [x] **T-001** Write PRD.md, PRD.json, PLAN.md, ARCHITECTURE.md, task.md, assets.md, CONTINUE.md  
  _note: Planning docs written_

## P1 Foundation (Rust + Bevy) (1/6)

- [x] **T-010** Cargo project, Bevy app skeleton, game states (Boot/Menu/Playing/Paused), window + Metal on M1  
  _note: Bevy 0.20 window, GameState, placeholder scene; cargo check OK_
- [~] **T-011** Asset manifest loader: assets/manifest.json -> every asset id resolves to a file path (easy replace)  
  _note: Generator side done (assetlib.reg); Rust loader = T-011b_
- [ ] **T-012** Config + keybinds (RON/JSON), settings menu stub
- [ ] **T-013** GitHub Actions CI: cargo check/clippy/test on macOS, Windows, Linux
- [ ] **T-014** Try cloning reference repos (RoR, eduke32, NBlood, shadow-warrior) into ref/ (gitignored) and write docs/REFERENCE_NOTES.md
- [ ] **T-011b** Rust side: load assets/manifest.json into AssetManifest resource (id->path)

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

## P3 World (0/9)

- [ ] **T-040** Heightmap terrain chunks (noise, flat Finnish countryside, gentle hills), LOD + streaming, ~8x8 km
- [ ] **T-041** Snow terrain material + footprints/tire tracks decals
- [ ] **T-042** Billboard system (Build-engine style: y-axis billboards, 8-angle selection, distance fade, batching/instancing)
- [ ] **T-043** Forest/bush scatter (spruce/pine/birch), density map, fields, clearings
- [ ] **T-044** Roads (gravel/asphalt spline), junctions, bridges, a frozen lake, a river
- [ ] **T-045** Nurmijarvi village: houses (2D/3D hybrid), home cabin + garage, K-market, gas station, church, sauna
- [ ] **T-046** Day/night cycle, sun/moon, aurora borealis at night, fog, sky
- [ ] **T-047** Snowfall particles + weather states (clear, snow, blizzard)
- [ ] **T-048** Lighting: headlights, streetlamps, window lights, fire/sauna glow, good-looking night (bloom, tonemapping)

## P4 Player + FPS (0/7)

- [ ] **T-060** First-person controller (walk/run/crouch/jump, slippery-ice option), collision vs terrain/buildings
- [ ] **T-061** Weapon system: hitscan + projectile, ammo, reload, recoil, 5 weapons
- [ ] **T-062** Enemy AI: billboard sprites, states (idle/patrol/chase/attack/flee/dead), line of sight, door/obstacle handling
- [ ] **T-063** Enemy roster: ex-agency goons, rogue hunters, wolves, angry moose, drunk neighbour
- [ ] **T-064** Pickups, health, armor, items, interaction raycast (E key)
- [ ] **T-065** Damage/hit feedback, gibs/blood (snow-red), enemy drops
- [ ] **T-066** Agency missions: 'side jobs' for money, combat arenas in the countryside

## P5 Vehicle simulation (0/9)

- [ ] **T-080** Raycast vehicle: chassis, 4 wheels, suspension, steering, drivetrain (RWD/FWD), gearbox
- [ ] **T-081** Tire/snow friction model: surface-dependent grip, summer/winter/studded tires, slide, ice, snowbank drag
- [ ] **T-082** Engine model: torque curve, temperature, oil, fuel, battery, starter, choke
- [ ] **T-083** Enter/exit car, steering wheel view, interior cam, mirrors, handbrake
- [ ] **T-084** Crash damage: deformable-look (mesh vertex displacement + swap states), per-part HP, broken parts drop
- [ ] **T-085** Windshield snow accumulation (Toyota Celica / Amiga style), wipers, defrost, washer fluid
- [ ] **T-086** Lights: headlights/high-beam/fog, brake lights, interior light; battery drain
- [ ] **T-087** Car audio: engine pitch by RPM, tires on snow, wipers, crash, radio
- [ ] **T-088** RoR truck-file (.truck/.car) importer to load RoR cars as placeholder vehicles (parse nodes/beams -> simplified chassis)

## P6 Car building + repair (0/7)

- [ ] **T-100** Part database (JSON): engine, gearbox, brakes, tires, battery, etc. each swappable, with condition + stats
- [ ] **T-101** Garage mode: lift car, remove/install parts, bolt tightening (simplified), tool wheel (wrench, jack)
- [ ] **T-102** Malfunction system: realistic faults (dead battery, fouled plugs, flat tire, clogged fuel filter, blown fuse, leaking coolant, worn brakes, frozen fuel line)
- [ ] **T-103** Diagnosis hints: symptoms -> cause (sounds, smoke, dashboard lights), in-game manual 'Korjaamo-opas'
- [ ] **T-104** Easy-repair flow after crash (simple repair minigames, cheaper than MSC)
- [ ] **T-105** Starting project car: old Finnish-style winter wreck delivered in boxes; goal = build + pass inspection (katsastus)
- [ ] **T-106** Upgrades: winter tires, studs, block heater (moottorinlampo), better lights, snow chains, engine tuning

## P7 Game loop + life sim (0/7)

- [ ] **T-120** Needs: hunger, thirst/coffee, warmth/cold, sleep, sauna; simple and forgiving
- [ ] **T-121** Economy: unemployment benefit, odd jobs, agency gigs, shops (K-market, auto parts, gas)
- [ ] **T-122** Home cabin: bed, sauna, stove (wood), fridge, TV, mailbox, phone
- [ ] **T-123** Story: premise, intro cutscene, goals (build car -> katsastus -> drive to Helsinki for final mission)
- [ ] **T-124** Save/load system (JSON), autosave, multiple slots
- [ ] **T-125** Finnish humour/flavour: NPC dialogue (Finnish+English), radio chatter, easter eggs
- [ ] **T-126** Localization (EN default, FI)

## P8 UI + audio integration (0/4)

- [ ] **T-140** HUD: health, ammo, needs, speed, tacho, fuel, temp, dash lights
- [ ] **T-141** Main menu, pause, settings, credits (by Crowelian 2026 + Sonnet)
- [ ] **T-142** Audio engine integration: spatial SFX, music state machine, volume sliders
- [ ] **T-143** Inventory + tool wheel UI

## P9 Polish + release (0/4)

- [ ] **T-160** Performance pass (M1 Max 60fps+, instancing, culling)
- [ ] **T-161** macOS .app bundle (cargo-bundle / scripts), Apple Silicon universal build
- [ ] **T-162** Windows + Linux builds, CI release artifacts
- [ ] **T-163** Playtest balance pass, bug list, README screenshots
