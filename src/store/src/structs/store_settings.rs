use std::time::Duration;

use minimap_core::config::StoreConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreSettings {
    pub journal_max_bytes: u64,
    pub sync_interval: Duration,
    pub queue_capacity: usize,
    pub idle_compaction: Duration,
}

impl StoreSettings {
    pub const IDLE_COMPACTION: Duration = Duration::from_secs(30);

    pub fn new(config: &StoreConfig) -> Self {
        Self {
            journal_max_bytes: config.journal_max_mb.max(1) as u64 * 1024 * 1024,
            sync_interval: Duration::from_millis(config.sync_interval_ms.max(1) as u64),
            queue_capacity: config.write_queue_capacity.max(1) as usize,
            idle_compaction: Self::IDLE_COMPACTION,
        }
    }
}
