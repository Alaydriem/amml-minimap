use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::input::KeybindInput;
use minimap_core::structs::pos::ChunkPos;

use crate::structs::{SubmitStatus, TickInput};

// one call from the host into the runtime, in the order the runtime received it
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureEvent {
    Enter { dim: i32 },
    Leave,
    Tick(TickInput),
    Submit { pos: ChunkPos, status: SubmitStatus, probe: Box<ChunkProbe> },
    // the game's answer for a biome ID's internal name; empty when it has none
    BiomeName { id: u16, name: String },
    Column { pos: ChunkPos, index: u32, sample: ColumnSample },
    Block { x: i32, z: i32, dim: i32 },
    Keybind(KeybindInput),
    // a key, mouse or frame record from before the manager drew the overlay; read and skipped
    Retired(u8),
}
