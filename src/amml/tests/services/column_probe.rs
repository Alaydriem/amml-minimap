use alaydriem_minimap::services::{ColumnColors, ColumnProbe};
use amml_api::testing::{FakeGame, FakeWorld};
use amml_api::{Area, ChunkLoad, ColumnSample};
use bytemuck::Zeroable;
use minimap_core::structs::chunk::ChunkProbe;
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::structs::SubmitStatus;

use crate::support::{FakeTerrain, Frame, MinimapRig, RecordingSink};

fn game(world: FakeWorld) -> FakeGame {
    let game = MinimapRig::game();
    game.set_client(MinimapRig::player(0, world));
    game
}

fn probe(game: &FakeGame) -> ColumnProbe {
    ColumnProbe::new(&MinimapRig::ctx(game))
}

// probes the chunk at `pos` in one tick
fn probe_chunk(game: &FakeGame, pos: (i32, i32)) -> (SubmitStatus, ChunkProbe) {
    let mut probe = probe(game);
    Frame::run(game, move |client| {
        let source = client.local_player().unwrap().block_source().unwrap();
        let mut out = Box::new(ChunkProbe::zeroed());
        let status =
            probe.probe_chunk(&source, ChunkPos { x: pos.0, z: pos.1, dim: 0 }, &RecordingSink::default(), &mut out);
        (status, *out)
    })
}

#[test]
fn a_chunk_the_game_lacks_or_has_not_loaded_is_not_loaded() {
    let mut world = FakeTerrain::world(&[]);
    world.chunks.insert((1, 0), FakeTerrain::chunk(ChunkLoad::NotLoaded, FakeTerrain::stone()));
    world.chunks.insert((2, 0), FakeTerrain::chunk(ChunkLoad::TopPlaceholder, FakeTerrain::stone()));
    let game = game(world);

    assert_eq!(probe_chunk(&game, (0, 0)).0, SubmitStatus::NotLoaded);
    assert_eq!(probe_chunk(&game, (1, 0)).0, SubmitStatus::NotLoaded);
    assert_eq!(probe_chunk(&game, (2, 0)).0, SubmitStatus::TopPlaceholder);
}

#[test]
fn a_loaded_chunk_fills_every_column_with_colour_heights_and_biome() {
    let game = game(FakeTerrain::world(&[(0, 0)]));

    let (status, out) = probe_chunk(&game, (0, 0));

    assert_eq!(status, SubmitStatus::Loaded);
    assert!(!out.surface.any_skipped());
    assert_eq!(out.surface.surface[17], minimap_core::structs::Rgba::new(1, 2, 3, 255).packed());
    assert_eq!((out.surface.height[17], out.surface.solid[17]), (65, 65));
    assert_eq!(out.biome_ids[17], FakeTerrain::PLAINS);
}

#[test]
fn skipped_and_retry_columns_are_marked() {
    let mut chunk = FakeTerrain::chunk(ChunkLoad::Loaded, FakeTerrain::stone());
    chunk.samples[3].flags = ColumnSample::SKIPPED;
    chunk.samples[4].flags = ColumnSample::RETRY;
    let mut world = FakeTerrain::world(&[]);
    world.chunks.insert((0, 0), chunk);
    let game = game(world);

    let (_, out) = probe_chunk(&game, (0, 0));

    assert!(out.surface.is_skipped(3));
    assert!(out.surface.is_retry(4));
    assert!(!out.surface.is_skipped(4));
}

#[test]
fn without_the_map_colour_area_the_fixed_colours_show() {
    let game = game(FakeTerrain::world(&[(0, 0)]));
    game.withdraw_area(Area::WORLD_MAP_COLOR);

    let (_, out) = probe_chunk(&game, (0, 0));

    assert_eq!(out.surface.surface[0], ColumnColors::FALLBACK_BLOCK.packed());
}

#[test]
fn one_column_is_read_alone_and_not_from_a_chunk_that_is_not_loaded() {
    let mut chunk = FakeTerrain::chunk(ChunkLoad::Loaded, FakeTerrain::stone());
    chunk.samples[33].height = 99;
    chunk.samples[34].flags = ColumnSample::SKIPPED;
    let mut world = FakeTerrain::world(&[]);
    world.chunks.insert((0, 0), chunk);
    world.chunks.insert((1, 0), FakeTerrain::chunk(ChunkLoad::TopPlaceholder, FakeTerrain::stone()));
    let game = game(world);
    let mut probe = probe(&game);

    let (read, skipped, placeholder) = Frame::run(&game, move |client| {
        let source = client.local_player().unwrap().block_source().unwrap();
        let sink = RecordingSink::default();
        let at = |x| ChunkPos { x, z: 0, dim: 0 };
        (
            probe.probe_column(&source, at(0), 33, &sink),
            probe.probe_column(&source, at(0), 34, &sink),
            probe.probe_column(&source, at(1), 33, &sink),
        )
    });

    assert_eq!(read.map(|sample| sample.height), Some(99));
    assert_eq!(skipped, None);
    assert_eq!(placeholder, None);
}
