use std::ffi::c_void;

use super::Vtable;

// a fake `Dimension` with an id
#[repr(C)]
pub struct FakeDimension {
    pub vtable: *const usize,
    pub id: i32,
}

impl FakeDimension {
    pub const GET_DIMENSION_ID: u64 = 1;

    pub fn new(id: i32) -> Box<Self> {
        let get_id: extern "C" fn(*mut c_void, *mut i32) -> *mut i32 = Self::get_dimension_id;
        Box::new(Self { vtable: Vtable::leak(&[get_id as usize]), id })
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut *self).cast()
    }

    extern "C" fn get_dimension_id(this: *mut c_void, out: *mut i32) -> *mut i32 {
        // SAFETY: `this` is a FakeDimension and the caller passes room for one DimensionType
        unsafe { *out = (*this.cast::<FakeDimension>()).id };
        out
    }
}
