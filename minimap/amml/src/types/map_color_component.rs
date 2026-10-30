use std::mem::offset_of;

use crate::types::MceColor;

// the game's BlockMapColorComponent: the same on 26.51 and 26.60
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapColorComponent {
    pub base: MceColor, // 0x00 mMapColor
    pub tint: u8,       // 0x10 mTintMethod: the index of the biome policy that tints `base`
}

const _: () = assert!(offset_of!(MapColorComponent, tint) == 0x10);
