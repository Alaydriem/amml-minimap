use minimap_store::structs::WorldIdentity;

use crate::errors::RuntimeError;
use crate::services::{CaptureReader, Runtime};
use crate::structs::{CaptureEvent, ScanPlan};

// drives a runtime with recorded events, the way the host called it; worlds are entered at `identity`
pub struct CaptureReplay {
    identity: WorldIdentity,
    plan: ScanPlan,
}

impl CaptureReplay {
    pub fn new(identity: WorldIdentity) -> Self {
        Self { identity, plan: ScanPlan::default() }
    }

    pub fn apply(&mut self, runtime: &Runtime, event: &CaptureEvent) -> Result<(), RuntimeError> {
        match event {
            CaptureEvent::Enter { dim } => runtime.enter_world(&self.identity, *dim)?,
            CaptureEvent::Leave => runtime.leave_world(),
            CaptureEvent::Tick(input) => runtime.tick(input, &mut self.plan),
            CaptureEvent::Submit { pos, status, probe } => runtime.submit_chunk(*pos, *status, probe),
            CaptureEvent::BiomeName { id, name } => runtime.biome_name(*id, name),
            CaptureEvent::Column { pos, index, sample } => runtime.submit_column(*pos, *index as usize, *sample),
            CaptureEvent::Block { x, z, dim } => {
                runtime.block_changed(*x, *z, *dim);
            }
            CaptureEvent::Keybind(input) => {
                runtime.keybind(*input);
            }
            CaptureEvent::Retired(_) => {}
        }
        Ok(())
    }

    // replays every event; returns how many were applied
    pub fn run(&mut self, runtime: &Runtime, reader: &mut CaptureReader) -> Result<u64, RuntimeError> {
        let mut count = 0;
        while let Some(event) = reader.next_event()? {
            self.apply(runtime, &event)?;
            count += 1;
        }
        Ok(count)
    }
}
