use amml_sdk::DetourGuard;

use crate::services::WorldStorage;

// bool DBStorage::checkShutdownDone(): true once the storage has shut down and is about to be destroyed. The
// pointer is cleared whatever the stop latch says: a clear never reaches the game
pub static GUARD: DetourGuard = DetourGuard::new("db_storage_shutdown");

pub type CheckShutdownDoneFn = unsafe extern "C" fn(usize) -> u8;

#[allow(unsafe_code)]
pub extern "C" fn detour(this: usize) -> u8 {
    let original: CheckShutdownDoneFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature
    let done = unsafe { original(this) };
    if done != 0 {
        GUARD.run(|| WorldStorage::global().shut_down(this), || ());
    }
    done
}
