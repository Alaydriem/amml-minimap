use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU32, Ordering};

use amml_game::{ChunkBlockPos, LevelChunkLayout2651};

use super::FakeSubChunk;

// a fake `LevelChunk` laid out as the 26.51 mirror, with the test state after it. Every column holds `stack` from
// the lowest block up, then `air`. The fake getBlock and the sub-chunk guard count every read outside the chunk
#[repr(C)]
pub struct FakeChunk {
    pub layout: LevelChunkLayout2651,
    pub stack: Vec<*mut c_void>,
    pub air: *mut c_void,
    pub subs: Vec<FakeSubChunk>,
    pub block_reads: AtomicU32,
    pub reads_outside: AtomicU32,
}

impl FakeChunk {
    pub const LOADED: u8 = 11;
    // two sub-chunks: y 0 to 31, the highest inclusive
    pub const MIN_Y: i32 = 0;
    pub const MAX_Y: i32 = 31;

    // a loaded chunk at (`x`, `z`) with `subs` sub-chunks, the last one a placeholder when `top_placeholder`
    pub fn new(
        x: i32,
        z: i32,
        stack: Vec<*mut c_void>,
        air: *mut c_void,
        subs: usize,
        top_placeholder: bool,
    ) -> Box<Self> {
        let height = stack.len() as i16;
        // SAFETY: the mirror holds integers, raw pointers and byte spans, for which all zero bytes are valid
        let mut layout: LevelChunkLayout2651 = unsafe { MaybeUninit::zeroed().assume_init() };
        layout.min = [x * 16, Self::MIN_Y, z * 16];
        layout.max = [x * 16 + 15, Self::MAX_Y, z * 16 + 15];
        layout.position = [x, z];
        layout.load_state = Self::LOADED;
        layout.heightmap = [height; 256];
        layout.render_heightmap = [height; 256];
        let mut chunk = Box::new(Self {
            layout,
            stack,
            air,
            subs: Vec::new(),
            block_reads: AtomicU32::new(0),
            reads_outside: AtomicU32::new(0),
        });
        let counter: *const AtomicU32 = &chunk.reads_outside;
        for i in 0..=subs {
            let placeholder = u8::from(top_placeholder && subs > 0 && i == subs - 1);
            let guard = u8::from(i == subs);
            chunk.subs.push(FakeSubChunk::new(placeholder, guard, counter));
        }
        let begin = chunk.subs.as_ptr() as usize;
        let size = size_of::<FakeSubChunk>();
        let vector = [begin, begin + subs * size, begin + (subs + 1) * size];
        // SAFETY: `mSubChunks` is a std::vector, three pointers: first, last and end of capacity
        unsafe { (&raw mut chunk.layout.sub_chunks).cast::<[usize; 3]>().write(vector) };
        chunk
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut *self).cast()
    }

    pub fn reads_outside(&self) -> u32 {
        self.reads_outside.load(Ordering::SeqCst)
    }

    pub fn block_reads(&self) -> u32 {
        self.block_reads.load(Ordering::SeqCst)
    }

    // `Block const& LevelChunk::getBlock(ChunkBlockPos const&) const`
    pub extern "C" fn get_block(this: *mut c_void, pos: *const ChunkBlockPos) -> *mut c_void {
        // SAFETY: the probe calls this with a FakeChunk and a live position
        let (chunk, pos) = unsafe { (&*this.cast::<FakeChunk>(), *pos) };
        chunk.block_reads.fetch_add(1, Ordering::SeqCst);
        if pos.x >= 16 || pos.z >= 16 || pos.y < 0 || i32::from(pos.y) > Self::MAX_Y - Self::MIN_Y {
            chunk.reads_outside.fetch_add(1, Ordering::SeqCst);
            return std::ptr::null_mut();
        }
        chunk.stack.get(pos.y as usize).copied().unwrap_or(chunk.air)
    }
}
