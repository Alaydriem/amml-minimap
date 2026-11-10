use crate::services::draw::{ChunkGridPainter, OverlayGate, WorldMapLayout};
use crate::services::slime::SlimeMaskCache;
use crate::structs::Rgba;
use crate::structs::draw::{DrawCommand, DrawList, ExploredChunks, WorldMapFrame};
use crate::structs::pos::ChunkPos;

// a green tint over the Overworld's explored slime chunks; neighbours in a row are drawn as one run
pub struct SlimeOverlayPainter {
    cache: SlimeMaskCache,
}

impl SlimeOverlayPainter {
    pub const MASKS_PER_FRAME: usize = 32;
    pub const COLOUR: Rgba = Rgba { r: 70, g: 210, b: 70, a: 80 };
    const OVERWORLD: i32 = 0;

    pub fn new(cache_regions: usize) -> Self {
        Self { cache: SlimeMaskCache::new(cache_regions) }
    }

    pub fn cached_regions(&self) -> usize {
        self.cache.len()
    }

    // a region whose mask is not cached once the frame's budget is spent is drawn on a later frame
    pub fn paint(&mut self, frame: &WorldMapFrame<'_>, explored: &ExploredChunks, draw: &mut DrawList) {
        if frame.dim != Self::OVERWORLD || !OverlayGate::open(frame.zoom) {
            return;
        }
        let (cx0, cx1, cz0, cz1) = WorldMapLayout::visible_chunks(frame.centre, frame.zoom, frame.display, frame.angle);
        let mut budget = Self::MASKS_PER_FRAME;
        for cz in cz0..=cz1 {
            let mut run: Option<i32> = None;
            let mut region: Option<(i32, Option<[u64; 4]>)> = None;
            for cx in cx0..=cx1 + 1 {
                let slime = cx <= cx1 && explored.contains(cx, cz) && self.is_slime(cx, cz, &mut region, &mut budget);
                match (slime, run) {
                    (true, None) => run = Some(cx),
                    (false, Some(start)) => {
                        Self::run(frame, (start, cx - 1), cz, draw);
                        run = None;
                    }
                    _ => {}
                }
            }
        }
    }

    // `region` keeps the mask of the region the row is in, so a row looks each region up once
    fn is_slime(&mut self, cx: i32, cz: i32, region: &mut Option<(i32, Option<[u64; 4]>)>, budget: &mut usize) -> bool {
        let rx = cx.div_euclid(16);
        let mask = match *region {
            Some((x, mask)) if x == rx => mask,
            _ => {
                let mask = self.cache.mask(rx, cz.div_euclid(16), budget);
                *region = Some((rx, mask));
                mask
            }
        };
        let Some(mask) = mask else { return false };
        let bit = (cz.rem_euclid(16) * 16 + cx.rem_euclid(16)) as usize;
        (mask[bit / 64] >> (bit % 64)) & 1 == 1
    }

    fn run(frame: &WorldMapFrame<'_>, columns: (i32, i32), cz: i32, draw: &mut DrawList) {
        let side = ChunkPos::BLOCKS;
        let (x0, x1, z0, z1) = (columns.0 * side, (columns.1 + 1) * side, cz * side, (cz + 1) * side);
        let colour = Self::COLOUR.packed();
        let corner = |x, z| ChunkGridPainter::screen(frame, (x, z));
        if frame.angle == 0.0 {
            draw.push(DrawCommand::RectFilled { p0: corner(x0, z0), p1: corner(x1, z1), colour });
            return;
        }
        let (a, b, c, d) = (corner(x0, z0), corner(x1, z0), corner(x1, z1), corner(x0, z1));
        draw.push(DrawCommand::TriangleFilled { a, b, c, colour });
        draw.push(DrawCommand::TriangleFilled { a, b: c, c: d, colour });
    }
}
