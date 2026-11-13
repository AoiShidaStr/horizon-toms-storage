#!/usr/bin/env python3
"""Pack this mod's two files the way docs/api/mod-files.md describes them.

  toms_storage.hmod   horizon-mod.toml + assets/ + data/
  toms_storage.hnmod  horizon-native.json + mixins/toms_storage/...

Run: python tools/pack.py
"""
import hashlib
import json
import os
import zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
ID = "toms_storage"
HORIZON = "a85af177a82504f357ff10d5d40d0b7230e2ac32"
# The horizon-mixins release the pinned Horizon was built with (the same one
# the other ports in this repo pin, a85af17).
MIXINS = "5a64af938ec86b00d53d16a408aabc0038fb3e57"
GAME = "26.3"
# NATIVE_HOOKS_API of the pinned Horizon commit
# (crates/horizon-client/src/game/native_hooks.rs).
HOOKS = 415

BUILD = os.path.join(ROOT, "target")


def manifest_version():
    with open(os.path.join(ROOT, "content", "horizon-mod.toml"), encoding="utf-8") as f:
        for line in f:
            if line.strip().startswith("version"):
                return line.split("=", 1)[1].strip().strip('"')
    raise SystemExit("no version in content/horizon-mod.toml")


def add_tree(zf, base, prefix="", skip=()):
    for dirpath, dirnames, filenames in os.walk(base):
        dirnames[:] = sorted(d for d in dirnames if d not in skip and not d.startswith("."))
        for name in sorted(filenames):
            if name.startswith("."):
                continue
            full = os.path.join(dirpath, name)
            rel = os.path.relpath(full, base).replace("\\", "/")
            zf.write(full, (prefix + rel) if prefix else rel)


def build_content():
    out = os.path.join(BUILD, ID + ".hmod")
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as zf:
        zf.write(os.path.join(ROOT, "content", "horizon-mod.toml"), "horizon-mod.toml")
        add_tree(zf, os.path.join(ROOT, "content", "assets"), "assets/")
        add_tree(zf, os.path.join(ROOT, "content", "data"), "data/")
    return out


def build_native():
    out = os.path.join(BUILD, ID + ".hnmod")
    bundle = os.path.join(ROOT, "native", ID)
    native = {
        "schema": 1,
        "id": ID,
        "version": manifest_version(),
        "game": GAME,
        "horizon": HORIZON,
        "mixins": MIXINS,
        "hooks": HOOKS,
    }
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as zf:
        zf.writestr("horizon-native.json", json.dumps(native, indent=2) + "\n")
        add_tree(zf, bundle, "mixins/" + ID + "/", skip=("target", ".build"))
    return out


def main():
    os.makedirs(BUILD, exist_ok=True)
    for path in (build_content(), build_native()):
        with open(path, "rb") as f:
            digest = hashlib.sha256(f.read()).hexdigest()
        print(f"{os.path.relpath(path)}  {os.path.getsize(path)} bytes  sha256 {digest}")


if __name__ == "__main__":
    main()
