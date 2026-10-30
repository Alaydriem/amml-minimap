use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use alaydriem_minimap::errors::ListenerError;
use alaydriem_minimap::services::{BlockListener, ListenerHub};
use alaydriem_minimap::types::{BlockChange, BlockPos, ListenerSlots};

use crate::support::{FakeHeap, FakeWorld};

static CHANGES: AtomicUsize = AtomicUsize::new(0);

fn count(_: &BlockChange) {
    CHANGES.fetch_add(1, Ordering::SeqCst);
}

type BlockChanged = extern "C" fn(
    *mut c_void,
    *mut c_void,
    *const BlockPos,
    u32,
    *mut c_void,
    *mut c_void,
    i32,
    *const c_void,
    u8,
    *mut c_void,
);
type SourceDestroyed = extern "C" fn(*mut c_void, *mut c_void);

fn slots(block_changed: u64) -> ListenerSlots {
    ListenerSlots {
        add_listener: 2,
        remove_listener: 3,
        dimension_id: 4,
        source_destroyed: FakeWorld::LISTENER_SOURCE_DESTROYED,
        block_changed,
    }
}

fn listener_with_hub() -> (BlockListener, &'static ListenerHub) {
    let hub: &'static ListenerHub = Box::leak(Box::new(ListenerHub::new(count)));
    let heap = FakeHeap::msvc();
    // SAFETY: the base is a 9-slot table and FakeHeap a valid heap
    let listener =
        unsafe { BlockListener::new(FakeWorld::listener_base(), slots(FakeWorld::LISTENER_BLOCK_CHANGED), hub, &heap) };
    (listener.unwrap(), hub)
}

fn listener() -> BlockListener {
    listener_with_hub().0
}

// calls the listener's slot as the game does: through the object's own vtable
fn fire_block_changed(listener: &BlockListener, source: *mut c_void) {
    let thunk = listener.vtable()[FakeWorld::LISTENER_BLOCK_CHANGED as usize];
    // SAFETY: the slot holds the onBlockChanged thunk
    let thunk: BlockChanged = unsafe { std::mem::transmute(thunk) };
    let pos = BlockPos { x: 1, y: 2, z: 3 };
    let null = std::ptr::null_mut();
    thunk(listener.object(), source, &pos, 0, null, null, 3, std::ptr::null(), 0, null);
}

fn fire_source_destroyed(listener: &BlockListener, source: *mut c_void) {
    let thunk = listener.vtable()[FakeWorld::LISTENER_SOURCE_DESTROYED as usize];
    // SAFETY: the slot holds the onSourceDestroyed thunk
    let thunk: SourceDestroyed = unsafe { std::mem::transmute(thunk) };
    thunk(listener.object(), source);
}

#[test]
fn the_copied_table_keeps_the_game_defaults_and_replaces_slots_0_2_and_4() {
    let listener = listener();
    let base = FakeWorld::listener_base_slots();
    let table = listener.vtable();

    for slot in [1, 3, 5, 6, 7, 8] {
        assert_eq!(table[slot], base[slot], "slot {slot} is the game's");
    }
    for slot in [0, 2, 4] {
        assert_ne!(table[slot], base[slot], "slot {slot} is the listener's");
    }
    // SAFETY: the object starts with its vtable pointer
    let object_table = unsafe { *listener.object().cast::<*const usize>() };
    assert_eq!(object_table, table.as_ptr());
}

#[test]
fn a_listener_slot_outside_the_table_is_refused() {
    let hub: &'static ListenerHub = Box::leak(Box::new(ListenerHub::new(count)));

    // SAFETY: as `listener`
    let result = unsafe { BlockListener::new(FakeWorld::listener_base(), slots(9), hub, &FakeHeap::msvc()) };

    assert!(matches!(result, Err(ListenerError::SlotOutOfRange { slot: 9 })));
}

#[test]
fn detach_removes_the_same_object_from_the_source_it_was_added_to() {
    let mut world = FakeWorld::new("minecraft:stone");
    let entries = FakeWorld::entries(true);
    let listener = listener();

    // SAFETY: a live fake source
    unsafe { listener.attach(&entries, world.source.ptr()) };
    listener.detach(&entries);

    assert_eq!(world.source.added, vec![listener.object() as usize]);
    assert_eq!(world.source.removed, world.source.added);
    assert!(!listener.is_attached());
}

#[test]
fn a_block_change_with_the_latch_raised_does_nothing() {
    let (listener, hub) = listener_with_hub();
    let source = 0x9000 as *mut c_void;

    hub.latch().raise();
    fire_block_changed(&listener, source);
    let while_raised = CHANGES.load(Ordering::SeqCst);
    hub.latch().lower();
    fire_block_changed(&listener, source);

    assert_eq!(while_raised, 0);
    assert_eq!(CHANGES.load(Ordering::SeqCst), 1, "a lowered latch passes the change on");
}

#[test]
fn a_destroyed_source_is_forgotten_even_with_the_latch_raised() {
    let mut world = FakeWorld::new("minecraft:stone");
    let entries = FakeWorld::entries(true);
    let (listener, hub) = listener_with_hub();
    // SAFETY: a live fake source
    unsafe { listener.attach(&entries, world.source.ptr()) };

    hub.latch().raise();
    fire_source_destroyed(&listener, world.source.ptr());
    listener.detach(&entries);

    assert!(world.source.removed.is_empty());
}

#[test]
fn a_destroyed_source_is_never_asked_to_remove_the_listener() {
    let mut world = FakeWorld::new("minecraft:stone");
    let entries = FakeWorld::entries(true);
    let listener = listener();
    // SAFETY: a live fake source
    unsafe { listener.attach(&entries, world.source.ptr()) };

    fire_source_destroyed(&listener, world.source.ptr());
    listener.detach(&entries);

    assert_eq!(world.source.added.len(), 1);
    assert!(world.source.removed.is_empty());
}

#[test]
fn a_registered_listener_is_not_freed_and_a_detached_one_is_freed_once() {
    let mut world = FakeWorld::new("minecraft:stone");
    let entries = FakeWorld::entries(true);
    let listener = listener();
    let object = listener.object() as usize;
    // SAFETY: a live fake source
    unsafe { listener.attach(&entries, world.source.ptr()) };

    // SAFETY: FakeHeap made the object; the fake source runs no thunk on its own
    let listener = unsafe { listener.release(&FakeHeap::msvc()) }.unwrap_err();
    let while_registered = FakeHeap::frees_of(object);
    listener.detach(&entries);
    // SAFETY: as above
    let released = unsafe { listener.release(&FakeHeap::msvc()) };

    assert_eq!(while_registered, 0, "no free while a source holds the object");
    assert!(released.is_ok());
    assert_eq!(world.source.removed, vec![object]);
    assert_eq!(FakeHeap::frees_of(object), 1);
}
