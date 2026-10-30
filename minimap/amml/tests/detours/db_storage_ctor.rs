use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use alaydriem_minimap::detours::db_storage_ctor;
use alaydriem_minimap::services::WorldStorage;

static SEEN: Mutex<Vec<(usize, usize, usize)>> = Mutex::new(Vec::new());

extern "C" fn original(this: usize, config: usize, env: usize) -> usize {
    SEEN.lock().unwrap().push((this, config, env));
    this
}

// the detour writes the process-global WorldStorage; see the shutdown test
#[test]
fn the_detour_passes_all_three_arguments_and_holds_the_constructed_storage() {
    let next: extern "C" fn(usize, usize, usize) -> usize = original;
    db_storage_ctor::GUARD.original().store(next as *mut c_void, Ordering::Release);

    let result = db_storage_ctor::detour(0x5000, 0x6000, 0x7000);

    assert_eq!(result, 0x5000);
    assert_eq!(*SEEN.lock().unwrap(), vec![(0x5000, 0x6000, 0x7000)]);
    assert_eq!(WorldStorage::global().current().map(|storage| storage.as_ptr() as usize), Some(0x5000));
}
