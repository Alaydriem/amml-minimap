use std::mem::offset_of;

use amml_game::BlockLayout;

use super::FakeBlockType;

// a fake `Block`: the block type pointer at mBlockType, as BlockLayout has it
#[repr(C)]
pub struct FakeBlock {
    pub head: [u64; 13],
    pub block_type: *mut FakeBlockType,
}

impl FakeBlock {
    pub fn new(block_type: *mut FakeBlockType) -> Self {
        Self { head: [0; 13], block_type }
    }
}

const _: () = assert!(offset_of!(FakeBlock, block_type) == offset_of!(BlockLayout, block_type));
