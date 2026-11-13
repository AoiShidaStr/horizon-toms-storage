//! The client half: a search box over the terminal's menu.
//!
//! The terminal's menu is an ordinary six-row chest menu on the wire, so a
//! stock client shows it as one. This half draws a bar just above that menu
//! while it is open, takes the letters typed there (they would go nowhere in a
//! chest) and sends the text to the server, which filters the list. The bar is
//! only drawn for the title the server gives the terminal, so another chest's
//! menu is left alone.
//!
//! The screen hooks get a `&dyn Host`, which cannot send a message, so typing
//! only changes the box and `tick` (`client_tick_post`, a `&mut dyn Host`)
//! sends what changed.

use std::sync::{Mutex, MutexGuard};

use horizon_client::mod_api::{
    Flow, Host, NativeScreenView, ScreenCharTypedPreEvent, ScreenKeyPressedPreEvent,
    ScreenRenderPostEvent,
};

use crate::model;

/// What the player has typed into the box of the open terminal.
static QUERY: Mutex<String> = Mutex::new(String::new());
/// The text the server last had, so only a change is sent.
static SENT: Mutex<String> = Mutex::new(String::new());

fn query() -> MutexGuard<'static, String> {
    QUERY.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn sent() -> MutexGuard<'static, String> {
    SENT.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Whether `screen` is a terminal's menu: a six-row chest menu with our
/// title. The title is what tells it from any other six-row menu.
pub fn is_terminal(screen: &NativeScreenView<'_>) -> bool {
    screen.menu == Some("minecraft:generic_9x6")
        && screen
            .title
            .is_some_and(|title| title.plain().trim_start().starts_with(model::TITLE))
}

/// Draws the search bar above the open terminal: 18 GUI pixels tall, two
/// above the menu's panel.
pub fn render(host: &dyn Host, event: &mut ScreenRenderPostEvent<'_>) {
    if !is_terminal(event.screen) {
        return;
    }
    let [x, y, w, _h] = event.screen.rect;
    let top = y - 22.0;
    host.fill(event.batch, [x, top, w, 18.0], 0xFF1E_2228);
    host.fill(event.batch, [x, top, w, 1.0], 0xFFB0_7440);
    host.fill(event.batch, [x, top + 17.0, w, 1.0], 0xFFB0_7440);
    let query = query();
    let (text, colour) = if query.is_empty() {
        ("Search items\u{2026}", 0xFF8A_8F98)
    } else {
        (&**query, 0xFFF2_F4F7)
    };
    host.text(event.batch, text, [x + 5.0, top + 5.0], colour, false);
}

/// Sends the box's text once it differs from what the server has.
pub fn sync(host: &mut dyn Host) {
    let query = query();
    {
        let mut sent = sent();
        if *sent == *query {
            return;
        }
        *sent = query.clone();
    }
    let _ = host.net_send_to_server(crate::server::QUERY_CHANNEL, query.as_bytes());
}

/// Takes the letters typed while the terminal is open, so they reach the box
/// and nothing else.
pub fn char_typed(event: &mut ScreenCharTypedPreEvent<'_>) -> Flow {
    if !is_terminal(event.screen) {
        return Flow::Continue;
    }
    let c = event.c;
    if c.is_control() {
        return Flow::Cancel;
    }
    let mut query = query();
    if query.chars().count() < model::QUERY_MAX {
        query.push(c);
    }
    Flow::Cancel
}

/// Backspace removes the last letter; nothing else in the terminal is ours.
pub fn key_pressed(event: &mut ScreenKeyPressedPreEvent<'_>) -> Flow {
    if !is_terminal(event.screen) {
        return Flow::Continue;
    }
    if event.key == "key.keyboard.backspace" {
        query().pop();
        return Flow::Cancel;
    }
    Flow::Continue
}

/// The box as the server wants it (empty when a terminal opens).
pub fn reset() {
    query().clear();
    sent().clear();
}

/// The terminal closed: forget what was typed.
pub fn closed() {
    query().clear();
    sent().clear();
}
