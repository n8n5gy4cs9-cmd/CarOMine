#!/usr/bin/env bash
# Run CarOMine from the source tree (assets are read from ./assets).
#   ./run.sh              optimised build (default, smooth framerate)
#   ./run.sh --dev        debug build (fast compile, slow game)
#   ./run.sh --app        open the bundled app made by ./build.sh (macOS: dist/CarOMine.app)
#   ./run.sh --portable   keep saves/settings in ./data instead of the OS data folder
# Everything after `--` goes to the game, e.g.  ./run.sh -- ...   Env knobs: see src/dev.rs (CAROMINE_*).
set -euo pipefail
cd "$(dirname "$0")"

MODE=release
for a in "$@"; do
  case "$a" in
    --dev) MODE=debug; shift ;;
    --app) MODE=app; shift ;;
    --portable) export CAROMINE_DATA_DIR="$PWD/data"; shift ;;
    --) shift; break ;;
    -h|--help) sed -n '2,8p' "$0"; exit 0 ;;
  esac
done

if [ "$MODE" = app ]; then
  case "$(uname -s)" in
    Darwin) [ -d dist/CarOMine.app ] || { echo "no app yet - run ./build.sh first"; exit 1; }; exec open dist/CarOMine.app ;;
    Linux) [ -x dist/caromine-linux/run.sh ] || { echo "no build yet - run ./build.sh first"; exit 1; }; exec dist/caromine-linux/run.sh "$@" ;;
    *) echo "--app is for macOS/Linux builds"; exit 1 ;;
  esac
fi

command -v cargo >/dev/null || { echo "cargo not found. Install Rust: https://rustup.rs  (then: rustup update stable)"; exit 1; }
[ -f assets/manifest.json ] || { echo "assets missing - generating (python3 + pillow numpy scikit-image)"; python3 -m pip install --user -q pillow numpy scikit-image; sh tools/gen_all.sh; }
if [ "$MODE" = release ]; then exec cargo run --release -- "$@"; else exec cargo run -- "$@"; fi
