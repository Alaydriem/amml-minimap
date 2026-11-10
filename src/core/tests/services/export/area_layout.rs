use minimap_core::services::export::AreaLayout;
use minimap_core::structs::pos::RegionPos;

fn regions(places: &[(i32, i32)]) -> Vec<RegionPos> {
    places.iter().map(|&(x, z)| RegionPos::new(x, z, 0)).collect()
}

#[test]
fn the_bounding_box_of_the_regions_is_drawn_at_one_block_per_pixel_when_it_fits() {
    let layout = AreaLayout::new(&regions(&[(-2, 3), (1, -1), (0, 0)]), 16384).unwrap();
    assert_eq!((layout.x0, layout.x1, layout.z0, layout.z1), (-2, 1, -1, 3));
    assert_eq!((layout.blocks_per_pixel, layout.width, layout.height), (1, 4 * 256, 5 * 256));
    assert_eq!(layout.region_pixels(), 256);
    assert_eq!(layout.row_bytes(), 4 * 256 * 256 * 4);
}

#[test]
fn a_wide_area_halves_the_scale_until_it_fits() {
    let layout = AreaLayout::new(&regions(&[(0, 0), (69, 0)]), 16384).unwrap();
    assert_eq!((layout.blocks_per_pixel, layout.width, layout.height), (2, 70 * 128, 128));
    let tall = AreaLayout::new(&regions(&[(0, -300), (0, 0)]), 16384).unwrap();
    assert_eq!((tall.blocks_per_pixel, tall.height), (8, 301 * 32));
    assert!(tall.height <= 16384);
}

#[test]
fn nothing_or_an_area_too_big_even_at_one_pixel_per_region_gives_no_layout() {
    assert_eq!(AreaLayout::new(&[], 16384), None);
    assert_eq!(AreaLayout::new(&regions(&[(0, 0), (256, 0)]), 256), None);
    assert!(AreaLayout::new(&regions(&[(0, 0), (255, 0)]), 256).is_some());
}
