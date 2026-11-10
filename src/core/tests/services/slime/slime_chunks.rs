use minimap_core::services::slime::SlimeChunks;
use minimap_core::structs::pos::ChunkPos;

#[test]
fn the_reference_vectors_match() {
    assert!(SlimeChunks::is_slime(-1, 0));
    assert!(SlimeChunks::is_slime(109, 3));
    assert!(!SlimeChunks::is_slime(0, 0));
    assert!(!SlimeChunks::is_slime(110, 3));
}

#[test]
fn negative_blocks_use_floor_division_into_chunks() {
    let chunk = |x, z| ChunkPos::from_block(x, z, 0);
    let c = chunk(-8, 8);
    assert!(SlimeChunks::is_slime(c.x, c.z));
    let c = chunk(-1, 0);
    assert!(SlimeChunks::is_slime(c.x, c.z));
    let c = chunk(8, 8);
    assert!(!SlimeChunks::is_slime(c.x, c.z));
}

#[test]
fn about_one_chunk_in_ten_is_a_slime_chunk() {
    let n =
        (-50..50).flat_map(|x| (-50..50).map(move |z| (x, z))).filter(|&(x, z)| SlimeChunks::is_slime(x, z)).count();
    assert!((800..=1200).contains(&n), "{n} of 10000");
}

#[test]
fn a_region_mask_agrees_with_the_chunk_test_at_negative_regions() {
    let mask = SlimeChunks::region_mask(-1, 0);
    for lz in 0..16 {
        for lx in 0..16 {
            let bit = (mask[(lz * 16 + lx) / 64] >> ((lz * 16 + lx) % 64)) & 1 == 1;
            assert_eq!(bit, SlimeChunks::is_slime(-16 + lx as i32, lz as i32));
        }
    }
}
