use std::path::Path;
use std::sync::Arc;

use amm_client::link::ManagerLink;
use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_link::errors::SessionError;
use minimap_link::services::{MinimapSession, SystemClipboard};
use minimap_runtime::errors::RuntimeError;
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::{ProbeRequest, RuntimePaths, ScanPlan, SubmitStatus, TickInput};
use minimap_store::structs::WorldIdentity;

use crate::traits::MapSink;

// minimap registered with the mod manager, and its running runtime
pub struct MapSession {
    session: MinimapSession<ManagerLink, SystemClipboard>,
    runtime: Arc<Runtime>,
}

impl MapSession {
    // registers with the manager and starts the runtime's threads; `config_dir` is where the manager keeps
    // config.hcl, `data_dir` where the caches of servers and unnamed worlds go
    pub fn connect(config_dir: &Path, data_dir: &Path) -> Result<Self, SessionError> {
        let paths = RuntimePaths { data_dir: data_dir.to_path_buf() };
        Ok(Self::new(MinimapSession::connect(config_dir, paths)?))
    }

    pub fn new(session: MinimapSession<ManagerLink, SystemClipboard>) -> Self {
        let runtime = Arc::clone(session.runtime());
        Self { session, runtime }
    }

    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    pub fn session_mut(&mut self) -> &mut MinimapSession<ManagerLink, SystemClipboard> {
        &mut self.session
    }

    // gives every lease back, unregisters, then stops the runtime's threads
    pub fn shutdown(self) {
        self.session.shutdown();
    }
}

impl MapSink for MapSession {
    fn enter_world(&mut self, identity: &WorldIdentity, dim: i32) -> Result<(), RuntimeError> {
        self.runtime.enter_world(identity, dim)
    }

    fn leave_world(&mut self) {
        self.runtime.leave_world();
    }

    fn tick(&mut self, input: &TickInput, plan: &mut ScanPlan) {
        self.session.tick(input, plan);
    }

    fn submit_chunk(&mut self, pos: ChunkPos, status: SubmitStatus, probe: &ChunkProbe) {
        self.runtime.submit_chunk(pos, status, probe);
    }

    fn block_changed(&mut self, x: i32, z: i32, dim: i32) -> ProbeRequest {
        self.runtime.block_changed(x, z, dim)
    }

    fn submit_column(&mut self, pos: ChunkPos, index: usize, sample: ColumnSample) {
        self.runtime.submit_column(pos, index, sample);
    }

    fn biome_name(&mut self, id: u16, name: &str) {
        self.runtime.biome_name(id, name);
    }

    fn resolve_void_color(&self, biome: &str, fallback: u32) -> u32 {
        self.runtime.resolve_void_color(biome, fallback)
    }

    fn block_colors(&self) -> &[(String, u32)] {
        self.runtime.block_colors()
    }
}
