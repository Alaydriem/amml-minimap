use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::errors::RuntimeError;
use minimap_runtime::structs::{ProbeRequest, ScanPlan, SubmitStatus, TickInput};
use minimap_store::structs::WorldIdentity;

// what the game side hands to the minimap and asks of it: the calls of minimap_runtime::Runtime, with the tick
// going through the manager session
pub trait MapSink {
    fn enter_world(&mut self, identity: &WorldIdentity, dim: i32) -> Result<(), RuntimeError>;

    fn leave_world(&mut self);

    // drains the manager's events, runs the runtime and fills `plan`
    fn tick(&mut self, input: &TickInput, plan: &mut ScanPlan);

    fn submit_chunk(&mut self, pos: ChunkPos, status: SubmitStatus, probe: &ChunkProbe);

    fn block_changed(&mut self, x: i32, z: i32, dim: i32) -> ProbeRequest;

    fn submit_column(&mut self, pos: ChunkPos, index: usize, sample: ColumnSample);

    fn biome_name(&mut self, id: u16, name: &str);

    // the colour of a column with no blocks: the biome's override, else `fallback`
    fn resolve_void_color(&self, biome: &str, fallback: u32) -> u32;

    // block name and packed colour pairs drawn in place of the game's map colour
    fn block_colors(&self) -> &[(String, u32)];
}
