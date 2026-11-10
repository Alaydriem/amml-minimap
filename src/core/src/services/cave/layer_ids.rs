// map layer ids: a surface keeps its dimension id; a cave band is 1000 + dim * 100 + band + 50
pub struct LayerIds;

impl LayerIds {
    pub const CAVE_BASE: i32 = 1000;
    pub const BANDS_PER_DIM: i32 = 100;
    pub const BAND_OFFSET: i32 = 50;
    // bands outside this range share its end bands
    pub const MIN_BAND: i32 = -Self::BAND_OFFSET;
    pub const MAX_BAND: i32 = Self::BANDS_PER_DIM - Self::BAND_OFFSET - 1;

    pub fn surface(dim: i32) -> i32 {
        dim
    }

    pub fn cave(dim: i32, band: i32) -> i32 {
        Self::CAVE_BASE + dim * Self::BANDS_PER_DIM + band.clamp(Self::MIN_BAND, Self::MAX_BAND) + Self::BAND_OFFSET
    }

    pub fn is_cave(id: i32) -> bool {
        id >= Self::CAVE_BASE
    }

    // the real dimension of a layer
    pub fn dim(id: i32) -> i32 {
        if Self::is_cave(id) { (id - Self::CAVE_BASE) / Self::BANDS_PER_DIM } else { id }
    }

    pub fn band(id: i32) -> Option<i32> {
        Self::is_cave(id).then(|| (id - Self::CAVE_BASE) % Self::BANDS_PER_DIM - Self::BAND_OFFSET)
    }
}
