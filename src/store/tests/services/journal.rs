use std::fs;

use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_store::services::{Journal, RegionFiles};

use crate::support::TestRecords;

const MAX: u64 = 64 * 1024 * 1024;

#[test]
fn reopening_replays_the_latest_record_of_each_chunk() {
    let dir = tempfile::tempdir().unwrap();
    let pos = ChunkPos::new(3, 3, 0);
    {
        let mut journal = Journal::open(dir.path(), MAX).unwrap();
        journal.append(&TestRecords::record(pos, 64, 1)).unwrap();
        journal.append(&TestRecords::record(pos, 90, 2)).unwrap();
        journal.append(&TestRecords::record(ChunkPos::new(40, 0, 0), 64, 3)).unwrap();
        journal.sync().unwrap();
    }
    let journal = Journal::open(dir.path(), MAX).unwrap();
    assert_eq!(journal.indexed_len(), 2);
    let mut records = Vec::new();
    journal.records_for_region(RegionPos::new(0, 0, 0), &mut records).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].surface.height[0], 90);
    assert_eq!(records[0].patch, Some([2; 256]));
}

#[test]
fn compaction_moves_records_into_region_files_and_empties_the_journal() {
    let dir = tempfile::tempdir().unwrap();
    let files = RegionFiles::new(dir.path().join("cache"));
    let mut journal = Journal::open(&dir.path().join("journal"), MAX).unwrap();
    journal.append(&TestRecords::record(ChunkPos::new(1, 1, 0), 70, 5)).unwrap();
    journal.append(&TestRecords::record(ChunkPos::new(17, 1, 0), 71, 6)).unwrap();
    assert_eq!(journal.compact(&files).unwrap(), 2);
    assert_eq!(journal.indexed_len(), 0);
    assert_eq!(journal.live_bytes(), 0);
    let region = files.read(RegionPos::new(0, 0, 0)).unwrap().unwrap();
    assert_eq!(region.surfaces[16 + 1].as_ref().unwrap().height[0], 70);
    drop(journal);
    let reopened = Journal::open(&dir.path().join("journal"), MAX).unwrap();
    assert_eq!(reopened.indexed_len(), 0);
}

#[test]
fn compaction_lays_new_records_over_an_existing_region_file() {
    let dir = tempfile::tempdir().unwrap();
    let files = RegionFiles::new(dir.path().join("cache"));
    let mut journal = Journal::open(&dir.path().join("journal"), MAX).unwrap();
    journal.append(&TestRecords::record(ChunkPos::new(0, 0, 0), 70, 5)).unwrap();
    journal.compact(&files).unwrap();
    journal.append(&TestRecords::record(ChunkPos::new(1, 0, 0), 80, 6)).unwrap();
    journal.compact(&files).unwrap();
    let region = files.read(RegionPos::new(0, 0, 0)).unwrap().unwrap();
    assert_eq!(region.surface_count(), 2);
}

#[test]
fn a_torn_tail_after_a_crash_keeps_the_earlier_records() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut journal = Journal::open(dir.path(), MAX).unwrap();
        journal.append(&TestRecords::record(ChunkPos::new(0, 0, 0), 64, 1)).unwrap();
        journal.append(&TestRecords::record(ChunkPos::new(1, 0, 0), 64, 2)).unwrap();
        journal.sync().unwrap();
    }
    let segment = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| fs::metadata(p).unwrap().len() > 64)
        .max()
        .unwrap();
    let length = fs::metadata(&segment).unwrap().len();
    fs::OpenOptions::new().write(true).open(&segment).unwrap().set_len(length - 5).unwrap();
    let journal = Journal::open(dir.path(), MAX).unwrap();
    assert_eq!(journal.indexed_len(), 1);
}

#[test]
fn the_journal_asks_for_compaction_past_its_size_limit() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::open(dir.path(), 200).unwrap();
    assert!(!journal.needs_compaction());
    for x in 0..8 {
        journal.append(&TestRecords::record(ChunkPos::new(x, 0, 0), 64 + x as i16, x as u32)).unwrap();
    }
    assert!(journal.needs_compaction());
}

#[test]
fn written_bytes_count_records_and_compacted_region_files() {
    let dir = tempfile::tempdir().unwrap();
    let files = RegionFiles::new(dir.path().join("regions"));
    let mut journal = Journal::open(&dir.path().join("journal"), MAX).unwrap();
    assert_eq!(journal.bytes_written(), 0);
    journal.append(&TestRecords::record(ChunkPos::new(1, 1, 0), 64, 1)).unwrap();
    let after_append = journal.bytes_written();
    assert!(after_append > 0);
    journal.compact(&files).unwrap();
    let region_file =
        minimap_store::services::CachePaths::region_file(&dir.path().join("regions"), RegionPos::new(0, 0, 0));
    let file_len = fs::metadata(region_file).unwrap().len();
    assert_eq!(journal.bytes_written(), after_append + file_len);
}
