use minimap_core::services::draw::SlimeOverlayPainter;
use minimap_core::structs::draw::ExploredChunks;
use minimap_core::structs::draw::{DrawCommand, DrawList};

use crate::support::TestFrames;

#[test]
fn slime_chunk_minus_one_zero_is_tinted_and_chunk_zero_zero_is_not() {
    let mut p = SlimeOverlayPainter::new(64);
    let mut d = DrawList::new();
    p.paint(&TestFrames::world_frame((-8.0, 8.0), 1.0, (800.0, 600.0), 0.0, 0), &ExploredChunks::everywhere(), &mut d);
    let rects = TestFrames::rects(&d);
    assert!(rects.iter().any(|(p0, p1)| p0.0 <= 392.0 && p1.0 >= 408.0 && p0.1 == 292.0 && p1.1 == 308.0));
    assert!(!rects.iter().any(|(p0, p1)| (p0.0..p1.0).contains(&416.0) && (p0.1..p1.1).contains(&300.0)));
}

#[test]
fn slime_draws_nothing_in_the_nether_or_below_the_gate() {
    let mut p = SlimeOverlayPainter::new(64);
    let mut d = DrawList::new();
    p.paint(&TestFrames::world_frame((0.0, 0.0), 1.0, (800.0, 600.0), 0.0, -1), &ExploredChunks::everywhere(), &mut d);
    assert!(d.commands.is_empty());
    p.paint(&TestFrames::world_frame((0.0, 0.0), 0.3, (800.0, 600.0), 0.0, 0), &ExploredChunks::everywhere(), &mut d);
    assert!(d.commands.is_empty());
}

#[test]
fn a_turned_map_draws_runs_as_triangle_pairs() {
    let mut p = SlimeOverlayPainter::new(64);
    let mut d = DrawList::new();
    p.paint(&TestFrames::world_frame((0.0, 0.0), 1.0, (800.0, 600.0), 0.4, 0), &ExploredChunks::everywhere(), &mut d);
    let triangles = d.commands.iter().filter(|c| matches!(c, DrawCommand::TriangleFilled { .. })).count();
    assert!(triangles > 0 && triangles % 2 == 0);
    assert!(TestFrames::rects(&d).is_empty());
}

#[test]
fn a_cold_cache_fills_over_frames_with_at_most_thirty_two_new_masks_each() {
    let mut p = SlimeOverlayPainter::new(1024);
    let frame = TestFrames::world_frame((0.0, 0.0), 0.375, (1920.0, 1080.0), 0.0, 0);
    let mut d = DrawList::new();
    p.paint(&frame, &ExploredChunks::everywhere(), &mut d);
    assert_eq!(p.cached_regions(), 32);
    let first = d.commands.len();
    for _ in 0..12 {
        d.clear();
        p.paint(&frame, &ExploredChunks::everywhere(), &mut d);
    }
    assert!(d.commands.len() > first);
}
