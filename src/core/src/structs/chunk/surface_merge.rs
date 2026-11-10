use crate::structs::chunk::{ChangeKinds, ChunkSurface};

#[derive(Debug, Clone, Copy)]
pub struct SurfaceMerge {
    pub surface: ChunkSurface,
    pub changes: ChangeKinds,
    pub first_load: bool,
    pub previous_range: Option<(i16, i16)>,
    pub needs_retry: bool,
}
