use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct DebugConfig {
    pub telemetry: bool,
    pub telemetry_interval_ms: i32,
    pub capture_probes: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self { telemetry: false, telemetry_interval_ms: 1000, capture_probes: false }
    }
}
