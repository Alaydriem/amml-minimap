use std::path::{Path, PathBuf};

use minimap_core::structs::pos::RegionPos;
use minimap_store::services::CachePaths;
use minimap_store::structs::WorldIdentity;

#[test]
fn a_local_world_keeps_its_cache_beside_the_world() {
    let identity = WorldIdentity::Local { world_dir: PathBuf::from("worlds/abc") };
    assert_eq!(CachePaths::root(&identity, Path::new("mods/data")), Path::new("worlds/abc/minimap_cache"));
}

#[test]
fn a_server_cache_is_keyed_by_a_sanitized_address_port_and_seed() {
    let identity = WorldIdentity::Remote { host: "::1".to_string(), port: 19132, seed: 42 };
    let root = CachePaths::root(&identity, Path::new("mods/data"));
    assert_eq!(root, Path::new("mods/data/servers/__1_19132_S42/minimap_cache"));
}

#[test]
fn an_empty_host_becomes_unknown() {
    assert_eq!(CachePaths::sanitize(""), "unknown");
    assert_eq!(CachePaths::sanitize("a*b?c"), "a_b_c");
}

#[test]
fn region_files_and_journals_are_split_by_dimension() {
    let root = Path::new("cache");
    assert_eq!(CachePaths::region_file(root, RegionPos::new(-3, 4, 1)), Path::new("cache/regions/1/r.-3.4.lmr"));
    assert_eq!(CachePaths::journal_dir(root, 2), Path::new("cache/journal/2"));
}

#[test]
fn the_waypoints_file_sits_beside_the_cache_folder_for_worlds_and_servers() {
    let local = WorldIdentity::Local { world_dir: PathBuf::from("worlds/abc") };
    assert_eq!(CachePaths::waypoints_file(&local, Path::new("mods/data")), Path::new("worlds/abc/waypoints.json"));
    let remote = WorldIdentity::Remote { host: "play.example".to_string(), port: 19132, seed: 7 };
    assert_eq!(
        CachePaths::waypoints_file(&remote, Path::new("mods/data")),
        Path::new("mods/data/servers/play.example_19132_S7/waypoints.json")
    );
}

#[test]
fn an_old_cache_folder_moves_to_the_new_name() {
    let dir = tempfile::tempdir().unwrap();
    let legacy = dir.path().join(CachePaths::LEGACY_FOLDER);
    std::fs::create_dir_all(legacy.join("regions")).unwrap();
    let root = dir.path().join(CachePaths::FOLDER);
    assert!(CachePaths::adopt_legacy(&root).unwrap());
    assert!(root.join("regions").is_dir());
    assert!(!legacy.exists());
}

#[test]
fn an_existing_new_cache_is_kept_and_the_old_one_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let legacy = dir.path().join(CachePaths::LEGACY_FOLDER);
    let root = dir.path().join(CachePaths::FOLDER);
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::create_dir_all(&root).unwrap();
    assert!(!CachePaths::adopt_legacy(&root).unwrap());
    assert!(legacy.is_dir());
}

#[test]
fn nothing_moves_without_an_old_cache() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join(CachePaths::FOLDER);
    assert!(!CachePaths::adopt_legacy(&root).unwrap());
    assert!(!root.exists());
}
