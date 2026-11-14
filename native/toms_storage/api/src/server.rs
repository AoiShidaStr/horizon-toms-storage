//! The server half of the terminal: what the menu shows, what a click takes or
//! stores, and what a search keeps.
//!
//! Every stack stays where it was. The menu is a view of the network
//! (`native_open_menu`): a click on a line takes the real items out of the
//! linked containers (`native_extract_item`) and hands them to the player, and
//! a shift-click on the player's inventory takes that stack and puts it in the
//! network (`native_insert_item`), so nothing is duplicated and nothing is
//! lost if the menu closes.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use horizon_client::mod_api::{Flow, Host, ItemStack, Text};

use crate::model::{self, Entry};
use crate::world;

/// The channel the client's search box sends its text on, and the one the
/// server clears it with when a terminal opens.
pub const QUERY_CHANNEL: &str = "toms_storage:query";

/// How often an open terminal re-reads its network, in server ticks: once a
/// second, enough for a hopper or another player to show up without a click
/// while keeping the walk cheap on a busy server.
const REFRESH_TICKS: u64 = 20;

/// One player's open terminal: the network it found, what it holds, and the
/// search and page showing.
struct Session {
    dimension: String,
    query: String,
    page: usize,
    containers: Vec<[i32; 3]>,
    entries: Vec<Entry>,
    matched: Vec<usize>,
}

static SESSIONS: Mutex<Option<HashMap<u128, Session>>> = Mutex::new(None);

fn sessions() -> MutexGuard<'static, Option<HashMap<u128, Session>>> {
    SESSIONS.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Takes a player's session out of the map, so the game is called without the
/// lock held (a hook may run again inside a helper).
fn take(uuid: u128) -> Option<Session> {
    sessions().as_mut().and_then(|map| map.remove(&uuid))
}

fn put(uuid: u128, session: Session) {
    sessions()
        .get_or_insert_with(HashMap::new)
        .insert(uuid, session);
}

fn max_stack(host: &dyn Host, item: &str) -> u32 {
    host.max_stack_size(item).unwrap_or(64).max(1)
}

/// A line in the server log when `TOMS_TRACE` is set, for a headless test and
/// for a server owner wondering why a terminal sees what it sees.
fn trace(line: &str) {
    if std::env::var_os("TOMS_TRACE").is_some() {
        eprintln!("toms_storage: {line}");
    }
}

/// Re-reads the network and applies the search again.
fn rebuild(host: &mut dyn Host, session: &mut Session) {
    session.entries = model::aggregate(&world::read_all(
        host,
        &session.dimension,
        &session.containers,
    ));
    session.matched = model::filter(&session.entries, &session.query);
    session.page = model::clamp_page(session.page, session.matched.len());
}

/// Sends the menu's 54 slots as the session says.
fn render(host: &mut dyn Host, uuid: u128, session: &Session) {
    let items = {
        let host: &dyn Host = host;
        model::menu_items(&session.entries, &session.matched, session.page, |item| {
            max_stack(host, item)
        })
    };
    host.set_menu_items(uuid, items);
}

/// Opens the terminal at `pos` for `uuid`: its network, its items, its menu.
pub fn open(host: &mut dyn Host, uuid: u128, pos: [i32; 3]) -> Flow {
    let dimension = host.dimension().to_string();
    let scan = world::scan(host, &dimension, pos);
    let containers = scan.containers;
    trace(&format!(
        "open at {:?} in {dimension}: {} container(s){}",
        pos,
        containers.len(),
        if scan.truncated { ", cut by a limit" } else { "" }
    ));
    if scan.truncated {
        host.tell(
            uuid,
            Text::translate_with_fallback(
                "toms_storage.terminal.too_big",
                "Storage Terminal: network too big, some containers are not shown (max 512 blocks, 64 containers)",
                &[],
            ),
        );
    }
    if containers.is_empty() {
        host.tell(
            uuid,
            Text::translate_with_fallback(
                "toms_storage.terminal.no_link",
                "Storage Terminal: no inventory linked",
                &[],
            ),
        );
        return Flow::Cancel;
    }
    let entries = model::aggregate(&world::read_all(host, &dimension, &containers));
    trace(&format!("open: {} line(s) of items", entries.len()));
    let matched = model::filter(&entries, "");
    let items = {
        let host: &dyn Host = host;
        model::menu_items(&entries, &matched, 0, |item| max_stack(host, item))
    };
    if host
        .open_menu(uuid, model::ROWS, Text::literal(model::TITLE), items)
        .is_none()
    {
        return Flow::Cancel;
    }
    // A fresh terminal starts with an empty search box.
    let _ = host.net_send_to_player(uuid, QUERY_CHANNEL, b"");
    put(
        uuid,
        Session {
            dimension,
            query: String::new(),
            page: 0,
            containers,
            entries,
            matched,
        },
    );
    Flow::Cancel
}

/// A click in the open terminal: a line takes its item, a button turns the
/// page, a shift-click on the player's inventory stores that stack.
pub fn click(host: &mut dyn Host, uuid: u128, slot: i32, button: i32, mode: i32) {
    let Some(mut session) = take(uuid) else {
        return;
    };
    trace(&format!(
        "click slot {slot} button {button} mode {mode} (page {} of {})",
        session.page + 1,
        model::pages(session.matched.len())
    ));
    if let Some(page_slot) = usize::try_from(slot).ok().filter(|s| *s < model::PAGE) {
        take_line(host, uuid, &session, page_slot, button);
    } else if slot == model::PREV_SLOT as i32 {
        if session.page > 0 {
            session.page -= 1;
        }
    } else if slot == model::NEXT_SLOT as i32 {
        session.page = model::clamp_page(session.page + 1, session.matched.len());
    } else if mode == 1 {
        // A shift-click on the player's inventory puts the stack in the
        // network; nothing else in the player's inventory is the terminal's.
        if let Some(inv) = model::inventory_slot(slot) {
            store_inventory_slot(host, uuid, &session, inv);
        }
    }
    rebuild(host, &mut session);
    render(host, uuid, &session);
    put(uuid, session);
}

/// Takes one line's worth into the player's inventory.
fn take_line(host: &mut dyn Host, uuid: u128, session: &Session, page_slot: usize, button: i32) {
    let Some(&index) = model::page_slice(&session.matched, session.page).get(page_slot) else {
        return;
    };
    let entry = session.entries[index].clone();
    let room = host.room_for(entry.item());
    let want = model::click_amount(button, max_stack(host, entry.item()), room);
    trace(&format!(
        "take {} x{want} (total {}, room {room})",
        entry.item(),
        entry.total
    ));
    if want == 0 {
        return;
    }
    for stack in world::take(host, &session.dimension, &session.containers, &entry.key, want) {
        if let Some(rest) = host.give_stack(uuid, stack) {
            // The player filled up between the count and the take: the rest
            // goes back into the network rather than into the void.
            back_into_network(host, uuid, &session.containers, rest);
        }
    }
}

/// Moves one stack out of a player's slot and into the network.
fn store_inventory_slot(host: &mut dyn Host, uuid: u128, session: &Session, slot: usize) {
    let Some(stacks) = host.player_stacks(uuid) else {
        return;
    };
    let Some(Some(stack)) = stacks.get(slot).cloned() else {
        return;
    };
    let Ok(taken) = host.take_stacks(uuid, &[(slot, stack)]) else {
        return;
    };
    for stack in taken {
        if let Some(rest) = world::store(host, &session.containers, stack) {
            back_into_network(host, uuid, &session.containers, rest);
        }
    }
}

/// Puts a stack back in the network, then in the player's inventory through
/// `give`, so it is never destroyed.
fn back_into_network(
    host: &mut dyn Host,
    uuid: u128,
    containers: &[[i32; 3]],
    stack: ItemStack,
) {
    let Some(rest) = world::store(host, containers, stack) else {
        return;
    };
    if rest.count == 0 {
        return;
    }
    let name = host
        .server_players()
        .into_iter()
        .find(|p| p.uuid == uuid)
        .map(|p| p.name);
    let line = format!(
        "give {} {} {}",
        name.as_deref().unwrap_or(""),
        rest.item,
        rest.count
    );
    if name.is_none() || !host.run_command(&line) {
        eprintln!(
            "toms_storage: {} x{} could not be given back and stays in the terminal's network",
            rest.item, rest.count
        );
    }
}

/// A search from the client's box: filters the list and shows the first page.
pub fn search(host: &mut dyn Host, uuid: u128, query: &str) {
    let Some(mut session) = take(uuid) else {
        return;
    };
    session.query = model::clean_query(query);
    trace(&format!(
        "search {query:?}: {} line(s)",
        model::filter(&session.entries, &session.query).len()
    ));
    session.page = 0;
    rebuild(host, &mut session);
    render(host, uuid, &session);
    put(uuid, session);
}

/// The server tick: once a second, every open terminal re-reads its network and
/// re-sends its menu only when what it holds changed. A chest filled by a
/// hopper or another player then appears on its own, and a menu nobody touched
/// costs nothing on the wire until its items differ.
pub fn tick(host: &mut dyn Host, tick: u64, world_running: bool) {
    if !world_running || tick % REFRESH_TICKS != 0 {
        return;
    }
    let uuids: Vec<u128> = match sessions().as_ref() {
        Some(map) => map.keys().copied().collect(),
        None => return,
    };
    for uuid in uuids {
        let Some(mut session) = take(uuid) else {
            continue;
        };
        let entries = model::aggregate(&world::read_all(
            host,
            &session.dimension,
            &session.containers,
        ));
        if entries != session.entries {
            trace(&format!("refresh: {} line(s) of items", entries.len()));
            session.entries = entries;
            session.matched = model::filter(&session.entries, &session.query);
            session.page = model::clamp_page(session.page, session.matched.len());
            render(host, uuid, &session);
        }
        put(uuid, session);
    }
}

/// The menu closed: forget the session, so a later click is nobody's.
pub fn closed(uuid: u128) {
    if let Some(map) = sessions().as_mut() {
        map.remove(&uuid);
    }
}
