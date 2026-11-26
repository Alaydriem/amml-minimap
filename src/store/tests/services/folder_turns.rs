use std::path::Path;
use std::time::Duration;

use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_store::services::{FolderTurns, StoreService};
use minimap_store::structs::{StoreCommand, StoreEvent};

use crate::support::TestRecords;

#[test]
fn a_turn_waits_for_the_earlier_turn_on_its_folder_only() {
    let turns = FolderTurns::new_shared();
    let first = turns.take(Path::new("world-a"));
    let second = turns.take(Path::new("world-a"));
    let other = turns.take(Path::new("world-b"));
    assert!(first.is_due() && other.is_due());
    assert!(!second.is_due());
    assert!(!second.wait_due(Duration::from_millis(20)));
    drop(first);
    assert!(second.is_due());
    assert_eq!(turns.len(), 2);
}

#[test]
fn a_folder_leaves_the_book_when_its_latest_turn_ends() {
    let turns = FolderTurns::new_shared();
    let first = turns.take(Path::new("world-a"));
    let second = turns.take(Path::new("world-a"));
    drop(first);
    assert_eq!(turns.len(), 1);
    let third = turns.take(Path::new("world-a"));
    drop(second);
    assert_eq!(turns.len(), 1);
    assert!(third.is_due());
    drop(third);
    assert!(turns.is_empty());
}

#[test]
fn a_store_handles_no_command_until_the_earlier_writer_of_its_folder_is_done() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let turns = FolderTurns::new_shared();
    let earlier = turns.take(&root);
    let mut service =
        StoreService::start(Some(root.clone()), TestRecords::settings(), Some(turns.take(&root))).unwrap();
    let record = Box::new(TestRecords::record(ChunkPos::new(0, 0, 0), 70, 0xFF00_00CC));
    service.send(StoreCommand::Append { record, token: 5 }).unwrap();
    assert!(TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::Appended { .. })).is_none());
    let done = earlier.done();
    drop(earlier);
    assert!(done.is_open());
    assert!(TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::Appended { token: 5, .. })).is_some());
    service.send(StoreCommand::LoadTile(RegionPos::new(0, 0, 0))).unwrap();
    assert!(TestRecords::wait_for(&service, |e| matches!(e, StoreEvent::TileLoaded { .. })).is_some());
    assert!(service.shutdown(Duration::from_secs(5)).finished);
    assert!(turns.is_empty());
}

#[test]
fn a_store_that_is_shut_down_while_waiting_still_writes_its_queue_once_its_turn_comes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let turns = FolderTurns::new_shared();
    let earlier = turns.take(&root);
    let mut service =
        StoreService::start(Some(root.clone()), TestRecords::settings(), Some(turns.take(&root))).unwrap();
    let record = Box::new(TestRecords::record(ChunkPos::new(1, 1, 0), 70, 0xFF00_00DD));
    service.send(StoreCommand::Append { record, token: StoreCommand::NO_ACK }).unwrap();
    assert!(!service.shutdown(Duration::from_millis(50)).finished);
    let mut after = StoreService::start(Some(root.clone()), TestRecords::settings(), Some(turns.take(&root))).unwrap();
    drop(earlier);
    after.send(StoreCommand::LoadTile(RegionPos::new(0, 0, 0))).unwrap();
    let event =
        TestRecords::wait_for(&after, |e| matches!(e, StoreEvent::TileLoaded { .. } | StoreEvent::TileMissing(_)));
    assert!(matches!(event, Some(StoreEvent::TileLoaded { .. })), "{event:?}");
    assert!(after.shutdown(Duration::from_secs(5)).finished);
}
