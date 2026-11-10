use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};

// a fake `SubChunk` element of `LevelChunk::mSubChunks`, the size of the game's (SubChunkLayout). The vector is
// followed by one guard element, so a read past its end is counted instead of reaching other memory
#[repr(C)]
pub struct FakeSubChunk {
    pub placeholder: u8,
    pub guard: u8,
    pub pad: [u8; 6],
    pub reads_past_end: *const AtomicU32,
    pub rest: [u8; 0x58],
}

const _: () = assert!(size_of::<FakeSubChunk>() == size_of::<amml_game::SubChunkLayout>());

impl FakeSubChunk {
    pub fn new(placeholder: u8, guard: u8, reads_past_end: *const AtomicU32) -> Self {
        Self { placeholder, guard, pad: [0; 6], reads_past_end, rest: [0; 0x58] }
    }

    // `bool SubChunk::isPlaceHolderSubChunk() const`
    pub extern "C" fn is_placeholder(this: *mut c_void) -> u8 {
        // SAFETY: the probe calls this with an element of a FakeChunk's sub-chunk vector or its guard
        let sub = unsafe { &*this.cast::<FakeSubChunk>() };
        if sub.guard != 0 {
            // SAFETY: the counter is a field of the FakeChunk that owns the vector
            unsafe { (*sub.reads_past_end).fetch_add(1, Ordering::SeqCst) };
        }
        sub.placeholder
    }
}
