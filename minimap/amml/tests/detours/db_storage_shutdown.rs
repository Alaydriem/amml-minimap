use std::ffi::c_void;
use std::sync::atomic::{AtomicU8, Ordering};

use alaydriem_minimap::detours::db_storage_shutdown;
use alaydriem_minimap::services::WorldStorage;

static DONE: AtomicU8 = AtomicU8::new(0);

extern "C" fn original(_this: usize) -> u8 {
    DONE.load(Ordering::SeqCst)
}

// the detours write the process-global WorldStorage, so this test and the constructor test rely on nextest running
// each test in its own process; they use different storage addresses
#[test]
fn the_storage_is_forgotten_only_once_its_shutdown_is_done() {
    let next: extern "C" fn(usize) -> u8 = original;
    db_storage_shutdown::GUARD.original().store(next as *mut c_void, Ordering::Release);
    WorldStorage::global().constructed(0x8000);

    assert_eq!(db_storage_shutdown::detour(0x8000), 0);
    let while_running = WorldStorage::global().current();
    DONE.store(1, Ordering::SeqCst);
    assert_eq!(db_storage_shutdown::detour(0x4000), 1);
    let other_storage = WorldStorage::global().current();
    assert_eq!(db_storage_shutdown::detour(0x8000), 1);

    assert!(while_running.is_some(), "a shutdown not yet done keeps the storage");
    assert!(other_storage.is_some(), "another storage's shutdown keeps it");
    assert!(WorldStorage::global().current().is_none(), "its own finished shutdown forgets it");
}
