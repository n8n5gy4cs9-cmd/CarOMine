# Reference repo notes (T-014)

Cloned (shallow) into `ref/` (gitignored; re-clone with the commands below). **Read-only study material.**

| Repo | Cloned? | Licence | Use for |
|---|---|---|---|
| https://github.com/RigsOfRods/rigs-of-rods | yes | GPLv3 | vehicle physics ideas, ground friction tables, truck file format |
| https://github.com/Azarien/shadow-warrior | yes | GPL (source only) | Build-engine game logic: enemy AI (`source/ai.c`, `actor.c`), sprite behaviour |
| https://github.com/NBlood/NBlood | yes | GPL (source only) | EDuke32-derived Build engine + Blood/SW/Duke ports; AI in `source/blood/src/ai*.cpp`, SW in `source/sw/src/*.cpp`, renderer in `source/build/src` |
| https://voidpoint.io/terminx/eduke32 | **NO - HTTP 403 from sandbox proxy (host not allowed)** | GPL | NBlood is an EDuke32 fork, same Build engine -> use `NBlood/source/build` + `NBlood/source/duke3d` instead. Can retry on a machine with open network. |

```
git clone --depth 1 https://github.com/RigsOfRods/rigs-of-rods ref/rigs-of-rods
git clone --depth 1 https://github.com/Azarien/shadow-warrior ref/shadow-warrior
git clone --depth 1 https://github.com/NBlood/NBlood ref/NBlood
git clone --depth 1 https://voidpoint.io/terminx/eduke32 ref/eduke32   # blocked in cloud sandbox
```

## Rigs of Rods findings
- **No car content in the repo** - vehicles (`.truck`/`.car`) are separate downloadable mods from the RoR repository. The repo only has engine + default sounds/materials. So for T-088 we need the file format and user-provided/third-party cars (check each licence).
- **File format docs:** `ref/rigs-of-rods/source/main/resources/rig_def_fileformat/` (truck/car definition), `odef_fileformat`, `terrn2_fileformat`, `tuneup_fileformat`. Doc: `doc/angelscript/*.md`.
- Soft-body: nodes+beams in `source/main/physics/Actor*.cpp` (ActorForcesEuler.cpp = integration, slip/traction control around lines 362-490). We do NOT port this; we use a raycast vehicle and, for imports, derive bounding box/wheel positions/mass from nodes.
- **Surface friction (adopt as baseline for our table)** from `resources/skeleton/config/ground_models.cfg`:

| surface | static mu | sliding mu | stribeck vel |
|---|---|---|---|
| ice | 0.30 | 0.20 | 3 |
| snow | 0.65 | 0.40 | 6 |
| gravel | see file | | |
| asphalt/concrete | see file | | |
Our grip = `surface_mu * tyre_mult`; start with snow 0.65/0.40, ice 0.30/0.20, then tune for fun (more forgiving).
- RoR default sounds in `resources/sounds` (71 wavs, e.g. `default_car_idle.wav`) are GPL-project assets; licence per file unclear -> **not copied**; we ship generated placeholders. Can be swapped in later by the user after checking licence.

## Build-engine (NBlood / Shadow Warrior) findings
- Enemies are **sprites with 8 rotational frames** (`angle -> frame`) plus per-state animation tables; AI = state machine with function pointers (`ai.c` / `aipatrol.cpp`). Our `enemy.*` sheets use this layout (10 cols x 8 angle rows).
- Hitscan weapons trace through sectors against sprites; ours: ray vs. billboard capsule + terrain/building colliders.
- Keep the *feel*: fast movement (no head-bob excess), instant weapon swap, pickups auto-collect, sector-style doors on buildings.
- Do not copy art/sound data (not free). Code ideas only; any ported code stays GPL.
