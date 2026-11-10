use minimap_core::structs::biome::RegionBiomes;
use minimap_core::structs::chunk::ChunkSurface;
use std::path::PathBuf;

use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_core::structs::waypoint::Waypoint;

#[derive(Debug)]
pub enum StoreEvent {
    Appended { pos: ChunkPos, token: u64 },
    TileLoaded { region: RegionPos, pixels: Box<[u8]>, settings_hash: u64 },
    // nothing is stored for the region
    TileMissing(RegionPos),
    // the bool is true when the stored tile already holds baked pixels for that chunk
    SurfacesLoaded { region: RegionPos, chunks: Vec<(ChunkPos, Box<ChunkSurface>, bool)> },
    // empty when the file is missing or was set aside as unreadable
    WaypointsLoaded { path: PathBuf, waypoints: Vec<Waypoint> },
    // the regions of the dimension with a region file, sorted by z then x
    RegionList { dim: i32, regions: Vec<RegionPos> },
    // always answered; empty when nothing is stored for the region
    BiomesLoaded { region: RegionPos, biomes: Box<RegionBiomes> },
    // None without a disk cache
    BiomeKeysLoaded { table: Option<(u64, Vec<String>)> },
    // the key table could not be written; the runtime sends it again
    BiomeKeysNotSaved,
    Failed(String),
}
