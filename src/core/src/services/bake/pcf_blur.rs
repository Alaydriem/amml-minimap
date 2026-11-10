use crate::services::bake::ColourMath;
use crate::structs::Rgba;
use crate::structs::chunk::ShadowMask;
use crate::structs::pos::ChunkPos;
use crate::traits::BakeEnvironment;

// separable box blur of shadow values, then colour *= 1 - average shadow over the column
pub struct PcfBlur {
    north: Vec<f32>,
    centre: Vec<f32>,
    south: Vec<f32>,
}

impl PcfBlur {
    pub fn new() -> Self {
        Self { north: Vec::new(), centre: Vec::new(), south: Vec::new() }
    }

    pub fn apply(
        &mut self,
        pos: ChunkPos,
        mask: &ShadowMask,
        env: &impl BakeEnvironment,
        radius: usize,
        colours: &mut [Rgba; 256],
    ) {
        let scale = usize::from(mask.scale());
        if scale == 0 {
            return;
        }
        if radius == 0 {
            Self::apply_unblurred(mask, colours);
            return;
        }
        let side = 16 * scale;
        let neighbour = |dx: i32, dz: i32| env.mask(pos.offset(dx, dz)).filter(|m| m.is_baked_at(mask.scale()));
        let mut north = std::mem::take(&mut self.north);
        let mut centre = std::mem::take(&mut self.centre);
        let mut south = std::mem::take(&mut self.south);
        Self::horizontal(
            [neighbour(-1, -1), neighbour(0, -1), neighbour(1, -1)],
            side,
            radius,
            side - radius,
            radius,
            &mut north,
        );
        Self::horizontal([neighbour(-1, 0), Some(mask), neighbour(1, 0)], side, radius, 0, side, &mut centre);
        Self::horizontal([neighbour(-1, 1), neighbour(0, 1), neighbour(1, 1)], side, radius, 0, radius, &mut south);

        let row = |sz: isize| -> &[f32] {
            let (band, r) = if sz < 0 {
                (&north, (radius as isize + sz) as usize)
            } else if (sz as usize) < side {
                (&centre, sz as usize)
            } else {
                (&south, sz as usize - side)
            };
            &band[r * side..(r + 1) * side]
        };
        let divisor = ((2 * radius + 1) * scale * scale) as f32;
        for bz in 0..16 {
            for bx in 0..16 {
                let mut sum = 0.0f32;
                for sz in 0..scale {
                    for sx in 0..scale {
                        let centre_z = (bz * scale + sz) as isize;
                        for z in (centre_z - radius as isize)..=(centre_z + radius as isize) {
                            sum += row(z)[bx * scale + sx];
                        }
                    }
                }
                let i = bz * 16 + bx;
                colours[i] = ColourMath::multiply(colours[i], 1.0 - sum / divisor);
            }
        }
        self.north = north;
        self.centre = centre;
        self.south = south;
    }

    fn apply_unblurred(mask: &ShadowMask, colours: &mut [Rgba; 256]) {
        let scale = usize::from(mask.scale());
        for bz in 0..16 {
            for bx in 0..16 {
                let mut sum = 0.0f32;
                for sz in 0..scale {
                    for sx in 0..scale {
                        sum += mask.value(bx * scale + sx, bz * scale + sz);
                    }
                }
                let i = bz * 16 + bx;
                colours[i] = ColourMath::multiply(colours[i], 1.0 - sum / (scale * scale) as f32);
            }
        }
    }

    // averages each row over `radius` samples either side; samples in missing chunks are not counted
    fn horizontal(
        masks: [Option<&ShadowMask>; 3],
        side: usize,
        radius: usize,
        first_row: usize,
        rows: usize,
        out: &mut Vec<f32>,
    ) {
        out.clear();
        out.resize(rows * side, 0.0);
        if masks.iter().all(Option::is_none) {
            return;
        }
        for r in 0..rows {
            let sz = first_row + r;
            for sx in 0..side {
                let mut sum = 0.0f32;
                let mut count = 0u32;
                for index in (sx as isize - radius as isize)..=(sx as isize + radius as isize) {
                    let (mask, x) = if index < 0 {
                        (masks[0], (index + side as isize) as usize)
                    } else if (index as usize) < side {
                        (masks[1], index as usize)
                    } else {
                        (masks[2], index as usize - side)
                    };
                    if let Some(m) = mask {
                        sum += m.value(x, sz);
                        count += 1;
                    }
                }
                if count > 0 {
                    out[r * side + sx] = sum / count as f32;
                }
            }
        }
    }
}

impl Default for PcfBlur {
    fn default() -> Self {
        Self::new()
    }
}
