use std::collections::HashMap;

use amml_api::{Biomes, ColumnSample};

// the biome names of the current world, by the game's biome id. An id can name another biome in another world, so
// `clear` runs at every world entry and exit. At most CAPACITY ids are kept; a new id past that empties the cache
// first. A name the loader does not have yet is not kept, so it is asked again
pub struct BiomeNames {
    biomes: Biomes,
    names: HashMap<u16, Box<str>>,
}

impl BiomeNames {
    pub const CAPACITY: usize = 512;

    pub fn new(biomes: Biomes) -> Self {
        Self { biomes, names: HashMap::with_capacity(Self::CAPACITY) }
    }

    pub fn get(&mut self, id: u16) -> Option<&str> {
        if id == ColumnSample::NO_BIOME {
            return None;
        }
        if !self.names.contains_key(&id) {
            let name = self.biomes.name(id).filter(|name| !name.is_empty())?;
            if self.names.len() >= Self::CAPACITY {
                self.names.clear();
            }
            self.names.insert(id, name.into());
        }
        self.names.get(&id).map(|name| &**name)
    }

    pub fn clear(&mut self) {
        self.names.clear();
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}
