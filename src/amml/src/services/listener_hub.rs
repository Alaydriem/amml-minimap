use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_game::StopLatch;

use crate::types::BlockChange;

// what the block listener's thunks reach: the mod's stop latch, the BlockSource the listener is registered with,
// and where a block change goes. The source is forgotten as soon as the game says it is destroyed, so no call
// ever reaches a freed source
pub struct ListenerHub {
    latch: StopLatch,
    source: AtomicPtr<c_void>,
    changed: fn(&BlockChange),
}

impl ListenerHub {
    pub const fn new(changed: fn(&BlockChange)) -> Self {
        Self { latch: StopLatch::new(), source: AtomicPtr::new(std::ptr::null_mut()), changed }
    }

    pub fn latch(&self) -> &StopLatch {
        &self.latch
    }

    // the source the listener is registered with; null when none
    pub fn attached(&self) -> *mut c_void {
        self.source.load(Ordering::SeqCst)
    }

    pub fn set_attached(&self, source: *mut c_void) {
        self.source.store(source, Ordering::SeqCst);
    }

    // the registered source, leaving none
    pub fn take_attached(&self) -> *mut c_void {
        self.source.swap(std::ptr::null_mut(), Ordering::SeqCst)
    }

    // the game destroys `source`: if it is the registered one, it is forgotten
    pub fn source_destroyed(&self, source: *mut c_void) {
        let _ = self.source.compare_exchange(source, std::ptr::null_mut(), Ordering::SeqCst, Ordering::SeqCst);
    }

    pub fn block_changed(&self, change: &BlockChange) {
        (self.changed)(change);
    }
}
