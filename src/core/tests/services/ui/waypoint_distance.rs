use minimap_core::services::ui::WaypointDistance;

#[test]
fn distance_is_across_the_ground_and_rounded() {
    assert_eq!(WaypointDistance::blocks((0, 64, 0), (3, 10, 4)), 5);
    assert_eq!(WaypointDistance::blocks((-10, 0, -10), (-10, 200, -10)), 0);
    assert_eq!(WaypointDistance::blocks((214, 70, -88), (342, 70, -31)), 140);
}

#[test]
fn three_blocks_or_less_is_here() {
    assert!(WaypointDistance::is_here(WaypointDistance::HERE_BLOCKS));
    assert!(!WaypointDistance::is_here(WaypointDistance::HERE_BLOCKS + 1));
}
