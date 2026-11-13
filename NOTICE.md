# Notice

## The original

**Tom's Simple Storage Mod** by tom54541 (TomTheDeveloper) — <https://modrinth.com/mod/toms-storage>,
source at <https://github.com/tom5454/Toms-Storage>. MIT licensed.

Its licence allows a translation of its code and its art with the notice kept, and this port keeps it.
It is nevertheless a **clean-room reimplementation written from the mod's public behaviour** (its
Modrinth and CurseForge pages, its in-game descriptions and play): no Tom's Simple Storage source file
was read or translated, and none of its textures, models, sounds or language files are included.

What that means for the files here:

- `native/toms_storage/api/` — all new Rust, written against Horizon's mod API.
- `content/` — all new JSON; the textures (16x16) are drawn by `tools/gen_textures.py`, which is part
  of this repository.
- The **names** "Tom's Simple Storage", "Storage Terminal", "Inventory Connector" and "Inventory Cable"
  and the idea of a searchable terminal over linked vanilla containers belong to the original mod.

If the original author objects to this port, it will be taken down.

## Horizon

The native half is a Horizon **trait mod**: it implements `horizon_client::mod_api::Hooks` and is
linked into the game by the launcher. It pins Horizon commit
`a85af177a82504f357ff10d5d40d0b7230e2ac32` (Nightly of 2026-11-11, mod API level 415) and Horizon
Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`. No Horizon source file is modified.

## No Minecraft assets

No Mojang or Microsoft asset is included: the textures are new, the models only point at this mod's own
textures, and the block JSON copies vanilla *templates* (which Horizon reads from the game).
