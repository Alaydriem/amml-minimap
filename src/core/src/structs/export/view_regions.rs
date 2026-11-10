use crate::structs::pos::RegionPos;

// the regions the world map shows, as inclusive bounds on one map layer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewRegions {
    pub x0: i32,
    pub x1: i32,
    pub z0: i32,
    pub z1: i32,
    pub layer: i32,
}

impl ViewRegions {
    // every region in the bounds, row by row
    pub fn regions(&self) -> Vec<RegionPos> {
        (self.z0..=self.z1).flat_map(|z| (self.x0..=self.x1).map(move |x| RegionPos::new(x, z, self.layer))).collect()
    }
}
