use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};

// the DBStorage of the local world now open, as the storage detours saw it: set when one is constructed, cleared
// when its shutdown is done
pub struct WorldStorage {
    storage: AtomicPtr<c_void>,
}

static WORLD_STORAGE: WorldStorage = WorldStorage::new();

impl WorldStorage {
    pub const fn new() -> Self {
        Self { storage: AtomicPtr::new(std::ptr::null_mut()) }
    }

    // the one the detours write
    pub fn global() -> &'static WorldStorage {
        &WORLD_STORAGE
    }

    pub fn constructed(&self, storage: usize) {
        self.storage.store(storage as *mut c_void, Ordering::SeqCst);
    }

    // forgets `storage` if it is the one held; a newer storage stays
    pub fn shut_down(&self, storage: usize) {
        let _ = self.storage.compare_exchange(
            storage as *mut c_void,
            std::ptr::null_mut(),
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
    }

    pub fn current(&self) -> Option<NonNull<c_void>> {
        NonNull::new(self.storage.load(Ordering::SeqCst))
    }
}

impl Default for WorldStorage {
    fn default() -> Self {
        Self::new()
    }
}
