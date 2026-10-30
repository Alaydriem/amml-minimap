use std::path::PathBuf;
use std::sync::Arc;

use alaydriem_minimap::services::MapSession;
use alaydriem_minimap::traits::MapSink;
use amm_client::test_support::FakeTable;
use minimap_core::config::Config;
use minimap_core::structs::draw::TextureGrant;
use minimap_link::services::{MinimapSession, SystemClipboard};
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::RuntimePaths;
use minimap_store::structs::WorldIdentity;

// a MapSession over a temporary folder, registered with amm-client's fake manager table
pub struct TestSession {
    pub dir: tempfile::TempDir,
    session: Option<MapSession>,
    world: PathBuf,
}

impl TestSession {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let paths = RuntimePaths { data_dir: dir.path().join("data") };
        let runtime = Arc::new(Runtime::new(Config::default(), paths, TextureGrant::OLD_MANAGER).unwrap());
        let session = MinimapSession::with_port(FakeTable::link_without_frame(), runtime, SystemClipboard);
        let world = dir.path().join("world");
        Self { dir, session: Some(MapSession::new(session)), world }
    }

    pub fn sink(&mut self) -> &mut MapSession {
        self.session.as_mut().unwrap()
    }

    pub fn enter(&mut self) {
        let world = WorldIdentity::Local { world_dir: self.world.clone() };
        self.sink().enter_world(&world, 0).unwrap();
    }
}

impl Drop for TestSession {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            session.shutdown();
        }
    }
}
