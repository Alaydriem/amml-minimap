// which visible chunks have map data, one 256-bit mask per region in a row-major block of
// regions; the world map's overlays draw only over these chunks. Keeps its capacity
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExploredChunks {
    // the block's first region and its width in regions
    origin: (i32, i32),
    width: i32,
    masks: Vec<[u64; 4]>,
    // every chunk counts as explored (benches and footprints that measure the overlays alone)
    everywhere: bool,
}

impl ExploredChunks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn everywhere() -> Self {
        Self { everywhere: true, ..Self::default() }
    }

    // starts a new block of regions; masks follow in row-major order
    pub fn begin(&mut self, first_region: (i32, i32), width: i32) {
        self.origin = first_region;
        self.width = width.max(0);
        self.masks.clear();
        self.everywhere = false;
    }

    pub fn push(&mut self, mask: [u64; 4]) {
        self.masks.push(mask);
    }

    pub fn contains(&self, cx: i32, cz: i32) -> bool {
        if self.everywhere {
            return true;
        }
        let (rx, rz) = (cx.div_euclid(16) - self.origin.0, cz.div_euclid(16) - self.origin.1);
        if rx < 0 || rz < 0 || rx >= self.width {
            return false;
        }
        let Some(mask) = self.masks.get((rz * self.width + rx) as usize) else { return false };
        let bit = (cz.rem_euclid(16) * 16 + cx.rem_euclid(16)) as usize;
        (mask[bit / 64] >> (bit % 64)) & 1 == 1
    }
}
