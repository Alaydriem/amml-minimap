use crate::structs::pos::RegionPos;

// the image an explored-area export writes: the regions' bounding box at a power-of-two scale
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AreaLayout {
    // inclusive region bounds
    pub x0: i32,
    pub x1: i32,
    pub z0: i32,
    pub z1: i32,
    pub blocks_per_pixel: u32,
    pub width: u32,
    pub height: u32,
}

impl AreaLayout {
    pub const MAX_BLOCKS_PER_PIXEL: u32 = RegionPos::BLOCKS as u32;

    // None when there is nothing to export, or even one pixel per region is wider than max_pixels
    pub fn new(regions: &[RegionPos], max_pixels: u32) -> Option<Self> {
        let first = regions.first()?;
        let (mut x0, mut x1, mut z0, mut z1) = (first.x, first.x, first.z, first.z);
        for r in regions {
            (x0, x1, z0, z1) = (x0.min(r.x), x1.max(r.x), z0.min(r.z), z1.max(r.z));
        }
        let side = RegionPos::BLOCKS as u64;
        let span = ((x1 - x0 + 1) as u64 * side).max((z1 - z0 + 1) as u64 * side);
        let mut scale = 1u32;
        while span / u64::from(scale) > u64::from(max_pixels) {
            scale *= 2;
            if scale > Self::MAX_BLOCKS_PER_PIXEL {
                return None;
            }
        }
        let per_region = RegionPos::BLOCKS as u32 / scale;
        Some(Self {
            x0,
            x1,
            z0,
            z1,
            blocks_per_pixel: scale,
            width: (x1 - x0 + 1) as u32 * per_region,
            height: (z1 - z0 + 1) as u32 * per_region,
        })
    }

    // pixels one region takes on each side
    pub fn region_pixels(&self) -> usize {
        RegionPos::BLOCKS as usize / self.blocks_per_pixel as usize
    }

    // bytes of one row of regions: the export's whole working buffer
    pub fn row_bytes(&self) -> usize {
        self.width as usize * self.region_pixels() * 4
    }
}
