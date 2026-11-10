use minimap_core::services::bake::GradientStyle;
use minimap_core::structs::Rgba;
use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::pos::ChunkPos;

use crate::support::{FakeEnvironment, TestSurfaces};

fn shade(center: &ChunkSurface, env: &FakeEnvironment) -> [Rgba; 256] {
    let mut colours = [Rgba::new(100, 100, 100, 255); 256];
    GradientStyle::apply(ChunkPos::new(0, 0, 0), center, env, 130, &mut colours);
    colours
}

#[test]
fn flat_terrain_is_not_shaded_anywhere_in_the_chunk() {
    let colours = shade(&TestSurfaces::stored_flat(64, TestSurfaces::grey(100)), &FakeEnvironment::new());
    assert!(colours.iter().all(|c| *c == Rgba::new(100, 100, 100, 255)));
}

#[test]
fn a_raised_inner_column_is_brightened_and_its_east_neighbour_darkened() {
    let center = TestSurfaces::with_height(TestSurfaces::stored_flat(64, TestSurfaces::grey(100)), 5, 5, 70);
    let colours = shade(&center, &FakeEnvironment::new());
    assert_eq!(colours[ChunkSurface::index(5, 5)], Rgba::new(130, 130, 130, 255));
    assert_eq!(colours[ChunkSurface::index(6, 5)], Rgba::new(76, 76, 76, 255));
    assert_eq!(colours[ChunkSurface::index(4, 5)], Rgba::new(100, 100, 100, 255));
}

#[test]
fn the_west_edge_compares_with_the_west_neighbour_chunk() {
    let center = TestSurfaces::stored_flat(64, TestSurfaces::grey(100));
    let env = FakeEnvironment::new()
        .with_surface(ChunkPos::new(-1, 0, 0), TestSurfaces::stored_flat(80, TestSurfaces::grey(100)));
    let colours = shade(&center, &env);
    assert_eq!(colours[ChunkSurface::index(0, 3)], Rgba::new(76, 76, 76, 255));
    assert_eq!(colours[ChunkSurface::index(1, 3)], Rgba::new(100, 100, 100, 255));
}
