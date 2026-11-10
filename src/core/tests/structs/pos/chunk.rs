use minimap_core::structs::pos::{ChunkPos, RegionPos};

#[test]
fn from_block_floors_negative_coordinates() {
    assert_eq!(ChunkPos::from_block(-1, -1, 0), ChunkPos::new(-1, -1, 0));
    assert_eq!(ChunkPos::from_block(-16, -17, 0), ChunkPos::new(-1, -2, 0));
    assert_eq!(ChunkPos::from_block(15, 16, 0), ChunkPos::new(0, 1, 0));
}

#[test]
fn region_and_local_position_of_negative_chunks() {
    assert_eq!(ChunkPos::new(-1, -16, 0).region(), RegionPos::new(-1, -1, 0));
    assert_eq!(ChunkPos::new(-1, -16, 0).local_in_region(), (15, 0));
    assert_eq!(ChunkPos::new(-17, 16, 0).region(), RegionPos::new(-2, 1, 0));
    assert_eq!(ChunkPos::new(-17, 16, 0).local_in_region(), (15, 0));
}

#[test]
fn distance_squared_counts_both_axes() {
    assert_eq!(ChunkPos::new(-2, 3, 0).distance_squared(&ChunkPos::new(1, -1, 0)), 25);
}

#[test]
fn positions_at_the_world_border_do_not_overflow() {
    let west = ChunkPos::from_block(-30_000_000, 30_000_000, 0);
    assert_eq!(west, ChunkPos::new(-1_875_000, 1_875_000, 0));
    assert_eq!(west.min_block_x(), -30_000_000);
    assert_eq!(west.region(), RegionPos::new(-117_188, 117_187, 0));
    assert_eq!(west.distance_squared(&ChunkPos::new(1_875_000, -1_875_000, 0)), 2 * 3_750_000i64 * 3_750_000);
}
