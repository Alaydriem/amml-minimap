use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct BakeConfig {
    pub worker_threads: i32,
    pub queue_capacity: i32,
}

impl Default for BakeConfig {
    fn default() -> Self {
        Self { worker_threads: 0, queue_capacity: 4096 }
    }
}
