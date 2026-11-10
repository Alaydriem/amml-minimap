use std::ffi::c_void;
use std::ptr::NonNull;

use amml_game::{GameEntries, RawObject};

use crate::types::{DbStorageLayout, MinimapEntry};

// a `DBStorage*` of the game: the storage of a local world
pub struct DbStorage<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> DbStorage<'a> {
    /// `None` for a null pointer
    ///
    /// # Safety
    /// `ptr` must be null or a live `DBStorage` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    // the world folder (`mFullPath`, whose std::string is at the start of its PathBuffer), copied; `None` when the
    // entry is off, the object does not start with DBStorage's vtable, or the text is not UTF-8
    #[allow(unsafe_code)]
    pub fn full_path(&self) -> Option<String> {
        let vtable = usize::try_from(self.entries.get(MinimapEntry::DB_STORAGE_VTABLE)?).ok()?;
        // SAFETY: `from_raw` guarantees a live object, whose first qword is readable
        if unsafe { RawObject::read::<usize>(self.ptr.as_ptr(), 0) } != Some(vtable) {
            return None;
        }
        let layout = self.ptr.as_ptr().cast::<DbStorageLayout>();
        if !layout.is_aligned() {
            return None;
        }
        // SAFETY: the object starts with DBStorage's vtable, so it is a live DBStorage, whose head DbStorageLayout
        // mirrors; the pointer is aligned
        let layout = unsafe { &*layout };
        layout.full_path.to_str().ok().map(str::to_string)
    }
}
