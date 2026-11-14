# Tom's Simple Storage for Horizon

A **storage terminal** and the **connectors** that link chests to it: right-click the terminal and every
item in the inventories joined to it is listed in one menu, with a search box above it. Taking a stack
takes the real items out of the real chests; shift-clicking a stack in your inventory puts it back in
the network.

A port of [Tom's Simple Storage](https://modrinth.com/mod/toms-storage) by tom54541 (MIT) to
[Horizon](https://github.com/horizon-rust/horizon), Minecraft: Java Edition 26.3 rewritten in Rust.
The idea and the block names come from it; **all code, models, textures and names here are new**
(see [NOTICE.md](NOTICE.md)). Not made or endorsed by tom54541, Mojang, Microsoft or the Horizon team.

## Two parts, both needed

| Part | Folder | File | What it is |
|---|---|---|---|
| Content | `content/` | `toms_storage.hmod` | the terminal, the connector and the cable, their models, textures, loot, recipes, names and creative tab. No code. |
| Native | `native/toms_storage/` | `toms_storage.hnmod` | the terminal itself: the network walk, the list, the search, the clicks. A Horizon **trait mod** (`[api]`), so nothing of Horizon's source is patched. |

The content mod alone gives decorative blocks; the native mod alone does nothing (its hooks never fire
for blocks the game does not have). Install both.

## Playing

1. Put the terminal against your chests and run **inventory connectors** and **inventory cables** from
   it. A connector or a cable joins to another one, to the terminal, and to the inventories beside it.
   Only blocks joined to the terminal are part of its network; a cable diags into a chest, it does not
   go through it.
2. Right-click the terminal. Every container your network touches (chests, barrels, hoppers,
   dispensers, droppers, shulker boxes, furnaces, and a mod's own containers) is read and its items
   are summed into one list, 45 lines a page.
3. **Left click** a line takes a full stack of it, **right click** takes one. They come out of the
   containers themselves, so nothing is copied and nothing is left behind if you close the menu.
4. **Shift-click** any slot of your own inventory to send that stack into the network.
5. **Type** in the bar above the menu to search: every word you type must appear in the item's id,
   in any case (`oak plank` finds `minecraft:oak_planks`, `stone` finds stone and stone bricks).
   Backspace deletes; the arrows on the last row turn the page.

The list is in network order (the order the walk finds the containers and their slots), the count on
each line is the total in the network capped at the item's stack size, and the stack you see is the
real stored one (its components and damage included).

## Done

- The terminal block, the inventory connector and the inventory cable, with models that join along the
  six directions (`connects_to` on the chorus-plant template), original textures, loot, recipes, English
  and French names, a creative tab, and `minecraft:mineable/pickaxe`.
- Right-clicking the terminal opens its menu at the next server tick: a player's block hook runs with the
  server out of the game's hands, where `native_open_menu` refuses, so the open is queued and done at the
  tick's start.
- The network walk: the terminal, every cable and connector joined to it up to 512 blocks, and up to
  64 containers beside them. A container is any block whose block entity saves an `Items` list (plus
  the containers that may leave it out while empty, such as a shulker box).
- The list: items summed by their canonical stack (`canon_stack`), so a named sword keeps its own line
  while plain stone from three chests is one line; the total, capped at the stack size, is what the
  menu shows.
- The terminal's menu (six rows), its paging (45 lines a page, previous/next arrows and a page number)
  and the search bar above it, which filters on the server and re-sends the menu.
- Taking (left click a stack, right click one, never more than the room in your inventory) and storing
  (shift-click a slot of your inventory), both on the server, both moving the real stacks through
  Horizon's container helpers. Leftovers go back into the network. A double chest is read half by
  half, and a slot of its left half is mapped to the 27 slots later the game's shared chest view
  numbers it, so a click takes the item it showed.
- A network cut by the block or container limit says so in chat when the terminal opens, instead of
  silently showing a part of it.
- The menu keeps itself up to date: once a second (`server_tick`, every 20 ticks) an open terminal
  re-reads its network and re-sends its menu only when the items changed, so a chest filled by a
  hopper or another player appears on its own. An unchanged network sends nothing.
- One `.hnmod` and one `.hmod` for client and dedicated server (`side = "both"`). The terminal's menu is
  the native half's: without it the terminal is a block that does nothing (a content block gets no block
  entity and no menu). A client with different content is refused when it joins, as any content mod's.
- 16 unit tests of the list, the search, the paging and the network helpers; the mod builds and passes
  its tests on the Horizon commit it pins.

## Missing

- **The crafting terminal**: not ported. A content block copies its template's states, model and shapes
  but not its behaviour: a `crafting_table`-templated block is a plain block, so right click never opens
  the 3x3 grid. A real one needs a mod menu type with a `crafting` grid (`data/<id>/horizon/menu/`), a
  native block entity to hold the grid, and a `native_screen` to draw it on a Horizon client; none of
  that is here. One was added last session and removed again once it was proved inert.
- **Crafting from the network**: the original crafts out of the network. Not ported.
- **Open crate**: tried and dropped. A content block keeps its vanilla template's states and shapes but
  gets no block entity, so a barrel-templated crate registered but held nothing and the network could
  not read it; a real crate needs a native block with its own container, which this port does not have
  yet. Do not add it back as a decorative block.
- **Wireless terminal, filtered connector, inventory hopper/interface, level emitter, trims and paint
  kits**: not ported. Three block types now: storage terminal, connector, cable.
- **Search finds item ids only**: not display names, tooltips, mod names, tags, or an item you have
  never seen. No tabs, no filters, no sort modes.
- **The search box is only on a Horizon client**: it is drawn over the menu by the native half. A stock
  Java client (or a Horizon client without the native mod) sees an ordinary six-row chest menu and can
  still take and store items, but cannot search. Typing there does nothing, as in any chest.
- **No linked-slot view**: the terminal shows *items*, not the chests' slots, so you cannot put an item
  into a specific chest or slot from it; shift-click fills the network in its own order. Horizon's
  `links` menu type does show real linked slots, but such a menu cannot be searched or summed, which is
  what this mod is for.
- **Limits**: 512 network blocks, 64 containers, 64 slots read per container (a double chest's half is
  27), 45 lines a page. A bigger network is cut off, and the terminal says so in chat when it opens.
- **A container's own rules decide what it gives**: a furnace's fuel comes out of its bottom, its
  output only from the front, and a mod container only from its `outputs` faces; the terminal tries
  every side, so what a hopper could not take, it cannot take either.
- **Container entities** (chest minecarts, chest boats) are not reached.
- Not tried in a running client by me: the mod is tested by `cargo test` and on a headless dedicated
  server (see below); the search bar's drawing on a real client was not seen.

## Build

```sh
python tools/gen_textures.py          # the 16x16 textures (original art)
python tools/pack.py                  # target/toms_storage.hmod and target/toms_storage.hnmod
cargo horizon-mod check target/toms_storage.hmod
```

The native half is built by the launcher (`horizon-launcher native-build native/toms_storage
--trust-native-code`), from the `[api]` crate pinned to Horizon commit `a85af177`.

On the VPS used for this port (never on a small PC):

```sh
bash tools/vps/stage.sh   # native-api/toms_storage + crates/native-game in the worktree
bash tools/vps/build.sh   # cargo test, dedicated-server engine, client-flavour check
```

## Licence

MIT, see [LICENSE](LICENSE). The original mod's licence and the clean-room statement are in
[NOTICE.md](NOTICE.md).
