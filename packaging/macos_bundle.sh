#!/bin/sh
# Build CarOMine.app for macOS (Apple Silicon: run on an M1/M2/M3 Mac; add --universal for arm64+x86_64).
# Usage: sh packaging/macos_bundle.sh [--universal]
set -e
cd "$(dirname "$0")/.."
APP=dist/CarOMine.app
if [ "$1" = "--universal" ]; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  cargo build --release --target aarch64-apple-darwin
  cargo build --release --target x86_64-apple-darwin
  mkdir -p target/universal
  lipo -create target/aarch64-apple-darwin/release/caromine target/x86_64-apple-darwin/release/caromine -output target/universal/caromine
  BIN=target/universal/caromine
else
  cargo build --release
  BIN=target/release/caromine
fi
rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/caromine-bin"
cp -R assets "$APP/Contents/Resources/assets"
# launcher: Bevy resolves ./assets from the current dir (see asset_dir() in main.rs)
cat > "$APP/Contents/MacOS/CarOMine" <<'LAUNCH'
#!/bin/sh
DIR="$(cd "$(dirname "$0")/../Resources" && pwd)"
cd "$DIR"
mkdir -p "$HOME/Library/Application Support/CarOMine"
exec "$DIR/../MacOS/caromine-bin"
LAUNCH
chmod +x "$APP/Contents/MacOS/CarOMine"
cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>CarOMine</string>
<key>CFBundleDisplayName</key><string>CarOMine</string>
<key>CFBundleIdentifier</key><string>fi.crowelian.caromine</string>
<key>CFBundleExecutable</key><string>CarOMine</string>
<key>CFBundleVersion</key><string>0.1.0</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>NSHighResolutionCapable</key><true/>
<key>LSMinimumSystemVersion</key><string>12.0</string>
</dict></plist>
PLIST
echo "Built $APP  (unsigned: right-click > Open the first time, or: xattr -dr com.apple.quarantine $APP)"
