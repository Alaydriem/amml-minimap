use crate::services::bake::region_environment::RegionEnvironment;
use crate::services::bake::{ChunkBaker, ShadowCaster};
use crate::structs::bake::{BakeSettings, RegionBakeOutput, RenderStyle};
use crate::structs::pos::RegionPos;
use crate::traits::BakeEnvironment;

// bakes every loaded chunk of a region: all shadow masks first, then shading with the new masks
pub struct RegionBaker {
    chunk: ChunkBaker,
}

impl RegionBaker {
    pub fn new() -> Self {
        Self { chunk: ChunkBaker::new() }
    }

    pub fn bake(
        &mut self,
        region: RegionPos,
        env: &impl BakeEnvironment,
        settings: &BakeSettings,
        out: &mut RegionBakeOutput,
    ) {
        for lz in 0..16 {
            for lx in 0..16 {
                let i = lz * 16 + lx;
                let pos = region.chunk(lx, lz);
                let Some(surface) = env.surface(pos) else {
                    out.present[i] = false;
                    continue;
                };
                out.present[i] = true;
                if settings.style == RenderStyle::ShadowMap {
                    ShadowCaster::cast(pos, surface, env, settings.light(), settings.scale, &mut out.masks[i]);
                } else {
                    out.masks[i].reset(0);
                }
            }
        }
        let overlay = RegionEnvironment { inner: env, region, present: &out.present, masks: &out.masks };
        for lz in 0..16 {
            for lx in 0..16 {
                let i = lz * 16 + lx;
                if !out.present[i] {
                    continue;
                }
                let pos = region.chunk(lx, lz);
                if let Some(surface) = env.surface(pos) {
                    self.chunk.render(pos, surface, &out.masks[i], &overlay, settings, &mut out.pixels[i]);
                }
            }
        }
    }
}

impl Default for RegionBaker {
    fn default() -> Self {
        Self::new()
    }
}
