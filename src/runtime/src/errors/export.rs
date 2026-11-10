#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("png error: {0}")]
    Png(#[from] png::EncodingError),
    #[error("store error: {0}")]
    Store(#[from] minimap_store::errors::StoreError),
    #[error("the world was left")]
    Stopped,
}
