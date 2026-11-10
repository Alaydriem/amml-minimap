use std::mem::offset_of;

use amml_game::game::HashedStringLayout;

// the head of the game's Biome, up to and with mHash. The same on 26.51 and 26.60. The game owns every Biome;
// the mod only reads one through the game's pointer, so the members after mHash are left out
#[repr(C)]
pub struct BiomeLayout {
    _head: [u8; 0x168],           // 0x000 the vtable, the climate values, mMobs, mMobsMap
    pub id: u16,                  // 0x168 mId (BiomeIdType)
    _components: [u8; 0x26],      // 0x16A mBiomeComponentStorage
    pub hash: HashedStringLayout, // 0x190 mHash
}

const _: () = assert!(offset_of!(BiomeLayout, id) == 0x168);
const _: () = assert!(offset_of!(BiomeLayout, hash) == 0x190);
