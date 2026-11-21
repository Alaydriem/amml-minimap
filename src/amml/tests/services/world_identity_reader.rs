use std::path::PathBuf;

use alaydriem_minimap::services::WorldIdentityReader;
use amml_api::{Connection, ConnectionKind, WorldInfo};
use minimap_store::structs::WorldIdentity;

fn info(folder: Option<&str>, seed: Option<u64>, connection: Option<Connection>) -> WorldInfo {
    WorldInfo {
        storage_folder: folder.map(PathBuf::from),
        level_name: None,
        seed,
        realm: None,
        commands_enabled: None,
        connection,
    }
}

fn remote(host: &str) -> Connection {
    Connection { kind: ConnectionKind::Remote, host: host.to_string(), port: 19132 }
}

#[test]
fn a_remote_connection_names_a_server_by_host_port_and_low_seed_bits() {
    let dir = tempfile::tempdir().unwrap();
    let reader = WorldIdentityReader::new(dir.path());

    let identity = reader.read(Some(&info(Some("C:/worlds/a"), Some(0x1_0000_0007), Some(remote("play.example")))));

    assert_eq!(identity, WorldIdentity::Remote { host: "play.example".to_string(), port: 19132, seed: 7 });
    assert!(!reader.from_world_folder(&identity));
}

#[test]
fn a_local_world_keeps_its_cache_in_its_folder() {
    let dir = tempfile::tempdir().unwrap();
    let reader = WorldIdentityReader::new(dir.path());
    let local = Connection { kind: ConnectionKind::Local, host: String::new(), port: 0 };

    let identity = reader.read(Some(&info(Some("C:/worlds/a"), Some(5), Some(local))));

    let world_dir = PathBuf::from("C:/worlds/a").join("@alaydriem").join("minimap").join("data");
    assert_eq!(identity, WorldIdentity::Local { world_dir });
    assert!(reader.from_world_folder(&identity));
}

#[test]
fn a_remote_connection_without_a_host_is_not_a_server() {
    let dir = tempfile::tempdir().unwrap();
    let reader = WorldIdentityReader::new(dir.path());

    let identity = reader.read(Some(&info(Some("C:/worlds/a"), Some(5), Some(remote("")))));

    assert!(matches!(identity, WorldIdentity::Local { .. }));
}

#[test]
fn without_a_folder_the_seed_names_the_cache_under_the_data_folder() {
    let dir = tempfile::tempdir().unwrap();
    let reader = WorldIdentityReader::new(dir.path());

    let identity = reader.read(Some(&info(None, Some(0x1_0000_0009), None)));

    assert_eq!(identity, WorldIdentity::Local { world_dir: dir.path().join("local").join("S9") });
    assert!(!reader.from_world_folder(&identity));
}

#[test]
fn without_a_folder_or_a_seed_each_entry_gets_a_cache_of_its_own() {
    let dir = tempfile::tempdir().unwrap();
    let reader = WorldIdentityReader::new(dir.path());
    let pid = std::process::id();

    let zero = reader.read(Some(&info(None, Some(0), None)));
    let none = reader.read(None);

    let local = dir.path().join("local");
    assert_eq!(zero, WorldIdentity::Local { world_dir: local.join(format!("unseeded-{pid}-0")) });
    assert_eq!(none, WorldIdentity::Local { world_dir: local.join(format!("unseeded-{pid}-1")) });
}

#[test]
fn the_sweep_removes_unseeded_caches_of_ended_processes_only() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new("cmd").args(["/C", "exit 0"]).spawn().unwrap();
    let ended = child.id();
    child.wait().unwrap();
    let local = dir.path().join("local");
    let mine = local.join(format!("unseeded-{}-0", std::process::id()));
    let gone = local.join(format!("unseeded-{ended}-0"));
    let seeded = local.join("S9");
    for folder in [&mine, &gone, &seeded] {
        std::fs::create_dir_all(folder).unwrap();
    }

    WorldIdentityReader::sweep_unseeded(dir.path());

    assert!(mine.exists());
    assert!(!gone.exists());
    assert!(seeded.exists());
}
