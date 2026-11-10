use minimap_core::services::memory::TileStore;
use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_core::structs::tile::Tile;

fn region_chunk(x: i32) -> ChunkPos {
    ChunkPos::new(x * 16, 0, 0)
}

fn region(x: i32) -> RegionPos {
    RegionPos::new(x, 0, 0)
}

fn load(store: &mut TileStore, x: i32, evicted: &mut Vec<(RegionPos, Tile)>) {
    store.insert_loaded(region(x), &vec![0u8; Tile::BYTES], 1, evicted).expect("valid length");
}

#[test]
fn clean_tiles_are_evicted_before_dirty_ones() {
    let mut store = TileStore::new(4, 25);
    let mut evicted = Vec::new();
    store.write_chunk(region_chunk(0), &[1; 256], 1, &mut evicted);
    for x in 1..4 {
        load(&mut store, x, &mut evicted);
    }
    load(&mut store, 4, &mut evicted);
    assert_eq!(store.len(), 4);
    assert!(store.peek(region(0)).is_some());
    assert!(store.peek(region(1)).is_none());
    assert!(evicted.is_empty());
}

#[test]
fn only_dirty_tiles_left_means_a_flush_request_and_a_capped_overrun() {
    let mut store = TileStore::new(4, 25);
    let mut evicted = Vec::new();
    for x in 0..5 {
        store.write_chunk(region_chunk(x), &[1; 256], 1, &mut evicted);
    }
    assert_eq!(store.len(), 5);
    assert_eq!(store.overrun(), 1);
    assert!(store.take_flush_request());
    assert!(!store.take_flush_request());
    assert!(evicted.is_empty());

    store.write_chunk(region_chunk(5), &[1; 256], 1, &mut evicted);
    assert_eq!(store.len(), 5);
    assert_eq!(evicted.len(), 1);
    assert_eq!(evicted[0].0, region(0));
    assert!(evicted[0].1.is_dirty());
}

#[test]
fn reading_a_tile_protects_it_from_eviction() {
    let mut store = TileStore::new(4, 0);
    let mut evicted = Vec::new();
    for x in 0..4 {
        load(&mut store, x, &mut evicted);
    }
    assert!(store.get(region(0)).is_some());
    load(&mut store, 4, &mut evicted);
    assert!(store.peek(region(0)).is_some());
    assert!(store.peek(region(1)).is_none());
}

#[test]
fn mark_clean_keeps_a_tile_written_after_the_flush_snapshot() {
    let mut store = TileStore::new(4, 0);
    let mut evicted = Vec::new();
    store.write_chunk(region_chunk(0), &[1; 256], 1, &mut evicted);
    let version = store.peek(region(0)).expect("present").version();
    store.write_chunk(region_chunk(0), &[2; 256], 1, &mut evicted);
    assert!(!store.mark_clean_if(region(0), version));
    let mut dirty = Vec::new();
    store.dirty_regions(&mut dirty);
    assert_eq!(dirty, vec![region(0)]);
}

#[test]
fn a_late_disk_load_never_covers_pixels_baked_before_it_arrived() {
    let mut store = TileStore::new(4, 0);
    let mut evicted = Vec::new();
    store.write_chunk(region_chunk(0), &[0xFF00_0022; 256], 1, &mut evicted);
    store.insert_loaded(region(0), &vec![0x55u8; Tile::BYTES], 1, &mut evicted).expect("valid length");
    let tile = store.peek(region(0)).expect("present");
    assert_eq!(tile.pixel(0, 0), 0xFF00_0022);
    assert_eq!(tile.pixel(200, 200), 0x5555_5555);
    assert!(tile.is_dirty());
}

#[test]
fn writing_a_chunk_reports_whether_the_tile_was_created() {
    let mut store = TileStore::new(4, 25);
    let mut evicted = Vec::new();
    assert!(store.write_chunk(ChunkPos::new(0, 0, 0), &[1; 256], 1, &mut evicted));
    assert!(!store.write_chunk(ChunkPos::new(1, 0, 0), &[2; 256], 1, &mut evicted));
    assert!(store.write_chunk(region_chunk(1), &[3; 256], 1, &mut evicted));
}

#[test]
fn clearing_drops_every_tile_and_the_flush_request() {
    let mut store = TileStore::new(TileStore::MIN_CAPACITY, 0);
    let mut evicted = Vec::new();
    for x in 0..=TileStore::MIN_CAPACITY as i32 {
        store.write_chunk(region_chunk(x), &[1; 256], 1, &mut evicted);
    }
    store.clear();
    assert_eq!(store.len(), 0);
    assert!(!store.take_flush_request());
    assert_eq!(store.capacity(), TileStore::MIN_CAPACITY);
}

#[test]
fn the_regions_held_in_memory_are_listed_for_one_dimension() {
    let mut store = TileStore::new(8, 0);
    let mut evicted = Vec::new();
    let pixels = vec![0u8; Tile::BYTES];
    for region in [RegionPos::new(1, 1, 0), RegionPos::new(-2, 0, 0), RegionPos::new(1, 1, 1)] {
        store.insert_loaded(region, &pixels, 7, &mut evicted).unwrap();
    }
    let mut out = vec![RegionPos::new(99, 99, 0)];
    store.regions(0, &mut out);
    out.sort_by_key(|r| r.x);
    assert_eq!(out, vec![RegionPos::new(-2, 0, 0), RegionPos::new(1, 1, 0)]);
}
