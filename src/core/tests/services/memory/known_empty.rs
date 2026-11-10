use minimap_core::services::memory::KnownEmptySet;
use minimap_core::structs::pos::RegionPos;

#[test]
fn panning_over_many_empty_regions_stays_within_capacity() {
    let mut set = KnownEmptySet::new(8);
    for x in 0..1000 {
        set.insert(RegionPos::new(x, 0, 0));
    }
    assert_eq!(set.len(), 8);
    assert!(set.contains(RegionPos::new(999, 0, 0)));
    assert!(!set.contains(RegionPos::new(0, 0, 0)));
}

#[test]
fn a_region_that_gains_data_is_removed() {
    let mut set = KnownEmptySet::new(8);
    set.insert(RegionPos::new(1, 1, 0));
    set.remove(RegionPos::new(1, 1, 0));
    assert!(!set.contains(RegionPos::new(1, 1, 0)));
}

#[test]
fn clearing_forgets_every_region() {
    let mut set = KnownEmptySet::new(8);
    set.insert(RegionPos::new(1, 1, 0));
    set.insert(RegionPos::new(2, 1, 0));
    set.clear();
    assert!(set.is_empty());
    assert!(!set.contains(RegionPos::new(1, 1, 0)));
}
