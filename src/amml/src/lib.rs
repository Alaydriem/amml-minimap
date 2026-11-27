//! @alaydriem/minimap as an AMML mod: reads the player and the loaded chunks around it through `amml-api` on each
//! client tick, and drives minimap-runtime with them.

#![deny(unsafe_code)]

pub mod minimap_mod;
pub mod services;
pub mod traits;
pub mod types;

pub use minimap_mod::MinimapMod;

amml_api::amml_mod!(MinimapMod);
