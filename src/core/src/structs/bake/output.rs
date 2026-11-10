use crate::structs::chunk::ShadowMask;

#[derive(Debug, Clone)]
pub struct BakeOutput {
    pub pixels: [u32; 256],
    pub mask: ShadowMask,
}

impl BakeOutput {
    pub fn new() -> Self {
        Self { pixels: [0; 256], mask: ShadowMask::new(0) }
    }
}

impl Default for BakeOutput {
    fn default() -> Self {
        Self::new()
    }
}
