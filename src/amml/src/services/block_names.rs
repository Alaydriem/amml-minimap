use std::collections::HashMap;

use amml_api::{BlockTypeId, Blocks};

use crate::types::BlockName;

// the names behind the block type ids `read_columns` gives. An id names one type for the rest of the process, so a
// kept name never goes stale. At most CAPACITY ids are kept; a new id past that empties the cache first
pub struct BlockNames {
    blocks: Blocks,
    names: HashMap<BlockTypeId, Option<BlockName>>,
}

impl BlockNames {
    pub const CAPACITY: usize = 4096;

    pub fn new(blocks: Blocks) -> Self {
        Self { blocks, names: HashMap::with_capacity(Self::CAPACITY) }
    }

    // the name of `id`; `None` for `NONE` and for an id the loader has no name for (kept, so it is asked once)
    pub fn get(&mut self, id: BlockTypeId) -> Option<&BlockName> {
        if id.is_none() {
            return None;
        }
        if !self.names.contains_key(&id) && self.names.len() >= Self::CAPACITY {
            self.names.clear();
        }
        let blocks = self.blocks;
        self.names.entry(id).or_insert_with(|| blocks.type_name(id).map(|name| BlockName::new(&name))).as_ref()
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}
