use std::ffi::c_void;

use amml_game::{ChunkBlockPos, GameEntries, LevelChunk};

use crate::game::Biome;
use crate::types::MinimapEntry;

// the biome lookup of a LevelChunk
pub struct ChunkBiomes<'a> {
    entries: &'a GameEntries,
}

impl<'a> ChunkBiomes<'a> {
    pub fn new(entries: &'a GameEntries) -> Self {
        Self { entries }
    }

    // the biome at `pos` of `chunk`; `None` when the entry is off or the game returns null. Game thread
    #[allow(unsafe_code)]
    pub fn at(&self, chunk: &LevelChunk<'_>, pos: ChunkBlockPos) -> Option<Biome<'a>> {
        let function = usize::try_from(self.entries.get(MinimapEntry::LEVEL_CHUNK_GET_BIOME)?).ok()?;
        if function == 0 {
            return None;
        }
        // SAFETY: the entry is the address of `Biome const& LevelChunk::getBiome(ChunkBlockPos const&) const`, a
        // non-static member that returns a reference in rax
        let call: unsafe extern "C" fn(*mut c_void, *const ChunkBlockPos) -> *mut c_void =
            unsafe { std::mem::transmute(function as *const c_void) };
        // SAFETY: the signature is as above, the wrapper holds a live chunk and `pos` lives across the call
        let biome = unsafe { call(chunk.as_ptr(), &raw const pos) };
        // SAFETY: the game returns a live Biome reference, or null
        unsafe { Biome::from_raw(biome) }
    }
}
