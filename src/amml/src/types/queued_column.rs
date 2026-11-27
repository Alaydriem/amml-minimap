// a block column a block change asks to read again at the next client tick, in blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueuedColumn {
    pub dimension: i32,
    pub x: i32,
    pub z: i32,
}
