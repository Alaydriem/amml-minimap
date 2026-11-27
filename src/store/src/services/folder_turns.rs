use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::services::{FolderTurn, TurnLatch};

// the order in which writers use each cache folder: a writer takes a turn and starts only when the turn before it on
// the same folder has ended, so two store threads never write one folder at once. A folder's entry is the latest
// turn on it and is removed when that turn ends, so the map holds only folders with a live turn (bounded by the live
// stores: the current world's and the flushes still running)
#[derive(Debug, Default)]
pub struct FolderTurns {
    latest: Mutex<HashMap<PathBuf, TurnLatch>>,
}

impl FolderTurns {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    // the next turn on `root`; it waits for the turn that was the latest on `root` when it was taken
    pub fn take(self: &Arc<Self>, root: &Path) -> FolderTurn {
        let done = TurnLatch::new();
        let before = self.lock().insert(root.to_path_buf(), done.clone());
        FolderTurn::new(Arc::clone(self), root.to_path_buf(), before, done)
    }

    // folders with a turn that has not ended
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    // a later turn on the folder keeps its entry
    pub(crate) fn end(&self, root: &Path, done: &TurnLatch) {
        let mut latest = self.lock();
        if latest.get(root).is_some_and(|latch| latch.same(done)) {
            latest.remove(root);
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<PathBuf, TurnLatch>> {
        self.latest.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
