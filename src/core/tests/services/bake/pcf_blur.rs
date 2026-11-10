use minimap_core::services::bake::PcfBlur;
use minimap_core::structs::Rgba;
use minimap_core::structs::chunk::{ChunkSurface, ShadowMask};
use minimap_core::structs::pos::ChunkPos;

use crate::support::FakeEnvironment;

fn blur(mask: &ShadowMask, env: &FakeEnvironment, radius: usize) -> [Rgba; 256] {
    let mut colours = [Rgba::new(200, 200, 200, 255); 256];
    PcfBlur::new().apply(ChunkPos::new(0, 0, 0), mask, env, radius, &mut colours);
    colours
}

fn grey(level: u8) -> Rgba {
    Rgba::new(level, level, level, 255)
}

#[test]
fn without_blur_a_column_is_darkened_by_its_share_of_shadowed_sub_pixels() {
    let mut mask = ShadowMask::new(2);
    mask.set(10, 10);
    let colours = blur(&mask, &FakeEnvironment::new(), 0);
    assert_eq!(colours[ChunkSurface::index(5, 5)], grey(174));
    assert_eq!(colours[ChunkSurface::index(6, 5)], grey(200));
}

#[test]
fn radius_one_spreads_one_shadowed_cell_over_its_neighbours() {
    let mut mask = ShadowMask::new(1);
    mask.set(5, 5);
    let colours = blur(&mask, &FakeEnvironment::new(), 1);
    assert_eq!(colours[ChunkSurface::index(5, 5)], grey(188));
    assert_eq!(colours[ChunkSurface::index(5, 4)], grey(188));
    assert_eq!(colours[ChunkSurface::index(4, 6)], grey(188));
    assert_eq!(colours[ChunkSurface::index(7, 5)], grey(200));
}

#[test]
fn samples_in_a_missing_neighbour_are_not_counted() {
    let mut mask = ShadowMask::new(1);
    mask.set(0, 5);
    let colours = blur(&mask, &FakeEnvironment::new(), 1);
    assert_eq!(colours[ChunkSurface::index(0, 5)], grey(182));
}

#[test]
fn a_neighbour_mask_at_another_scale_is_treated_as_missing() {
    let mut mask = ShadowMask::new(1);
    mask.set(0, 5);
    let env = FakeEnvironment::new().with_mask(ChunkPos::new(-1, 0, 0), ShadowMask::new(2));
    let colours = blur(&mask, &env, 1);
    assert_eq!(colours[ChunkSurface::index(0, 5)], grey(182));
}

#[test]
fn shadow_in_the_north_neighbour_reaches_the_first_row() {
    let mask = ShadowMask::new(1);
    let mut north = ShadowMask::new(1);
    north.set(3, 15);
    let env = FakeEnvironment::new().with_mask(ChunkPos::new(0, -1, 0), north);
    let colours = blur(&mask, &env, 1);
    assert_eq!(colours[ChunkSurface::index(3, 0)], grey(188));
    assert_eq!(colours[ChunkSurface::index(3, 1)], grey(200));
}
