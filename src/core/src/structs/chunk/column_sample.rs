#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnSample {
    pub surface: u32,
    pub water: u32,
    pub height: i16,
    pub solid: i16,
    pub retry: bool,
    // the raw game biome ID at the column's surface
    pub biome_id: u16,
}

impl ColumnSample {
    pub fn new(surface: u32, water: u32, height: i16, solid: i16, retry: bool, biome_id: u16) -> Self {
        Self { surface, water, height, solid, retry, biome_id }
    }
}
