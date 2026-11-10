use std::collections::{BTreeMap, HashMap};

use crate::structs::Rgba;

// colours from terrain.biomeColorOverrides, shown for columns with no blocks in place of the game's colour
#[derive(Debug, Clone, Default)]
pub struct BiomeColors {
    entries: HashMap<String, Rgba>,
}

impl BiomeColors {
    const NAMESPACE: &'static str = "minecraft:";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_overrides(overrides: &BTreeMap<String, [u8; 4]>) -> Self {
        let entries = overrides
            .iter()
            .map(|(name, [r, g, b, a])| (Self::bare(name).to_string(), Rgba::new(*r, *g, *b, *a)))
            .collect();
        Self { entries }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // the game's biome names carry a namespace; an override may be written with or without it
    pub fn lookup(&self, biome: &str) -> Option<Rgba> {
        self.entries.get(Self::bare(biome)).copied()
    }

    fn bare(name: &str) -> &str {
        name.strip_prefix(Self::NAMESPACE).unwrap_or(name)
    }
}
