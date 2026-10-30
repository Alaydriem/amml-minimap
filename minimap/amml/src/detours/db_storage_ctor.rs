use amml_sdk::DetourGuard;

use crate::services::{MinimapRuntime, WorldStorage};

// DBStorage::DBStorage(DBStorageConfig, Bedrock::NotNullNonOwnerPtr<LevelDbEnv>): both arguments are passed by
// hidden pointer, so the detour takes (this, config, env) and returns the original's rax, which is `this`. While
// the stop latch is raised the storage is not recorded: `stop` has cleared the pointer and no shutdown detour may
// clear it again
pub static GUARD: DetourGuard = DetourGuard::new("db_storage_ctor");

pub type DbStorageCtorFn = unsafe extern "C" fn(usize, usize, usize) -> usize;

#[allow(unsafe_code)]
pub extern "C" fn detour(this: usize, config: usize, env: usize) -> usize {
    let original: DbStorageCtorFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature
    let result = unsafe { original(this, config, env) };
    let stopped = MinimapRuntime::get().is_some_and(|runtime| runtime.latch().raised());
    if !stopped {
        GUARD.run(|| WorldStorage::global().constructed(this), || ());
    }
    result
}
