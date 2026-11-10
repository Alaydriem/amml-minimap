use alaydriem_minimap::traits::MapSink;
use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::errors::RuntimeError;
use minimap_runtime::structs::{ProbeRequest, ScanPlan, SubmitStatus, TickInput};
use minimap_store::structs::WorldIdentity;

use super::SinkCall;

// a MapSink that records each call; its tick hands out `plan`, its block change answers `request`
pub struct RecordingSink {
    pub calls: Vec<SinkCall>,
    pub probes: Vec<ChunkProbe>,
    pub samples: Vec<ColumnSample>,
    pub plan: ScanPlan,
    pub request: ProbeRequest,
    pub colors: Vec<(String, u32)>,
}

impl RecordingSink {
    pub fn new(plan: ScanPlan) -> Self {
        Self {
            calls: Vec::new(),
            probes: Vec::new(),
            samples: Vec::new(),
            plan,
            request: ProbeRequest::None,
            colors: vec![("minecraft:lava".to_string(), u32::from_le_bytes([230, 110, 20, 255]))],
        }
    }

    // a plan that asks for the chunk at (0, 0) with the camera above the world
    pub fn probing() -> Self {
        let plan = ScanPlan {
            candidates: vec![ChunkPos::new(0, 0, 0)],
            limit: 8,
            camera_height: 320,
            transparent_water: true,
            ..ScanPlan::default()
        };
        Self::new(plan)
    }
}

impl MapSink for RecordingSink {
    fn enter_world(&mut self, identity: &WorldIdentity, dim: i32) -> Result<(), RuntimeError> {
        self.calls.push(SinkCall::Enter(identity.clone(), dim));
        Ok(())
    }

    fn leave_world(&mut self) {
        self.calls.push(SinkCall::Leave);
    }

    fn tick(&mut self, input: &TickInput, plan: &mut ScanPlan) {
        self.calls.push(SinkCall::Tick { present: input.player.present });
        *plan = if input.player.present { self.plan.clone() } else { ScanPlan::default() };
    }

    fn submit_chunk(&mut self, pos: ChunkPos, status: SubmitStatus, probe: &ChunkProbe) {
        self.calls.push(SinkCall::Submit(pos, status));
        self.probes.push(*probe);
    }

    fn block_changed(&mut self, x: i32, z: i32, dim: i32) -> ProbeRequest {
        self.calls.push(SinkCall::BlockChanged(x, z, dim));
        self.request
    }

    fn submit_column(&mut self, pos: ChunkPos, index: usize, sample: ColumnSample) {
        self.calls.push(SinkCall::Column(pos, index));
        self.samples.push(sample);
    }

    fn biome_name(&mut self, id: u16, name: &str) {
        self.calls.push(SinkCall::BiomeName(id, name.to_string()));
    }

    fn resolve_void_color(&self, _biome: &str, fallback: u32) -> u32 {
        fallback
    }

    fn block_colors(&self) -> &[(String, u32)] {
        &self.colors
    }
}
