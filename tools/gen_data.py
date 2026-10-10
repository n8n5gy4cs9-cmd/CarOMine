#!/usr/bin/env python3
"""Copy hand-edited data (tools/strings/*.json) into assets/data and register in the manifest."""
import json, shutil
from assetlib import *
for lang, name in (("en", "English"), ("fi", "Finnish")):
    shutil.copy(ROOT / f"tools/strings/{lang}.json", out(f"data/strings_{lang}.json"))
    reg(f"data.strings_{lang}", f"data/strings_{lang}.json", "data", f"UI text table: {name}", source="hand-written (tools/strings)", notes="key->string; {n} placeholders. Add a language by adding strings_xx.json + registering here")
print("data ok")
