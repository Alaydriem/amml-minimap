use std::path::PathBuf;

use amm_abi::types::{AMM_ABI_MAJOR, AmmStatus};
use amm_client::link::{ApiTable, ManagerLink};
use amm_core::structs::frame::LayerKind;
use amm_ffi::api::amm_api_for;
use amm_ffi::host::{amm_host_create, amm_host_destroy, amm_host_frame, amm_host_key, amm_host_pointer, amm_host_tick};
use amm_ffi::services::HostCell;
use amm_ffi::types::{AmmHostFrame, AmmHostFrameInput, AmmHostLayer};
use minimap_core::structs::view::PlayerSnapshot;
use minimap_link::services::{MinimapSession, SystemClipboard};
use minimap_runtime::structs::{RuntimePaths, ScanPlan, TickInput};
use tempfile::TempDir;

// the real mod manager, linked into the test process, with minimap registered on it the way enable() does;
// the host is process-global, so each test runs alone in its process (nextest)
pub struct Host {
    dir: TempDir,
    pub session: Option<MinimapSession<ManagerLink, SystemClipboard>>,
    pub now_ms: u64,
}

impl Host {
    pub const CTRL: u8 = 1;
    pub const SHIFT: u8 = 2;
    const WIDTH: u32 = 1280;
    const HEIGHT: u32 = 720;
    const FRAME_MS: u64 = 16;

    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("manager").display().to_string();
        let game = dir.path().join("game").display().to_string();
        let version = "1.0.0-minimap-e2e";
        // SAFETY: the three texts live for the call
        let status = unsafe {
            amm_host_create(config.as_ptr(), config.len(), game.as_ptr(), game.len(), version.as_ptr(), version.len())
        };
        assert_eq!(status, AmmStatus::Ok);
        let ptr = amm_api_for(AMM_ABI_MAJOR);
        // SAFETY: amm_api_for returns null or the manager's static table
        let table = unsafe { ApiTable::new(ptr) }.unwrap();
        let paths = RuntimePaths { data_dir: dir.path().join("data") };
        let session = MinimapSession::connect_with(table, &dir.path().join("mods").join("minimap"), paths).unwrap();
        let mut host = Self { dir, session: Some(session), now_ms: 1_000 };
        host.tick();
        host
    }

    pub fn world_dir(&self) -> PathBuf {
        self.dir.path().join("world")
    }

    pub fn config_file(&self) -> PathBuf {
        self.dir.path().join("mods").join("minimap").join("config.hcl")
    }

    pub fn session(&mut self) -> &mut MinimapSession<ManagerLink, SystemClipboard> {
        self.session.as_mut().unwrap()
    }

    // a key press and release through the manager's router, then a game tick
    pub fn key(&mut self, vk: u32, mods: u8) {
        amm_host_key(vk, true, false, mods, self.now_ms);
        amm_host_key(vk, false, false, mods, self.now_ms + 1);
        self.tick();
    }

    // the manager's tick, then minimap's, with the player in the overworld holding the mouse
    pub fn tick(&mut self) -> ScanPlan {
        self.now_ms += 50;
        amm_host_tick(self.now_ms);
        let player = PlayerSnapshot { present: true, x: 8.5, y: 80.0, z: 8.5, yaw: 0.0, dim: 0 };
        let input =
            TickInput { player, foreground: true, cursor_hidden: true, mouse_grabbed: true, ..TickInput::default() };
        let mut plan = ScanPlan::default();
        self.session().tick(&input, &mut plan);
        plan
    }

    pub fn pointer_move(&mut self, x: f32, y: f32) {
        const MOVE: u8 = 0;
        amm_host_pointer(MOVE, x, y, 0, false, 0.0);
    }

    // the mod index and whether it is the Overlay layer, of the layer the manager routes presses to now
    pub fn pointer_layer(&self) -> Option<(u8, bool)> {
        HostCell::global()
            .with(|runtime| Ok(runtime.manager().pointer_layer()))
            .ok()
            .flatten()
            .map(|entry| (entry.mod_index.0, entry.kind == LayerKind::Overlay))
    }

    // one manager frame; returns the owner of each layer the manager drew
    pub fn frame(&mut self) -> Vec<u8> {
        self.now_ms += Self::FRAME_MS;
        let input = AmmHostFrameInput {
            width: Self::WIDTH,
            height: Self::HEIGHT,
            pixels_per_point: 1.0,
            time_secs: self.now_ms as f64 / 1000.0,
            now_ms: self.now_ms,
            menu_screen: 0,
            // DXGI_FORMAT_R8G8B8A8_UNORM, with the client area the size of the back buffer
            format: 28,
            client_width: Self::WIDTH,
            client_height: Self::HEIGHT,
        };
        let mut out = AmmHostFrame::empty();
        // SAFETY: input and out live for the call
        assert_eq!(unsafe { amm_host_frame(&input, &mut out) }, AmmStatus::Ok);
        if out.layer_count == 0 {
            return Vec::new();
        }
        // SAFETY: the host keeps layer_count layers at layers until the next frame
        let layers: &[AmmHostLayer] = unsafe { std::slice::from_raw_parts(out.layers, out.layer_count as usize) };
        layers.iter().map(|layer| layer.owner).collect()
    }

    // gives the leases back, unregisters and stops the runtime, then stops the host; the folder comes back so
    // a test can read what the manager wrote
    pub fn shutdown(mut self) -> TempDir {
        if let Some(session) = self.session.take() {
            session.shutdown();
        }
        amm_host_destroy();
        self.dir
    }
}

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}
