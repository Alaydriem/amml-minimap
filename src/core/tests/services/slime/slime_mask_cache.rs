use minimap_core::services::slime::SlimeMaskCache;

#[test]
fn the_mask_cache_holds_its_capacity_and_a_new_region_evicts_the_old_one() {
    let mut cache = SlimeMaskCache::new(1);
    let mut budget = 10;
    assert!(cache.mask(0, 0, &mut budget).is_some());
    assert!(cache.mask(1, 0, &mut budget).is_some());
    assert_eq!(cache.len(), 1);
    let mut none = 0;
    assert!(cache.mask(0, 0, &mut none).is_none());
    assert!(cache.mask(1, 0, &mut none).is_some());
}

#[test]
fn misses_stop_when_the_frame_budget_is_spent() {
    let mut cache = SlimeMaskCache::new(64);
    let mut budget = 2;
    assert!(cache.mask(0, 0, &mut budget).is_some());
    assert!(cache.mask(0, 1, &mut budget).is_some());
    assert!(cache.mask(0, 2, &mut budget).is_none());
    assert_eq!(budget, 0);
}

#[test]
fn every_region_of_a_view_stays_cached_when_the_cache_is_larger_than_the_view() {
    let mut cache = SlimeMaskCache::new(1024);
    let regions: Vec<(i32, i32)> = (-11..11).flat_map(|x| (-7..7).map(move |z| (x, z))).collect();
    let mut budget = usize::MAX;
    for &(x, z) in &regions {
        assert!(cache.mask(x, z, &mut budget).is_some());
    }
    let mut none = 0;
    let missing = regions.iter().filter(|&&(x, z)| cache.mask(x, z, &mut none).is_none()).count();
    assert_eq!(missing, 0, "regions evicted by their neighbours");
    assert_eq!(cache.len(), regions.len());
}
