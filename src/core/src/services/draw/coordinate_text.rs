// the second coordinate line: where the player's position lands in the linked dimension
pub struct CoordinateText;

impl CoordinateText {
    pub const OVERWORLD: i32 = 0;
    pub const NETHER: i32 = 1;
    // an overworld block is 1/8 of a nether block
    pub const SCALE: i32 = 8;

    pub fn linked(dim: i32, x: i32, z: i32) -> Option<String> {
        match dim {
            Self::OVERWORLD => Some(format!("Nether {}, {}", x.div_euclid(Self::SCALE), z.div_euclid(Self::SCALE))),
            Self::NETHER => Some(format!("Overworld {}, {}", x * Self::SCALE, z * Self::SCALE)),
            _ => None,
        }
    }
}
