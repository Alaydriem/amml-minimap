use std::collections::HashMap;

use crate::services::biome::{BiomeIdNames, BiomeKeyTable};
use crate::structs::biome::{BiomeKey, BiomePalette};
use crate::structs::chunk::ChunkSurface;

// turns the probe's raw game biome IDs into world-local keys in a chunk's palette
pub struct BiomeResolver {
    names: BiomeIdNames,
    // game ID to key for the table `memo_table`; bounded by BiomeIdNames::CAPACITY
    memo: HashMap<u16, BiomeKey>,
    memo_table: u64,
}

impl BiomeResolver {
    pub fn new() -> Self {
        Self { names: BiomeIdNames::new(BiomeIdNames::CAPACITY), memo: HashMap::new(), memo_table: 0 }
    }

    pub fn key(&mut self, id: u16, table: &mut BiomeKeyTable) -> Option<BiomeKey> {
        if self.memo_table != table.table_id() {
            self.memo.clear();
            self.memo_table = table.table_id();
        }
        if let Some(key) = self.memo.get(&id) {
            return Some(*key);
        }
        let Some(name) = self.names.name(id) else {
            self.names.request(id);
            return None;
        };
        let key = table.key_for(name)?;
        if self.memo.len() < BiomeIdNames::CAPACITY {
            self.memo.insert(id, key);
        }
        Some(key)
    }

    // sets the palette of every probed column; a run of equal IDs is looked up and placed once, and
    // the rest of the run copies its slot. A column whose biome waits for the game's name or for the
    // key table is marked for a retry, so the scan comes back for it soon; true when any column waits
    pub fn fill(&mut self, ids: &[u16; 256], surface: &mut ChunkSurface, table: &mut BiomeKeyTable) -> bool {
        // the run's ID, its first column and whether its biome waits
        let mut run: Option<(u16, usize, bool)> = None;
        let mut overflow = false;
        let mut waiting = false;
        for (column, &id) in ids.iter().enumerate() {
            if surface.is_skipped(column) {
                run = None;
                continue;
            }
            let waits = match run {
                Some((run_id, first, waits)) if run_id == id => {
                    surface.biome.copy_column(first, column);
                    waits
                }
                _ => {
                    let key = self.key(id, table);
                    overflow |= !surface.biome.set(column, key);
                    let waits = key.is_none() && (!table.is_loaded() || !self.names.is_answered(id));
                    run = Some((id, column, waits));
                    waits
                }
            };
            if waits {
                surface.set_retry(column, true);
                waiting = true;
            }
        }
        if overflow {
            curia::debug!("a chunk has more biomes than a palette holds; the extra columns take its first biome", { slots: BiomePalette::SLOTS });
        }
        waiting
    }

    pub fn answer(&mut self, id: u16, name: &str) {
        self.names.answer(id, name);
    }

    pub fn take_requests(&mut self, out: &mut Vec<u16>) {
        self.names.take_requests(out);
    }
}

impl Default for BiomeResolver {
    fn default() -> Self {
        Self::new()
    }
}
