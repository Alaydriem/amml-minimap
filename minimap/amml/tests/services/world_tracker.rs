use std::path::Path;

use alaydriem_minimap::services::{ListenerHub, WorldStorage, WorldTracker};
use alaydriem_minimap::types::{BlockChange, BlockPos};
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::structs::{ProbeRequest, ScanPlan, SubmitStatus};
use minimap_store::structs::WorldIdentity;

use crate::support::{FakeHeap, FakePalette, FakeWorld, RecordingSink, SinkCall};

fn ignore(_: &BlockChange) {}

fn tracker(sink: RecordingSink, optional: bool) -> WorldTracker<RecordingSink> {
    let hub: &'static ListenerHub = Box::leak(Box::new(ListenerHub::new(ignore)));
    let storage: &'static WorldStorage = Box::leak(Box::new(WorldStorage::new()));
    WorldTracker::new(FakeWorld::entries(optional), sink, hub, Some(FakeHeap::msvc()), Path::new("data"), storage)
}

#[test]
fn entering_a_world_names_it_ticks_with_the_player_and_probes_the_plan() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let mut tracker = tracker(RecordingSink::probing(), true);

    tracker.tick(world.client_ptr());

    let local = WorldIdentity::Local { world_dir: Path::new("data").join("local").join("S1234") };
    assert_eq!(
        tracker.sink().calls,
        vec![
            SinkCall::Enter(local, 0),
            SinkCall::Tick { present: true },
            SinkCall::Submit(ChunkPos::new(0, 0, 0), SubmitStatus::Loaded),
        ]
    );
    assert!(tracker.in_world());
}

#[test]
fn out_of_a_world_the_tracker_leaves_once_then_ticks_without_a_player() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let mut tracker = tracker(RecordingSink::probing(), true);
    tracker.tick(world.client_ptr());
    let entered = tracker.sink().calls.len();

    world.client.world_active = 0;
    for _ in 0..3 {
        tracker.tick(world.client_ptr());
    }

    let after: Vec<_> = tracker.sink().calls[entered..].to_vec();
    assert_eq!(
        after,
        vec![
            SinkCall::Leave,
            SinkCall::Tick { present: false },
            SinkCall::Tick { present: false },
            SinkCall::Tick { present: false },
        ]
    );
    assert!(!tracker.in_world());
}

#[test]
fn chunks_past_the_plan_limit_are_not_probed() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let plan = ScanPlan {
        candidates: vec![ChunkPos::new(0, 0, 0), ChunkPos::new(1, 0, 0)],
        limit: 1,
        camera_height: 320,
        ..ScanPlan::default()
    };
    let mut tracker = tracker(RecordingSink::new(plan), true);

    tracker.tick(world.client_ptr());

    let submits: Vec<_> =
        tracker.sink().calls.iter().filter(|call| matches!(call, SinkCall::Submit(..))).cloned().collect();
    assert_eq!(
        submits,
        vec![
            SinkCall::Submit(ChunkPos::new(0, 0, 0), SubmitStatus::Loaded),
            SinkCall::Submit(ChunkPos::new(1, 0, 0), SubmitStatus::NotProbed),
        ]
    );
}

#[test]
fn reset_removes_the_listener_from_the_source_it_was_added_to() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let mut tracker = tracker(RecordingSink::probing(), true);
    tracker.tick(world.client_ptr());
    let object = tracker.listener().unwrap().object() as usize;

    tracker.reset(true);

    assert_eq!(world.source.added, vec![object]);
    assert_eq!(world.source.removed, vec![object]);
}

#[test]
fn after_the_source_is_destroyed_leaving_the_world_never_reaches_it() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let mut tracker = tracker(RecordingSink::probing(), true);
    tracker.tick(world.client_ptr());
    let listener = tracker.listener().unwrap();
    let thunk = listener.vtable()[FakeWorld::LISTENER_SOURCE_DESTROYED as usize];
    // SAFETY: the slot holds the onSourceDestroyed thunk
    let thunk: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) = unsafe { std::mem::transmute(thunk) };

    thunk(listener.object(), world.source.ptr());
    world.client.world_active = 0;
    tracker.tick(world.client_ptr());
    tracker.reset(true);

    assert_eq!(world.source.added.len(), 1);
    assert!(world.source.removed.is_empty());
}

#[test]
fn without_the_block_changes_feature_no_listener_is_made() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let mut tracker = tracker(RecordingSink::probing(), false);

    tracker.tick(world.client_ptr());

    assert!(tracker.listener().is_none());
    assert!(world.source.added.is_empty());
}

#[test]
fn a_block_that_changes_type_probes_the_column_the_minimap_asks_for() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let mut sink = RecordingSink::probing();
    sink.request = ProbeRequest::Column { pos: ChunkPos::new(0, 0, 0), index: 5 };
    let mut tracker = tracker(sink, true);
    tracker.tick(world.client_ptr());
    let ticked = tracker.sink().calls.len();
    let pos = BlockPos { x: 5, y: 4, z: 0 };
    let source = world.source.ptr();
    let same = BlockChange {
        source,
        pos,
        block: world.palette.block(FakePalette::STONE),
        old_block: world.palette.block(FakePalette::STONE),
    };
    let changed = BlockChange { block: world.palette.block(FakePalette::LAVA), ..same };

    tracker.block_changed(&same);
    tracker.block_changed(&changed);

    let after: Vec<_> = tracker.sink().calls[ticked..].to_vec();
    assert_eq!(after, vec![SinkCall::BlockChanged(5, 0, 0), SinkCall::Column(ChunkPos::new(0, 0, 0), 5)]);
}

#[test]
fn each_enable_makes_one_listener_and_its_disable_frees_it_once_after_the_detach() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let object_size = 2 * size_of::<usize>();
    let mut objects = Vec::new();

    for cycle in 1..=2 {
        let mut tracker = tracker(RecordingSink::probing(), true);
        tracker.tick(world.client_ptr());
        let object = tracker.listener().unwrap().object() as usize;
        let before_reset = FakeHeap::frees_of(object);
        tracker.reset(true);

        assert_eq!(FakeHeap::allocations_of(object_size), cycle, "one allocation per enable");
        assert_eq!(before_reset, 0, "no free before the detach");
        assert_eq!(FakeHeap::frees_of(object), 1, "one free per disable");
        assert_eq!(world.source.removed.last(), Some(&object), "removed before it was freed");
        assert!(tracker.listener().is_none());
        objects.push(object);
    }
    assert_eq!(world.source.added.len(), 2, "re-enable registers a fresh object");
}

#[test]
fn a_listener_whose_source_was_destroyed_is_freed_at_disable_without_a_remove() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let mut tracker = tracker(RecordingSink::probing(), true);
    tracker.tick(world.client_ptr());
    let listener = tracker.listener().unwrap();
    let object = listener.object() as usize;
    let thunk = listener.vtable()[FakeWorld::LISTENER_SOURCE_DESTROYED as usize];
    // SAFETY: the slot holds the onSourceDestroyed thunk
    let thunk: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) = unsafe { std::mem::transmute(thunk) };

    thunk(listener.object(), world.source.ptr());
    tracker.reset(true);

    assert!(world.source.removed.is_empty());
    assert_eq!(FakeHeap::frees_of(object), 1);
}

#[test]
fn off_the_game_thread_with_a_source_attached_the_listener_is_removed_but_stays_allocated() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let mut tracker = tracker(RecordingSink::probing(), true);
    tracker.tick(world.client_ptr());
    let object = tracker.listener().unwrap().object() as usize;

    tracker.reset(false);

    assert_eq!(world.source.removed, vec![object], "it still leaves its source");
    assert_eq!(FakeHeap::frees_of(object), 0, "the object is not freed");
    assert!(tracker.listener().is_none());
    assert!(!tracker.in_world());
}

#[test]
fn off_the_game_thread_with_no_source_attached_the_listener_is_freed() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    let mut tracker = tracker(RecordingSink::probing(), true);
    tracker.tick(world.client_ptr());
    let listener = tracker.listener().unwrap();
    let object = listener.object() as usize;
    let thunk = listener.vtable()[FakeWorld::LISTENER_SOURCE_DESTROYED as usize];
    // SAFETY: the slot holds the onSourceDestroyed thunk
    let thunk: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) = unsafe { std::mem::transmute(thunk) };

    thunk(listener.object(), world.source.ptr());
    tracker.reset(false);

    assert_eq!(FakeHeap::frees_of(object), 1);
}
