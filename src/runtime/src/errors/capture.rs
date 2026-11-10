#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("capture i/o: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a capture file")]
    BadMagic,
    #[error("capture version {0} is not supported")]
    UnsupportedVersion(u16),
    #[error("capture compression {0} is not supported")]
    UnsupportedCompression(u8),
    #[error("unknown capture record tag {0}")]
    UnknownTag(u8),
    #[error("capture record {tag} has {actual} bytes, expected {expected}")]
    WrongSize { tag: u8, expected: usize, actual: usize },
    #[error("capture record {0} holds a value out of range")]
    BadValue(u8),
    #[error("capture settings: {0}")]
    Settings(#[from] serde_json::Error),
}
