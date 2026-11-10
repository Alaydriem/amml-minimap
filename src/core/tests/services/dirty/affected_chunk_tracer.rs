use minimap_core::services::dirty::AffectedChunkTracer;
use minimap_core::structs::LightDirection;
use minimap_core::structs::chunk::HeightChange;
use minimap_core::structs::pos::{BlockRect, ChunkPos};

use crate::support::FakeEnvironment;

// light from the north-west: shadows fall toward the south-east
fn light() -> LightDirection {
    LightDirection::new(315.0, 60.0)
}

fn chunk_change(higher: i32, lower: i32) -> HeightChange {
    HeightChange::new(BlockRect::new(0, 0, 16, 16), higher, lower, 0)
}

fn trace(change: HeightChange, env: &FakeEnvironment) -> Vec<ChunkPos> {
    let mut out = Vec::new();
    AffectedChunkTracer::new().trace(&change, light(), env, &mut out);
    out
}

fn c(x: i32, z: i32) -> ChunkPos {
    ChunkPos::new(x, z, 0)
}

#[test]
fn the_changed_chunk_and_its_far_edge_neighbours_are_always_included() {
    let out = trace(chunk_change(100, 90), &FakeEnvironment::new());
    for chunk in [c(0, 0), c(1, 0), c(0, 1), c(1, 1)] {
        assert!(out.contains(&chunk), "{chunk:?}");
    }
}

#[test]
fn a_downstream_chunk_within_reach_is_included_and_an_upstream_one_never() {
    let env = FakeEnvironment::new().with_range(c(2, 2), 60, 70).with_range(c(-1, -1), 0, 5);
    let out = trace(chunk_change(100, 90), &env);
    assert!(out.contains(&c(2, 2)));
    assert!(!out.contains(&c(-1, -1)));
}

#[test]
fn the_ray_stops_where_the_higher_height_cannot_reach_the_chunk() {
    let env = FakeEnvironment::new().with_range(c(2, 2), 60, 70).with_range(c(3, 3), 60, 70);
    let out = trace(chunk_change(100, 90), &env);
    assert!(out.contains(&c(2, 2)));
    assert!(!out.contains(&c(3, 3)));
}

#[test]
fn unscanned_chunks_are_passed_through_without_being_marked() {
    let env = FakeEnvironment::new().with_range(c(3, 3), 0, 5);
    let out = trace(chunk_change(100, 90), &env);
    assert!(!out.contains(&c(2, 2)));
    assert!(out.contains(&c(3, 3)));
}

#[test]
fn a_chunk_shadowed_in_both_states_is_skipped_but_the_ray_continues() {
    let env = FakeEnvironment::new().with_range(c(2, 2), 0, 5).with_range(c(3, 3), 0, 5);
    let out = trace(chunk_change(100, 90), &env);
    assert!(!out.contains(&c(2, 2)));
    assert!(out.contains(&c(3, 3)));
}

#[test]
fn a_first_load_never_skips_a_chunk_as_always_shadowed() {
    let env = FakeEnvironment::new().with_range(c(2, 2), 0, 5);
    let out = trace(chunk_change(100, HeightChange::FIRST_LOAD_LOWER), &env);
    assert!(out.contains(&c(2, 2)));
}
