use std::ffi::c_void;

use super::Vtable;

// a fake `BlockSource` holding one chunk; it records every listener added and removed
#[repr(C)]
pub struct FakeSource {
    pub vtable: *const usize,
    pub chunk: *mut c_void,
    pub chunk_pos: [i32; 2],
    pub dim: i32,
    pub added: Vec<usize>,
    pub removed: Vec<usize>,
}

impl FakeSource {
    pub const GET_CHUNK: u64 = 1;
    pub const ADD_LISTENER: u64 = 2;
    pub const REMOVE_LISTENER: u64 = 3;
    pub const GET_DIMENSION_ID: u64 = 4;

    pub fn new(chunk: *mut c_void, chunk_pos: [i32; 2], dim: i32) -> Box<Self> {
        let get_chunk: extern "C" fn(*mut c_void, i32, i32) -> *mut c_void = Self::get_chunk;
        let add: extern "C" fn(*mut c_void, *mut c_void) = Self::add_listener;
        let remove: extern "C" fn(*mut c_void, *mut c_void) = Self::remove_listener;
        let dimension: extern "C" fn(*mut c_void, *mut i32) -> *mut i32 = Self::get_dimension_id;
        let vtable = Vtable::leak(&[get_chunk as usize, add as usize, remove as usize, dimension as usize]);
        Box::new(Self { vtable, chunk, chunk_pos, dim, added: Vec::new(), removed: Vec::new() })
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut *self).cast()
    }

    fn this<'a>(this: *mut c_void) -> &'a mut FakeSource {
        // SAFETY: the fake's vtable is only reached through a FakeSource
        unsafe { &mut *this.cast::<FakeSource>() }
    }

    extern "C" fn get_chunk(this: *mut c_void, x: i32, z: i32) -> *mut c_void {
        let source = Self::this(this);
        if [x, z] == source.chunk_pos { source.chunk } else { std::ptr::null_mut() }
    }

    extern "C" fn add_listener(this: *mut c_void, listener: *mut c_void) {
        Self::this(this).added.push(listener as usize);
    }

    extern "C" fn remove_listener(this: *mut c_void, listener: *mut c_void) {
        Self::this(this).removed.push(listener as usize);
    }

    // `DimensionType getDimensionId() const`, through the hidden return pointer
    extern "C" fn get_dimension_id(this: *mut c_void, out: *mut i32) -> *mut i32 {
        // SAFETY: the caller passes room for one DimensionType
        unsafe { *out = Self::this(this).dim };
        out
    }
}
