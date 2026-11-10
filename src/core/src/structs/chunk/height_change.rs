use crate::structs::pos::BlockRect;

// a change of surface heights inside `rect`; `higher` / `lower` span the old and new state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightChange {
    pub rect: BlockRect,
    pub higher: i32,
    pub lower: i32,
    pub dim: i32,
}

impl HeightChange {
    // lower for a chunk seen for the first time: no "always shadowed" pruning can apply
    pub const FIRST_LOAD_LOWER: i32 = i32::MIN / 2;

    pub fn new(rect: BlockRect, higher: i32, lower: i32, dim: i32) -> Self {
        Self { rect, higher, lower, dim }
    }
}
