use crate::services::bake::ColourMath;
use crate::structs::Rgba;
use crate::structs::chunk::ChunkSurface;
use crate::structs::pos::ChunkPos;
use crate::traits::BakeEnvironment;

// style 1: brighter where a column is above its west and north neighbours, darker where below
pub struct GradientStyle;

impl GradientStyle {
    pub fn apply(
        pos: ChunkPos,
        center: &ChunkSurface,
        env: &impl BakeEnvironment,
        shadow_level: i32,
        colours: &mut [Rgba; 256],
    ) {
        let level = if shadow_level <= 0 { 100 } else { shadow_level };
        let west = env.surface(pos.offset(-1, 0));
        let north = env.surface(pos.offset(0, -1));
        for z in 0..16 {
            for x in 0..16 {
                let i = ChunkSurface::index(x, z);
                let cur = i32::from(center.height[i]);
                let west_h = match (x, west) {
                    (0, Some(w)) => i32::from(w.height[ChunkSurface::index(15, z)]),
                    (0, None) => cur,
                    _ => i32::from(center.height[ChunkSurface::index(x - 1, z)]),
                };
                let north_h = match (z, north) {
                    (0, Some(n)) => i32::from(n.height[ChunkSurface::index(x, 15)]),
                    (0, None) => cur,
                    _ => i32::from(center.height[ChunkSurface::index(x, z - 1)]),
                };
                let sum = west_h + north_h;
                if cur * 2 > sum {
                    colours[i] = ColourMath::scale_ratio(colours[i], level, 100);
                } else if cur * 2 < sum {
                    colours[i] = ColourMath::scale_ratio(colours[i], 100, level);
                }
            }
        }
    }
}
