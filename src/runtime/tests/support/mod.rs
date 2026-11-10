// shared helpers; each test module uses a different subset
#![allow(dead_code, unused_imports)]

pub mod painted;
pub mod synthetic_world;

pub use painted::Painted;
pub use synthetic_world::SyntheticWorld;
