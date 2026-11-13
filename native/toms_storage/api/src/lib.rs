//! Tom's Simple Storage for Horizon: the native (trait mod) half.
//!
//! A clean-room port: the idea (a terminal that shows every item of the chests
//! joined to it, in one searchable list) comes from Tom's Simple Storage by
//! TomTheDeveloper, but no line of its Java code and none of its art is used
//! here. Everything is written for Horizon's mod API; see README.md and
//! NOTICE.md in the mod folder.
//!
//! The launcher links this crate into the game and calls `hooks()`. The same
//! `Hooks` serve a dedicated server (the server hooks fire there) and a client
//! (the screen hooks), so one `.hnmod` is installable on both.
pub mod model;
pub mod world;

mod client;
mod server;

use horizon_client::mod_api::{
    ClientTickPostEvent, Flow, Hooks, Host, MenuClickedEvent, MenuClosedEvent, NetFromClientEvent,
    NetFromServerEvent, RightClickBlockEvent, ScreenCharTypedPreEvent, ScreenClosingEvent,
    ScreenKeyPressedPreEvent, ScreenRenderPostEvent,
};

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Mod)
}

struct Mod;

impl Hooks for Mod {
    /// Right-clicking the terminal opens its menu, in place of anything the
    /// block itself would do.
    fn right_click_block(&self, host: &mut dyn Host, event: &mut RightClickBlockEvent) -> Flow {
        if !world::is_terminal(&host.block(event.pos)) {
            return Flow::Continue;
        }
        let Some(player) = host.acting_player() else {
            return Flow::Continue;
        };
        server::open(host, player.uuid, event.pos)
    }

    /// Every click in that menu: a line takes its item, the arrows turn the
    /// page, a shift-click on the player's inventory stores a stack.
    fn menu_clicked(&self, host: &mut dyn Host, event: &mut MenuClickedEvent) {
        server::click(host, event.uuid, event.slot, event.button, event.mode);
    }

    fn menu_closed(&self, _host: &mut dyn Host, event: &mut MenuClosedEvent) {
        server::closed(event.uuid);
    }

    /// The search box's text, from our own client half.
    fn net_from_client(&self, host: &mut dyn Host, event: &mut NetFromClientEvent<'_>) {
        if event.channel != server::QUERY_CHANNEL {
            return;
        }
        let query = String::from_utf8_lossy(event.data);
        server::search(host, event.player, &query);
    }

    /// The server cleared the box because a terminal opened.
    fn net_from_server(&self, _host: &mut dyn Host, event: &mut NetFromServerEvent<'_>) {
        if event.channel == server::QUERY_CHANNEL && event.data.is_empty() {
            client::reset();
        }
    }

    fn screen_render_post(&self, host: &dyn Host, event: &mut ScreenRenderPostEvent<'_>) {
        client::render(host, event);
    }

    fn screen_char_typed_pre(
        &self,
        _host: &dyn Host,
        event: &mut ScreenCharTypedPreEvent<'_>,
    ) -> Flow {
        client::char_typed(event)
    }

    fn screen_key_pressed_pre(
        &self,
        _host: &dyn Host,
        event: &mut ScreenKeyPressedPreEvent<'_>,
    ) -> Flow {
        client::key_pressed(event)
    }

    fn screen_closing(&self, _host: &dyn Host, event: &mut ScreenClosingEvent<'_>) {
        if client::is_terminal(event.screen) {
            client::closed();
        }
    }

    fn client_tick_post(&self, host: &mut dyn Host, _event: &mut ClientTickPostEvent) {
        client::sync(host);
    }
}
