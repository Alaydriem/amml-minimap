use std::ffi::c_void;

use super::Vtable;

// a fake `Actor`: component pointers that point into the actor itself, the level, and what the dimension and
// block source functions return
#[repr(C)]
pub struct FakeActor {
    pub vtable: *const usize,
    pub state_vector: *const [f32; 3],
    pub rotation: *const [f32; 2],
    pub level: *mut c_void,
    pub dimension: *mut c_void,
    pub source: *mut c_void,
    pub alive: u8,
    pub pos: [f32; 3],
    pub rot: [f32; 2],
}

impl FakeActor {
    pub const IS_ALIVE: u64 = 1;

    pub fn new(pos: [f32; 3], level: *mut c_void, dimension: *mut c_void, source: *mut c_void) -> Box<Self> {
        let is_alive: extern "C" fn(*mut c_void) -> u8 = Self::is_alive;
        let mut actor = Box::new(Self {
            vtable: Vtable::leak(&[is_alive as usize]),
            state_vector: std::ptr::null(),
            rotation: std::ptr::null(),
            level,
            dimension,
            source,
            alive: 1,
            pos,
            rot: [0.0, 90.0],
        });
        actor.state_vector = &actor.pos;
        actor.rotation = &actor.rot;
        actor
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut *self).cast()
    }

    extern "C" fn is_alive(this: *mut c_void) -> u8 {
        // SAFETY: `this` is a FakeActor
        unsafe { (*this.cast::<FakeActor>()).alive }
    }

    // `Dimension& Actor::getDimension() const`
    pub extern "C" fn get_dimension(this: *mut c_void) -> *mut c_void {
        // SAFETY: `this` is a FakeActor
        unsafe { (*this.cast::<FakeActor>()).dimension }
    }

    // `BlockSource& Actor::getDimensionBlockSource() const`
    pub extern "C" fn get_dimension_block_source(this: *mut c_void) -> *mut c_void {
        // SAFETY: `this` is a FakeActor
        unsafe { (*this.cast::<FakeActor>()).source }
    }
}
