use std::sync::Arc;

use minimap_core::config::Config;
use minimap_core::services::cave::LayerIds;
use minimap_core::services::memory::{ChunkStore, KnownEmptySet, TileStore};
use minimap_core::services::ui::MapUi;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::Rgba;
use minimap_core::structs::bake::BakeSettings;
use minimap_core::structs::chunk::{ChunkProbe, ChunkSurface, ColumnSample};
use minimap_core::structs::layer::MapLayer;
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::ViewState;
use minimap_core::traits::BakeEnvironment;
use minimap_runtime::services::{ExportService, WorldSession};
use minimap_runtime::structs::{ProbeRequest, Shared, SubmitStatus};

const BAND: i32 = 2;

fn session(dir: &std::path::Path) -> (Arc<Shared>, WorldSession) {
    let config = Config::default();
    let shared = Arc::new(Shared::new(
        ChunkStore::new(64),
        TileStore::new(16, 25),
        KnownEmptySet::new(16),
        BakeSettings::new(&config.terrain),
        ViewState::new(4),
        WaypointStore::new(8, 1),
        MapUi::new(1.0),
    ));
    let exports = Arc::new(ExportService::start(dir, Arc::clone(&shared)).unwrap());
    let mut session = WorldSession::start(Arc::clone(&shared), &config, None, 0, exports).unwrap();
    let cave =
        MapLayer { id: LayerIds::cave(0, BAND), dim: 0, band: Some(BAND), camera_height: 47, scan_radius_chunks: 4 };
    session.change_layer(&cave);
    (shared, session)
}

#[test]
fn a_cave_band_stores_see_through_columns_as_rock_and_every_column_flat_at_its_floor() {
    let dir = tempfile::tempdir().unwrap();
    let (shared, mut session) = session(dir.path());
    let pos = ChunkPos::new(0, 0, LayerIds::cave(0, BAND));
    let ore = Rgba::new(200, 60, 20, 255).packed();
    let mut probe = ChunkSurface::empty();
    for i in 0..ChunkSurface::COLUMNS {
        let colour = if i == 7 { ore } else { 0 };
        probe.write_column(i, ColumnSample::new(colour, 0, 70, 70, false, 0));
    }
    probe.skipped = [0; 4];
    session.submit_chunk(pos, SubmitStatus::Loaded, &ChunkProbe::new(probe, [0; 256]));
    {
        let chunks = shared.chunks.read().unwrap();
        let stored = chunks.surface(pos).expect("the chunk is stored");
        assert_eq!(stored.surface[0], Rgba::new(96, 96, 96, 255).packed());
        assert_eq!(stored.surface[7], ore);
        assert_eq!((stored.height[0], stored.solid[200]), (32, 32));
    }
    session.finish();
}

#[test]
fn a_block_change_in_the_real_dimension_asks_for_the_cave_layer_column() {
    let dir = tempfile::tempdir().unwrap();
    let (_shared, mut session) = session(dir.path());
    let layer = LayerIds::cave(0, BAND);
    let mut probe = ChunkSurface::empty();
    for i in 0..ChunkSurface::COLUMNS {
        probe.write_column(i, ColumnSample::new(0xFF40_4040, 0, 70, 70, false, 0));
    }
    probe.skipped = [0; 4];
    session.submit_chunk(ChunkPos::new(0, 0, layer), SubmitStatus::Loaded, &ChunkProbe::new(probe, [0; 256]));
    assert_eq!(
        session.block_changed(3, 4, 0),
        ProbeRequest::Column { pos: ChunkPos::new(0, 0, layer), index: 4 * 16 + 3 }
    );
    assert_eq!(session.block_changed(3, 4, 1), ProbeRequest::None);
    session.finish();
}
