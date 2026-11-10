use std::alloc::{Layout, alloc, dealloc};
use std::sync::Mutex;

use amml_msvc::MsvcHeap;

use super::HeapEvent;

static EVENTS: Mutex<Vec<HeapEvent>> = Mutex::new(Vec::new());

// a game heap over the Rust allocator that records every block it hands out and frees, in order, beside the
// events the fakes add
pub struct FakeHeap;

impl FakeHeap {
    pub fn msvc() -> MsvcHeap {
        MsvcHeap::new(Self::allocate, Self::free)
    }

    pub fn record(event: HeapEvent) {
        EVENTS.lock().unwrap().push(event);
    }

    pub fn events() -> Vec<HeapEvent> {
        EVENTS.lock().unwrap().clone()
    }

    // how often the block at `ptr` was freed
    pub fn frees_of(ptr: usize) -> usize {
        Self::events().iter().filter(|event| **event == HeapEvent::Free(ptr)).count()
    }

    // how many blocks of `size` bytes were handed out
    pub fn allocations_of(size: usize) -> usize {
        Self::events().iter().filter(|event| matches!(event, HeapEvent::Allocate(_, bytes) if *bytes == size)).count()
    }

    unsafe extern "C" fn allocate(size: usize) -> *mut u8 {
        // SAFETY: the heap is asked for non-zero sizes only
        let ptr = unsafe { alloc(Layout::from_size_align(size, 16).unwrap()) };
        Self::record(HeapEvent::Allocate(ptr as usize, size));
        ptr
    }

    unsafe extern "C" fn free(ptr: *mut u8, size: usize) {
        Self::record(HeapEvent::Free(ptr as usize));
        // SAFETY: `ptr` came from `allocate` with this size
        unsafe { dealloc(ptr, Layout::from_size_align(size, 16).unwrap()) };
    }
}
