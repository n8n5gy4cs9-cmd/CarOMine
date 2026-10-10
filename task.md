# CarOMine - TASKS

> Generated from PRD.json by tools/tasks.py - do not hand-edit. **51/86 done.**
> Legend: `[x]` done, `[~]` in progress, `[ ]` not started.

## NEXT UP

- [~] **T-032** ART PASS 2 - Player first-person hands + weapons: regenerate fists, pistol, shotgun, SMG, hunting rifle (+flashlight attachment, muzzle flash, shell ejection) as Doom/ROTT-style hand-shaded sprites: gloved/knuckled hands w/ sleeves, metal highlights, wood grain, 4+ frames (idle, fire, recoil, reload), higher res (e.g. 192x144)
- [ ] **T-033** ART PASS 3 - World sprites: trees (spruce/pine/birch/dead), bushes, rocks, fences, lamps, items/pickups, building facades regenerated with the same pixel-realistic shaded style + lighting ramps; consistent palette (limited ~64 colour master palette, cold blue shadows / warm highlights)
- [ ] **T-034** ART PASS 4 - UI/HUD in Doom-style: status bar frame, face/portrait that reacts to damage (Doom marine face idea), bitmap font, weapon/ammo icons, windshield overlays re-done by hand-shaded technique
- [ ] **T-041** Snow terrain material + footprints/tire tracks decals
- [~] **T-042** Billboard system (Build-engine style: y-axis billboards, 8-angle selection, distance fade, batching/instancing)

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

## P2 Asset generation (placeholders) (13/16)

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
- [x] **T-031** ART PASS 1 - Characters: regenerate ALL enemy sheets (goon, hunter, neighbour, wolf, moose + boss Director) in retro 'Doom / Rise of the Triad' pixel-realistic style: 8 angles, shaded forms (3-4 tone ramps per material, dithering, hard dark outline), anatomically proportioned humans (head/torso/arms/legs, clothing folds, boots, facial features), animal anatomy for wolf/moose, walk x4 / attack x2 / pain / death x4 frames  
  _note: tools/gen_chars.py + sdfrender.py + rig_humanoid.py + rig_quad.py: 6 characters (goon, hunter, neighbour, wolf, moose, director/boss) rendered from 3D rigs into 72x96 (wolf 112x64, moose 144x112) Doom/ROTT-style shaded pixel sheets, 12 cols (idle, walk x4, attack x2, pain, death x4) x 8 angles; enemy.rs reads world size from sprites.json_
- [~] **T-032** ART PASS 2 - Player first-person hands + weapons: regenerate fists, pistol, shotgun, SMG, hunting rifle (+flashlight attachment, muzzle flash, shell ejection) as Doom/ROTT-style hand-shaded sprites: gloved/knuckled hands w/ sleeves, metal highlights, wood grain, 4+ frames (idle, fire, recoil, reload), higher res (e.g. 192x144)  
  _note: tools/gen_weapons.py: pistol/shotgun/SMG/rifle/fists with gloved hands, muzzle flash, recoil, reload stages, left/right punches (192x144 x8 frames), wired into HUD. TODO polish: hand detail, flashlight attachment visible, shell ejection, per-weapon tuning_
- [ ] **T-033** ART PASS 3 - World sprites: trees (spruce/pine/birch/dead), bushes, rocks, fences, lamps, items/pickups, building facades regenerated with the same pixel-realistic shaded style + lighting ramps; consistent palette (limited ~64 colour master palette, cold blue shadows / warm highlights)
- [ ] **T-034** ART PASS 4 - UI/HUD in Doom-style: status bar frame, face/portrait that reacts to damage (Doom marine face idea), bitmap font, weapon/ammo icons, windshield overlays re-done by hand-shaded technique
- [x] **T-035** Art tooling: tools/pixelkit.py (palette ramps, dither, outline, shading from a 3D-ish light model, sheet packer) so every sprite is built from shaded primitives; keep manifest ids/grids/sprites.json unchanged so replacement is drop-in  
  _note: tooling is sdfrender.py (SDF ray-marcher: ramp shading, ordered dither, AO, depth edge outlines) + rigs; replaces the planned pixelkit_

## P3 World (4/13)

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
- [~] **T-049** Proper lighting pass: bloom, tonemapping + exposure per time of day, SSAO, colour grading (cold blue night / warm low sun), light shafts, soft shadows tuned  
  _note: world/atmosphere.rs: HDR bloom, TonyMcMapface tonemapping, volumetric light shafts + FogVolume, moon+ambient. TODO: SSAO, colour grading per time of day, tune on real GPU (M1)_
- [x] **T-050** Fog + atmosphere: height fog, morning mist, weather-driven fog density, blizzard whiteout, horizon haze, fog colour tied to sky  
  _note: weather-driven linear fog (blizzard whiteout), morning/evening mist, fog colour tied to sky, volumetric density from weather+mist_
- [~] **T-051** Atmospheric effects: stars + moon/sun discs, aurora borealis, breath vapour, exhaust smoke, tyre snow spray, wind-blown snow gusts, snow on tree billboards, chimney smoke  
  _note: stars, sun+moon discs, aurora curtains (night+clear), breath vapour, exhaust (dark smoke if engine/plugs bad), hood steam when overheating, tyre snow spray, cabin chimney smoke. TODO: wind gusts of snow, sauna smoke, headlight beams in fog_
- [x] **T-068** Farmsteads: deterministic farm sites (house, barn, sauna, fenced yard, flattened + cleared) streamed with chunks  
  _note: layout::farm_site_in_cell + village::spawn_farm_

## P4 Player + FPS (8/10)

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
- [x] **T-066** Agency missions: 'side jobs' for money, combat arenas in the countryside  
  _note: life/missions.rs: gig board (phone) with Hunt/Deliver/Recover gigs, waypoint beacon, compass HUD, rewards; console gig cmds_
- [x] **T-052** Flashlight (F): handheld SpotLight, battery drain, flicker when low, sway  
  _note: combat/flashlight.rs: F toggle, battery drain 7min, flicker <15%, sway, batteries pickup, HUD %; shadows on_
- [x] **T-053** Weapon-mounted flashlight: attachment unlockable per weapon (pistol/shotgun/SMG/rifle), beam follows weapon, toggle F; console 'give flashlight'  
  _note: X mounts/unmounts light on equipped gun (muzzle offset, tighter beam); fists = hand-held. Works for ALL weapons_
- [x] **T-067** Enemy population director (combat/director.rs): spawns by area/time/weather near player, culls far  
  _note: combat/director.rs_

## P5 Vehicle simulation (5/9)

- [x] **T-080** Raycast vehicle: chassis, 4 wheels, suspension, steering, drivetrain (RWD/FWD), gearbox  
  _note: CUSTOM bicycle model (no physics crate): vehicle/physics.rs, auto gearbox, terrain follow, tree collisions + Crash message_
- [x] **T-081** Tire/snow friction model: surface-dependent grip, summer/winter/studded tires, slide, ice, snowbank drag  
  _note: friction circle; spec.rs mu(surface,tyre): summer/winter/studded x snow/packed/ice/gravel_
- [x] **T-082** Engine model: torque curve, temperature, oil, fuel, battery, starter, choke  
  _note: engine temp, battery, fuel, starter attempts, torque curve, belt, alternator (build/sim.rs)_
- [x] **T-083** Enter/exit car, steering wheel view, interior cam, mirrors, handbrake  
  _note: driver seat + chase camera (C, mouse orbit); mirrors/wheel model TODO_
- [~] **T-084** Crash damage: deformable-look (mesh vertex displacement + swap states), per-part HP, broken parts drop  
  _note: visible body squash/tilt from panel damage (vehicle::body_damage). TODO: real mesh deformation, dents/decals, broken-off parts_
- [x] **T-085** Windshield snow accumulation (Toyota Celica / Amiga style), wipers, defrost, washer fluid  
  _note: windshield: 20 strips, snow per strip, wiper sweeps a fan-shaped cleared arc, wiper blade wear scales clearing, frost, night tint_
- [~] **T-086** Lights: headlights/high-beam/fog, brake lights, interior light; battery drain  
  _note: headlights toggle L. TODO: brake lights, interior light, battery drain_
- [ ] **T-087** Car audio: engine pitch by RPM, tires on snow, wipers, crash, radio
- [ ] **T-088** RoR truck-file (.truck/.car) importer to load RoR cars as placeholder vehicles (parse nodes/beams -> simplified chassis)

## P6 Car building + repair (6/9)

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
- [x] **T-107** Physical shops (K-market, auto parts) with buy UI + mailbox delivery; currently buy anywhere in workshop  
  _note: K-market + Auto parts shop dialogs at village doors (lit lamps mark entrances). Mailbox delivery still TODO_
- [x] **T-108** Katsastus (inspection) mini-flow: checks brakes, lights, glass, body, emissions  
  _note: Katsastus station in village: 9-point inspection (brakes, tyres, winter tyres, lights, glass, wipers, body, emissions...), 30 EUR fee, +100 EUR on pass_

## P7 Game loop + life sim (4/9)

- [x] **T-120** Needs: hunger, thirst/coffee, warmth/cold, sleep, sauna; simple and forgiving  
  _note: life/needs.rs: hunger/energy/warmth, freezing/starving damage, car heater, pass-out, HUD_
- [x] **T-121** Economy: unemployment benefit, odd jobs, agency gigs, shops (K-market, auto parts, gas)  
  _note: Kela benefit +35 EUR each 08:00, gigs for money, K-market + auto parts shops, enemy loot_
- [~] **T-122** Home cabin: bed, sauna, stove (wood), fridge, TV, mailbox, phone  
  _note: life/dialog.rs: cabin door dialog (sleep w/ autosave, fridge, coffee, firewood, phone), sauna (stove w/ logs, bathe). No walkable interiors yet; TV/mailbox TODO_
- [x] **T-123** Story: premise, intro cutscene, goals (build car -> katsastus -> drive to Helsinki for final mission)  
  _note: src/story.rs: intro dialog, auto-tracked objectives (build->run->winter tyres->katsastus->Helsinki), HQ compound + boss 'The Director', ending screen, story saved_
- [x] **T-124** Save/load system (JSON), autosave, multiple slots  
  _note: src/save.rs: JSON saves in ./saves (autosave every 3 min + slot1-3), player/car/parts/inventory/time/weather; console save/load; verified round trip_
- [~] **T-125** Finnish humour/flavour: NPC dialogue (Finnish+English), radio chatter, easter eggs  
  _note: flavour text: intro, Kela, katsastus, makkara, sauna. TODO: NPC dialogue, radio chatter, easter eggs, Finnish strings_
- [~] **T-126** Localization (EN default, FI)  
  _note: i18n.rs + assets/data/strings_{en,fi}.json (86 keys): menu, settings (language toggle), HUD, prompts, objectives, Kela toast localized. TODO: workshop/dialog/manual/sim toasts/missions text_
- [ ] **T-127** Mailbox: parts ordered by phone arrive next day (MSC Teemo style); TV with Finnish channels; cabin interior (walkable)
- [ ] **T-128** Boss fight polish: unique boss sprite, phases, minions; second endgame area; New Game+

## P8 UI + audio integration (4/8)

- [~] **T-140** HUD: health, ammo, needs, speed, tacho, fuel, temp, dash lights  
  _note: combat/hud.rs: hp/armor/money/ammo/speed/gear/rpm/fuel/clock/snow%. TODO: proper gauges, needs bars_
- [x] **T-141** Main menu, pause, settings, credits (by Crowelian 2026 + Sonnet)  
  _note: src/menu.rs: main menu (new/continue/load/settings/quit), pause menu (resume/save slots/load/settings/quit), settings sliders persisted to settings.json, credits line_
- [x] **T-142** Audio engine integration: spatial SFX, music state machine, volume sliders  
  _note: src/audio.rs: Sfx messages, music state machine (menu/drive/night/combat), engine/tyre/wind loops, spatial listener; hooks in weapons/enemy/pickups/vehicle/build/player. (cannot hear in sandbox: no audio device)_
- [ ] **T-143** Inventory + tool wheel UI
- [x] **T-144** Developer console (backtick or F1): pauses game, command history (up/down), help  
  _note: src/console.rs_
- [x] **T-145** Cheat codes + dev commands: god/fly/speed/give/ammo/fuel/spawn/killall/tp/time/snow/ws/car bring|goto|tyres/fov; iddqd idkfa idclip dnstuff  
  _note: src/console.rs exec_cmds; type help in game_
- [ ] **T-146** Pause menu: 'Quit to main menu' (needs world teardown/reset), new-game reset
- [ ] **T-147** Key rebinding UI + gamepad support

## P9 Polish + release (0/5)

- [ ] **T-160** Performance pass (M1 Max 60fps+, instancing, culling)
- [~] **T-161** macOS .app bundle (cargo-bundle / scripts), Apple Silicon universal build  
  _note: packaging/macos_bundle.sh (.app, universal option) written; untested (needs a Mac)_
- [~] **T-162** Windows + Linux builds, CI release artifacts  
  _note: packaging/linux_windows.md instructions; CI release artifacts TODO_
- [ ] **T-163** Playtest balance pass, bug list, README screenshots
- [ ] **T-165** Move saves/settings to OS data dir (dirs crate); macOS app bundle cwd handling
