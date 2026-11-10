use minimap_core::config::MiniMapConfig;
use minimap_core::services::draw::ChunkGridPainter;
use minimap_core::structs::draw::DrawList;
use minimap_core::structs::draw::ExploredChunks;

use crate::support::TestFrames;

#[test]
fn the_grid_draws_every_border_in_view_and_the_hovered_chunk() {
    let frame = TestFrames::world_frame((0.0, 0.0), 1.0, (800.0, 600.0), 0.0, 0);
    let grid = ChunkGridPainter::new(&MiniMapConfig::default());
    let mut d = DrawList::new();
    grid.paint(&frame, None, &ExploredChunks::everywhere(), &mut d);
    assert_eq!(TestFrames::lines(&d), 91);
    d.clear();
    grid.paint(&frame, Some((-1, 0)), &ExploredChunks::everywhere(), &mut d);
    assert_eq!(TestFrames::lines(&d), 95);
    d.clear();
    grid.paint(
        &TestFrames::world_frame((0.0, 0.0), 0.3, (800.0, 600.0), 0.0, 0),
        None,
        &ExploredChunks::everywhere(),
        &mut d,
    );
    assert!(d.commands.is_empty());
}

#[test]
fn on_a_turned_map_the_grid_covers_the_diagonal() {
    let grid = ChunkGridPainter::new(&MiniMapConfig::default());
    let mut d = DrawList::new();
    grid.paint(
        &TestFrames::world_frame((0.0, 0.0), 1.0, (800.0, 600.0), 0.3, 0),
        None,
        &ExploredChunks::everywhere(),
        &mut d,
    );
    assert_eq!(TestFrames::lines(&d), 65 + 65);
}

// the chunk (0, 0) explored, alone: region (0, 0) is the only region in the block
fn one_chunk() -> ExploredChunks {
    let mut explored = ExploredChunks::new();
    explored.begin((-1, -1), 2);
    for region in 0..4 {
        explored.push(if region == 3 { [1, 0, 0, 0] } else { [0; 4] });
    }
    explored
}

#[test]
fn the_grid_draws_only_the_borders_of_explored_chunks() {
    let frame = TestFrames::world_frame((0.0, 0.0), 1.0, (800.0, 600.0), 0.0, 0);
    let grid = ChunkGridPainter::new(&MiniMapConfig::default());
    let mut d = DrawList::new();

    grid.paint(&frame, None, &one_chunk(), &mut d);

    assert_eq!(TestFrames::lines(&d), 4, "one chunk, four borders");
}

#[test]
fn an_unexplored_chunk_under_the_cursor_is_not_highlighted() {
    let frame = TestFrames::world_frame((0.0, 0.0), 1.0, (800.0, 600.0), 0.0, 0);
    let grid = ChunkGridPainter::new(&MiniMapConfig::default());
    let mut d = DrawList::new();

    grid.paint(&frame, Some((-1, 0)), &one_chunk(), &mut d);
    let unexplored = TestFrames::lines(&d);
    d.clear();
    grid.paint(&frame, Some((0, 0)), &one_chunk(), &mut d);

    assert_eq!(unexplored, 4);
    assert_eq!(TestFrames::lines(&d), 8, "the explored chunk gets its outline");
}
