#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TileError {
    #[error("tile pixel data has {0} bytes, expected 262144")]
    WrongLength(usize),
}
