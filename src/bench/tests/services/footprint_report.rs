use minimap_bench::services::FootprintReport;
use minimap_core::config::TerrainConfig;
use minimap_core::services::memory::ChunkStore;
use minimap_core::structs::bake::BakeSettings;
use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::draw::{DrawCommand, DrawList, TextAlign};

#[test]
fn the_footprint_names_the_surface_size_and_the_chunks_per_budget_the_store_uses() {
    let rows = FootprintReport::rows();
    let get = |name| rows.iter().find(|(n, _)| *n == name).map(|(_, v)| *v).unwrap();
    assert_eq!(get("chunk_surface_bytes"), std::mem::size_of::<ChunkSurface>() as u64);
    let scale = BakeSettings::new(&TerrainConfig::default()).scale;
    assert_eq!(get("chunks_per_64_mib"), ChunkStore::capacity_for_budget(64 << 20, scale) as u64);
    assert!(get("region_file_bytes") > 0);
}

#[test]
fn vertices_count_four_per_line_and_rect_three_per_triangle_and_four_per_text_byte() {
    let mut draw = DrawList::new();
    draw.push(DrawCommand::Line { a: (0.0, 0.0), b: (1.0, 1.0), colour: 0, thickness: 1.0 });
    draw.push(DrawCommand::RectFilled { p0: (0.0, 0.0), p1: (1.0, 1.0), colour: 0 });
    draw.push(DrawCommand::TriangleFilled { a: (0.0, 0.0), b: (1.0, 0.0), c: (0.0, 1.0), colour: 0 });
    draw.text((0.0, 0.0), 0, 10.0, TextAlign::Left, "abc");
    assert_eq!(FootprintReport::vertices(&draw), 4 + 4 + 3 + 12);
}

#[test]
fn the_footprint_counts_overlay_vertices_and_the_grid_stays_bounded_at_the_gate() {
    let rows = FootprintReport::rows();
    let get = |name| rows.iter().find(|(n, _)| *n == name).map(|(_, v)| *v).unwrap();
    assert!(get("grid_vertices_gate") > 0);
    assert!(get("grid_vertices_gate") <= 4 * (2 * 360 + 8));
    assert!(get("grid_vertices_max_zoom") > 0);
    assert!(get("slime_vertices_gate") > 0);
    assert_eq!(get("chunk_surface_bytes"), 3264);
}
