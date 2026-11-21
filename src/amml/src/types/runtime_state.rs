use crate::services::{MapSession, WorldTracker};
use crate::types::LinkState;

// what one enable holds: the link state, and the tracker once the session runs
pub struct RuntimeState {
    pub link: LinkState,
    pub tracker: Option<WorldTracker<MapSession>>,
}

impl RuntimeState {
    pub fn new(link: LinkState) -> Self {
        Self { link, tracker: None }
    }
}
