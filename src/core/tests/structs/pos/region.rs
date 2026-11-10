use minimap_core::structs::pos::{ChunkPos, RegionPos};

#[test]
fn from_block_floors_negative_coordinates() {
    assert_eq!(RegionPos::from_block(-1, 255, 1), RegionPos::new(-1, 0, 1));
    assert_eq!(RegionPos::from_block(-257, 256, 1), RegionPos::new(-2, 1, 1));
}

#[test]
fn chunk_and_contains_agree_for_negative_regions() {
    let region = RegionPos::new(-1, -1, 0);
    let chunk = region.chunk(15, 0);
    assert_eq!(chunk, ChunkPos::new(-1, -16, 0));
    assert!(region.contains(&chunk));
    assert!(!region.contains(&ChunkPos::new(0, -16, 0)));
}
