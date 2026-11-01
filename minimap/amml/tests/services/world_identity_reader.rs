use std::ffi::c_void;
use std::path::Path;
use std::ptr::NonNull;

use alaydriem_minimap::game::{GameConnectionInfo, Level};
use alaydriem_minimap::services::WorldIdentityReader;
use alaydriem_minimap::types::MinimapEntry;
use amml_game::{ClientInstance, GameEntries};
use amml_msvc::MsvcString;
use minimap_store::structs::WorldIdentity;

use crate::support::{FakeClient, FakeConnection, FakeHeap, FakeStorage, FakeWorld, HeapEvent};

fn read(world: &mut FakeWorld, storage: Option<NonNull<c_void>>) -> WorldIdentity {
    read_with(world, storage, &FakeWorld::entries(true))
}

fn read_with(world: &mut FakeWorld, storage: Option<NonNull<c_void>>, entries: &GameEntries) -> WorldIdentity {
    let reader = WorldIdentityReader::new(Path::new("data"), Some(FakeHeap::msvc()));
    // SAFETY: live fakes
    let client = unsafe { ClientInstance::from_raw(world.client.ptr(), entries) }.unwrap();
    // SAFETY: as above
    let level = unsafe { Level::from_raw(world.level.ptr(), entries) };
    reader.read(entries, &client, level.as_ref(), storage)
}

// the block the reader copied the connection info into: the one allocation of the optional's size
fn copy_block() -> usize {
    let size = GameConnectionInfo::optional_size(FakeClient::INFO_SIZE_26_51 as usize);
    let blocks: Vec<usize> = FakeHeap::events()
        .into_iter()
        .filter_map(|event| match event {
            HeapEvent::Allocate(ptr, bytes) if bytes == size => Some(ptr),
            _ => None,
        })
        .collect();
    assert_eq!(blocks.len(), 1, "one copy block");
    blocks[0]
}

#[test]
fn no_connection_info_and_no_storage_give_the_local_seed_key() {
    let mut world = FakeWorld::new("minecraft:stone");

    let identity = read(&mut world, None);

    assert_eq!(identity, WorldIdentity::Local { world_dir: Path::new("data").join("local").join("S1234") });
    assert_eq!(world.client.host_block, 0, "no host string was made");
}

#[test]
fn a_server_is_named_by_host_port_and_seed_and_its_host_string_is_freed_once() {
    let mut world = FakeWorld::new("minecraft:stone");
    let host = "play.a-very-long-server-name.example".to_string();
    world.client.connection = Some(FakeConnection { connection_type: 1, host: host.clone(), port: 19132 });

    let identity = read(&mut world, None);

    assert_eq!(identity, WorldIdentity::Remote { host, port: 19132, seed: 1234 });
    assert_ne!(world.client.host_block, 0);
    assert_eq!(FakeHeap::frees_of(world.client.host_block), 1);
}

#[test]
fn a_local_connection_is_not_a_server_and_its_host_string_is_still_freed_once() {
    let mut world = FakeWorld::new("minecraft:stone");
    let host = "a-local-host-name-longer-than-fifteen".to_string();
    world.client.connection = Some(FakeConnection { connection_type: 0, host, port: 19132 });

    let identity = read(&mut world, None);

    assert_eq!(identity, WorldIdentity::Local { world_dir: Path::new("data").join("local").join("S1234") });
    assert_eq!(FakeHeap::frees_of(world.client.host_block), 1);
}

// a fake DBStorage of `C:/worlds/abc=` and the entries that read it; read with `connection` on the client
fn read_with_storage(connection: Option<FakeConnection>) -> (WorldIdentity, FakeWorld) {
    let mut world = FakeWorld::new("minecraft:stone");
    world.client.connection = connection;
    // SAFETY: FakeHeap's functions are a valid allocate and free pair
    let path = unsafe { MsvcString::new_in(&FakeHeap::msvc(), "C:/worlds/abc=") }.unwrap();
    let mut storage = FakeStorage::new(FakeStorage::VTABLE, path);
    let storage_ptr = NonNull::new((&raw mut storage).cast::<c_void>()).unwrap();
    let entries = FakeWorld::entries_with(&[(MinimapEntry::DB_STORAGE_VTABLE, FakeStorage::VTABLE as u64)]);
    let reader = WorldIdentityReader::new(Path::new("data"), Some(FakeHeap::msvc()));
    // SAFETY: a live fake
    let client = unsafe { ClientInstance::from_raw(world.client.ptr(), &entries) }.unwrap();

    let identity = reader.read(&entries, &client, None, Some(storage_ptr));

    // SAFETY: made on FakeHeap above and not used again
    unsafe { storage.path.release(&FakeHeap::msvc()) };
    (identity, world)
}

#[test]
fn a_local_world_caches_under_the_namespaced_data_folder() {
    let connection =
        FakeConnection { connection_type: 0, host: "a-local-host-name-longer-than-fifteen".into(), port: 1 };

    let (identity, world) = read_with_storage(Some(connection));

    let folder = Path::new("C:/worlds/abc=").join("@alaydriem").join("minimap").join("data");
    assert_eq!(identity, WorldIdentity::Local { world_dir: folder });
    assert_ne!(world.client.host_block, 0, "the connection is read before the storage");
    assert_eq!(FakeHeap::frees_of(world.client.host_block), 1);
}

#[test]
fn a_live_storage_and_a_remote_connection_give_the_server_key() {
    let host = "play.a-very-long-server-name.example".to_string();
    let connection = FakeConnection { connection_type: 1, host: host.clone(), port: 19132 };

    let (identity, world) = read_with_storage(Some(connection));

    assert_eq!(identity, WorldIdentity::Remote { host, port: 19132, seed: 0 });
    assert_eq!(FakeHeap::frees_of(world.client.host_block), 1);
    let block = copy_block();
    assert_eq!(FakeHeap::events().iter().filter(|event| **event == HeapEvent::Destroy(block)).count(), 1);
    assert_eq!(FakeHeap::frees_of(block), 1);
}

#[test]
fn the_copy_block_is_sized_as_the_optional_value_flag_and_padding() {
    assert_eq!(GameConnectionInfo::optional_size(0x310), 0x318);
    assert_eq!(GameConnectionInfo::optional_size(0x2f0), 0x2f8);
    assert_eq!(GameConnectionInfo::optional_size(0x2f1), 0x2f8);
    let mut world = FakeWorld::new("minecraft:stone");
    world.client.connection = Some(FakeConnection { connection_type: 1, host: "server".into(), port: 1 });

    read(&mut world, None);

    assert_eq!(FakeHeap::allocations_of(0x318), 1);
    assert_eq!(FakeHeap::allocations_of(0x311), 0);
}

// a fake connection info of the game layout, `size` bytes long, read with the size entry at `size`
fn read_connection_of_size(size: u64) -> (WorldIdentity, FakeWorld) {
    let mut world = FakeWorld::new("minecraft:stone");
    let host = "play.a-very-long-server-name.example".to_string();
    world.client.connection = Some(FakeConnection { connection_type: 1, host, port: 19132 });
    world.client.info_size = size;
    let entries = FakeWorld::entries_with(&[(MinimapEntry::GAME_CONNECTION_INFO_SIZE, size)]);
    let identity = read_with(&mut world, None, &entries);
    (identity, world)
}

// the server key from a connection info of `size` bytes: the mirror reads the type, host and port
fn assert_connection_info_of_size_is_read(size: u64) {
    let (identity, world) = read_connection_of_size(size);

    let host = "play.a-very-long-server-name.example".to_string();
    assert_eq!(identity, WorldIdentity::Remote { host, port: 19132, seed: 1234 });
    assert_eq!(FakeHeap::allocations_of(GameConnectionInfo::optional_size(size as usize)), 1);
    assert_eq!(FakeHeap::frees_of(world.client.host_block), 1);
}

#[test]
fn the_connection_info_mirror_reads_the_26_51_layout() {
    assert_connection_info_of_size_is_read(FakeClient::INFO_SIZE_26_51);
}

#[test]
fn the_connection_info_mirror_reads_the_26_60_layout() {
    assert_connection_info_of_size_is_read(FakeClient::INFO_SIZE_26_60);
}

#[test]
fn a_size_entry_smaller_than_the_mirrored_head_is_refused() {
    let (identity, world) = read_connection_of_size(0x8C);

    let WorldIdentity::Local { .. } = identity else { panic!("no server key from a refused layout") };
    assert_eq!(world.client.host_block, 0, "the getter was not called");
}

#[test]
fn a_storage_that_is_no_longer_a_db_storage_is_not_read() {
    let mut world = FakeWorld::new("minecraft:stone");
    // SAFETY: FakeHeap's functions are a valid allocate and free pair
    let path = unsafe { MsvcString::new_in(&FakeHeap::msvc(), "C:/worlds/gone") }.unwrap();
    let mut storage = FakeStorage::new(0x0BAD, path);
    let storage_ptr = NonNull::new((&raw mut storage).cast::<c_void>()).unwrap();
    let entries = FakeWorld::entries_with(&[(MinimapEntry::DB_STORAGE_VTABLE, FakeStorage::VTABLE as u64)]);
    let reader = WorldIdentityReader::new(Path::new("data"), Some(FakeHeap::msvc()));
    // SAFETY: a live fake
    let client = unsafe { ClientInstance::from_raw(world.client.ptr(), &entries) }.unwrap();

    let identity = reader.read(&entries, &client, None, Some(storage_ptr));

    let WorldIdentity::Local { world_dir } = identity else { panic!("a local key") };
    assert!(world_dir.file_name().unwrap().to_str().unwrap().starts_with("unseeded-"), "{world_dir:?}");
    // SAFETY: made on FakeHeap above and not used again
    unsafe { storage.path.release(&FakeHeap::msvc()) };
}

#[test]
fn the_copy_is_destroyed_once_and_its_block_is_freed_once_after_that() {
    let mut world = FakeWorld::new("minecraft:stone");
    let host = "play.a-very-long-server-name.example".to_string();
    world.client.connection = Some(FakeConnection { connection_type: 1, host, port: 19132 });

    read(&mut world, None);

    let block = copy_block();
    let events = FakeHeap::events();
    let destroyed: Vec<usize> =
        events.iter().enumerate().filter(|(_, e)| **e == HeapEvent::Destroy(block)).map(|(i, _)| i).collect();
    let freed: Vec<usize> =
        events.iter().enumerate().filter(|(_, e)| **e == HeapEvent::Free(block)).map(|(i, _)| i).collect();
    assert_eq!(destroyed.len(), 1, "destroyed once");
    assert_eq!(freed.len(), 1, "freed once");
    assert!(destroyed[0] < freed[0], "destroyed before the block is freed");
    assert_eq!(FakeHeap::frees_of(world.client.host_block), 1, "the destructor freed the host string");
}

#[test]
fn without_the_destructor_only_the_host_string_is_freed_and_the_block_once() {
    let mut world = FakeWorld::new("minecraft:stone");
    let host = "play.a-very-long-server-name.example".to_string();
    world.client.connection = Some(FakeConnection { connection_type: 1, host: host.clone(), port: 19132 });
    let entries = FakeWorld::entries_without(&[], &[MinimapEntry::GAME_CONNECTION_INFO_DTOR]);

    let identity = read_with(&mut world, None, &entries);

    assert_eq!(identity, WorldIdentity::Remote { host, port: 19132, seed: 1234 });
    assert!(!FakeHeap::events().iter().any(|event| matches!(event, HeapEvent::Destroy(_))));
    assert_eq!(FakeHeap::frees_of(world.client.host_block), 1);
    assert_eq!(FakeHeap::frees_of(copy_block()), 1);
}

#[test]
fn an_empty_optional_frees_its_block_once_and_destroys_nothing() {
    let mut world = FakeWorld::new("minecraft:stone");

    read(&mut world, None);

    assert!(!FakeHeap::events().iter().any(|event| matches!(event, HeapEvent::Destroy(_))));
    assert_eq!(FakeHeap::frees_of(copy_block()), 1);
}

#[test]
fn a_world_with_no_seed_gets_a_key_of_its_own_on_each_entry() {
    let mut world = FakeWorld::new("minecraft:stone");
    let entries = FakeWorld::entries(true);
    let reader = WorldIdentityReader::new(Path::new("data"), Some(FakeHeap::msvc()));
    // SAFETY: a live fake
    let client = unsafe { ClientInstance::from_raw(world.client.ptr(), &entries) }.unwrap();

    let first = reader.read(&entries, &client, None, None);
    let second = reader.read(&entries, &client, None, None);

    assert_ne!(first, second);
    for identity in [first, second] {
        let WorldIdentity::Local { world_dir } = identity else { panic!("a local key") };
        assert!(world_dir.starts_with(Path::new("data").join("local")));
        assert!(world_dir.file_name().unwrap().to_str().unwrap().starts_with("unseeded-"), "{world_dir:?}");
    }
}

#[test]
fn the_sweep_removes_only_the_caches_of_ended_processes() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local");
    let mut ended = std::process::Command::new("cmd").args(["/C", "exit"]).spawn().unwrap();
    let ended_pid = ended.id();
    ended.wait().unwrap();
    let mut running = std::process::Command::new("cmd").stdin(std::process::Stdio::piped()).spawn().unwrap();
    let folders = [
        format!("unseeded-{ended_pid}-0"),
        format!("unseeded-{}-0", running.id()),
        format!("unseeded-{}-3", std::process::id()),
        "S5".to_string(),
    ];
    for folder in &folders {
        std::fs::create_dir_all(local.join(folder).join("minimap_cache")).unwrap();
    }

    WorldIdentityReader::sweep_unseeded(dir.path());
    let kept: Vec<bool> = folders.iter().map(|folder| local.join(folder).exists()).collect();
    running.kill().unwrap();
    running.wait().unwrap();

    assert_eq!(kept, vec![false, true, true, true], "ended, another live game, this process, a seeded world");
}
