use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use amm_client::link::{LinkState as ManagerState, ManagerLink};
use amml_api::amml_abi::KeyInput;
use amml_api::{Client, Ctx, Logger};
use minimap_link::services::{KeyFeed, MinimapFrameHandler};
use minimap_link::structs::Descriptor;

use crate::services::{KeyListener, MapSession, WorldIdentityReader, WorldTracker};
use crate::types::{LinkState, QueuedColumn, RuntimeState};

// what the listeners share: the mod's context, its data folder and the state of the current enable. Every listener
// and disable run on the game thread
pub struct MinimapRuntime {
    ctx: Ctx,
    data_dir: PathBuf,
    log: Logger,
    // the world map's Escape and Ctrl+Z; outside the state lock, so a key listener on the input thread never waits
    keys: Arc<KeyFeed>,
    state: Mutex<RuntimeState>,
}

impl MinimapRuntime {
    // starts the manager link with the minimap's frame handler; the session starts at the first tick that finds the
    // link connected. The caches of worlds with no seed from game processes that have ended are removed first
    pub fn start(ctx: &Ctx, data_dir: &Path) -> Result<Arc<Self>, minimap_link::errors::SessionError> {
        WorldIdentityReader::sweep_unseeded(data_dir);
        let slot = Arc::new(OnceLock::new());
        let keys = Arc::new(KeyFeed::new());
        let handler = MinimapFrameHandler::new(Arc::clone(&slot), Arc::clone(&keys));
        let link = ManagerLink::start(ctx, Descriptor::TEXT, Some(handler))?;
        let state = RuntimeState::new(LinkState::Waiting { link, slot, keys: Arc::clone(&keys) });
        let data_dir = data_dir.to_path_buf();
        Ok(Arc::new(Self { ctx: *ctx, data_dir, log: ctx.log(), keys, state: Mutex::new(state) }))
    }

    // once per client tick: the session starts when the manager is there, then the tracker ticks
    pub fn tick(&self, client: &Client<'_>) {
        let Ok(mut state) = self.state.lock() else { return };
        if matches!(state.link, LinkState::Waiting { .. }) {
            self.attach(&mut state);
        }
        if let Some(tracker) = state.tracker.as_mut() {
            tracker.tick(client);
        }
    }

    // a real key: Escape and Ctrl+Z go to the world map while it is open
    pub fn key(&self, event: &KeyInput) {
        KeyListener::on_key(&self.keys, event, self.ctx.input().modifiers().f3());
    }

    pub fn world_left(&self) {
        let Ok(mut state) = self.state.lock() else { return };
        if let Some(tracker) = state.tracker.as_mut() {
            tracker.leave();
        }
    }

    // a block change can come while the state is busy (from inside a game call of the tick); it is then dropped and
    // the scan plan reads the chunk again later
    pub fn block_changed(&self, column: QueuedColumn) {
        let Ok(mut state) = self.state.try_lock() else { return };
        if let Some(tracker) = state.tracker.as_mut() {
            tracker.block_changed(column);
        }
    }

    // disable, after the listeners are gone: the cursor is given back, the session gives its leases back and
    // unregisters, and the runtime's threads stop
    pub fn stop(&self) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(mut tracker) = state.tracker.take() {
            tracker.reset();
            tracker.into_sink().shutdown();
        }
        state.link = LinkState::Off;
        self.keys.set_armed(false);
    }

    fn attach(&self, state: &mut RuntimeState) {
        let LinkState::Waiting { link, .. } = &state.link else { return };
        match link.state() {
            ManagerState::Waiting => return,
            ManagerState::Gone => {
                self.log.error("minimap off: the mod manager is gone", &[]);
                state.link = LinkState::Off;
                return;
            }
            ManagerState::Connected => {}
        }
        let LinkState::Waiting { link, slot, keys } = std::mem::replace(&mut state.link, LinkState::Off) else {
            return;
        };
        match MapSession::open(link, &slot, keys, &self.data_dir) {
            Ok(session) => {
                state.tracker = Some(WorldTracker::new(&self.ctx, session, &self.data_dir));
                state.link = LinkState::Attached;
                self.log.info("minimap linked", &[]);
            }
            Err(error) => self.log.error("minimap off", &[("error", &error.to_string())]),
        }
    }
}
