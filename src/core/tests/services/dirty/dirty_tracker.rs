use minimap_core::services::dirty::DirtyTracker;
use minimap_core::services::memory::ChunkStore;
use minimap_core::structs::LightDirection;
use minimap_core::structs::bake::BakeJob;
use minimap_core::structs::biome::BiomeKey;
use minimap_core::structs::chunk::{ChangeKinds, ChunkSurface, ColumnSample};
use minimap_core::structs::pos::{ChunkPos, RegionPos};

use crate::support::TestSurfaces;

fn tracker(threshold: usize) -> DirtyTracker {
    DirtyTracker::new(LightDirection::new(315.0, 60.0), threshold)
}

fn submit(store: &mut ChunkStore, tracker: &mut DirtyTracker, pos: ChunkPos, surface: &ChunkSurface) {
    let result = store.submit(pos, surface, 1, ChunkPos::new(0, 0, 0), &mut Vec::new());
    tracker.on_chunk_submit(store, pos, &result);
}

fn run_ticks(tracker: &mut DirtyTracker, store: &ChunkStore, ticks: u32) -> Vec<BakeJob> {
    let mut jobs = Vec::new();
    for _ in 0..ticks {
        tracker.tick(store, &mut jobs);
    }
    jobs
}

#[test]
fn a_first_load_is_baked_as_a_chunk_job_after_the_debounce() {
    let mut store = ChunkStore::new(64);
    let mut tracker = tracker(64);
    let pos = ChunkPos::new(0, 0, 0);
    submit(&mut store, &mut tracker, pos, &TestSurfaces::probe_flat(64, TestSurfaces::grey(100)));
    assert!(run_ticks(&mut tracker, &store, 9).is_empty());
    assert_eq!(run_ticks(&mut tracker, &store, 1), vec![BakeJob::Chunk { pos, soft_only: false }]);
    assert_eq!(tracker.pending_regions(), 0);
}

#[test]
fn a_height_change_marks_loaded_downstream_chunks() {
    let mut store = ChunkStore::new(64);
    let mut tracker = tracker(64);
    let flat = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    let downstream = ChunkPos::new(1, 1, 0);
    submit(&mut store, &mut tracker, downstream, &flat);
    let pos = ChunkPos::new(0, 0, 0);
    submit(&mut store, &mut tracker, pos, &flat);
    run_ticks(&mut tracker, &store, 60);
    let snapshot = store.dirty(downstream).expect("stored").epochs();
    tracker.complete_chunk(&mut store, downstream, false, snapshot, false);

    submit(&mut store, &mut tracker, pos, &TestSurfaces::probe_flat(90, TestSurfaces::grey(100)));
    assert!(store.dirty(downstream).expect("stored").is_full());
}

#[test]
fn a_solid_only_change_soft_marks_the_ring() {
    let mut store = ChunkStore::new(64);
    let mut tracker = tracker(64);
    let flat = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    let east = ChunkPos::new(1, 0, 0);
    let pos = ChunkPos::new(0, 0, 0);
    submit(&mut store, &mut tracker, east, &flat);
    submit(&mut store, &mut tracker, pos, &flat);
    let snapshot = store.dirty(east).expect("stored").epochs();
    tracker.complete_chunk(&mut store, east, false, snapshot, false);

    let mut probe = flat;
    probe.solid[0] = 60;
    submit(&mut store, &mut tracker, pos, &probe);
    let east_flags = store.dirty(east).expect("stored");
    assert!(east_flags.is_soft());
    assert!(!east_flags.is_full());
}

#[test]
fn more_dirty_chunks_than_the_threshold_become_one_region_job() {
    let mut store = ChunkStore::new(64);
    let mut tracker = tracker(2);
    let flat = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    for x in 0..3 {
        submit(&mut store, &mut tracker, ChunkPos::new(x * 4, 0, 0), &flat);
    }
    let jobs = run_ticks(&mut tracker, &store, 10);
    assert_eq!(jobs, vec![BakeJob::Region(RegionPos::new(0, 0, 0))]);
}

#[test]
fn a_mark_during_the_bake_keeps_the_chunk_dirty_and_rearms_the_region() {
    let mut store = ChunkStore::new(64);
    let mut tracker = tracker(64);
    let pos = ChunkPos::new(0, 0, 0);
    submit(&mut store, &mut tracker, pos, &TestSurfaces::probe_flat(64, TestSurfaces::grey(100)));
    run_ticks(&mut tracker, &store, 10);
    let snapshot = store.dirty(pos).expect("stored").epochs();
    submit(&mut store, &mut tracker, pos, &TestSurfaces::probe_flat(64, TestSurfaces::grey(90)));
    run_ticks(&mut tracker, &store, 10);
    tracker.complete_chunk(&mut store, pos, false, snapshot, false);
    assert!(store.dirty(pos).expect("stored").is_full());
    assert_eq!(tracker.pending_regions(), 1);
}

#[test]
fn a_changed_mask_soft_marks_neighbours() {
    let mut store = ChunkStore::new(64);
    let mut tracker = tracker(64);
    let flat = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    let west = ChunkPos::new(-1, 0, 0);
    let pos = ChunkPos::new(0, 0, 0);
    submit(&mut store, &mut tracker, west, &flat);
    submit(&mut store, &mut tracker, pos, &flat);
    let west_snapshot = store.dirty(west).expect("stored").epochs();
    tracker.complete_chunk(&mut store, west, false, west_snapshot, false);
    let snapshot = store.dirty(pos).expect("stored").epochs();
    tracker.complete_chunk(&mut store, pos, false, snapshot, true);
    assert!(store.dirty(west).expect("stored").is_soft());
}

#[test]
fn a_region_completion_soft_marks_only_neighbours_outside_the_region() {
    let mut store = ChunkStore::new(64);
    let mut tracker = tracker(64);
    let flat = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    let edge = ChunkPos::new(15, 0, 0);
    let inside = ChunkPos::new(14, 0, 0);
    let outside = ChunkPos::new(16, 0, 0);
    for pos in [edge, inside, outside] {
        submit(&mut store, &mut tracker, pos, &flat);
        let snapshot = store.dirty(pos).expect("stored").epochs();
        tracker.complete_chunk(&mut store, pos, false, snapshot, false);
    }
    let region = RegionPos::new(0, 0, 0);
    let baked = [(edge, store.dirty(edge).expect("stored").epochs())];
    tracker.complete_region(&mut store, region, &baked, &[edge]);
    assert!(store.dirty(outside).expect("stored").is_soft());
    assert!(!store.dirty(inside).expect("stored").is_soft());
}

#[test]
fn a_column_height_change_marks_downstream_chunks() {
    let mut store = ChunkStore::new(64);
    let mut tracker = tracker(64);
    let flat = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    let pos = ChunkPos::new(0, 0, 0);
    let downstream = ChunkPos::new(1, 1, 0);
    for chunk in [pos, downstream] {
        submit(&mut store, &mut tracker, chunk, &flat);
        let snapshot = store.dirty(chunk).expect("stored").epochs();
        tracker.complete_chunk(&mut store, chunk, false, snapshot, false);
    }
    let index = ChunkSurface::index(15, 15);
    let column = ColumnSample::new(TestSurfaces::grey(100), 0, 120, 120, false, 0);
    let result = store.submit_column(pos, index, column, None).expect("stored with base data");
    tracker.on_column_submit(&mut store, pos, (15, 15), 64, 120, &result);
    assert!(store.dirty(pos).expect("stored").is_full());
    assert!(store.dirty(downstream).expect("stored").is_full());
}

#[test]
fn a_requeued_region_is_dispatched_again_after_its_debounce() {
    let mut store = ChunkStore::new(64);
    let mut tracker = tracker(64);
    let pos = ChunkPos::new(0, 0, 0);
    submit(&mut store, &mut tracker, pos, &TestSurfaces::probe_flat(64, TestSurfaces::grey(100)));
    assert_eq!(run_ticks(&mut tracker, &store, 10).len(), 1);
    tracker.requeue(pos.region());
    assert_eq!(run_ticks(&mut tracker, &store, 10), vec![BakeJob::Chunk { pos, soft_only: false }]);
}

#[test]
fn a_biome_only_change_is_journaled_but_not_rebaked() {
    let mut store = ChunkStore::new(64);
    let mut tracker = tracker(64);
    let pos = ChunkPos::new(0, 0, 0);
    let stored = TestSurfaces::stored_flat(64, TestSurfaces::grey(100));
    assert!(store.restore(pos, &stored, pos, &mut Vec::new()));
    let mut named = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    named.biome.set(0, Some(BiomeKey(1)));
    let result = store.submit(pos, &named, 1, pos, &mut Vec::new());
    assert_eq!(result.changes, ChangeKinds::BIOME);
    assert!(store.is_journal_dirty(pos));
    tracker.on_chunk_submit(&mut store, pos, &result);
    assert!(!store.dirty(pos).unwrap().is_full());
    assert!(run_ticks(&mut tracker, &store, 60).is_empty());
}
