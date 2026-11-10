use std::fs;

use minimap_core::structs::biome::BiomeKey;
use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_store::services::{CachePaths, RegionFiles};
use minimap_store::structs::RegionSnapshot;

use crate::support::TestRecords;

fn snapshot(region: RegionPos) -> RegionSnapshot {
    let mut snapshot = RegionSnapshot::new(region, 1);
    snapshot.apply(&TestRecords::record(region.chunk(3, 4), 70, 0xFF00_00AA));
    snapshot
}

#[test]
fn a_region_never_written_reads_as_none() {
    let dir = tempfile::tempdir().unwrap();
    assert!(RegionFiles::new(dir.path().to_path_buf()).read(RegionPos::new(0, 0, 0)).unwrap().is_none());
}

#[test]
fn a_written_region_reads_back_and_leaves_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let files = RegionFiles::new(dir.path().to_path_buf());
    let region = RegionPos::new(-1, 2, 0);
    files.write(&snapshot(region)).unwrap();
    files.write(&snapshot(region)).unwrap();
    assert_eq!(files.read(region).unwrap(), Some(snapshot(region)));
    let folder = CachePaths::region_file(dir.path(), region).parent().unwrap().to_path_buf();
    let names: Vec<String> =
        fs::read_dir(folder).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    assert_eq!(names, vec!["r.-1.2.lmr".to_string()]);
}

#[test]
fn a_corrupt_file_is_set_aside_and_reads_as_none() {
    let dir = tempfile::tempdir().unwrap();
    let files = RegionFiles::new(dir.path().to_path_buf());
    let region = RegionPos::new(0, 0, 0);
    files.write(&snapshot(region)).unwrap();
    let path = CachePaths::region_file(dir.path(), region);
    fs::write(&path, b"LMRG garbage").unwrap();
    assert!(files.read(region).unwrap().is_none());
    assert!(!path.exists());
    assert!(path.with_extension("lmr.corrupt").exists());
}

#[test]
fn a_file_naming_another_region_is_set_aside() {
    let dir = tempfile::tempdir().unwrap();
    let files = RegionFiles::new(dir.path().to_path_buf());
    files.write(&snapshot(RegionPos::new(5, 5, 0))).unwrap();
    let wrong = CachePaths::region_file(dir.path(), RegionPos::new(6, 5, 0));
    fs::copy(CachePaths::region_file(dir.path(), RegionPos::new(5, 5, 0)), &wrong).unwrap();
    assert!(files.read(RegionPos::new(6, 5, 0)).unwrap().is_none());
    assert!(!files.read(RegionPos::new(5, 5, 0)).unwrap().is_none());
    assert!(ChunkPos::new(80, 80, 0).region() == RegionPos::new(5, 5, 0));
}

#[test]
fn listing_a_dimension_finds_its_region_files_and_skips_other_names() {
    let dir = tempfile::tempdir().unwrap();
    let files = RegionFiles::new(dir.path().to_path_buf());
    assert!(files.list(0).unwrap().is_empty());
    for region in [RegionPos::new(-3, 2, 0), RegionPos::new(4, -1, 0), RegionPos::new(9, 9, 1)] {
        files.write(&snapshot(region)).unwrap();
    }
    let overworld = dir.path().join("regions").join("0");
    std::fs::write(overworld.join("r.1.2.lmr.corrupt"), b"x").unwrap();
    std::fs::write(overworld.join("r.a.2.lmr"), b"x").unwrap();
    std::fs::write(overworld.join("notes.txt"), b"x").unwrap();
    assert_eq!(files.list(0).unwrap(), vec![RegionPos::new(4, -1, 0), RegionPos::new(-3, 2, 0)]);
    assert_eq!(files.list(1).unwrap(), vec![RegionPos::new(9, 9, 1)]);
}

#[test]
fn an_older_region_file_is_removed_and_reads_as_missing() {
    let dir = tempfile::tempdir().unwrap();
    let files = RegionFiles::new(dir.path().to_path_buf());
    let region = RegionPos::new(0, 0, 0);
    files.write(&snapshot(region)).unwrap();
    let path = CachePaths::region_file(dir.path(), region);
    let mut bytes = fs::read(&path).unwrap();
    bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    fs::write(&path, bytes).unwrap();
    assert_eq!(files.read(region).unwrap(), None);
    assert!(!path.exists());
    assert!(!path.with_extension("lmr.corrupt").exists());
}

#[test]
fn a_region_read_under_another_table_has_no_biomes() {
    let dir = tempfile::tempdir().unwrap();
    let mut files = RegionFiles::new(dir.path().to_path_buf());
    let region = RegionPos::new(0, 0, 0);
    let mut written = snapshot(region);
    written.surfaces[4 * 16 + 3].as_mut().unwrap().biome.set(0, Some(BiomeKey(1)));
    written.table_id = 5;
    files.set_table_id(5);
    files.write(&written).unwrap();
    let same = files.read(region).unwrap().unwrap();
    assert_eq!(same.surfaces[4 * 16 + 3].as_ref().unwrap().biome.key(0), Some(BiomeKey(1)));
    files.set_table_id(6);
    let other = files.read(region).unwrap().unwrap();
    assert_eq!(other.surfaces[4 * 16 + 3].as_ref().unwrap().biome.key(0), None);
}
