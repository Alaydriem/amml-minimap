// block-coordinate rectangle; x1 and z1 are the exclusive far edges
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRect {
    pub x0: i32,
    pub z0: i32,
    pub x1: i32,
    pub z1: i32,
}

impl BlockRect {
    pub fn new(x0: i32, z0: i32, x1: i32, z1: i32) -> Self {
        Self { x0, z0, x1, z1 }
    }
}
