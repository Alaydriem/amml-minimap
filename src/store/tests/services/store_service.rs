use std::time::Duration;

use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::biome::BiomeKey;
use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_core::structs::tile::Tile;
use minimap_core::structs::waypoint::{Waypoint, WaypointSource};
use minimap_store::services::{CachePaths, StoreService, WaypointFile};
use minimap_store::structs::{JournalRecord, StoreCommand, StoreEvent};

use crate::support::TestRecords;

fn append(service: &StoreService, pos: ChunkPos, pixel: u32, token: u64) {
    let record = Box::new(TestRecords::record(pos, 70, pixel));
    service.send(StoreCommand::Append { record, token }).unwrap();
}

fn load_tile(service: &StoreService, region: RegionPos) -> StoreEvent {
    service.send(StoreCommand::LoadTile(region)).unwrap();
    TestRecords::wait_for(
        service,
        |e| matches!(e, StoreEvent::TileLoaded { region: r, .. } | StoreEvent::TileMissing(r) if *r == region),
    )
    .expect("a tile answer within two seconds")
}

fn first_pixel(event: &StoreEvent) -> u32 {
    match event {
        StoreEvent::TileLoaded { pixels, .. } => u32::from_le_bytes(pixels[0..4].try_into().unwrap()),
        other => panic!("expected a tile, got {other:?}"),
    }
}

#[test]
fn an_appended_chunk_is_acknowledged_and_its_tile_loads() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = StoreService::start(Some(dir.path().to_path_buf()), TestRecords::settings()).unwrap();
    append(&service, ChunkPos::new(0, 0, 0), 0xFF00_00AA, 41);
    let ack = TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::Appended { token: 41, .. }));
    assert!(ack.is_some());
    assert_eq!(first_pixel(&load_tile(&service, RegionPos::new(0, 0, 0))), 0xFF00_00AA);
    assert!(service.shutdown(Duration::from_secs(5)).finished);
}

#[test]
fn a_region_with_nothing_stored_is_reported_missing() {
    let dir = tempfile::tempdir().unwrap();
    let service = StoreService::start(Some(dir.path().to_path_buf()), TestRecords::settings()).unwrap();
    assert!(matches!(load_tile(&service, RegionPos::new(9, 9, 0)), StoreEvent::TileMissing(_)));
}

#[test]
fn shutdown_compacts_and_a_restart_still_loads_the_tile() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    {
        let mut service = StoreService::start(Some(root.clone()), TestRecords::settings()).unwrap();
        append(&service, ChunkPos::new(-1, -1, 0), 0xFF00_00BB, 1);
        assert!(service.shutdown(Duration::from_secs(5)).finished);
    }
    assert!(CachePaths::region_file(&root, RegionPos::new(-1, -1, 0)).exists());
    let service = StoreService::start(Some(root), TestRecords::settings()).unwrap();
    let event = load_tile(&service, RegionPos::new(-1, -1, 0));
    match event {
        StoreEvent::TileLoaded { pixels, settings_hash, .. } => {
            let i = (15 * 16 * Tile::SIDE + 15 * 16) * 4;
            assert_eq!(u32::from_le_bytes(pixels[i..i + 4].try_into().unwrap()), 0xFF00_00BB);
            assert_eq!(settings_hash, 7);
        }
        other => panic!("expected a tile, got {other:?}"),
    }
}

#[test]
fn surfaces_of_a_region_load_with_their_positions() {
    let dir = tempfile::tempdir().unwrap();
    let service = StoreService::start(Some(dir.path().to_path_buf()), TestRecords::settings()).unwrap();
    append(&service, ChunkPos::new(2, 3, 0), 0xFF00_0001, 1);
    append(&service, ChunkPos::new(5, 6, 0), 0xFF00_0001, 2);
    let unbaked = JournalRecord::new(ChunkPos::new(9, 9, 0), 7, TestRecords::surface(64), None);
    service.send(StoreCommand::Append { record: Box::new(unbaked), token: 3 }).unwrap();
    service.send(StoreCommand::LoadSurfaces(RegionPos::new(0, 0, 0))).unwrap();
    let event = TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::SurfacesLoaded { .. })).unwrap();
    let StoreEvent::SurfacesLoaded { chunks, .. } = event else { panic!("matched above") };
    let positions: Vec<ChunkPos> = chunks.iter().map(|(p, _, _)| *p).collect();
    assert_eq!(positions, vec![ChunkPos::new(2, 3, 0), ChunkPos::new(5, 6, 0), ChunkPos::new(9, 9, 0)]);
    let baked: Vec<bool> = chunks.iter().map(|(_, _, baked)| *baked).collect();
    assert_eq!(baked, vec![true, true, false]);
}

#[test]
fn without_a_disk_cache_appends_are_acknowledged_and_loads_miss() {
    let mut service = StoreService::start(None, TestRecords::settings()).unwrap();
    append(&service, ChunkPos::new(0, 0, 0), 1, 3);
    assert!(TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::Appended { token: 3, .. })).is_some());
    assert!(matches!(load_tile(&service, RegionPos::new(0, 0, 0)), StoreEvent::TileMissing(_)));
    assert!(service.shutdown(Duration::from_secs(5)).finished);
}

#[test]
fn after_shutdown_sending_reports_the_store_stopped() {
    let mut service = StoreService::start(None, TestRecords::settings()).unwrap();
    service.shutdown(Duration::from_secs(5));
    assert!(service.send(StoreCommand::Compact).is_err());
    assert!(service.try_send(StoreCommand::Compact).is_err());
}

#[test]
fn appends_without_an_ack_never_wait_on_a_full_event_queue() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = StoreService::start(Some(dir.path().to_path_buf()), TestRecords::settings()).unwrap();
    let count = 4 * TestRecords::settings().queue_capacity as i32;
    let started = std::time::Instant::now();
    for i in 0..count {
        append(&service, ChunkPos::new(i, 0, 0), 0xFF00_00CC, StoreCommand::NO_ACK);
    }
    assert!(service.shutdown(Duration::from_secs(5)).finished);
    assert!(started.elapsed() < Duration::from_secs(5), "took {:?}", started.elapsed());
}

#[test]
fn journal_writes_show_in_bytes_written_and_lines_reach_their_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = StoreService::start(Some(dir.path().join("cache")), TestRecords::settings()).unwrap();
    assert_eq!(service.bytes_written(), 0);
    append(&service, ChunkPos::new(0, 0, 0), 0xFF00_00AA, 1);
    assert!(TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::Appended { token: 1, .. })).is_some());
    let line_path = dir.path().join("telemetry.csv");
    let command = StoreCommand::AppendLine { path: line_path.clone(), line: "t_ms".to_string(), truncate: true };
    service.send(command).unwrap();
    assert!(service.shutdown(Duration::from_secs(5)).finished);
    assert!(service.bytes_written() > 0);
    assert_eq!(std::fs::read_to_string(line_path).unwrap(), "t_ms\n");
}

fn load_waypoints(service: &StoreService, path: &std::path::Path) -> Vec<Waypoint> {
    service.send(StoreCommand::LoadWaypoints { path: path.to_path_buf() }).unwrap();
    match TestRecords::wait_for(service, |e| matches!(e, StoreEvent::WaypointsLoaded { path: p, .. } if p == path)) {
        Some(StoreEvent::WaypointsLoaded { waypoints, .. }) => waypoints,
        other => panic!("expected the waypoints, got {other:?}"),
    }
}

#[test]
fn waypoints_are_written_and_loaded_on_the_store_thread_without_a_disk_cache() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = StoreService::start(None, TestRecords::settings()).unwrap();
    let path = dir.path().join("world").join(WaypointFile::NAME);
    assert!(load_waypoints(&service, &path).is_empty());
    let mut store = WaypointStore::new(8, 1);
    store.add("Home", 0, (1, 64, 1), WaypointSource::Manual, 5).unwrap();
    service.send(StoreCommand::WriteWaypoints { path: path.clone(), waypoints: store.waypoints().to_vec() }).unwrap();
    assert_eq!(load_waypoints(&service, &path), store.waypoints());
    assert!(service.shutdown(Duration::from_secs(5)).finished);
}

#[test]
fn a_region_list_answers_with_the_files_of_the_dimension_or_nothing_without_a_cache() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = StoreService::start(Some(dir.path().to_path_buf()), TestRecords::settings()).unwrap();
    append(&service, ChunkPos::new(0, 0, 0), 0xFF00_00AA, StoreCommand::NO_ACK);
    service.send(StoreCommand::Compact).unwrap();
    service.send(StoreCommand::ListRegions { dim: 0 }).unwrap();
    let listed = TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::RegionList { dim: 0, .. }));
    assert!(matches!(listed, Some(StoreEvent::RegionList { regions, .. }) if regions == vec![RegionPos::new(0, 0, 0)]));
    service.shutdown(Duration::from_secs(5));

    let mut bare = StoreService::start(None, TestRecords::settings()).unwrap();
    bare.send(StoreCommand::ListRegions { dim: 0 }).unwrap();
    let empty = TestRecords::wait_for(&bare, |e| matches!(e, StoreEvent::RegionList { .. }));
    assert!(matches!(empty, Some(StoreEvent::RegionList { regions, .. }) if regions.is_empty()));
    bare.shutdown(Duration::from_secs(5));
}

fn load_keys(service: &StoreService) -> Option<(u64, Vec<String>)> {
    service.send(StoreCommand::LoadBiomeKeys).unwrap();
    match TestRecords::wait_for(service, |e| matches!(e, StoreEvent::BiomeKeysLoaded { .. })) {
        Some(StoreEvent::BiomeKeysLoaded { table }) => table,
        other => panic!("expected biome keys, got {other:?}"),
    }
}

#[test]
fn a_world_with_no_key_file_gets_a_new_table_and_keeps_it() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let first = {
        let mut service = StoreService::start(Some(root.clone()), TestRecords::settings()).unwrap();
        let table = load_keys(&service).unwrap();
        assert_ne!(table.0, 0);
        assert!(table.1.is_empty());
        assert!(service.shutdown(Duration::from_secs(5)).finished);
        table.0
    };
    assert!(CachePaths::biome_keys_file(&root).exists());
    let service = StoreService::start(Some(root), TestRecords::settings()).unwrap();
    assert_eq!(load_keys(&service).map(|t| t.0), Some(first));
}

#[test]
fn biome_keys_written_are_loaded_back() {
    let dir = tempfile::tempdir().unwrap();
    let service = StoreService::start(Some(dir.path().to_path_buf()), TestRecords::settings()).unwrap();
    let names = vec!["plains".to_string(), "forest".to_string()];
    service.send(StoreCommand::WriteBiomeKeys { table_id: 9, names: names.clone() }).unwrap();
    assert_eq!(load_keys(&service), Some((9, names)));
}

#[test]
fn without_a_cache_there_is_no_key_table() {
    let service = StoreService::start(None, TestRecords::settings()).unwrap();
    assert_eq!(load_keys(&service), None);
}

#[test]
fn load_biomes_answers_with_the_journaled_palettes() {
    let dir = tempfile::tempdir().unwrap();
    let service = StoreService::start(Some(dir.path().to_path_buf()), TestRecords::settings()).unwrap();
    load_keys(&service).unwrap();
    let mut record = TestRecords::record(ChunkPos::new(17, -1, 0), 70, 1);
    record.surface.biome.set(5, Some(BiomeKey(2)));
    service.send(StoreCommand::Append { record: Box::new(record), token: 0 }).unwrap();
    let region = RegionPos::new(1, -1, 0);
    service.send(StoreCommand::LoadBiomes(region)).unwrap();
    match TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::BiomesLoaded { .. })) {
        Some(StoreEvent::BiomesLoaded { region: r, biomes }) => {
            assert_eq!(r, region);
            assert_eq!(biomes.key((1, 15), 5), Some(BiomeKey(2)));
        }
        other => panic!("expected biomes, got {other:?}"),
    }
}

#[test]
fn load_biomes_for_an_unexplored_region_answers_empty() {
    let dir = tempfile::tempdir().unwrap();
    let service = StoreService::start(Some(dir.path().to_path_buf()), TestRecords::settings()).unwrap();
    load_keys(&service);
    service.send(StoreCommand::LoadBiomes(RegionPos::new(4, 4, 0))).unwrap();
    match TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::BiomesLoaded { .. })) {
        Some(StoreEvent::BiomesLoaded { biomes, .. }) => assert!(biomes.is_empty()),
        other => panic!("expected biomes, got {other:?}"),
    }
}

#[test]
fn a_key_file_that_cannot_be_read_still_gives_a_table() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(CachePaths::biome_keys_file(dir.path())).unwrap();
    let service = StoreService::start(Some(dir.path().to_path_buf()), TestRecords::settings()).unwrap();
    let table = load_keys(&service).expect("a table even when the file cannot be read");
    assert_ne!(table.0, 0);
    assert!(table.1.is_empty());
}

#[test]
fn a_key_table_that_cannot_be_saved_is_reported_so_it_is_sent_again() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(CachePaths::biome_keys_file(dir.path())).unwrap();
    let service = StoreService::start(Some(dir.path().to_path_buf()), TestRecords::settings()).unwrap();
    service.send(StoreCommand::WriteBiomeKeys { table_id: 9, names: vec!["plains".to_string()] }).unwrap();
    let event = TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::BiomeKeysNotSaved));
    assert!(event.is_some());
}
