use std::sync::Arc;
use std::time::{Duration, Instant};

use minimap_core::config::TerrainConfig;
use minimap_core::services::memory::{ChunkStore, KnownEmptySet, TileStore};
use minimap_core::services::ui::MapUi;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::bake::{BakeJob, BakeSettings};
use minimap_core::structs::chunk::{ChunkSurface, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::ViewState;
use minimap_runtime::services::BakePool;
use minimap_runtime::structs::{BakeResult, Shared};

fn shared_with_chunk(pos: ChunkPos) -> Arc<Shared> {
    let mut chunks = ChunkStore::new(64);
    let mut probe = ChunkSurface::empty();
    for i in 0..ChunkSurface::COLUMNS {
        probe.write_column(i, ColumnSample::new(0xFF64_6464, 0, 70, 70, false, 0));
    }
    chunks.submit(pos, &probe, 1, pos, &mut Vec::new());
    Arc::new(Shared::new(
        chunks,
        TileStore::new(16, 25),
        KnownEmptySet::new(16),
        BakeSettings::new(&TerrainConfig::default()),
        ViewState::new(4),
        WaypointStore::new(8, 1),
        MapUi::new(1.0),
    ))
}

fn wait_for_result(pool: &BakePool) -> Option<BakeResult> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut out = Vec::new();
    while Instant::now() < deadline {
        pool.drain(&mut out, 1);
        if let Some(result) = out.pop() {
            return Some(result);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    None
}

#[test]
fn a_chunk_job_comes_back_baked() {
    let pos = ChunkPos::new(3, 4, 0);
    let mut pool = BakePool::new(1, 8, shared_with_chunk(pos)).unwrap();
    assert!(pool.try_submit(BakeJob::Chunk { pos, soft_only: false }));
    match wait_for_result(&pool) {
        Some(BakeResult::Chunk { chunk, .. }) => {
            assert_eq!(chunk.pos, pos);
            assert!(chunk.pixels.iter().all(|&p| p >> 24 == 0xFF), "every pixel is opaque");
        }
        other => panic!("expected a baked chunk, got {other:?}"),
    }
    pool.shutdown(Duration::from_secs(2));
}

#[test]
fn a_chunk_that_is_not_stored_is_skipped() {
    let mut pool = BakePool::new(1, 8, shared_with_chunk(ChunkPos::new(0, 0, 0))).unwrap();
    assert!(pool.try_submit(BakeJob::Chunk { pos: ChunkPos::new(50, 50, 0), soft_only: false }));
    assert!(matches!(wait_for_result(&pool), Some(BakeResult::Skipped)));
    pool.shutdown(Duration::from_secs(2));
}

#[test]
fn a_full_queue_refuses_jobs_without_blocking() {
    let pos = ChunkPos::new(0, 0, 0);
    let mut pool = BakePool::new(1, 1, shared_with_chunk(pos)).unwrap();
    let started = Instant::now();
    let accepted = (0..64).filter(|_| pool.try_submit(BakeJob::Chunk { pos, soft_only: false })).count();
    assert!(accepted < 64);
    assert!(started.elapsed() < Duration::from_secs(1));
    pool.shutdown(Duration::from_secs(2));
}

#[test]
fn finishing_returns_within_its_timeout_and_refuses_new_jobs() {
    let pos = ChunkPos::new(0, 0, 0);
    let mut pool = BakePool::new(2, 64, shared_with_chunk(pos)).unwrap();
    for _ in 0..64 {
        pool.try_submit(BakeJob::Chunk { pos, soft_only: false });
    }
    let started = Instant::now();
    let mut applied = 0;
    pool.finish(Duration::from_secs(2), |_| applied += 1);
    assert!(started.elapsed() < Duration::from_secs(4));
    assert!(!pool.try_submit(BakeJob::Chunk { pos, soft_only: false }));
}
