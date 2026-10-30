use std::mem::offset_of;

use amml_msvc::MsvcString;

// a fake `DBStorage`: its vtable pointer, then the world folder string at mFullPath
#[repr(C)]
pub struct FakeStorage {
    pub vtable: usize,
    pub members: [u8; 0x90],
    pub path: MsvcString,
}

impl FakeStorage {
    pub const VTABLE: usize = 0xDB00;
    // `DBStorage.mFullPath` as derived on 26.51 and 26.60 (amml data/), before it was folded
    pub const FULL_PATH: usize = 0x98;

    pub fn new(vtable: usize, path: MsvcString) -> Self {
        Self { vtable, members: [0; 0x90], path }
    }
}

const _: () = assert!(offset_of!(FakeStorage, path) == FakeStorage::FULL_PATH);
