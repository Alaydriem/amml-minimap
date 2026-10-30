use std::ffi::c_void;

use super::Vtable;

// a fake `Level` with a seed and no biome registry
#[repr(C)]
pub struct FakeLevel {
    pub vtable: *const usize,
    pub seed: u32,
}

impl FakeLevel {
    pub const GET_SEED: u64 = 1;
    pub const GET_BIOME_REGISTRY: u64 = 2;

    pub fn new(seed: u32) -> Box<Self> {
        let get_seed: extern "C" fn(*mut c_void) -> u32 = Self::get_seed;
        let get_registry: extern "C" fn(*mut c_void) -> *mut c_void = Self::get_biome_registry;
        Box::new(Self { vtable: Vtable::leak(&[get_seed as usize, get_registry as usize]), seed })
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut *self).cast()
    }

    extern "C" fn get_seed(this: *mut c_void) -> u32 {
        // SAFETY: `this` is a FakeLevel
        unsafe { (*this.cast::<FakeLevel>()).seed }
    }

    extern "C" fn get_biome_registry(_: *mut c_void) -> *mut c_void {
        std::ptr::null_mut()
    }
}
