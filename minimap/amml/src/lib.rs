//! @alaydriem/minimap as an AMML mod: hooks the client update and the world storage, reads the loaded chunks
//! around the player, and drives minimap-runtime with them.

#![deny(unsafe_code)]

pub mod detours;
pub mod errors;
pub mod game;
pub mod minimap_mod;
pub mod services;
pub mod traits;
pub mod types;

pub use minimap_mod::MinimapMod;

amml_sdk::amml_mod!(MinimapMod);
