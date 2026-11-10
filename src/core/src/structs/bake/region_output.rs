use crate::structs::chunk::ShadowMask;

// one slot per chunk of the region, row-major local_z * 16 + local_x
#[derive(Debug, Clone)]
pub struct RegionBakeOutput {
    pub present: [bool; 256],
    pub pixels: Vec<[u32; 256]>,
    pub masks: Vec<ShadowMask>,
}

impl RegionBakeOutput {
    pub fn new() -> Self {
        Self { present: [false; 256], pixels: vec![[0; 256]; 256], masks: vec![ShadowMask::new(0); 256] }
    }
}

impl Default for RegionBakeOutput {
    fn default() -> Self {
        Self::new()
    }
}
