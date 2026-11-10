use std::fs;
use std::path::Path;

use minimap_core::structs::waypoint::Waypoint;
use serde_json::{Value, json};

use crate::errors::StoreError;
use crate::services::JsonFile;

// waypoints.json: { "version": 1, "waypoints": [ ... ] }
pub struct WaypointFile;

impl WaypointFile {
    pub const NAME: &'static str = "waypoints.json";
    pub const VERSION: u64 = 1;

    // the entries that parse, and a warning for each one dropped
    pub fn parse(text: &str) -> Result<(Vec<Waypoint>, Vec<String>), StoreError> {
        let document: Value = serde_json::from_str(text)?;
        let version = document.get("version").and_then(Value::as_u64).ok_or(StoreError::NotWaypointFile)?;
        if version != Self::VERSION {
            return Err(StoreError::UnsupportedVersion(u16::try_from(version).unwrap_or(u16::MAX)));
        }
        let entries = document.get("waypoints").and_then(Value::as_array).ok_or(StoreError::NotWaypointFile)?;
        let mut waypoints = Vec::with_capacity(entries.len());
        let mut warnings = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            match serde_json::from_value::<Waypoint>(entry.clone()) {
                Ok(waypoint) => waypoints.push(waypoint),
                Err(error) => warnings.push(format!("waypoint {index} dropped: {error}")),
            }
        }
        Ok((waypoints, warnings))
    }

    pub fn to_json(waypoints: &[Waypoint]) -> Result<String, StoreError> {
        Ok(serde_json::to_string_pretty(&json!({ "version": Self::VERSION, "waypoints": waypoints }))?)
    }

    // a missing file is no waypoints; a file that does not parse is moved to waypoints.json.invalid,
    // so the next save cannot overwrite what the player may want back
    pub fn load(path: &Path) -> Result<Vec<Waypoint>, StoreError> {
        let Some(text) = JsonFile::read(path)? else { return Ok(Vec::new()) };
        match Self::parse(&text) {
            Ok((waypoints, warnings)) => {
                for warning in warnings {
                    curia::warn!("waypoint file warning", { path: path.display().to_string(), warning: warning.to_string() });
                }
                Ok(waypoints)
            }
            Err(error) => {
                curia::warn!("waypoint file is not readable; it is kept as waypoints.json.invalid", { path: path.display().to_string(), error: error.to_string() });
                fs::rename(path, path.with_extension("json.invalid"))?;
                Ok(Vec::new())
            }
        }
    }

    // replaces the file in one rename
    pub fn save(path: &Path, waypoints: &[Waypoint]) -> Result<(), StoreError> {
        JsonFile::write(path, &Self::to_json(waypoints)?)
    }
}
