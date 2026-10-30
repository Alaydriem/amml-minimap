use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use alaydriem_minimap::detours::client_update;

static SEEN: Mutex<Vec<(usize, u8, u64)>> = Mutex::new(Vec::new());

extern "C" fn original(this: usize, init_finished: u8, extra: u64) -> u8 {
    SEEN.lock().unwrap().push((this, init_finished, extra));
    7
}

#[test]
fn the_detour_passes_all_three_arguments_to_the_original_and_returns_its_result() {
    let next: extern "C" fn(usize, u8, u64) -> u8 = original;
    client_update::GUARD.original().store(next as *mut c_void, Ordering::Release);

    let result = client_update::detour(0x1234, 1, 0xDEAD_BEEF_0000_0001);

    assert_eq!(result, 7);
    assert_eq!(*SEEN.lock().unwrap(), vec![(0x1234, 1, 0xDEAD_BEEF_0000_0001)]);
}
