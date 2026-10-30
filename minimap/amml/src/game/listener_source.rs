use std::ffi::c_void;
use std::ptr::NonNull;

use amml_game::{GameEntries, RawObject};

use crate::types::{DimensionType, MinimapEntry};

// a `BlockSource*` of the game, for its listener calls
pub struct ListenerSource<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> ListenerSource<'a> {
    /// `None` for a null pointer
    ///
    /// # Safety
    /// `ptr` must be null or a live `BlockSource` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    // calls `void f(BlockSourceListener&)` in the slot `id` names; false when the entry is off or the slot null
    #[allow(unsafe_code)]
    fn listener_call(&self, id: amml_game::EntryId, listener: *mut c_void) -> bool {
        let Some(slot) = self.entries.get(id) else { return false };
        // SAFETY: `from_raw` guarantees a live BlockSource, whose first qword is its vtable, and `id` is a slot of
        // that vtable
        let Some(function) = (unsafe { RawObject::slot(self.ptr.as_ptr(), slot) }) else { return false };
        // SAFETY: the slot holds `void BlockSource::addListener(BlockSourceListener&)` or `removeListener`, with
        // the same signature
        let call: unsafe extern "C" fn(*mut c_void, *mut c_void) = unsafe { std::mem::transmute(function) };
        // SAFETY: the signature is as above, `this` is a live BlockSource and the caller passes a live listener
        unsafe { call(self.ptr.as_ptr(), listener) };
        true
    }

    // adds `listener`; false when the entry is off. The listener must stay live until it is removed or the
    // source calls its onSourceDestroyed. Game thread
    pub fn add_listener(&self, listener: *mut c_void) -> bool {
        self.listener_call(MinimapEntry::BLOCK_SOURCE_ADD_LISTENER, listener)
    }

    // removes `listener`; false when the entry is off. Game thread
    pub fn remove_listener(&self, listener: *mut c_void) -> bool {
        self.listener_call(MinimapEntry::BLOCK_SOURCE_REMOVE_LISTENER, listener)
    }

    // the dimension id of the source; `None` when the entry is off or the slot is null. Game thread
    #[allow(unsafe_code)]
    pub fn dimension_id(&self) -> Option<i32> {
        let slot = self.entries.get(MinimapEntry::BLOCK_SOURCE_GET_DIMENSION_ID)?;
        // SAFETY: as `listener_call`, for the slot of `getDimensionId`
        let function = unsafe { RawObject::slot(self.ptr.as_ptr(), slot) }?;
        // SAFETY: the slot holds `DimensionType BlockSource::getDimensionId() const`; a non-static member returns
        // the struct through a hidden pointer (this in rcx, the buffer in rdx), which MemberCall does
        let id: DimensionType = unsafe { amml_msvc::MemberCall::call_sret0(function, self.ptr.as_ptr()) };
        Some(id.value)
    }
}
