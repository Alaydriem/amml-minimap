use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct StoreConfig {
    pub journal_max_mb: i32,
    pub sync_interval_ms: i32,
    pub write_queue_capacity: i32,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self { journal_max_mb: 64, sync_interval_ms: 1000, write_queue_capacity: 1024 }
    }
}
