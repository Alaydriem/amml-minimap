#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShadowMask {
    scale: u8,
    bits: Vec<u64>,
}

impl ShadowMask {
    pub const DARKNESS: f32 = 0.52;

    // scale 0 is a mask that has not been baked
    pub fn new(scale: u8) -> Self {
        let mut mask = Self { scale: 0, bits: Vec::new() };
        mask.reset(scale);
        mask
    }

    pub fn scale(&self) -> u8 {
        self.scale
    }

    pub fn side(&self) -> usize {
        16 * usize::from(self.scale)
    }

    pub fn is_baked_at(&self, scale: u8) -> bool {
        self.scale != 0 && self.scale == scale
    }

    pub fn reset(&mut self, scale: u8) {
        self.scale = scale;
        self.bits.clear();
        self.bits.resize(Self::byte_len(scale) / 8, 0);
    }

    pub fn get(&self, sx: usize, sz: usize) -> bool {
        let i = sz * self.side() + sx;
        (self.bits[i / 64] >> (i % 64)) & 1 == 1
    }

    pub fn set(&mut self, sx: usize, sz: usize) {
        let i = sz * self.side() + sx;
        self.bits[i / 64] |= 1 << (i % 64);
    }

    pub fn value(&self, sx: usize, sz: usize) -> f32 {
        if self.get(sx, sz) { Self::DARKNESS } else { 0.0 }
    }

    pub fn copy_from(&mut self, other: &ShadowMask) {
        self.scale = other.scale;
        self.bits.clear();
        self.bits.extend_from_slice(&other.bits);
    }

    pub fn byte_len(scale: u8) -> usize {
        let side = 16 * usize::from(scale);
        (side * side).div_ceil(64) * 8
    }
}
