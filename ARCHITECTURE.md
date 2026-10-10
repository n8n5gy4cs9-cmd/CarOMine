# CarOMine - Architecture

Rust + Bevy ECS. One crate (split into a workspace only if compile times demand it).

## Module map (`src/`)
| Module | Responsibility |
|---|---|
| `main.rs` | App setup, plugin registration |
| `state.rs` | `GameState`: Boot, MainMenu, Playing, Paused, Garage |
| `assets_manifest.rs` | Loads `assets/manifest.json`; `AssetId -> path`; hot-reload friendly |
| `config.rs` | settings/keybinds |
| `world/terrain.rs` | heightmap chunks, noise, streaming, collision height query `height_at(x,z)` |
| `world/billboards.rs` | sprite quad material + 8-angle select + instancing |
| `world/scatter.rs` | trees/bushes/rocks from density map |
| `world/village.rs` | buildings (2D/3D hybrid), roads, lake |
| `world/sky.rs` | day/night, sun, aurora, fog |
| `world/weather.rs` | snowfall particles, wind, weather state |
| `player/` | controller, interaction, inventory, needs |
| `combat/` | weapons, hitscan, projectiles, damage events |
| `enemy/` | defs (JSON), AI state machine, spawning |
| `vehicle/` | chassis, wheels (raycast), drivetrain, engine, tyres, damage, lights, windshield |
| `build/` | part DB, garage, faults, diagnosis |
| `ui/` | HUD, menus, tool wheel |
| `audio.rs` | spatial sfx, music state machine |
| `save.rs` | JSON save slots |

## Key data contracts
- `assets/manifest.json`: `{ "id": {"path": "...", "kind": "sprite|texture|model|sfx|music|font|data", "source": "generated|...", "license": "CC0"}}`. Code asks for ids only.
- Sprite sheets: `assets/sprites/<name>.png`, grid of `cols x rows`, described in `assets/data/sprites.json` (frame size, angles=8, animations{name:[frames]}).
- Parts: `assets/data/parts.json` - `{id, name, slot, model, mass, condition, stats{}, fault_modes[]}`.
- Enemies: `assets/data/enemies.json`. Weapons: `assets/data/weapons.json`.

## Vehicle model (summary)
Rigid body chassis; each wheel: ray down -> suspension spring/damper force; longitudinal force from drivetrain torque / brake; lateral force from slip angle with friction circle limited by `mu = surface_mu * tyre_mu(type, temp) * (1 - wear)`. Engine: torque(rpm) curve * throttle * health; gearbox ratios; clutch simplified (auto-clutch option). Faults modify specific factors (e.g. fouled plugs -> torque x0.5 + misfire sound).

## Billboard rendering
Custom Bevy mesh quad per sprite instance (or instanced material). Each frame: face camera around Y; pick frame = `((angle_to_cam - entity_yaw) / 45deg).round() % 8`. Alpha-masked (no sort needed) to stay cheap with huge forests.

## Threading / perf notes
Terrain chunk mesh generation on `AsyncComputeTaskPool`. Scatter placement computed per chunk, deterministic from `(seed, chunk_xy)`.

## Platform notes
macOS: Metal backend default. Don't use dynamic linking (`bevy/dynamic_linking`) in release. Build: `cargo run --release`.
