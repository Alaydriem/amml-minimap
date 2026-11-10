use crate::services::bake::HeightSampler;
use crate::structs::LightDirection;
use crate::structs::bake::BakeSettings;
use crate::structs::chunk::{ChunkSurface, ShadowMask};
use crate::structs::pos::ChunkPos;
use crate::traits::BakeEnvironment;

// marches one ray per sub-pixel toward the light and marks the sub-pixels that hit terrain
pub struct ShadowCaster;

impl ShadowCaster {
    pub const MAX_STEPS: i32 = 96;

    pub fn cast(
        pos: ChunkPos,
        center: &ChunkSurface,
        env: &impl BakeEnvironment,
        light: LightDirection,
        scale: u8,
        out: &mut ShadowMask,
    ) {
        out.reset(scale);
        let (sun_x, sun_y, sun_z) = light.sun();
        let len_2d = (sun_x * sun_x + sun_z * sun_z).sqrt();
        if len_2d < 1e-4 {
            return;
        }
        let (sdx, sdz) = (sun_x / len_2d, sun_z / len_2d);
        let rise = sun_y / len_2d;
        let top = BakeSettings::dimension_top(pos.dim);
        let mut sampler = HeightSampler::new(env, pos, center);
        let (base_x, base_z) = (pos.min_block_x(), pos.min_block_z());
        let scale_f = f32::from(scale);
        let side = usize::from(scale);
        for bz in 0..16 {
            for bx in 0..16 {
                let h = i32::from(center.height[ChunkSurface::index(bx, bz)]);
                for sz in 0..side {
                    for sx in 0..side {
                        let off_x = bx as f32 + (sx as f32 + 0.5) / scale_f;
                        let off_z = bz as f32 + (sz as f32 + 0.5) / scale_f;
                        let mut last = (i32::MAX, i32::MAX);
                        for s in 1..=Self::MAX_STEPS {
                            let step = s as f32;
                            let cell = ((off_x + step * sdx).floor() as i32, (off_z + step * sdz).floor() as i32);
                            if cell == last {
                                continue;
                            }
                            last = cell;
                            let ray = h + (step * rise + 0.01) as i32;
                            if sampler.height_at(base_x + cell.0, base_z + cell.1) > ray {
                                out.set(bx * side + sx, bz * side + sz);
                                break;
                            }
                            if ray > top {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
