// a world-local biome key: an index into the world's biome name table, stable across game versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BiomeKey(pub u16);
