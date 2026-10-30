use minimap_core::structs::Rgba;

// one column's walk result: the colour shown, and the water above it when transparent water applies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WalkedColumn {
    pub colour: Rgba,
    pub water_colour: Rgba,
    pub water_depth: u8,
    // a block on the walk was still the client-request placeholder
    pub retry: bool,
}

impl WalkedColumn {
    // the water word of the chunk surface: the water colour with the depth in the alpha byte; 0 without water
    pub fn water_word(&self) -> u32 {
        if self.water_depth == 0 {
            return 0;
        }
        let water = Rgba::new(self.water_colour.r, self.water_colour.g, self.water_colour.b, 0);
        water.packed() | (u32::from(self.water_depth) << 24)
    }
}
