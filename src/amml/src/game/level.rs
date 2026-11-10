use std::ffi::c_void;
use std::ptr::NonNull;

use amml_game::{GameEntries, RawObject};

use crate::game::BiomeRegistry;
use crate::types::MinimapEntry;

// an `ILevel*` of the game
pub struct Level<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> Level<'a> {
    /// `None` for a null pointer
    ///
    /// # Safety
    /// `ptr` must be null or a live `Level` of the game build `entries` was resolved for, and must stay live while
    /// the wrapper is used
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    // the world seed through the `getSeed` virtual; `None` when the entry is off or the slot is null. Game thread
    #[allow(unsafe_code)]
    pub fn seed(&self) -> Option<u32> {
        let slot = self.entries.get(MinimapEntry::LEVEL_GET_SEED)?;
        // SAFETY: `from_raw` guarantees a live Level, whose first qword is its vtable, and the entry is the slot of
        // `getSeed` in that vtable
        let function = unsafe { RawObject::slot(self.ptr.as_ptr(), slot) }?;
        // SAFETY: the slot holds `uint ILevel::getSeed()`, which returns the int in eax
        let call: unsafe extern "C" fn(*mut c_void) -> u32 = unsafe { std::mem::transmute(function) };
        // SAFETY: the signature is as above and `this` is a live Level
        Some(unsafe { call(self.ptr.as_ptr()) })
    }

    // the biome registry through the `getBiomeRegistry` virtual; `None` when the entry is off or the game returns
    // null. Game thread
    #[allow(unsafe_code)]
    pub fn biome_registry(&self) -> Option<BiomeRegistry<'a>> {
        let slot = self.entries.get(MinimapEntry::LEVEL_GET_BIOME_REGISTRY)?;
        // SAFETY: as `seed`, for the slot of `getBiomeRegistry`
        let function = unsafe { RawObject::slot(self.ptr.as_ptr(), slot) }?;
        // SAFETY: the slot holds `BiomeRegistry& ILevel::getBiomeRegistry()`, a reference returned in rax
        let registry = unsafe { RawObject::call_pointer(function, self.ptr.as_ptr()) }?;
        // SAFETY: the game returns its live registry
        unsafe { BiomeRegistry::from_raw(registry, self.entries) }
    }
}
