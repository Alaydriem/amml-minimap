use amm_core::errors::LinkError;
use minimap_runtime::errors::RuntimeError;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("mod manager: {0}")]
    Link(#[from] LinkError),
    #[error("runtime: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("config folder: {0}")]
    Folder(#[from] std::io::Error),
}
