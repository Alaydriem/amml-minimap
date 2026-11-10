use alaydriem_minimap::services::{ColumnProbe, MapColorReader};
use amml_game::BlockSource;
use bytemuck::Zeroable;
use minimap_core::structs::chunk::ChunkProbe;
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::structs::SubmitStatus;

use crate::support::{FakePalette, FakeWorld, RecordingSink};

fn probe(world: &mut FakeWorld, optional: bool) -> (SubmitStatus, ChunkProbe) {
    let entries = FakeWorld::entries(optional);
    let sink = RecordingSink::probing();
    let mut out = ChunkProbe::zeroed();
    // SAFETY: a live fake source
    let region = unsafe { BlockSource::from_raw(world.source.ptr(), &entries) }.unwrap();
    let status = ColumnProbe::new(false).probe_chunk(&entries, &region, ChunkPos::new(0, 0, 0), &sink, &mut out);
    (status, out)
}

#[test]
fn a_chunk_that_is_not_loaded_is_not_loaded_and_no_block_is_read() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    world.chunk.layout.load_state = 0;

    let (status, _) = probe(&mut world, true);

    assert_eq!(status, SubmitStatus::NotLoaded);
    assert_eq!(world.chunk.block_reads(), 0);
    assert_eq!(world.chunk.reads_outside(), 0);
}

#[test]
fn a_chunk_with_no_sub_chunks_is_a_top_placeholder_and_no_block_is_read() {
    let mut world = FakeWorld::with_chunk(FakePalette::STONE, 0, false);

    let (status, _) = probe(&mut world, true);

    assert_eq!(status, SubmitStatus::TopPlaceholder);
    assert_eq!(world.chunk.block_reads(), 0);
    assert_eq!(world.chunk.reads_outside(), 0);
}

#[test]
fn a_placeholder_top_sub_chunk_is_a_top_placeholder_and_nothing_past_the_list_is_read() {
    let mut world = FakeWorld::with_chunk(FakePalette::STONE, 2, true);

    let (status, _) = probe(&mut world, true);

    assert_eq!(status, SubmitStatus::TopPlaceholder);
    assert_eq!(world.chunk.block_reads(), 0);
    assert_eq!(world.chunk.reads_outside(), 0);
}

#[test]
fn a_chunk_not_in_the_source_is_not_loaded() {
    let mut world = FakeWorld::new(FakePalette::STONE);
    world.source.chunk_pos = [5, 5];

    let (status, _) = probe(&mut world, true);

    assert_eq!(status, SubmitStatus::NotLoaded);
}

#[test]
fn a_loaded_chunk_reads_every_column_inside_its_height_range_only() {
    let mut world = FakeWorld::new(FakePalette::STONE);

    let (status, out) = probe(&mut world, true);

    assert_eq!(status, SubmitStatus::Loaded);
    assert!(world.chunk.block_reads() >= 256);
    assert_eq!(world.chunk.reads_outside(), 0);
    assert!(!out.surface.any_skipped());
    assert_eq!(out.surface.height[0], 5);
}

#[test]
fn without_the_minimap_features_the_probe_colours_from_the_name_table() {
    let lava = u32::from_le_bytes([230, 110, 20, 255]);
    let mut lava_world = FakeWorld::new(FakePalette::LAVA);
    let mut stone_world = FakeWorld::new(FakePalette::STONE);

    let (lava_status, lava_probe) = probe(&mut lava_world, false);
    let (stone_status, stone_probe) = probe(&mut stone_world, false);

    assert_eq!(lava_status, SubmitStatus::Loaded);
    assert!(lava_probe.surface.surface.iter().all(|colour| *colour == lava));
    assert_eq!(stone_status, SubmitStatus::Loaded);
    assert!(stone_probe.surface.surface.iter().all(|colour| *colour == MapColorReader::FALLBACK_BLOCK.packed()));
    assert!(stone_probe.biome_ids.iter().all(|id| *id == 0));
}

#[test]
fn transparent_water_shows_the_block_below_with_the_water_depth() {
    let mut world = FakeWorld::new(FakePalette::WATER);

    let (status, out) = probe(&mut world, false);

    assert_eq!(status, SubmitStatus::Loaded);
    assert_eq!(out.surface.surface[0], MapColorReader::FALLBACK_BLOCK.packed());
    assert_eq!(out.surface.water[0] >> 24, 1);
}
