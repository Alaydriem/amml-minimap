use alaydriem_minimap::game::Biome;
use amml_msvc::MsvcString;

use crate::support::FakeHeap;

// the bytes of a game Biome, 8-aligned, up to the end of mHash
#[repr(C, align(8))]
struct FakeBiome([u8; 0x1C0]);

impl FakeBiome {
    // `Biome.mId` and `Biome.mHash` as derived on 26.51 and 26.60 (amml data/), before they were folded
    const ID: usize = 0x168;
    const HASH: usize = 0x190;
    // the std::string of a HashedString, after its 8-byte hash
    const HASH_TEXT: usize = 8;

    fn new(id: u16, name: &str) -> Box<Self> {
        let mut biome = Box::new(Self([0; 0x1C0]));
        biome.0[Self::ID..Self::ID + 2].copy_from_slice(&id.to_le_bytes());
        // SAFETY: FakeHeap's functions are a valid allocate and free pair
        let text = unsafe { MsvcString::new_in(&FakeHeap::msvc(), name) }.unwrap();
        // SAFETY: the std::string fits inside the bytes at an 8-aligned offset
        unsafe { biome.0.as_mut_ptr().add(Self::HASH + Self::HASH_TEXT).cast::<MsvcString>().write(text) };
        biome
    }
}

#[test]
fn the_biome_mirror_reads_the_id_and_the_name_at_the_26_51_and_26_60_offsets() {
    let mut fake = FakeBiome::new(0x2A, "a-biome-name-longer-than-fifteen");

    // SAFETY: a live fake of the game layout
    let biome = unsafe { Biome::from_raw(fake.0.as_mut_ptr().cast()) }.unwrap();

    assert_eq!(biome.id(), 0x2A);
    assert_eq!(biome.name().to_str().unwrap(), "a-biome-name-longer-than-fifteen");
}

#[test]
fn a_null_or_unaligned_biome_is_not_read() {
    let mut fake = FakeBiome::new(1, "plains");

    // SAFETY: null is allowed
    assert!(unsafe { Biome::from_raw(std::ptr::null_mut()) }.is_none());
    // SAFETY: an unaligned pointer is refused before it is read
    assert!(unsafe { Biome::from_raw(fake.0.as_mut_ptr().wrapping_add(1).cast()) }.is_none());
}
