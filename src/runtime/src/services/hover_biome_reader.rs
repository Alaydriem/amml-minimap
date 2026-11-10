use minimap_core::services::biome::BiomeDisplayNames;
use minimap_core::services::ui::HoverDebounce;
use minimap_core::structs::biome::BiomeKey;
use minimap_core::structs::pos::ChunkPos;

use crate::structs::Shared;

// the biome name under the world map cursor, on the render thread: the resident chunk first, then
// the biomes read from disk for hover; any busy lock gives no name this frame
pub struct HoverBiomeReader {
    debounce: HoverDebounce,
    // the last name read; keeps its capacity between frames
    label: String,
}

impl HoverBiomeReader {
    pub fn new() -> Self {
        Self { debounce: HoverDebounce::new(), label: String::new() }
    }

    pub fn read(&mut self, shared: &Shared, block: (i32, i32), layer: i32) -> Option<&str> {
        let key = self.key(shared, block, layer)?;
        let keys = shared.biome_keys.try_lock().ok()?;
        let name = keys.name(key)?;
        BiomeDisplayNames::write_label(name, &mut self.label);
        Some(&self.label)
    }

    pub fn reset(&mut self) {
        self.debounce.reset();
    }

    // a region with no palettes in memory is asked for once the cursor rests on it
    fn key(&mut self, shared: &Shared, block: (i32, i32), layer: i32) -> Option<BiomeKey> {
        let resident = shared.chunks.try_read().ok()?.column_biome(block.0, block.1, layer);
        if resident.is_some() {
            return resident;
        }
        let mut hover = shared.hover_biomes.try_lock().ok()?;
        match hover.lookup(block.0, block.1, layer) {
            Some(key) => key,
            None => {
                let region = ChunkPos::from_block(block.0, block.1, layer).region();
                if self.debounce.observe(region) {
                    let _ = shared.biome_requests.0.try_send(region);
                }
                None
            }
        }
    }
}

impl Default for HoverBiomeReader {
    fn default() -> Self {
        Self::new()
    }
}
