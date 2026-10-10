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
- Last completed: T-001 (planning docs).
- In progress: none.
- Next exact step: T-010 - create Cargo project (`cargo init --name caromine`), add Bevy, window + states.
- Environment notes: Linux cloud sandbox (no GPU) -> can only `cargo check`, not run visuals. Python3 + Pillow + numpy available for asset generators.
- Known issues: none yet.
