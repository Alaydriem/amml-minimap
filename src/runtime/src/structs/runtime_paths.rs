use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePaths {
    // per-mod data folder; server caches live under it
    pub data_dir: PathBuf,
}
