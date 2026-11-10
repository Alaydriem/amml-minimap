use crate::structs::pos::RegionPos;

// one row of regions of an explored-area image, as RGBA rows ready for the PNG stream
pub struct AreaRow;

impl AreaRow {
    // averages `tile` (256 x 256 RGBA) down by `scale` into the row buffer `width` pixels wide, at
    // region column `column`; see-through texels do not darken the colour, and a block with none
    // drawn stays see-through
    pub fn place(row: &mut [u8], width: usize, column: usize, tile: &[u8], scale: usize) {
        let side = RegionPos::BLOCKS as usize;
        let out_side = side / scale;
        for oz in 0..out_side {
            for ox in 0..out_side {
                let mut sum = [0u32; 3];
                let (mut alpha, mut drawn) = (0u32, 0u32);
                for dz in 0..scale {
                    for dx in 0..scale {
                        let i = ((oz * scale + dz) * side + ox * scale + dx) * 4;
                        let a = u32::from(tile[i + 3]);
                        if a > 0 {
                            for (c, total) in sum.iter_mut().enumerate() {
                                *total += u32::from(tile[i + c]);
                            }
                            drawn += 1;
                        }
                        alpha += a;
                    }
                }
                let o = (oz * width + column * out_side + ox) * 4;
                if drawn == 0 {
                    row[o..o + 4].fill(0);
                    continue;
                }
                for (c, total) in sum.iter().enumerate() {
                    row[o + c] = (total / drawn) as u8;
                }
                row[o + 3] = (alpha / (scale * scale) as u32) as u8;
            }
        }
    }
}
