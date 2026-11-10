use std::ffi::c_void;
use std::ptr::NonNull;

use amml_game::GameEntries;

use crate::game::Biome;
use crate::types::MinimapEntry;

// a `BiomeRegistry*` of the game
pub struct BiomeRegistry<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> BiomeRegistry<'a> {
    /// `None` for a null pointer
    ///
    /// # Safety
    /// `ptr` must be null or a live `BiomeRegistry` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    // the biome with the game id `id`; `None` when the entry is off or the registry does not know the id. Game
    // thread
    #[allow(unsafe_code)]
    pub fn lookup(&self, id: u16) -> Option<Biome<'a>> {
        let function = usize::try_from(self.entries.get(MinimapEntry::BIOME_REGISTRY_LOOKUP_BY_ID)?).ok()?;
        if function == 0 {
            return None;
        }
        // SAFETY: the entry is the address of `Biome* BiomeRegistry::lookupById(BiomeIdType) const`; the id is a
        // two-byte struct passed in dx and the pointer comes back in rax
        let call: unsafe extern "C" fn(*mut c_void, u16) -> *mut c_void =
            unsafe { std::mem::transmute(function as *const c_void) };
        // SAFETY: the signature is as above and `from_raw` guarantees a live registry
        let biome = unsafe { call(self.ptr.as_ptr(), id) };
        // SAFETY: the game returns a live Biome, or null
        unsafe { Biome::from_raw(biome) }
    }
}
