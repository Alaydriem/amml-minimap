use std::sync::{Arc, OnceLock};

use amm_client::test_support::{FakeConnection, FakeTable};
use amm_core::errors::LinkError;
use amm_core::structs::manager::TextureGrant;
use minimap_link::errors::SessionError;
use minimap_link::services::{KeyFeed, MinimapSession};
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::RuntimePaths;

fn paths(dir: &tempfile::TempDir) -> RuntimePaths {
    RuntimePaths { data_dir: dir.path().join("data") }
}

#[test]
fn open_on_a_connected_link_puts_its_runtime_in_the_frame_slot() {
    let dir = tempfile::tempdir().unwrap();
    let FakeConnection { link, game } = FakeTable::link_without_frame();
    FakeTable::with(|state| state.texture_grant = TextureGrant { bytes: 96 * 1024 * 1024, count: 512 });
    let slot: Arc<OnceLock<Arc<Runtime>>> = Arc::new(OnceLock::new());

    let session = MinimapSession::open(link, &slot, Arc::new(KeyFeed::new()), paths(&dir)).unwrap();

    let in_slot = slot.get().unwrap();
    assert!(Arc::ptr_eq(in_slot, session.runtime()));
    session.shutdown();
    drop(game);
}

#[test]
fn open_on_a_link_that_waits_for_the_manager_is_not_connected() {
    let dir = tempfile::tempdir().unwrap();
    let game = FakeTable::game();
    let ctx = FakeTable::ctx_of(&game, FakeTable::DESCRIPTOR);
    let link = amm_client::link::ManagerLink::start_without_frame(&ctx, FakeTable::DESCRIPTOR).unwrap();
    let slot: Arc<OnceLock<Arc<Runtime>>> = Arc::new(OnceLock::new());

    let result = MinimapSession::open(link, &slot, Arc::new(KeyFeed::new()), paths(&dir));

    assert!(matches!(result, Err(SessionError::Link(LinkError::NotConnected))), "{:?}", result.err());
    assert!(slot.get().is_none());
}
