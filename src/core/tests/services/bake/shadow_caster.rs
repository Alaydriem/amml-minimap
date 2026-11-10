use minimap_core::services::bake::ShadowCaster;
use minimap_core::structs::LightDirection;
use minimap_core::structs::chunk::{ChunkSurface, ShadowMask};
use minimap_core::structs::pos::ChunkPos;

use crate::support::{FakeEnvironment, TestSurfaces};

fn light() -> LightDirection {
    LightDirection::new(315.0, 60.0)
}

fn cast(pos: ChunkPos, center: &ChunkSurface, env: &FakeEnvironment, scale: u8) -> ShadowMask {
    let mut mask = ShadowMask::new(0);
    ShadowCaster::cast(pos, center, env, light(), scale, &mut mask);
    mask
}

#[test]
fn flat_terrain_casts_no_shadow() {
    let mask = cast(ChunkPos::new(0, 0, 0), &TestSurfaces::stored_flat(64, 1), &FakeEnvironment::new(), 2);
    for sz in 0..32 {
        for sx in 0..32 {
            assert!(!mask.get(sx, sz));
        }
    }
}

#[test]
fn a_pillar_shadows_the_column_downstream_and_not_the_one_upstream() {
    let center = TestSurfaces::with_height(TestSurfaces::stored_flat(64, 1), 5, 5, 100);
    let mask = cast(ChunkPos::new(0, 0, 0), &center, &FakeEnvironment::new(), 1);
    assert!(mask.get(6, 6));
    assert!(!mask.get(4, 4));
    assert!(!mask.get(5, 5));
}

#[test]
fn a_pillar_in_the_west_neighbour_shadows_across_the_chunk_edge() {
    let center = TestSurfaces::stored_flat(64, 1);
    let west = TestSurfaces::with_height(TestSurfaces::stored_flat(64, 1), 15, 7, 100);
    let env = FakeEnvironment::new().with_surface(ChunkPos::new(-1, 0, 0), west);
    let mask = cast(ChunkPos::new(0, 0, 0), &center, &env, 1);
    assert!(mask.get(0, 8));
}

#[test]
fn a_missing_neighbour_reads_as_deep_ground() {
    let center = TestSurfaces::stored_flat(-70, 1);
    let mask = cast(ChunkPos::new(0, 0, 0), &center, &FakeEnvironment::new(), 1);
    assert!(mask.get(0, 0));
}

#[test]
fn rays_stop_at_the_dimension_top() {
    let center = TestSurfaces::with_height(TestSurfaces::stored_flat(126, 1), 5, 5, 200);
    let nether = cast(ChunkPos::new(0, 0, 1), &center, &FakeEnvironment::new(), 1);
    let overworld = cast(ChunkPos::new(0, 0, 0), &center, &FakeEnvironment::new(), 1);
    assert!(!nether.get(10, 10));
    assert!(overworld.get(10, 10));
}
