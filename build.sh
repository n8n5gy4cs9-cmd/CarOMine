#!/usr/bin/env bash
# CarOMine release build.
#   macOS   -> dist/CarOMine.app   (Apple Silicon native; --universal adds Intel)
#   Linux   -> dist/caromine-linux/
#   Windows -> dist/caromine-win/  (run from Git Bash / MSYS2)
# Usage: ./build.sh [--universal] [--zip] [--regen-assets] [--debug]
set -euo pipefail
cd "$(dirname "$0")"

UNIVERSAL=0; ZIP=0; REGEN=0; PROFILE=release
for a in "$@"; do
  case "$a" in
    --universal) UNIVERSAL=1 ;;
    --zip) ZIP=1 ;;
    --regen-assets) REGEN=1 ;;
    --debug) PROFILE=debug ;;
    -h|--help) sed -n '2,8p' "$0"; exit 0 ;;
    *) echo "unknown option: $a"; exit 1 ;;
  esac
done
CARGO_FLAGS=""; [ "$PROFILE" = release ] && CARGO_FLAGS="--release"

command -v cargo >/dev/null || { echo "cargo not found. Install Rust: https://rustup.rs"; exit 1; }
# Bevy 0.20 needs a recent compiler
need=1.97; have=$(rustc --version | awk '{print $2}')
if [ "$(printf '%s\n%s\n' "$need" "$have" | sort -V | head -1)" != "$need" ]; then
  echo "rustc $have is too old (need >= $need). Run: rustup update stable"; exit 1
fi

# Assets are generated placeholders; (re)create them if missing or asked to.
if [ "$REGEN" = 1 ] || [ ! -f assets/manifest.json ]; then
  echo "==> generating assets (needs: python3 + pillow numpy scikit-image)"
  python3 -c "import PIL, numpy, skimage" 2>/dev/null || python3 -m pip install --user pillow numpy scikit-image
  sh tools/gen_all.sh
fi

OS=$(uname -s)
build_target() { cargo build $CARGO_FLAGS ${1:+--target "$1"}; }

case "$OS" in
Darwin)
  APP=dist/CarOMine.app
  if [ "$UNIVERSAL" = 1 ]; then
    rustup target add aarch64-apple-darwin x86_64-apple-darwin
    build_target aarch64-apple-darwin; build_target x86_64-apple-darwin
    mkdir -p target/universal
    lipo -create "target/aarch64-apple-darwin/$PROFILE/caromine" "target/x86_64-apple-darwin/$PROFILE/caromine" -output target/universal/caromine
    BIN=target/universal/caromine
  else
    build_target; BIN="target/$PROFILE/caromine"
  fi
  echo "==> bundling $APP"
  rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
  cp "$BIN" "$APP/Contents/MacOS/caromine-bin"
  cp -R assets "$APP/Contents/Resources/assets"
  # Launcher: the game resolves ./assets from its working directory; saves/settings go to ~/Library/Application Support/CarOMine.
  cat > "$APP/Contents/MacOS/CarOMine" <<'LAUNCH'
#!/bin/sh
cd "$(dirname "$0")/../Resources" || exit 1
exec "$(dirname "$0")/caromine-bin" "$@"
LAUNCH
  chmod +x "$APP/Contents/MacOS/CarOMine" "$APP/Contents/MacOS/caromine-bin"
  # App icon from the generated sprites if sips/iconutil exist (optional)
  if command -v sips >/dev/null && command -v iconutil >/dev/null && [ -f assets/sprites/ui/faces.png ]; then
    ICONSET=$(mktemp -d)/CarOMine.iconset; mkdir -p "$ICONSET"
    python3 - "$ICONSET" <<'PY' || true
import sys
from PIL import Image
out = sys.argv[1]
faces = Image.open("assets/sprites/ui/faces.png").convert("RGBA").crop((0, 0, 36, 44))
bg = Image.new("RGBA", (44, 44), (28, 40, 70, 255)); bg.alpha_composite(faces, (4, 0))
for s in (16, 32, 64, 128, 256, 512, 1024):
    bg.resize((s, s), Image.NEAREST).save(f"{out}/icon_{s}x{s}.png")
    if s <= 512: bg.resize((s * 2, s * 2), Image.NEAREST).save(f"{out}/icon_{s}x{s}@2x.png")
PY
    iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/CarOMine.icns" 2>/dev/null && ICON_KEY="<key>CFBundleIconFile</key><string>CarOMine</string>" || ICON_KEY=""
  else ICON_KEY=""; fi
  VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')
  cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>CarOMine</string>
<key>CFBundleDisplayName</key><string>CarOMine</string>
<key>CFBundleIdentifier</key><string>fi.crowelian.caromine</string>
<key>CFBundleExecutable</key><string>CarOMine</string>
<key>CFBundleVersion</key><string>$VERSION</string>
<key>CFBundleShortVersionString</key><string>$VERSION</string>
<key>CFBundlePackageType</key><string>APPL</string>
$ICON_KEY
<key>NSHighResolutionCapable</key><true/>
<key>LSMinimumSystemVersion</key><string>12.0</string>
<key>NSMicrophoneUsageDescription</key><string>Not used.</string>
</dict></plist>
PLIST
  # Ad-hoc sign so Gatekeeper on Apple Silicon will launch a locally built app
  command -v codesign >/dev/null && codesign --force --deep --sign - "$APP" 2>/dev/null || true
  echo "==> built $APP"
  echo "    First launch if macOS complains it is from an unidentified developer:  xattr -dr com.apple.quarantine $APP"
  OUT="$APP"
  ;;
Linux)
  build_target
  D=dist/caromine-linux; rm -rf "$D"; mkdir -p "$D"
  cp "target/$PROFILE/caromine" "$D/caromine"; cp -R assets "$D/assets"
  printf '#!/bin/sh\ncd "$(dirname "$0")" && exec ./caromine "$@"\n' > "$D/run.sh"; chmod +x "$D/run.sh" "$D/caromine"
  echo "==> built $D (run: $D/run.sh)"; OUT="$D"
  ;;
MINGW*|MSYS*|CYGWIN*)
  build_target
  D=dist/caromine-win; rm -rf "$D"; mkdir -p "$D"
  cp "target/$PROFILE/caromine.exe" "$D/"; cp -R assets "$D/assets"
  echo "==> built $D (run caromine.exe)"; OUT="$D"
  ;;
*) echo "unsupported OS: $OS"; exit 1 ;;
esac

if [ "$ZIP" = 1 ]; then
  ( cd dist && zip -qry "$(basename "$OUT").zip" "$(basename "$OUT")" ) && echo "==> dist/$(basename "$OUT").zip"
fi
