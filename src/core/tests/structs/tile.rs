use minimap_core::errors::TileError;
use minimap_core::structs::tile::Tile;

#[test]
fn chunk_pixels_land_at_their_place_in_the_region() {
    let mut tile = Tile::new(1);
    let mut pixels = [0u32; 256];
    pixels[0] = 0x11;
    pixels[255] = 0x22;
    tile.write_chunk(1, 2, &pixels, 9);
    assert_eq!(tile.pixel(16, 32), 0x11);
    assert_eq!(tile.pixel(31, 47), 0x22);
    assert_eq!(tile.pixel(15, 32), 0);
    assert!(tile.is_dirty());
    assert_eq!(tile.settings_hash(), 9);
}

#[test]
fn a_write_after_the_flush_snapshot_keeps_the_tile_dirty() {
    let mut tile = Tile::new(1);
    tile.write_chunk(0, 0, &[1; 256], 1);
    let snapshot = tile.version();
    tile.write_chunk(0, 0, &[2; 256], 1);
    assert!(!tile.mark_clean_if(snapshot));
    assert!(tile.is_dirty());
    assert!(tile.mark_clean_if(tile.version()));
    assert!(!tile.is_dirty());
}

#[test]
fn replacing_pixels_rejects_a_wrong_length() {
    let mut tile = Tile::new(1);
    assert_eq!(tile.replace_pixels(&[0; 10], 1), Err(TileError::WrongLength(10)));
    assert!(tile.replace_pixels(&vec![7u8; Tile::BYTES], 3).is_ok());
    assert!(!tile.is_dirty());
    assert_eq!(tile.pixel(0, 0), 0x0707_0707);
}

#[test]
fn merging_older_data_fills_only_transparent_pixels() {
    let mut tile = Tile::new(1);
    tile.write_chunk(0, 0, &[0xFF00_0011; 256], 1);
    let older = vec![0xAAu8; Tile::BYTES];
    assert_eq!(tile.merge_under(&older), Ok(true));
    assert_eq!(tile.pixel(0, 0), 0xFF00_0011);
    assert_eq!(tile.pixel(16, 0), 0xAAAA_AAAA);
    assert_eq!(tile.merge_under(&older), Ok(false));
}
