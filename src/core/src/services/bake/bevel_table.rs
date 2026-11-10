// average colour factor per column for every west / east / north / south relation:
// 0 equal, 1 neighbour lower, 2 neighbour higher; index = w * 27 + e * 9 + n * 3 + s
#[derive(Debug, Clone, PartialEq)]
pub struct BevelTable {
    scale: u8,
    factors: [f32; 81],
}

impl BevelTable {
    const EDGE_BRIGHT: f32 = 1.18;
    const EDGE_DARK: f32 = 0.72;
    const CORNER_BOOST: f32 = 0.40;

    pub fn new(scale: u8) -> Self {
        let scale_i = i32::from(scale);
        let edge_w = (scale_i / 6).max(1);
        let rcp = 1.0 / edge_w as f32;
        let total = (scale_i * scale_i) as f32;
        let mut factors = [0.0f32; 81];
        for hl in 0..3 {
            for hr in 0..3 {
                for ht in 0..3 {
                    for hb in 0..3 {
                        let mut sum = 0.0f32;
                        for dy in 0..scale_i {
                            let ty_t = if dy < edge_w { 1.0 - dy as f32 * rcp } else { 0.0 };
                            let ty_b =
                                if dy >= scale_i - edge_w { (dy - (scale_i - edge_w)) as f32 * rcp } else { 0.0 };
                            for dx in 0..scale_i {
                                let tx_l = if dx < edge_w { 1.0 - dx as f32 * rcp } else { 0.0 };
                                let tx_r =
                                    if dx >= scale_i - edge_w { (dx - (scale_i - edge_w)) as f32 * rcp } else { 0.0 };
                                sum += Self::pixel_factor([tx_l, tx_r, ty_t, ty_b], [hl, hr, ht, hb]);
                            }
                        }
                        factors[(hl * 27 + hr * 9 + ht * 3 + hb) as usize] = sum / total;
                    }
                }
            }
        }
        Self { scale, factors }
    }

    pub fn scale(&self) -> u8 {
        self.scale
    }

    pub fn factor(&self, west: u8, east: u8, north: u8, south: u8) -> f32 {
        self.factors[usize::from(west) * 27 + usize::from(east) * 9 + usize::from(north) * 3 + usize::from(south)]
    }

    // t = [left, right, top, bottom] edge weights; rel = the matching neighbour relations
    fn pixel_factor(t: [f32; 4], rel: [i32; 4]) -> f32 {
        if t.iter().all(|v| *v == 0.0) {
            return 1.0;
        }
        let mut factor = 1.0f32;
        for side in 0..4 {
            if t[side] <= 0.0 {
                continue;
            }
            if rel[side] == 1 {
                factor *= 1.0 + (Self::EDGE_BRIGHT - 1.0) * t[side];
            } else if rel[side] == 2 {
                factor *= 1.0 + (Self::EDGE_DARK - 1.0) * t[side];
            }
        }
        for (a, b) in [(0, 2), (1, 2), (0, 3), (1, 3)] {
            let tc = t[a].min(t[b]);
            if tc <= 0.0 {
                continue;
            }
            if rel[a] == 1 && rel[b] == 1 {
                factor *= 1.0 + (Self::EDGE_BRIGHT - 1.0) * tc * Self::CORNER_BOOST;
            } else if rel[a] == 2 && rel[b] == 2 {
                factor *= 1.0 + (Self::EDGE_DARK - 1.0) * tc * Self::CORNER_BOOST;
            }
        }
        factor.clamp(0.55, 1.58)
    }
}
