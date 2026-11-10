use std::ffi::c_void;

use alaydriem_minimap::game::MapColorCalls;
use alaydriem_minimap::types::{MapColorComponent, MceColor, MinimapEntry};

use crate::support::FakeWorld;

// the bytes of a game BlockMapColorComponent, 4-aligned
#[repr(C, align(4))]
struct FakeComponent([u8; 0x14]);

impl FakeComponent {
    // `BlockMapColorComponent.mMapColor` and `mTintMethod` as derived on 26.51 and 26.60 (amml data/), before they
    // were folded
    const MAP_COLOR: usize = 0x0;
    const TINT_METHOD: usize = 0x10;

    fn new(color: [f32; 4], tint: u8) -> Self {
        let mut bytes = [0; 0x14];
        for (i, channel) in color.iter().enumerate() {
            let at = Self::MAP_COLOR + i * 4;
            bytes[at..at + 4].copy_from_slice(&channel.to_le_bytes());
        }
        bytes[Self::TINT_METHOD] = tint;
        Self(bytes)
    }

    // `Block::tryGetComponent<BlockMapColorComponent>()`: the fake block is its own component
    extern "C" fn try_get(block: *mut c_void) -> *mut c_void {
        block
    }

    // `Block::tryGetComponent<BlockMapColorComponent>()` of a block with no component
    extern "C" fn try_get_none(_: *mut c_void) -> *mut c_void {
        std::ptr::null_mut()
    }
}

fn calls_with(function: extern "C" fn(*mut c_void) -> *mut c_void) -> amml_game::GameEntries {
    FakeWorld::entries_with(&[(MinimapEntry::BLOCK_TRY_GET_MAP_COLOR_COMPONENT, function as usize as u64)])
}

#[test]
fn the_component_mirror_reads_the_colour_and_tint_at_the_26_51_and_26_60_offsets() {
    let mut fake = FakeComponent::new([0.25, 0.5, 0.75, 1.0], 3);
    let entries = calls_with(FakeComponent::try_get);

    // SAFETY: the fake block is a live component of the game layout
    let component = unsafe { MapColorCalls::new(&entries).component((&raw mut fake).cast()) };

    let base = MceColor { r: 0.25, g: 0.5, b: 0.75, a: 1.0 };
    assert_eq!(component, Some(MapColorComponent { base, tint: 3 }));
}

#[test]
fn a_block_with_no_component_gives_none() {
    let mut fake = FakeComponent::new([0.0; 4], 0);
    let entries = calls_with(FakeComponent::try_get_none);

    // SAFETY: the fake function never reads the block
    let component = unsafe { MapColorCalls::new(&entries).component((&raw mut fake).cast()) };

    assert_eq!(component, None);
}
