use minimap_core::services::draw::{GpuResidency, RegionOrder};
use minimap_core::structs::draw::ResidencyDecision;
use minimap_core::structs::pos::RegionPos;

// a 20 x 20 grid of regions around the origin, row by row
fn grid() -> Vec<RegionPos> {
    (-10..10).flat_map(|z| (-10..10).map(move |x| RegionPos::new(x, z, 0))).collect()
}

#[test]
fn the_region_under_the_centre_comes_first_and_the_corners_last() {
    let regions = grid();
    let mut order = Vec::new();

    RegionOrder::nearest_first(&regions, (128.0, 128.0), &mut order);

    assert_eq!(order.len(), regions.len());
    assert_eq!(regions[order[0]], RegionPos::new(0, 0, 0));
    let last = regions[order[order.len() - 1]];
    assert!(last.x.abs() == 10 || last.z.abs() == 10, "{last:?}");
}

#[test]
fn nearest_regions_win_when_over_budget() {
    let regions = grid();
    let mut order = Vec::new();
    RegionOrder::nearest_first(&regions, (128.0, 128.0), &mut order);
    let mut gpu = GpuResidency::new(240, 600, 4096);
    let mut ops = Vec::new();
    gpu.begin_frame(1);

    let resident: Vec<RegionPos> = order
        .iter()
        .map(|&i| regions[i])
        .filter(|region| matches!(gpu.decide(*region, Some(1), &mut ops), ResidencyDecision::Upload { .. }))
        .collect();

    assert_eq!(resident.len(), 240);
    let far = |r: &RegionPos| f64::from(r.x).hypot(f64::from(r.z));
    let nearest_left_out = order.iter().skip(240).map(|&i| far(&regions[i])).fold(f64::MAX, f64::min);
    assert!(resident.iter().all(|r| far(r) <= nearest_left_out));
}
