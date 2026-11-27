use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::{Duration, Instant};

// opens once and stays open; every clone sees the same state
#[derive(Debug, Clone, Default)]
pub struct TurnLatch {
    state: Arc<(Mutex<bool>, Condvar)>,
}

impl TurnLatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&self) {
        let (open, changed) = &*self.state;
        *open.lock().unwrap_or_else(PoisonError::into_inner) = true;
        changed.notify_all();
    }

    pub fn is_open(&self) -> bool {
        *self.state.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    // true when the latch opened within `timeout`; a timeout past the clock's range waits until it opens
    pub fn wait(&self, timeout: Duration) -> bool {
        let (open, changed) = &*self.state;
        let deadline = Instant::now().checked_add(timeout);
        let mut guard = open.lock().unwrap_or_else(PoisonError::into_inner);
        while !*guard {
            guard = match deadline {
                None => changed.wait(guard).unwrap_or_else(PoisonError::into_inner),
                Some(deadline) => {
                    let now = Instant::now();
                    if now >= deadline {
                        return false;
                    }
                    changed.wait_timeout(guard, deadline - now).unwrap_or_else(PoisonError::into_inner).0
                }
            };
        }
        true
    }

    pub fn same(&self, other: &TurnLatch) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }
}
