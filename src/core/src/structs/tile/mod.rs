use crate::errors::TileError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pixels: Box<[u8]>,
    version: u64,
    settings_hash: u64,
    dirty: bool,
}

impl Tile {
    pub const SIDE: usize = 256;
    pub const BYTES: usize = Self::SIDE * Self::SIDE * 4;

    pub fn new(settings_hash: u64) -> Self {
        Self { pixels: vec![0u8; Self::BYTES].into_boxed_slice(), version: 0, settings_hash, dirty: false }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn settings_hash(&self) -> u64 {
        self.settings_hash
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn pixel(&self, x: usize, z: usize) -> u32 {
        let i = (z * Self::SIDE + x) * 4;
        u32::from_le_bytes([self.pixels[i], self.pixels[i + 1], self.pixels[i + 2], self.pixels[i + 3]])
    }

    // writes one chunk's 16 x 16 baked pixels at its place in the region
    pub fn write_chunk(&mut self, local_x: usize, local_z: usize, pixels: &[u32; 256], settings_hash: u64) {
        for z in 0..16 {
            let row = (local_z * 16 + z) * Self::SIDE + local_x * 16;
            for x in 0..16 {
                let i = (row + x) * 4;
                self.pixels[i..i + 4].copy_from_slice(&pixels[z * 16 + x].to_le_bytes());
            }
        }
        self.settings_hash = settings_hash;
        self.version += 1;
        self.dirty = true;
    }

    // replaces every pixel with data read from disk; the tile is clean afterwards
    pub fn replace_pixels(&mut self, bytes: &[u8], settings_hash: u64) -> Result<(), TileError> {
        if bytes.len() != Self::BYTES {
            return Err(TileError::WrongLength(bytes.len()));
        }
        self.pixels.copy_from_slice(bytes);
        self.settings_hash = settings_hash;
        self.version += 1;
        self.dirty = false;
        Ok(())
    }

    // fills only pixels that are still transparent, so older disk data never covers a newer bake
    pub fn merge_under(&mut self, bytes: &[u8]) -> Result<bool, TileError> {
        if bytes.len() != Self::BYTES {
            return Err(TileError::WrongLength(bytes.len()));
        }
        let mut changed = false;
        for (dst, src) in self.pixels.as_chunks_mut::<4>().0.iter_mut().zip(bytes.as_chunks::<4>().0) {
            if dst[3] == 0 && src[3] != 0 {
                *dst = *src;
                changed = true;
            }
        }
        if changed {
            self.version += 1;
            self.dirty = true;
        }
        Ok(changed)
    }

    // a write that lands between snapshot and flush keeps the tile dirty
    pub fn mark_clean_if(&mut self, version: u64) -> bool {
        if self.version != version {
            return false;
        }
        self.dirty = false;
        true
    }
}
