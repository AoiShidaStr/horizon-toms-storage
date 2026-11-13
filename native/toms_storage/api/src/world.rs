//! The network around a terminal and the stacks in it, through the game's
//! helpers (`native_block_entity_tag`, `native_extract_item`,
//! `native_insert_item`).
//!
//! A network is the terminal plus every cable and connector joined to it
//! through `#toms_storage:cable` (the block tag the content mod declares), and
//! every block beside one of those that is a container. Reading a container is
//! reading its block entity's saved items, which is the same NBT a chest saves
//! for vanilla, so the terminal sees chests, barrels, hoppers, dispensers,
//! droppers, shulker boxes, furnaces and a mod's own containers alike.

use std::collections::{HashSet, VecDeque};

use horizon_client::mod_api::{Direction, Host, ItemStack};
use horizon_client::save::decode_items;

use crate::model;

/// The id the terminal block has in the content half.
pub const TERMINAL: &str = "toms_storage:storage_terminal";
/// The blocks a network walks through.
pub const CABLE_BLOCKS: [&str; 3] = [
    "toms_storage:storage_terminal",
    "toms_storage:inventory_connector",
    "toms_storage:inventory_cable",
];
/// The most network blocks one terminal walks: a big build, not a whole world.
const MAX_BLOCKS: usize = 512;
/// The most containers one terminal shows.
const MAX_CONTAINERS: usize = 64;
/// The most slots read from one container (a double chest's half is 27).
const MAX_SLOTS: usize = 64;
/// The sides a container is reached through, in the order tried. Up is a
/// hopper's input and a chest's whole inventory.
const FACES: [Direction; 6] = [
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
    Direction::Down,
];

pub fn is_cable(block: &str) -> bool {
    CABLE_BLOCKS.contains(&block)
}

pub fn is_terminal(block: &str) -> bool {
    block == TERMINAL
}

/// Container blocks whose block entity may save no `Items` while empty (a
/// shulker box does not), so a link to one is kept and simply shows nothing.
fn is_known_container(block: &str) -> bool {
    let path = block.strip_prefix("minecraft:").unwrap_or(block);
    matches!(
        path,
        "chest"
            | "trapped_chest"
            | "barrel"
            | "hopper"
            | "dispenser"
            | "dropper"
            | "brewing_stand"
            | "crafter"
            | "furnace"
            | "blast_furnace"
            | "smoker"
    ) || path.ends_with("shulker_box")
}

/// The slot a container's own saved tag numbers, in the container the game
/// sees. A double chest is one container whose view numbers the right half's
/// 27 slots first, so a slot read from the left half's tag sits 27 later; a
/// single chest or any other container keeps its slots as they are.
fn view_slot(half: Option<&str>, slot: usize) -> usize {
    if half == Some("left") {
        slot + 27
    } else {
        slot
    }
}

fn offset(pos: [i32; 3], face: Direction) -> [i32; 3] {
    let (dx, dy, dz) = match face {
        Direction::Down => (0, -1, 0),
        Direction::Up => (0, 1, 0),
        Direction::North => (0, 0, -1),
        Direction::South => (0, 0, 1),
        Direction::West => (-1, 0, 0),
        Direction::East => (1, 0, 0),
    };
    [pos[0] + dx, pos[1] + dy, pos[2] + dz]
}

/// Whether the block at `pos` holds items: its block entity saves an `Items`
/// list, or it is one of the containers that may leave that list out.
pub fn is_container(host: &mut dyn Host, dimension: &str, pos: [i32; 3]) -> bool {
    if read_slots(host, dimension, pos).is_some() {
        return true;
    }
    is_known_container(&host.block(pos))
}

/// The stacks of the container at `pos`, slot by slot, or `None` where the
/// block holds no items at all.
pub fn read_slots(
    host: &mut dyn Host,
    dimension: &str,
    pos: [i32; 3],
) -> Option<Vec<Option<ItemStack>>> {
    let tag = host.block_entity_tag(dimension, pos)?;
    let items = tag.get("Items")?;
    Some(decode_items(Some(items), MAX_SLOTS))
}

/// What a terminal's walk found: the containers, in the order the walk finds
/// them, and whether a limit stopped it from finding more (so the player can be
/// told instead of silently seeing a partial network).
pub struct Scan {
    pub containers: Vec<[i32; 3]>,
    pub truncated: bool,
}

/// Every container the terminal at `origin` links to, in the order the walk
/// finds them.
pub fn scan(host: &mut dyn Host, dimension: &str, origin: [i32; 3]) -> Scan {
    let mut containers: Vec<[i32; 3]> = Vec::new();
    let mut truncated = false;
    let mut seen_blocks: HashSet<[i32; 3]> = HashSet::new();
    let mut seen_containers: HashSet<[i32; 3]> = HashSet::new();
    let mut queue: VecDeque<[i32; 3]> = VecDeque::new();
    queue.push_back(origin);
    seen_blocks.insert(origin);
    while let Some(pos) = queue.pop_front() {
        for face in FACES {
            let next = offset(pos, face);
            if seen_blocks.contains(&next) || seen_containers.contains(&next) {
                continue;
            }
            let block = host.block(next);
            if is_cable(&block) {
                if seen_blocks.len() < MAX_BLOCKS {
                    seen_blocks.insert(next);
                    queue.push_back(next);
                } else {
                    truncated = true;
                }
                continue;
            }
            if block == "minecraft:air" || block == "minecraft:cave_air" || block == "minecraft:void_air" {
                continue;
            }
            if is_container(host, dimension, next) {
                if containers.len() < MAX_CONTAINERS {
                    seen_containers.insert(next);
                    containers.push(next);
                } else {
                    truncated = true;
                }
            }
        }
    }
    Scan {
        containers,
        truncated,
    }
}

/// What the containers hold, one list of slots each.
pub fn read_all(
    host: &mut dyn Host,
    dimension: &str,
    containers: &[[i32; 3]],
) -> Vec<Vec<Option<ItemStack>>> {
    containers
        .iter()
        .map(|&pos| read_slots(host, dimension, pos).unwrap_or_default())
        .collect()
}

/// Takes up to `want` items whose canonical form is `key` out of the network,
/// by the rules a hopper follows: the first container that gives wins, and
/// only what a side lets go of is taken.
pub fn take(
    host: &mut dyn Host,
    dimension: &str,
    containers: &[[i32; 3]],
    key: &str,
    want: u32,
) -> Vec<ItemStack> {
    let mut out: Vec<ItemStack> = Vec::new();
    let mut left = want;
    'containers: for &pos in containers {
        let Some(slots) = read_slots(host, dimension, pos) else {
            continue;
        };
        for (slot, stack) in slots.iter().enumerate() {
            if left == 0 {
                break 'containers;
            }
            let Some(stack) = stack else { continue };
            if model::key(stack) != key {
                continue;
            }
            let slot = view_slot(host.block_property(pos, "type").as_deref(), slot);
            for face in FACES {
                let Some(got) = host.extract_item(dimension, pos, face, slot, left) else {
                    continue;
                };
                left = left.saturating_sub(got.count);
                out.push(got);
                break;
            }
        }
    }
    out
}

/// Puts `stack` into the network, as a hopper would; what did not fit.
pub fn store(
    host: &mut dyn Host,
    containers: &[[i32; 3]],
    stack: ItemStack,
) -> Option<ItemStack> {
    let mut rest = stack;
    for &pos in containers {
        for face in FACES {
            match host.insert_item(pos, face, rest) {
                None => return None,
                Some(left) => rest = left,
            }
            if rest.count == 0 {
                return None;
            }
        }
    }
    Some(rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_follow_the_faces() {
        assert_eq!(offset([0, 0, 0], Direction::Up), [0, 1, 0]);
        assert_eq!(offset([0, 0, 0], Direction::Down), [0, -1, 0]);
        assert_eq!(offset([0, 0, 0], Direction::North), [0, 0, -1]);
        assert_eq!(offset([0, 0, 0], Direction::South), [0, 0, 1]);
        assert_eq!(offset([0, 0, 0], Direction::West), [-1, 0, 0]);
        assert_eq!(offset([0, 0, 0], Direction::East), [1, 0, 0]);
    }

    #[test]
    fn the_cable_tag_blocks_are_the_ones_walked() {
        assert!(is_cable("toms_storage:inventory_cable"));
        assert!(is_cable("toms_storage:inventory_connector"));
        assert!(is_cable("toms_storage:storage_terminal"));
        assert!(!is_cable("minecraft:chest"));
        assert!(is_terminal("toms_storage:storage_terminal"));
        assert!(!is_terminal("toms_storage:inventory_cable"));
    }

    #[test]
    fn a_double_chest_left_half_is_27_slots_later_in_the_view() {
        assert_eq!(view_slot(None, 3), 3);
        assert_eq!(view_slot(Some("single"), 3), 3);
        assert_eq!(view_slot(Some("right"), 3), 3);
        assert_eq!(view_slot(Some("left"), 3), 30);
    }

    #[test]
    fn known_containers_include_modded_shulker_boxes() {
        assert!(is_known_container("minecraft:chest"));
        assert!(is_known_container("minecraft:barrel"));
        assert!(is_known_container("minecraft:white_shulker_box"));
        assert!(is_known_container("othermod:big_shulker_box"));
        assert!(!is_known_container("minecraft:stone"));
        assert!(!is_known_container("toms_storage:inventory_cable"));
    }
}
