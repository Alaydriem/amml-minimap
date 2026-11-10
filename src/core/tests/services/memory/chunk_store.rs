use minimap_core::services::memory::ChunkStore;
use minimap_core::structs::biome::BiomeKey;
use minimap_core::structs::chunk::{ChunkSurface, ColumnSample, ShadowMask};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::traits::{BakeEnvironment, ScanHistory};

use crate::support::TestSurfaces;

fn flat() -> ChunkSurface {
    TestSurfaces::probe_flat(64, TestSurfaces::grey(100))
}

#[test]
fn the_store_never_holds_more_than_its_capacity() {
    let mut store = ChunkStore::new(32);
    let player = ChunkPos::new(0, 0, 0);
    let mut evicted = Vec::new();
    for x in 0..100 {
        store.submit(ChunkPos::new(x, 0, 0), &flat(), 1, player, &mut evicted);
        assert!(store.len() <= store.capacity());
    }
    assert_eq!(store.capacity(), 32);
}

#[test]
fn eviction_drops_the_chunks_farthest_from_the_player() {
    let mut store = ChunkStore::new(16);
    let player = ChunkPos::new(0, 0, 0);
    let mut evicted = Vec::new();
    for x in 0..16 {
        store.submit(ChunkPos::new(x, 0, 0), &flat(), 1, player, &mut evicted);
    }
    store.submit(ChunkPos::new(-1, 0, 0), &flat(), 1, player, &mut evicted);
    assert!(!store.contains(ChunkPos::new(15, 0, 0)));
    assert!(store.contains(ChunkPos::new(0, 0, 0)));
    assert!(store.contains(ChunkPos::new(-1, 0, 0)));
}

#[test]
fn evicted_chunks_that_were_not_journaled_are_handed_back() {
    let mut store = ChunkStore::new(16);
    let player = ChunkPos::new(0, 0, 0);
    let mut evicted = Vec::new();
    for x in 0..16 {
        store.submit(ChunkPos::new(x, 0, 0), &flat(), 1, player, &mut evicted);
    }
    store.mark_journaled(ChunkPos::new(15, 0, 0));
    store.submit(ChunkPos::new(-1, 0, 0), &flat(), 1, player, &mut evicted);
    assert_eq!(evicted.len(), 0);
    store.mark_journaled(ChunkPos::new(-1, 0, 0));
    store.submit(ChunkPos::new(-2, 0, 0), &flat(), 1, player, &mut evicted);
    assert_eq!(evicted.iter().map(|e| e.pos).collect::<Vec<_>>(), vec![ChunkPos::new(14, 0, 0)]);
}

#[test]
fn an_all_skipped_probe_of_an_unknown_chunk_takes_no_slot() {
    let mut store = ChunkStore::new(16);
    let mut probe = flat();
    probe.skipped = [u64::MAX; 4];
    let result = store.submit(ChunkPos::new(0, 0, 0), &probe, 1, ChunkPos::new(0, 0, 0), &mut Vec::new());
    assert!(result.needs_retry);
    assert!(store.is_empty());
}

#[test]
fn last_scan_is_recorded_only_for_scans_without_a_placeholder() {
    let mut store = ChunkStore::new(16);
    let pos = ChunkPos::new(0, 0, 0);
    let mut probe = flat();
    probe.set_retry(3, true);
    store.submit(pos, &probe, 5, pos, &mut Vec::new());
    assert_eq!(store.last_scan(pos), None);
    store.submit(pos, &flat(), 9, pos, &mut Vec::new());
    assert_eq!(store.last_scan(pos), Some(9));
}

#[test]
fn soft_marks_are_refused_for_full_dirty_and_unknown_chunks() {
    let mut store = ChunkStore::new(16);
    let pos = ChunkPos::new(0, 0, 0);
    assert!(!store.mark_soft(pos));
    store.submit(pos, &flat(), 1, pos, &mut Vec::new());
    assert!(store.mark_full(pos));
    assert!(!store.mark_soft(pos));
}

#[test]
fn storing_an_equal_mask_reports_no_change() {
    let mut store = ChunkStore::new(16);
    let pos = ChunkPos::new(0, 0, 0);
    store.submit(pos, &flat(), 1, pos, &mut Vec::new());
    let mut mask = ShadowMask::new(2);
    mask.set(1, 1);
    assert!(store.store_mask(pos, &mask));
    assert!(!store.store_mask(pos, &mask));
    assert_eq!(store.mask(pos), Some(&mask));
}

#[test]
fn capacity_for_the_default_budget_counts_slot_and_mask_bytes() {
    let capacity = ChunkStore::capacity_for_budget(64 * 1024 * 1024, 2);
    assert!(capacity > 15_000 && capacity < 21_000, "{capacity}");
}

#[test]
fn chunks_of_another_dimension_are_evicted_first() {
    let mut store = ChunkStore::new(16);
    let player = ChunkPos::new(0, 0, 0);
    let mut evicted = Vec::new();
    store.submit(ChunkPos::new(0, 0, 1), &flat(), 1, player, &mut evicted);
    for x in 1..16 {
        store.submit(ChunkPos::new(x, 0, 0), &flat(), 1, player, &mut evicted);
    }
    store.submit(ChunkPos::new(-1, 0, 0), &flat(), 1, player, &mut evicted);
    assert!(!store.contains(ChunkPos::new(0, 0, 1)));
    assert!(store.contains(ChunkPos::new(15, 0, 0)));
}

#[test]
fn restore_inserts_a_clean_chunk_but_never_replaces_a_live_one() {
    let mut store = ChunkStore::new(16);
    let pos = ChunkPos::new(0, 0, 0);
    let disk = TestSurfaces::stored_flat(70, TestSurfaces::grey(10));
    assert!(store.restore(pos, &disk, pos, &mut Vec::new()));
    assert!(!store.is_journal_dirty(pos));
    assert_eq!(store.last_scan(pos), None);
    assert_eq!(store.surface(pos).map(|s| s.height[0]), Some(70));

    let live = ChunkPos::new(1, 0, 0);
    store.submit(live, &flat(), 1, pos, &mut Vec::new());
    assert!(!store.restore(live, &disk, pos, &mut Vec::new()));
    assert_eq!(store.surface(live).map(|s| s.height[0]), Some(64));
    assert!(!store.restore(ChunkPos::new(2, 0, 0), &ChunkSurface::empty(), pos, &mut Vec::new()));
}

#[test]
fn a_journal_ack_clears_the_mark_only_for_the_version_it_wrote() {
    let mut store = ChunkStore::new(16);
    let pos = ChunkPos::new(0, 0, 0);
    store.submit(pos, &flat(), 1, pos, &mut Vec::new());
    let written = store.journal_version(pos).unwrap();
    store.submit(pos, &TestSurfaces::probe_flat(65, TestSurfaces::grey(100)), 2, pos, &mut Vec::new());
    assert!(!store.mark_journaled_if(pos, written));
    assert!(store.is_journal_dirty(pos));
    assert!(store.mark_journaled_if(pos, store.journal_version(pos).unwrap()));
    assert!(!store.is_journal_dirty(pos));
}

#[test]
fn a_column_height_is_known_only_for_a_held_and_probed_column() {
    let mut store = ChunkStore::new(32);
    let mut surface = ChunkSurface::empty();
    surface.write_column(ChunkSurface::index(15, 0), ColumnSample::new(1, 0, 91, 91, false, 0));
    store.submit(ChunkPos::new(-1, 0, 0), &surface, 1, ChunkPos::new(0, 0, 0), &mut Vec::new());
    assert_eq!(store.column_height(-1, 0, 0), Some(91));
    assert_eq!(store.column_height(-2, 0, 0), None);
    assert_eq!(store.column_height(-1, 0, 1), None);
    assert_eq!(store.column_height(5, 0, 0), None);
}

#[test]
fn the_biome_of_a_held_column_is_read_by_block_position_at_negative_blocks() {
    let mut store = ChunkStore::new(32);
    let mut surface = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    surface.biome.set(ChunkSurface::index(15, 0), Some(BiomeKey(4)));
    store.submit(ChunkPos::new(-1, 0, 0), &surface, 1, ChunkPos::new(0, 0, 0), &mut Vec::new());
    assert_eq!(store.column_biome(-1, 0, 0), Some(BiomeKey(4)));
    assert_eq!(store.column_biome(-2, 0, 0), None);
    assert_eq!(store.column_biome(0, 0, 0), None);
    assert_eq!(store.column_biome(-1, 0, 1), None);
}

#[test]
fn the_memory_budget_counts_the_larger_surface() {
    assert_eq!(std::mem::size_of::<ChunkSurface>(), 3264);
    let cap = ChunkStore::capacity_for_budget(64 << 20, 1);
    assert!(cap * std::mem::size_of::<ChunkSurface>() <= 64 << 20);
}
