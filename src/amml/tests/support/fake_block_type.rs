use std::mem::MaybeUninit;

use amml_game::{BlockTypePointerLayout, MaterialBoolsLayout};
use amml_msvc::MsvcString;

// a fake `BlockType` of the pointer layout (26.51): the material pointer and the full name
#[repr(C)]
pub struct FakeBlockType {
    pub layout: BlockTypePointerLayout,
}

impl FakeBlockType {
    pub fn new(name: MsvcString, material: *const MaterialBoolsLayout) -> Box<Self> {
        // SAFETY: the mirror holds integers, raw pointers, std::string mirrors and byte spans, for which all zero
        // bytes are valid
        let mut block_type: Box<Self> = Box::new(unsafe { MaybeUninit::zeroed().assume_init() });
        block_type.layout.material = material;
        // SAFETY: the field is a zeroed std::string mirror that owns nothing, so it is overwritten without a drop
        unsafe { (&raw mut block_type.layout.full_name.text).write(name) };
        block_type
    }
}
