use std::path::PathBuf;

use minimap_store::structs::WorldIdentity;

// a game world folder holding the mod's data folder, as the AMML loader lays it out: <world>/@namespace/name/data
fn world_with_mod_data(level_name: Option<&str>) -> (tempfile::TempDir, WorldIdentity) {
    let dir = tempfile::tempdir().unwrap();
    let world = dir.path().join("XuODV9RMt48=");
    let mod_data = world.join("@some").join("mod").join("data");
    std::fs::create_dir_all(&mod_data).unwrap();
    if let Some(name) = level_name {
        std::fs::write(world.join(WorldIdentity::LEVEL_NAME_FILE), name).unwrap();
    }
    (dir, WorldIdentity::Local { world_dir: mod_data })
}

#[test]
fn a_world_is_labelled_by_its_level_name_not_by_the_mod_data_folder() {
    let (_dir, identity) = world_with_mod_data(Some("  My Survival World\r\n"));
    assert_eq!(identity.label(), "My Survival World");
}

#[test]
fn without_a_level_name_the_world_folder_names_it() {
    let (_dir, identity) = world_with_mod_data(None);
    assert_eq!(identity.label(), "XuODV9RMt48=");
    let (_dir, blank) = world_with_mod_data(Some("   "));
    assert_eq!(blank.label(), "XuODV9RMt48=");
    let plain = WorldIdentity::Local { world_dir: PathBuf::from("worlds/JXaayLKvlr8=") };
    assert_eq!(plain.label(), "JXaayLKvlr8=");
}

#[test]
fn a_server_is_labelled_by_its_address() {
    let remote = WorldIdentity::Remote { host: "play.example".to_string(), port: 19132, seed: 3 };
    assert_eq!(remote.label(), "play.example_19132");
}
