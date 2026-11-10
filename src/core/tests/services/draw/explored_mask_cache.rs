use minimap_core::services::draw::ExploredMaskCache;
use minimap_core::structs::pos::RegionPos;
use minimap_core::structs::tile::Tile;

fn tile_with(chunks: &[(usize, usize)]) -> Tile {
    let mut tile = Tile::new(1);
    for &(x, z) in chunks {
        tile.write_chunk(x, z, &[0xFF33_6699; 256], 1);
    }
    tile
}

#[test]
fn a_chunk_is_explored_once_its_pixels_are_baked() {
    let mask = ExploredMaskCache::read(&tile_with(&[(0, 0), (15, 15), (3, 1)]));

    assert_eq!(mask[0], 1 | (1 << 19));
    assert_eq!(mask[3], 1 << 63);
    assert_eq!(mask[1] | mask[2], 0);
}

#[test]
fn the_mask_is_kept_while_the_tile_is_busy_and_read_again_when_it_changes() {
    let mut cache = ExploredMaskCache::new(4);
    let region = RegionPos::new(-2, 3, 0);
    let mut tile = tile_with(&[(0, 0)]);

    let first = cache.mask(region, Some(&tile));
    let busy = cache.mask(region, None);
    tile.write_chunk(1, 0, &[0xFF00_0000; 256], 1);
    let changed = cache.mask(region, Some(&tile));

    assert_eq!((first, busy), ([1, 0, 0, 0], [1, 0, 0, 0]));
    assert_eq!(changed, [0b11, 0, 0, 0]);
    assert_eq!(cache.mask(RegionPos::new(9, 9, 0), None), [0; 4], "an unknown region is unexplored");
}

#[test]
fn the_cache_keeps_at_most_its_capacity() {
    let mut cache = ExploredMaskCache::new(3);
    let tile = tile_with(&[(0, 0)]);

    for x in 0..5 {
        cache.mask(RegionPos::new(x, 0, 0), Some(&tile));
    }

    assert_eq!(cache.len(), 3);
    assert_eq!(cache.mask(RegionPos::new(0, 0, 0), None), [0; 4], "the oldest region was dropped");
}
