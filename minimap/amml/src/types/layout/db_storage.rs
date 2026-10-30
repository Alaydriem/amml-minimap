use std::mem::offset_of;

use amml_msvc::MsvcString;

// the head of the game's DBStorage, up to and with mFullPath. The same on 26.51 and 26.60. The game owns the
// storage; the mod only reads it through the game's pointer, so the members after mFullPath are left out
#[repr(C)]
pub struct DbStorageLayout {
    pub vtable: usize,         // 0x00
    _members: [u8; 0x90],      // 0x08 mEnvChain to mContentIdentity
    pub full_path: MsvcString, // 0x98 mFullPath (Core::PathBuffer<std::string>)
}

const _: () = assert!(offset_of!(DbStorageLayout, full_path) == 0x98);
