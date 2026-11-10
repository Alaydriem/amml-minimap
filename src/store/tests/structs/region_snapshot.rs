use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_core::structs::tile::Tile;
use minimap_store::structs::{JournalRecord, RegionSnapshot};

use crate::support::TestRecords;

fn pixel(snapshot: &RegionSnapshot, x: usize, z: usize) -> u32 {
    let i = (z * Tile::SIDE + x) * 4;
    u32::from_le_bytes(snapshot.tile[i..i + 4].try_into().unwrap())
}

#[test]
fn a_record_lands_in_its_chunk_slot_and_its_patch_in_the_tile() {
    let region = RegionPos::new(-1, 0, 0);
    let mut snapshot = RegionSnapshot::new(region, 1);
    snapshot.apply(&TestRecords::record(ChunkPos::new(-15, 2, 0), 70, 0xFF11_2233));
    assert_eq!(snapshot.surface_count(), 1);
    assert!(snapshot.surfaces[2 * 16 + 1].is_some());
    assert_eq!(pixel(&snapshot, 16, 32), 0xFF11_2233);
    assert_eq!(pixel(&snapshot, 15, 32), 0);
    assert_eq!(snapshot.present_bits(), [1 << 33, 0, 0, 0]);
    assert_eq!(snapshot.settings_hash, 7);
}

#[test]
fn a_record_without_a_patch_keeps_the_tile_and_its_hash() {
    let mut snapshot = RegionSnapshot::new(RegionPos::new(0, 0, 0), 1);
    let record = JournalRecord::new(ChunkPos::new(0, 0, 0), 9, TestRecords::surface(64), None);
    snapshot.apply(&record);
    assert!(!snapshot.has_pixels());
    assert_eq!(snapshot.settings_hash, 1);
    assert!(!snapshot.is_empty());
}

#[test]
fn a_new_snapshot_is_empty() {
    assert!(RegionSnapshot::new(RegionPos::new(0, 0, 0), 1).is_empty());
}
