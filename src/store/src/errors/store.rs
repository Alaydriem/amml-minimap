#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("journal error: {0}")]
    Journal(#[from] wal_db::WalError),
    #[error("data is shorter than its header says ({0} bytes)")]
    Truncated(usize),
    #[error("unknown magic number")]
    BadMagic,
    #[error("unsupported format version {0}")]
    UnsupportedVersion(u16),
    #[error("checksum mismatch in the {0} section")]
    ChecksumMismatch(&'static str),
    #[error("decompressed {section} section has {actual} bytes, expected {expected}")]
    WrongSize { section: &'static str, expected: usize, actual: usize },
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a waypoints file: it needs a version and a waypoints list")]
    NotWaypointFile,
    #[error("a biome name is empty or longer than {0} bytes")]
    BadBiomeName(usize),
    #[error("a biome key file names {0} biomes, more than a table holds")]
    TooManyBiomes(usize),
    #[error("the store worker has stopped")]
    Stopped,
}
