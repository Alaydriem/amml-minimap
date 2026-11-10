use minimap_core::services::bake::{Bevel, BevelTable};
use minimap_core::structs::Rgba;
use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::pos::ChunkPos;

use crate::support::{FakeEnvironment, TestSurfaces};

fn bevel(center: &ChunkSurface, scale: u8) -> [Rgba; 256] {
    let mut colours = [Rgba::new(100, 100, 100, 255); 256];
    Bevel::apply(ChunkPos::new(0, 0, 0), center, &FakeEnvironment::new(), &BevelTable::new(scale), &mut colours);
    colours
}

#[test]
fn equal_neighbours_leave_the_colour_unchanged() {
    let colours = bevel(&TestSurfaces::stored_flat(64, TestSurfaces::grey(100)), 2);
    assert!(colours.iter().all(|c| *c == Rgba::new(100, 100, 100, 255)));
}

// 100 * 1.18f32 is 117.99999, which truncates to 117
#[test]
fn at_scale_one_a_lower_west_neighbour_brightens_and_a_higher_one_darkens() {
    let pit = TestSurfaces::with_height(TestSurfaces::stored_flat(64, TestSurfaces::grey(100)), 4, 4, 60);
    let colours = bevel(&pit, 1);
    assert_eq!(colours[ChunkSurface::index(5, 4)], Rgba::new(117, 117, 117, 255));
    assert_eq!(colours[ChunkSurface::index(4, 5)], Rgba::new(117, 117, 117, 255));
}

#[test]
fn a_pit_with_four_higher_neighbours_is_darkened_to_the_clamp_floor() {
    let pit = TestSurfaces::with_height(TestSurfaces::stored_flat(64, TestSurfaces::grey(100)), 4, 4, 60);
    let colours = bevel(&pit, 1);
    assert_eq!(colours[ChunkSurface::index(4, 4)], Rgba::new(55, 55, 55, 255));
}

#[test]
fn a_west_and_north_step_down_add_the_corner_boost() {
    let flat = TestSurfaces::stored_flat(64, TestSurfaces::grey(100));
    let lowered = TestSurfaces::with_height(TestSurfaces::with_height(flat, 4, 5, 60), 5, 4, 60);
    let colours = bevel(&lowered, 1);
    assert_eq!(colours[ChunkSurface::index(5, 5)], Rgba::new(149, 149, 149, 255));
}

#[test]
fn table_factor_is_one_for_all_equal_relations() {
    for scale in [1, 2, 4, 16] {
        assert_eq!(BevelTable::new(scale).factor(0, 0, 0, 0), 1.0);
    }
}
