# Packaging notes (Windows / Linux)

Run from the repo root so `./assets` is found (or copy `assets/` next to the executable).

## Linux
```
sudo apt-get install -y libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
cargo build --release
mkdir -p dist/caromine-linux && cp target/release/caromine dist/caromine-linux/ && cp -r assets dist/caromine-linux/
cd dist/caromine-linux && ./caromine
```

## Windows (PowerShell)
```
cargo build --release
New-Item -ItemType Directory dist\caromine-win -Force
Copy-Item target\release\caromine.exe dist\caromine-win\
Copy-Item -Recurse assets dist\caromine-win\assets
cd dist\caromine-win; .\caromine.exe
```
(MSVC build tools required. DX12/Vulkan via wgpu.)

## Saves / settings
`saves/` and `settings.json` are created in the working directory. (TODO T-165: move to OS data dir.)
