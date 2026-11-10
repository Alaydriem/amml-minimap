use std::path::PathBuf;

use minimap_core::structs::pos::RegionPos;
use minimap_core::structs::waypoint::Waypoint;

use crate::structs::JournalRecord;

#[derive(Debug)]
pub enum StoreCommand {
    // journal the chunk; the token comes back in `StoreEvent::Appended`, except token 0 (no acknowledgement)
    Append { record: Box<JournalRecord>, token: u64 },
    LoadTile(RegionPos),
    LoadSurfaces(RegionPos),
    // the biome palettes of a region for hover; answered by `StoreEvent::BiomesLoaded`
    LoadBiomes(RegionPos),
    // loads the world's biome key table, or starts a new one; answered by `StoreEvent::BiomeKeysLoaded`
    LoadBiomeKeys,
    WriteBiomeKeys { table_id: u64, names: Vec<String> },
    // appends one line (a newline is added); truncate starts the file again first
    AppendLine { path: PathBuf, line: String, truncate: bool },
    Compact,
    // answered by `StoreEvent::WaypointsLoaded` with the same path
    LoadWaypoints { path: PathBuf },
    WriteWaypoints { path: PathBuf, waypoints: Vec<Waypoint> },
    // answered by `StoreEvent::RegionList`; empty without a disk cache
    ListRegions { dim: i32 },
}

impl StoreCommand {
    pub const NO_ACK: u64 = 0;
}
