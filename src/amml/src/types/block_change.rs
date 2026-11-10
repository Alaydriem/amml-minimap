use std::ffi::c_void;

use crate::types::BlockPos;

// one onBlockChanged call: the source it came from, where, and the new and old blocks. The pointers are valid for
// the call only
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockChange {
    pub source: *mut c_void,
    pub pos: BlockPos,
    pub block: *mut c_void,
    pub old_block: *mut c_void,
}
