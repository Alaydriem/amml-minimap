use std::ffi::c_void;

use amml_msvc::MsvcString;

use crate::types::BiomeLayout;

// a `Biome*` of the game
pub struct Biome<'a> {
    layout: &'a BiomeLayout,
}

impl<'a> Biome<'a> {
    /// `None` for a null or unaligned pointer
    ///
    /// # Safety
    /// `ptr` must be null or a live `Biome` of the game, and must stay live while the wrapper is used
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void) -> Option<Self> {
        let ptr = ptr.cast::<BiomeLayout>();
        if ptr.is_null() || !ptr.is_aligned() {
            return None;
        }
        // SAFETY: the caller guarantees a live Biome, whose head BiomeLayout mirrors; the pointer is aligned
        Some(Self { layout: unsafe { &*ptr } })
    }

    pub fn as_ptr(&self) -> *mut c_void {
        std::ptr::from_ref(self.layout).cast_mut().cast()
    }

    // the game biome id (`mId`). Game thread
    pub fn id(&self) -> u16 {
        self.layout.id
    }

    // the internal name, such as `plains`, as a view of the game's std::string (the text of `mHash`); valid while
    // the game keeps the biome. Game thread
    pub fn name(&self) -> &'a MsvcString {
        &self.layout.hash.text
    }
}
