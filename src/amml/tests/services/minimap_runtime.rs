use alaydriem_minimap::services::MinimapRuntime;
use alaydriem_minimap::types::LinkState;

use crate::support::FakeWorld;

#[test]
fn a_lock_poisoned_by_a_panic_is_cleared_by_the_next_enable() {
    let dir = tempfile::tempdir().unwrap();
    let runtime: &'static MinimapRuntime =
        Box::leak(Box::new(MinimapRuntime::new(&dir.path().join("config"), &dir.path().join("data"))));
    let panicked = std::panic::catch_unwind(|| runtime.with_state(|_| panic!("a tick panics")));
    let poisoned = runtime.with_state(|state| state.link);

    runtime.stop();
    runtime.start(FakeWorld::entries(true));

    assert!(panicked.is_err());
    assert_eq!(poisoned, None, "the panic poisoned the lock");
    assert_eq!(runtime.with_state(|state| state.link), Some(LinkState::Waiting), "no manager yet: the tick tries");
    assert!(!runtime.latch().raised());
}

#[test]
fn the_game_thread_is_the_one_the_first_tick_marks() {
    let dir = tempfile::tempdir().unwrap();
    let runtime: &'static MinimapRuntime =
        Box::leak(Box::new(MinimapRuntime::new(&dir.path().join("config"), &dir.path().join("data"))));
    let elsewhere = || std::thread::scope(|scope| scope.spawn(|| runtime.on_game_thread()).join().unwrap());

    assert!(runtime.on_game_thread() && elsewhere(), "before any tick every thread counts as the game thread");
    runtime.mark_game_thread();
    std::thread::scope(|scope| scope.spawn(|| runtime.mark_game_thread()).join().unwrap());

    assert!(runtime.on_game_thread(), "the first mark wins");
    assert!(!elsewhere(), "disable from another thread is off the game thread");
    runtime.stop();
}
