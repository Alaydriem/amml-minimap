#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("store error: {0}")]
    Store(#[from] minimap_store::errors::StoreError),
    #[error("capture error: {0}")]
    Capture(#[from] crate::errors::CaptureError),
    #[error("thread could not start: {0}")]
    Thread(#[from] std::io::Error),
}
