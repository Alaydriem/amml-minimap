use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// unique in the world; written to waypoints.json as 16 hex digits, because JSON numbers lose u64 precision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WaypointId(pub u64);

impl Serialize for WaypointId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{:016x}", self.0))
    }
}

impl<'de> Deserialize<'de> for WaypointId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        u64::from_str_radix(&text, 16)
            .map(WaypointId)
            .map_err(|_| D::Error::custom(format!("bad waypoint id {text:?}")))
    }
}
