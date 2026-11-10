use minimap_core::services::draw::MinimapComposer;
use minimap_core::services::memory::{ChunkStore, TileStore};
use minimap_core::structs::pos::ChunkPos;

use crate::support::TestSurfaces;

fn pixel(composer: &MinimapComposer, block: (i32, i32)) -> u32 {
    let (ox, oz) = composer.origin();
    let i = (((block.1 - oz) as usize) * composer.side() + (block.0 - ox) as usize) * 4;
    u32::from_le_bytes(composer.image()[i..i + 4].try_into().unwrap())
}

#[test]
fn baked_pixels_win_and_unbaked_chunks_show_their_surface_colour() {
    let mut tiles = TileStore::new(8, 0);
    tiles.write_chunk(ChunkPos::new(0, 0, 0), &[0xFF00_00AA; 256], 1, &mut Vec::new());
    let mut chunks = ChunkStore::new(16);
    let surface = TestSurfaces::probe_flat(64, 0xFF00_BB00);
    chunks.submit(ChunkPos::new(-1, 0, 0), &surface, 1, ChunkPos::new(0, 0, 0), &mut Vec::new());
    let mut composer = MinimapComposer::new();
    assert!(composer.update((0, 0), 0, 2, &tiles, Some(&chunks)));
    assert_eq!(composer.side(), 66);
    assert_eq!(pixel(&composer, (3, 3)), 0xFF00_00AA);
    assert_eq!(pixel(&composer, (-3, 3)), 0xFF00_BB00);
    assert_eq!(pixel(&composer, (3, -3)), 0);
}

#[test]
fn the_image_is_rebuilt_only_when_something_it_shows_changes() {
    let mut tiles = TileStore::new(8, 0);
    let chunks = ChunkStore::new(16);
    let mut composer = MinimapComposer::new();
    assert!(composer.update((5, 5), 0, 2, &tiles, Some(&chunks)));
    assert!(!composer.update((5, 5), 0, 2, &tiles, Some(&chunks)));
    tiles.write_chunk(ChunkPos::new(0, 0, 0), &[1; 256], 1, &mut Vec::new());
    assert!(composer.update((5, 5), 0, 2, &tiles, Some(&chunks)));
    assert!(composer.update((6, 5), 0, 2, &tiles, Some(&chunks)));
    assert!(composer.update((6, 5), 0, 3, &tiles, Some(&chunks)));
    assert!(composer.update((6, 5), 1, 3, &tiles, Some(&chunks)));
}

#[test]
fn the_image_spans_region_edges_at_negative_positions() {
    let mut tiles = TileStore::new(8, 0);
    tiles.write_chunk(ChunkPos::new(-1, -1, 0), &[0xFF11_1111; 256], 1, &mut Vec::new());
    tiles.write_chunk(ChunkPos::new(0, 0, 0), &[0xFF22_2222; 256], 1, &mut Vec::new());
    let mut composer = MinimapComposer::new();
    composer.update((0, 0), 0, 1, &tiles, None::<&ChunkStore>);
    assert_eq!(pixel(&composer, (-1, -1)), 0xFF11_1111);
    assert_eq!(pixel(&composer, (0, 0)), 0xFF22_2222);
}
