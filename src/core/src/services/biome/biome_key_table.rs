use std::collections::HashMap;

use crate::structs::biome::BiomeKey;

// the world's biome names in key order; a palette stores the key, the table gives the name back.
// No key is given before the saved table is loaded, so a new key never collides with a stored one
pub struct BiomeKeyTable {
    capacity: usize,
    names: Vec<Box<str>>,
    keys: HashMap<Box<str>, BiomeKey>,
    // non-zero once loaded; region files and journal records store the ID their palettes used
    table_id: u64,
    loaded: bool,
    dirty: bool,
}

impl BiomeKeyTable {
    pub const CAPACITY: usize = 1024;

    pub fn new(capacity: usize) -> Self {
        Self { capacity, names: Vec::new(), keys: HashMap::new(), table_id: 0, loaded: false, dirty: false }
    }

    pub fn mark_loaded(&mut self, table_id: u64, names: Vec<String>) {
        self.names.clear();
        self.keys.clear();
        for name in names.into_iter().take(self.capacity) {
            self.push(name.into_boxed_str());
        }
        self.table_id = table_id;
        self.loaded = true;
        self.dirty = false;
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn table_id(&self) -> u64 {
        self.table_id
    }

    // a new name gets the next key; None before load or when the table is full
    pub fn key_for(&mut self, internal: &str) -> Option<BiomeKey> {
        if !self.loaded {
            return None;
        }
        if let Some(key) = self.keys.get(internal) {
            return Some(*key);
        }
        if self.names.len() >= self.capacity {
            return None;
        }
        self.dirty = true;
        Some(self.push(internal.into()))
    }

    pub fn name(&self, key: BiomeKey) -> Option<&str> {
        self.names.get(usize::from(key.0)).map(|name| &**name)
    }

    // the names to save, once per change
    pub fn take_dirty(&mut self) -> Option<Vec<String>> {
        if !self.dirty {
            return None;
        }
        self.dirty = false;
        Some(self.names.iter().map(|name| name.to_string()).collect())
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn clear(&mut self) {
        self.names.clear();
        self.keys.clear();
        self.table_id = 0;
        self.loaded = false;
        self.dirty = false;
    }

    fn push(&mut self, name: Box<str>) -> BiomeKey {
        let key = BiomeKey(self.names.len() as u16);
        self.keys.insert(name.clone(), key);
        self.names.push(name);
        key
    }
}
