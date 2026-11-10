use minimap_core::structs::draw::ExploredChunks;

#[test]
fn chunks_are_found_in_their_region_at_negative_coordinates() {
    let mut explored = ExploredChunks::new();
    explored.begin((-2, -1), 2);
    // regions (-2, -1), (-1, -1), (-2, 0), (-1, 0); in (-1, -1) the chunk at local (15, 15)
    explored.push([0; 4]);
    explored.push([0, 0, 0, 1 << 63]);
    explored.push([0; 4]);
    explored.push([0; 4]);

    assert!(explored.contains(-1, -1));
    assert!(!explored.contains(-2, -1));
    assert!(!explored.contains(0, 0), "outside the block");
    assert!(!explored.contains(-1, 30), "below the block");
    assert!(ExploredChunks::everywhere().contains(1000, -1000));
}
