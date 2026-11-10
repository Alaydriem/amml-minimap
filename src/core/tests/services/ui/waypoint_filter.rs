use minimap_core::services::ui::WaypointFilter;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::waypoint::WaypointSource;

#[test]
fn the_filter_is_case_insensitive_substring_of_the_name() {
    let mut f = WaypointFilter::new();
    f.set_query("BA");
    assert!(f.matches("Base") && f.matches("abandoned base") && f.matches("Bastion"));
    assert!(!f.matches("Farm"));
    f.set_query("\u{e9}co");
    assert!(f.matches("\u{c9}COLE"));
    f.set_query("");
    assert!(f.matches("anything"));
}

#[test]
fn collect_keeps_store_order_in_one_dimension() {
    let mut store = WaypointStore::new(8, 1);
    store.add("Base", 0, (0, 64, 0), WaypointSource::Manual, 1).unwrap();
    store.add("Base", -1, (0, 64, 0), WaypointSource::Manual, 1).unwrap();
    store.add("Farm", 0, (5, 64, 5), WaypointSource::Manual, 1).unwrap();
    let mut f = WaypointFilter::new();
    let mut out = Vec::new();
    f.collect(store.waypoints(), 0, &mut out);
    assert_eq!(out, vec![0, 2]);
    f.set_query("far");
    f.collect(store.waypoints(), 0, &mut out);
    assert_eq!(out, vec![2]);
    f.set_query("nothing");
    f.collect(store.waypoints(), 0, &mut out);
    assert!(out.is_empty());
}

#[test]
fn count_in_counts_one_dimension_whatever_the_query() {
    let mut store = minimap_core::services::waypoints::WaypointStore::new(8, 1);
    for (dim, x) in [(0, 0), (0, 16), (0, 32), (1, 0), (1, 16)] {
        store.add("w", dim, (x, 64, 0), minimap_core::structs::waypoint::WaypointSource::Manual, 0).unwrap();
    }

    assert_eq!(WaypointFilter::count_in(store.waypoints(), 1), 2);
    assert_eq!(WaypointFilter::count_in(store.waypoints(), 0), 3);
    assert_eq!(WaypointFilter::count_in(store.waypoints(), 2), 0);
}
