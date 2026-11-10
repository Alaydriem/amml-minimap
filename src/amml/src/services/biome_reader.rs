use amml_game::{ChunkBlockPos, GameEntries, LevelChunk};

use crate::game::{Biome, ChunkBiomes, Level};
use crate::traits::MapSink;

// the biomes of probed columns, and the names the scan plan asks for
pub struct BiomeReader;

impl BiomeReader {
    pub fn new() -> Self {
        Self
    }

    // the biome at the column's surface block; the heightmap names the first block above it. Game thread
    pub fn surface<'e>(&self, entries: &'e GameEntries, chunk: &LevelChunk<'_>, index: usize) -> Option<Biome<'e>> {
        let top = chunk.max_y()?.checked_sub(chunk.min_y()?)?.max(0);
        let height = i32::from(chunk.height(index)?);
        let local = (height - 1).clamp(0, top);
        let pos = ChunkBlockPos { x: (index % 16) as u8, z: (index / 16) as u8, y: local as i16 };
        ChunkBiomes::new(entries).at(chunk, pos)
    }

    // the game biome id at the column's surface; 0 when it cannot be read
    pub fn surface_id(&self, entries: &GameEntries, chunk: &LevelChunk<'_>, index: usize) -> u16 {
        self.surface(entries, chunk, index).map_or(0, |biome| biome.id())
    }

    // names the requested ids the level's registry knows; an id it does not know stays asked, and a probed chunk
    // holding that biome names it
    pub fn answer_registry<S: MapSink>(&self, level: &Level<'_>, requests: &[u16], sink: &mut S) {
        if requests.is_empty() {
            return;
        }
        let Some(registry) = level.biome_registry() else { return };
        for &id in requests {
            let Some(biome) = registry.lookup(id) else { continue };
            if let Ok(name) = biome.name().to_str()
                && !name.is_empty()
            {
                sink.biome_name(id, name);
            }
        }
    }

    // names each requested id that `probe_ids` holds from the chunk's own biome objects: the client's registry
    // does not know every id, but the chunk's biome carries its name
    pub fn answer_chunk<S: MapSink>(
        &self,
        entries: &GameEntries,
        chunk: &LevelChunk<'_>,
        probe_ids: &[u16; 256],
        requests: &[u16],
        sink: &mut S,
    ) {
        for &id in requests {
            let Some(index) = probe_ids.iter().position(|probed| *probed == id) else { continue };
            let Some(biome) = self.surface(entries, chunk, index) else { continue };
            if let Ok(name) = biome.name().to_str()
                && !name.is_empty()
            {
                sink.biome_name(id, name);
            }
        }
    }
}

impl Default for BiomeReader {
    fn default() -> Self {
        Self::new()
    }
}
