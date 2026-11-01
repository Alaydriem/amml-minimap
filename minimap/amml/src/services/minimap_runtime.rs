use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use amm_core::errors::LinkError;
use amml_game::{GameEntries, GameHeap, StopLatch};
use amml_sdk::{HookHandle, ModRuntime};
use minimap_link::errors::SessionError;

use crate::services::{ListenerHub, MapSession, WorldIdentityReader, WorldStorage, WorldTracker};
use crate::types::{BlockChange, LinkState, RuntimeState};

// what the detours and the listener share: the folders, the stop latch and listener hub, and the state of the
// current enable. The tick and the listener run on the game thread; disable may run on another and waits for the
// state lock
pub struct MinimapRuntime {
    config_dir: PathBuf,
    data_dir: PathBuf,
    hub: ListenerHub,
    state: Mutex<RuntimeState>,
    // the OS id of the thread that runs the tick, 0 until the first tick
    game_thread: AtomicU32,
    // the disable thread is logged once per process
    disable_thread_logged: AtomicBool,
}

static RUNTIME: OnceLock<MinimapRuntime> = OnceLock::new();

impl MinimapRuntime {
    pub const HOOK_CAPACITY: usize = 4;

    pub fn new(config_dir: &Path, data_dir: &Path) -> Self {
        Self {
            config_dir: config_dir.to_path_buf(),
            data_dir: data_dir.to_path_buf(),
            hub: ListenerHub::new(Self::on_block_changed),
            state: Mutex::new(RuntimeState::new(Self::HOOK_CAPACITY)),
            game_thread: AtomicU32::new(0),
            disable_thread_logged: AtomicBool::new(false),
        }
    }

    // the runtime of this DLL; the first call wins
    pub fn install(config_dir: &Path, data_dir: &Path) -> &'static MinimapRuntime {
        RUNTIME.get_or_init(|| Self::new(config_dir, data_dir))
    }

    pub fn get() -> Option<&'static MinimapRuntime> {
        RUNTIME.get()
    }

    // raised by disable and by a panic in the tick or the listener
    pub fn latch(&self) -> &StopLatch {
        self.hub.latch()
    }

    // enable: keeps the entries and reaches the manager now; when the manager is not there yet, the first tick
    // tries again. A lock poisoned by a panic before the last disable is cleared: `stop` dropped the state the
    // panic left half done. The caches of worlds with no seed from game processes that have ended are removed
    // before the runtime starts
    pub fn start(&'static self, entries: GameEntries) {
        if self.state.is_poisoned() {
            self.state.clear_poison();
            if let Some(runtime) = ModRuntime::get() {
                runtime.logger().warn("a panic before the last disable left the minimap state locked; it is reset");
            }
        }
        WorldIdentityReader::sweep_unseeded(&self.data_dir);
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.pending = Some(entries);
        state.link = LinkState::Waiting;
        self.latch().lower();
        self.attach(&mut state, false);
    }

    // keeps an installed hook; false, keeping nothing, when HOOK_CAPACITY hooks are already kept
    pub fn keep_hook(&self, hook: HookHandle) -> bool {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if state.hooks.len() >= Self::HOOK_CAPACITY {
            return false;
        }
        state.hooks.push(hook);
        true
    }

    // forgets a hook the caller removed again
    pub fn forget_hook(&self, hook: HookHandle) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.hooks.retain(|kept| *kept != hook);
    }

    // runs `body` on the state of this enable; `None` while a panic has the lock poisoned
    pub fn with_state<R>(&self, body: impl FnOnce(&mut RuntimeState) -> R) -> Option<R> {
        let mut state = self.state.lock().ok()?;
        Some(body(&mut state))
    }

    // the installed hooks, newest first, leaving none behind
    pub fn take_hooks(&self) -> Vec<HookHandle> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let mut hooks = std::mem::take(&mut state.hooks);
        hooks.reverse();
        hooks
    }

    // the body of the client update detour
    pub fn tick(&'static self, client: NonNull<c_void>) {
        self.mark_game_thread();
        let Ok(mut state) = self.state.lock() else { return };
        if state.link == LinkState::Waiting {
            self.attach(&mut state, true);
        }
        if let Some(tracker) = state.tracker.as_mut() {
            tracker.tick(client);
        }
    }

    // disable, after the hooks are gone: the listener leaves its source, the session gives its leases back and
    // unregisters, and the runtime's threads stop. The storage pointer is forgotten, since no detour follows it
    // while disabled
    pub fn stop(&self) {
        self.latch().raise();
        let on_game_thread = self.on_game_thread();
        self.log_disable_thread(on_game_thread);
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(mut tracker) = state.tracker.take() {
            tracker.reset(on_game_thread);
            tracker.into_sink().shutdown();
        }
        state.pending = None;
        state.link = LinkState::Off;
        if let Some(storage) = WorldStorage::global().current() {
            WorldStorage::global().shut_down(storage.as_ptr() as usize);
        }
    }

    // the tick runs on the game thread: the first one records it
    pub fn mark_game_thread(&self) {
        let _ = self.game_thread.compare_exchange(0, Self::current_thread(), Ordering::Relaxed, Ordering::Relaxed);
    }

    // whether the calling thread is the one the tick runs on. Before the first tick it counts as the game thread: no
    // source can hold the listener yet, since only the tick attaches it
    pub fn on_game_thread(&self) -> bool {
        let game = self.game_thread.load(Ordering::Relaxed);
        game == 0 || game == Self::current_thread()
    }

    // once per process: which thread runs disable, and whether it is the game thread
    fn log_disable_thread(&self, on_game_thread: bool) {
        if self.disable_thread_logged.swap(true, Ordering::Relaxed) {
            return;
        }
        let game = self.game_thread.load(Ordering::Relaxed);
        let game = if game == 0 { "unknown: no tick yet".to_string() } else { game.to_string() };
        curia::info!("disable thread", {
            thread: Self::current_thread(),
            game_thread: game,
            on_game_thread: on_game_thread
        });
    }

    #[allow(unsafe_code)]
    fn current_thread() -> u32 {
        // SAFETY: GetCurrentThreadId has no preconditions
        unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() }
    }

    fn attach(&'static self, state: &mut RuntimeState, last_try: bool) {
        let Some(entries) = state.pending.take() else {
            state.link = LinkState::Off;
            return;
        };
        match MapSession::connect(&self.config_dir, &self.data_dir) {
            Ok(session) => {
                state.tracker = Some(WorldTracker::new(
                    entries,
                    session,
                    &self.hub,
                    GameHeap::msvc(),
                    &self.data_dir,
                    WorldStorage::global(),
                ));
                state.link = LinkState::Attached;
            }
            Err(error) if last_try => {
                Self::log_link_error(&error);
                state.link = LinkState::Off;
            }
            Err(_) => state.pending = Some(entries),
        }
    }

    // the hub's target: the change goes to the tracker unless the state is busy, which only a call from inside the
    // tick itself could find
    fn on_block_changed(change: &BlockChange) {
        let Some(runtime) = Self::get() else { return };
        let Ok(mut state) = runtime.state.try_lock() else { return };
        if let Some(tracker) = state.tracker.as_mut() {
            tracker.block_changed(change);
        }
    }

    fn log_link_error(error: &SessionError) {
        let Some(runtime) = ModRuntime::get() else { return };
        match error {
            SessionError::Link(LinkError::ManagerMissing | LinkError::AbiMismatch { .. }) => {
                runtime.logger().error("@alaydriem/minimap needs @amml/modmanager; the minimap stays off");
            }
            other => {
                runtime.logger().error(&format!("@alaydriem/minimap could not start ({other}); the minimap stays off"));
            }
        }
    }
}
