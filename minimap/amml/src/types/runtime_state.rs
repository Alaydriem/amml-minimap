use amml_game::GameEntries;
use amml_sdk::HookHandle;

use crate::services::{MapSession, WorldTracker};
use crate::types::LinkState;

// what one enable holds: the link state, the entries until the session exists, the tracker once it does, and the
// installed hooks (at most MinimapRuntime::HOOK_CAPACITY)
pub struct RuntimeState {
    pub link: LinkState,
    pub pending: Option<GameEntries>,
    pub tracker: Option<WorldTracker<MapSession>>,
    pub hooks: Vec<HookHandle>,
}

impl RuntimeState {
    pub fn new(hook_capacity: usize) -> Self {
        Self { link: LinkState::Off, pending: None, tracker: None, hooks: Vec::with_capacity(hook_capacity) }
    }
}
