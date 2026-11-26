use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::services::{FolderTurns, TurnLatch};

// one writer's turn on a cache folder; dropping it ends the turn, also during a panic
#[derive(Debug)]
pub struct FolderTurn {
    turns: Arc<FolderTurns>,
    root: PathBuf,
    before: Option<TurnLatch>,
    done: TurnLatch,
}

impl FolderTurn {
    pub(crate) fn new(turns: Arc<FolderTurns>, root: PathBuf, before: Option<TurnLatch>, done: TurnLatch) -> Self {
        Self { turns, root, before, done }
    }

    // true once the turn before this one has ended (or there was none)
    pub fn is_due(&self) -> bool {
        self.before.as_ref().is_none_or(TurnLatch::is_open)
    }

    // true when the turn before this one ended within `timeout`
    pub fn wait_due(&self, timeout: Duration) -> bool {
        self.before.as_ref().is_none_or(|before| before.wait(timeout))
    }

    // opens when this turn ends
    pub fn done(&self) -> TurnLatch {
        self.done.clone()
    }
}

impl Drop for FolderTurn {
    fn drop(&mut self) {
        self.done.open();
        self.turns.end(&self.root, &self.done);
    }
}
