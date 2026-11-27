use std::collections::HashMap;

use alaydriem_minimap::traits::MapSink;
use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::errors::RuntimeError;
use minimap_runtime::structs::{ProbeRequest, ScanPlan, SubmitStatus, TickInput};
use minimap_store::structs::WorldIdentity;

use crate::support::SinkCall;

// a MapSink that records each call and answers with the plan, colour table and void colours a test sets
#[derive(Default)]
pub struct RecordingSink {
    pub calls: Vec<SinkCall>,
    pub plan: ScanPlan,
    pub colors: Vec<(String, u32)>,
    pub void_colors: HashMap<String, u32>,
    // the column a block change asks for, by block x and z
    pub column_requests: HashMap<(i32, i32), (ChunkPos, usize)>,
    pub ticks: Vec<TickInput>,
    pub submitted: Vec<(ChunkPos, SubmitStatus, ChunkProbe)>,
}

impl RecordingSink {
    pub fn submitted_status(&self) -> Vec<(ChunkPos, SubmitStatus)> {
        self.submitted.iter().map(|(pos, status, _)| (*pos, *status)).collect()
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
        self.ticks.push(*input);
        *plan = self.plan.clone();
    }

    fn submit_chunk(&mut self, pos: ChunkPos, status: SubmitStatus, probe: &ChunkProbe) {
        self.submitted.push((pos, status, *probe));
    }

    fn block_changed(&mut self, x: i32, z: i32, dim: i32) -> ProbeRequest {
        self.calls.push(SinkCall::BlockChanged(x, z, dim));
        match self.column_requests.get(&(x, z)) {
            Some((pos, index)) => ProbeRequest::Column { pos: *pos, index: *index },
            None => ProbeRequest::None,
        }
    }

    fn submit_column(&mut self, pos: ChunkPos, index: usize, sample: ColumnSample) {
        self.calls.push(SinkCall::Column(pos, index, sample));
    }

    fn biome_name(&mut self, id: u16, name: &str) {
        self.calls.push(SinkCall::BiomeName(id, name.to_string()));
    }

    fn resolve_void_color(&self, biome: &str, fallback: u32) -> u32 {
        self.void_colors.get(biome).copied().unwrap_or(fallback)
    }

    fn block_colors(&self) -> &[(String, u32)] {
        &self.colors
    }
}
