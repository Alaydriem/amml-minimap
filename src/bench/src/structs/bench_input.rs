use minimap_core::config::TerrainConfig;
use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::pos::ChunkPos;

// a named set of stored chunks and the terrain settings they were recorded with
#[derive(Debug, Clone)]
pub struct BenchInput {
    pub name: String,
    pub terrain: TerrainConfig,
    pub chunks: Vec<(ChunkPos, ChunkSurface)>,
}
