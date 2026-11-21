//! The plain OS calls of minimap on its own process: process memory for telemetry and whether a process id still
//! runs (the sweep of unseeded world caches). No game access.

#![deny(unsafe_code)]

pub mod services;
