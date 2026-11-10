// shared helpers; each test module uses a different subset
#![allow(dead_code, unused_imports)]

pub mod fake_environment;
pub mod test_frames;
pub mod test_surfaces;

pub use fake_environment::FakeEnvironment;
pub use test_frames::TestFrames;
pub use test_surfaces::TestSurfaces;
