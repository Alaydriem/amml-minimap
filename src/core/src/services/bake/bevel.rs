use crate::services::bake::{BevelTable, ColourMath};
use crate::structs::Rgba;
use crate::structs::chunk::ChunkSurface;
use crate::structs::pos::ChunkPos;
use crate::traits::BakeEnvironment;

// edge lighting from solid-height steps between a column and its four neighbours
pub struct Bevel;

impl Bevel {
    pub fn apply(
        pos: ChunkPos,
        center: &ChunkSurface,
        env: &impl BakeEnvironment,
        table: &BevelTable,
        colours: &mut [Rgba; 256],
    ) {
        let west = env.surface(pos.offset(-1, 0));
        let east = env.surface(pos.offset(1, 0));
        let north = env.surface(pos.offset(0, -1));
        let south = env.surface(pos.offset(0, 1));
        let solid = |x: usize, z: usize| center.solid[ChunkSurface::index(x, z)];
        for z in 0..16 {
            let mut h = solid(0, z);
            let mut hw = west.map_or(h, |w| w.solid[ChunkSurface::index(15, z)]);
            for x in 0..16 {
                let he = match (x, east) {
                    (15, Some(e)) => e.solid[ChunkSurface::index(0, z)],
                    (15, None) => h,
                    _ => solid(x + 1, z),
                };
                let hn = match (z, north) {
                    (0, Some(n)) => n.solid[ChunkSurface::index(x, 15)],
                    (0, None) => h,
                    _ => solid(x, z - 1),
                };
                let hs = match (z, south) {
                    (15, Some(s)) => s.solid[ChunkSurface::index(x, 0)],
                    (15, None) => h,
                    _ => solid(x, z + 1),
                };
                let factor = table.factor(
                    Self::relation(hw, h),
                    Self::relation(he, h),
                    Self::relation(hn, h),
                    Self::relation(hs, h),
                );
                if factor != 1.0 {
                    let i = ChunkSurface::index(x, z);
                    colours[i] = ColourMath::multiply(colours[i], factor);
                }
                hw = h;
                h = he;
            }
        }
    }

    fn relation(neighbour: i16, height: i16) -> u8 {
        match neighbour.cmp(&height) {
            std::cmp::Ordering::Greater => 2,
            std::cmp::Ordering::Less => 1,
            std::cmp::Ordering::Equal => 0,
        }
    }
}
