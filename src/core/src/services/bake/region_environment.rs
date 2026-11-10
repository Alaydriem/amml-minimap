use crate::structs::chunk::{ChunkSurface, ShadowMask};
use crate::structs::pos::{ChunkPos, RegionPos};
use crate::traits::BakeEnvironment;

// the outer environment, with masks inside the region replaced by the ones this bake cast
pub(crate) struct RegionEnvironment<'a, E: BakeEnvironment> {
    pub(crate) inner: &'a E,
    pub(crate) region: RegionPos,
    pub(crate) present: &'a [bool; 256],
    pub(crate) masks: &'a [ShadowMask],
}

impl<E: BakeEnvironment> BakeEnvironment for RegionEnvironment<'_, E> {
    fn surface(&self, pos: ChunkPos) -> Option<&ChunkSurface> {
        self.inner.surface(pos)
    }

    fn mask(&self, pos: ChunkPos) -> Option<&ShadowMask> {
        if !self.region.contains(&pos) {
            return self.inner.mask(pos);
        }
        let (lx, lz) = pos.local_in_region();
        let i = lz * 16 + lx;
        self.present[i].then(|| &self.masks[i]).filter(|m| m.scale() != 0)
    }
}
