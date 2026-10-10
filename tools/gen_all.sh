#!/bin/sh
# Regenerate ALL placeholder assets + assets.md  (needs python3, pillow, numpy)
set -e
cd "$(dirname "$0")"
rm -f ../assets/manifest.json
python3 gen_sprites.py && python3 gen_world.py && python3 gen_chars.py && python3 gen_weapons.py && python3 gen_audio.py && python3 gen_models.py && python3 gen_car.py && python3 gen_dash.py && python3 gen_hud.py && python3 gen_data.py && python3 gen_assets_md.py
