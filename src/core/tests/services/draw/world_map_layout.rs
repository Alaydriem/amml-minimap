use minimap_core::services::draw::WorldMapLayout;
use minimap_core::structs::pos::ChunkPos;

#[test]
fn visible_regions_cover_the_screen_including_negative_ones() {
    // a screen edge exactly on a region boundary includes that region
    assert_eq!(WorldMapLayout::visible((0.0, 0.0), 1.0, (1024.0, 512.0)), (-2, 2, -1, 1));
    assert_eq!(WorldMapLayout::visible((0.0, 0.0), 0.2, (1920.0, 1080.0)), (-19, 18, -11, 10));
}

#[test]
fn the_centre_block_maps_to_the_screen_centre() {
    let p = WorldMapLayout::to_screen((100.0, -50.0), (100.0, -50.0), 3.0, (800.0, 600.0));
    assert_eq!(p, (400.0, 300.0));
    let q = WorldMapLayout::to_screen((110.0, -50.0), (100.0, -50.0), 3.0, (800.0, 600.0));
    assert_eq!(q, (430.0, 300.0));
}

#[test]
fn a_turned_screen_needs_the_regions_under_its_diagonal() {
    let flat = WorldMapLayout::visible((0.0, 0.0), 1.0, (800.0, 200.0));
    let turned = WorldMapLayout::visible_turned((0.0, 0.0), 1.0, (800.0, 200.0));
    assert_eq!(flat, (-2, 1, -1, 0));
    assert_eq!(turned, (-2, 1, -2, 1));
}

#[test]
fn a_screen_point_maps_back_to_the_world_block_under_it_at_negative_positions() {
    let world = WorldMapLayout::to_world((370.0, 300.0), (-100.0, 40.0), 3.0, (800.0, 600.0));
    assert_eq!(world, (-110.0, 40.0));
    let back = WorldMapLayout::to_screen(world, (-100.0, 40.0), 3.0, (800.0, 600.0));
    assert_eq!(back, (370.0, 300.0));
}

#[test]
fn a_point_left_of_the_origin_is_block_minus_one_and_chunk_minus_one() {
    let b = WorldMapLayout::block_at((399.5, 300.0), (0.0, 0.0), 1.0, (800.0, 600.0), 0.0);
    assert_eq!(b, (-1, 0));
    assert_eq!(ChunkPos::from_block(b.0, b.1, 0), ChunkPos::new(-1, 0, 0));
}

#[test]
fn chunk_boundaries_fall_on_whole_blocks() {
    assert_eq!(WorldMapLayout::block_at((400.0, 300.0), (16.0, -16.0), 2.0, (800.0, 600.0), 0.0), (16, -16));
    assert_eq!(WorldMapLayout::block_at((399.0, 299.0), (16.0, -16.0), 2.0, (800.0, 600.0), 0.0), (15, -17));
}

#[test]
fn a_turned_map_is_turned_back_before_the_block_is_found() {
    let half_turn = std::f32::consts::FRAC_PI_2;
    assert_eq!(WorldMapLayout::block_at((403.0, 287.5), (0.0, 0.0), 1.0, (800.0, 600.0), half_turn), (-13, -3));
}

#[test]
fn far_coordinates_keep_whole_block_precision() {
    let b = WorldMapLayout::block_at((400.0, 300.0), (29_999_999.5, -29_999_999.5), 1.0, (800.0, 600.0), 0.0);
    assert_eq!(b, (29_999_999, -30_000_000));
}

#[test]
fn visible_chunks_cover_the_screen_and_the_diagonal_when_turned() {
    assert_eq!(WorldMapLayout::visible_chunks((0.0, 0.0), 1.0, (800.0, 600.0), 0.0), (-25, 25, -19, 18));
    assert_eq!(WorldMapLayout::visible_chunks((0.0, 0.0), 1.0, (800.0, 600.0), 0.3), (-32, 31, -32, 31));
}
