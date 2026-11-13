//! The terminal's list, worked out without the game: what the linked
//! inventories hold, how a search narrows it, and what fills the menu's slots.
//!
//! Everything here is pure so it can be unit-tested (`cargo test`), which is
//! most of the mod's behaviour: the game side only reads containers, feeds
//! these functions and moves the stacks they pick.

use horizon_client::mod_api::ItemStack;
use horizon_client::item_data::{canon_stack, parse_stack};

/// The terminal's menu: six rows, as `native_open_menu` takes them.
pub const ROWS: u8 = 6;
/// Slots of that menu, its own 54; the player's 36 come after them.
pub const SLOTS: usize = 54;
/// Slots one page lists, the rest of the page being its buttons. 45 rows of
/// items leave the last row clear, as Tom's Simple Storage lays it out.
pub const PAGE: usize = 45;
pub const PREV_SLOT: usize = 45;
pub const INFO_SLOT: usize = 46;
pub const NEXT_SLOT: usize = 47;
/// The title the server gives the menu; the client matches it to know the
/// search box belongs to a terminal and not to another chest.
pub const TITLE: &str = "Storage Terminal";
/// The most characters a search may hold.
pub const QUERY_MAX: usize = 32;

/// One line of the terminal: an item, how many of it the network holds, and
/// the stack the menu shows for it (the first one seen, so components and
/// damage are the real ones).
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub key: String,
    pub stack: ItemStack,
    pub total: u32,
}

impl Entry {
    /// The item's id, `minecraft:stone`.
    pub fn item(&self) -> &str {
        &self.stack.item
    }
}

/// The line a stack belongs to: its canonical form (`canon_stack`) with the
/// count set to one. `canon_stack` is `ItemStack.CODEC`, which carries the
/// count, so without this the same item in two chests would be two lines
/// whenever its counts differ.
pub fn key(stack: &ItemStack) -> String {
    let mut one = stack.clone();
    one.count = 1;
    canon_stack(Some(&one))
}

/// Sums what the containers hold by item, in the order the containers and
/// their slots come: the terminal lists a network the way it is filled.
///
/// Two stacks are one line when [`key`] matches, so a named or enchanted item
/// keeps its own line as in vanilla's inventory sorting while the same plain
/// item in three chests is one line.
pub fn aggregate(containers: &[Vec<Option<ItemStack>>]) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for slots in containers {
        for stack in slots.iter().flatten() {
            if stack.count == 0 {
                continue;
            }
            let line = key(stack);
            match seen.get(&line) {
                Some(&i) => out[i].total = out[i].total.saturating_add(stack.count),
                None => {
                    seen.insert(line.clone(), out.len());
                    out.push(Entry {
                        key: line,
                        stack: stack.clone(),
                        total: stack.count,
                    });
                }
            }
        }
    }
    out
}

/// The entries a search keeps: every whitespace-separated word must appear in
/// the item's id, in any case, so `oak plank` finds `minecraft:oak_planks`
/// and `stone` finds both `minecraft:stone` and `minecraft:stone_bricks`.
/// An empty search keeps everything.
pub fn filter(entries: &[Entry], query: &str) -> Vec<usize> {
    let words: Vec<String> = query
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .filter(|w| !w.is_empty())
        .collect();
    entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            let id = e.item().to_lowercase();
            words.iter().all(|w| id.contains(w.as_str()))
        })
        .map(|(i, _)| i)
        .collect()
}

/// How many pages `len` matches need; one even when there is none, so the
/// terminal always has a page to show.
pub fn pages(len: usize) -> usize {
    len.div_ceil(PAGE).max(1)
}

/// `page`, kept inside the pages that exist.
pub fn clamp_page(page: usize, len: usize) -> usize {
    page.min(pages(len) - 1)
}

/// The entries page `page` lists, by index into `matched`.
pub fn page_slice(matched: &[usize], page: usize) -> &[usize] {
    let start = clamp_page(page, matched.len()) * PAGE;
    let end = (start + PAGE).min(matched.len());
    &matched[start..end]
}

/// The menu's 54 slots for `page`: the page's items, then the buttons.
///
/// A shown stack keeps its item and components and takes `min(total, stack
/// size)` for its count, as a menu cannot show more than one stack's worth;
/// the true total is what a click takes from.
pub fn menu_items(
    entries: &[Entry],
    matched: &[usize],
    page: usize,
    max_stack: impl Fn(&str) -> u32,
) -> Vec<Option<ItemStack>> {
    let mut items = vec![None; SLOTS];
    for (slot, &index) in page_slice(matched, page).iter().enumerate() {
        let entry = &entries[index];
        let mut stack = entry.stack.clone();
        stack.count = entry
            .total
            .min(max_stack(entry.item()).max(1))
            .max(1);
        items[slot] = Some(stack);
    }
    let last = pages(matched.len()) - 1;
    if page > 0 {
        items[PREV_SLOT] = button("minecraft:arrow");
    }
    items[INFO_SLOT] = {
        let mut stack = button("minecraft:paper").expect("paper is a vanilla item");
        stack.count = (page + 1).min(99) as u32;
        Some(stack)
    };
    if page < last {
        items[NEXT_SLOT] = button("minecraft:spectral_arrow");
    }
    items
}

/// A button's one-item stack, from vanilla's item text so no `ItemStack`
/// field has to be spelled here.
fn button(item: &str) -> Option<ItemStack> {
    parse_stack(&format!("{{id:\"{item}\",count:1}}"))
        .ok()
        .flatten()
}

/// The player-inventory slot a menu slot of our six-row menu is, as
/// `native_player_stacks` numbers them (0..9 the hotbar): the menu's own 54
/// slots come first, then the main inventory's 27, then the hotbar's 9.
pub fn inventory_slot(menu_slot: i32) -> Option<usize> {
    let i = usize::try_from(menu_slot).ok()?;
    if !(SLOTS..SLOTS + 36).contains(&i) {
        return None;
    }
    let i = i - SLOTS;
    Some(if i < 27 { i + 9 } else { i - 27 })
}

/// What one click takes: a right click one item, a left click a stack.
pub fn click_amount(button: i32, max_stack: u32, room: u32) -> u32 {
    let want = if button == 1 { 1 } else { max_stack.max(1) };
    want.min(room)
}

/// A search line, cleaned: trimmed, at most [`QUERY_MAX`] characters, lower
/// case kept as typed.
pub fn clean_query(query: &str) -> String {
    query.trim().chars().take(QUERY_MAX).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: &str, count: u32) -> ItemStack {
        let mut s = parse_stack(&format!("{{id:\"{item}\",count:1}}"))
            .expect("parses")
            .expect("not air");
        s.count = count;
        s
    }

    fn container(items: &[(&str, u32)]) -> Vec<Option<ItemStack>> {
        items
            .iter()
            .map(|(item, count)| Some(stack(item, *count)))
            .collect()
    }

    #[test]
    fn aggregate_sums_the_same_item_across_containers_and_slots() {
        let cs = vec![
            container(&[("minecraft:stone", 64), ("minecraft:dirt", 3)]),
            container(&[("minecraft:stone", 12)]),
        ];
        let entries = aggregate(&cs);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].item(), "minecraft:stone");
        assert_eq!(entries[0].total, 76);
        assert_eq!(entries[1].item(), "minecraft:dirt");
        assert_eq!(entries[1].total, 3);
    }

    #[test]
    fn the_line_key_ignores_the_count_but_not_the_components() {
        let plain = stack("minecraft:stone", 64);
        let fewer = stack("minecraft:stone", 3);
        assert_eq!(key(&plain), key(&fewer));
        let named = parse_stack(
            "{id:\"minecraft:stone\",count:64,components:{\"minecraft:custom_name\":\"Rock\"}}",
        )
        .expect("parses")
        .expect("not air");
        assert_ne!(key(&plain), key(&named));
    }

    #[test]
    fn aggregate_keeps_named_and_enchanted_stacks_apart() {
        let plain = stack("minecraft:diamond_sword", 1);
        let named = parse_stack(
            "{id:\"minecraft:diamond_sword\",count:1,components:{\"minecraft:custom_name\":\"Sharp\"}}",
        )
        .expect("parses")
        .expect("not air");
        let entries = aggregate(&[vec![Some(plain), Some(named), None]]);
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.total == 1));
    }

    #[test]
    fn aggregate_ignores_empty_slots_and_negative_counts() {
        let entries = aggregate(&[vec![None, Some(stack("minecraft:stone", 0))]]);
        assert!(entries.is_empty());
    }

    #[test]
    fn filter_matches_words_of_the_id_in_any_case() {
        let entries = aggregate(&[
            container(&[("minecraft:oak_planks", 4), ("minecraft:stone", 4)]),
            container(&[("minecraft:stone_bricks", 4), ("minecraft:stick", 8)]),
        ]);
        let st = filter(&entries, "STONE");
        assert_eq!(st.len(), 2);
        let oak = filter(&entries, " oak plank ");
        assert_eq!(oak.len(), 1);
        assert_eq!(entries[oak[0]].item(), "minecraft:oak_planks");
        assert_eq!(filter(&entries, "nothing here").len(), 0);
        assert_eq!(filter(&entries, "").len(), entries.len());
    }

    #[test]
    fn pages_and_clamping_follow_the_page_size() {
        assert_eq!(pages(0), 1);
        assert_eq!(pages(1), 1);
        assert_eq!(pages(PAGE), 1);
        assert_eq!(pages(PAGE + 1), 2);
        assert_eq!(clamp_page(7, 10), 0);
        assert_eq!(clamp_page(1, PAGE + 1), 1);
    }

    #[test]
    fn page_slice_never_runs_past_the_matches() {
        let matched: Vec<usize> = (0..PAGE + 2).collect();
        assert_eq!(page_slice(&matched, 0), &matched[..PAGE]);
        assert_eq!(page_slice(&matched, 1), &matched[PAGE..]);
        assert_eq!(page_slice(&matched, 9), &matched[PAGE..]);
        assert!(page_slice(&[], 0).is_empty());
    }

    #[test]
    fn menu_items_fill_a_page_then_its_buttons() {
        let cs = vec![container(
            &(0..PAGE + 3)
                .map(|i| match i {
                    0 => ("minecraft:stone", 100u32),
                    _ => ("minecraft:dirt", 1),
                })
                .collect::<Vec<_>>(),
        )];
        let entries = aggregate(&cs);
        // The 100 stone are one line; the rest are dirt, all on one line too.
        assert_eq!(entries.len(), 2);
        let matched = filter(&entries, "");
        let items = menu_items(&entries, &matched, 0, |_| 64);
        assert!(items[0].as_ref().is_some_and(|s| s.count == 64));
        assert!(items[PREV_SLOT].is_none(), "one page: no previous");
        assert!(items[NEXT_SLOT].is_none(), "one page: no next");
        assert_eq!(items[INFO_SLOT].as_ref().map(|s| s.count), Some(1));
    }

    #[test]
    fn menu_items_page_two_shows_both_arrows_and_the_page_number() {
        let mut cs = Vec::new();
        let names: Vec<String> = (0..PAGE + 5).map(|i| format!("test:item_{i}")).collect();
        for name in &names {
            cs.push(container(&[(name.as_str(), 1)]));
        }
        let entries = aggregate(&cs);
        assert_eq!(entries.len(), PAGE + 5);
        let matched = filter(&entries, "");
        let items = menu_items(&entries, &matched, 1, |_| 64);
        assert_eq!(items[0].as_ref().map(|s| &*s.item), Some("test:item_45"));
        assert!(items[PREV_SLOT].is_some());
        assert!(items[NEXT_SLOT].is_none());
        assert_eq!(items[INFO_SLOT].as_ref().map(|s| s.count), Some(2));
        assert_eq!(page_slice(&matched, 1).len(), 5);
    }

    #[test]
    fn inventory_slots_map_the_menu_rows_to_the_player() {
        assert_eq!(inventory_slot(54), Some(9));
        assert_eq!(inventory_slot(80), Some(35));
        assert_eq!(inventory_slot(81), Some(0));
        assert_eq!(inventory_slot(89), Some(8));
        assert_eq!(inventory_slot(53), None);
        assert_eq!(inventory_slot(90), None);
        assert_eq!(inventory_slot(-999), None);
    }

    #[test]
    fn a_click_takes_a_stack_or_one_and_never_more_than_the_room() {
        assert_eq!(click_amount(0, 64, 1000), 64);
        assert_eq!(click_amount(1, 64, 1000), 1);
        assert_eq!(click_amount(0, 64, 5), 5);
        assert_eq!(click_amount(0, 64, 0), 0);
    }

    #[test]
    fn clean_query_trims_and_limits() {
        assert_eq!(clean_query("  oak  "), "oak");
        assert_eq!(clean_query(&"x".repeat(100)).len(), QUERY_MAX);
    }
}
