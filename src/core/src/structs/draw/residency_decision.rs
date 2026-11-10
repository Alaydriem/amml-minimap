#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidencyDecision {
    // a texture with the current pixels is on the GPU
    Draw(u32),
    // upload the tile into this texture, then report it with `uploaded`
    Upload { tex: u32, created: bool },
    // nothing to draw yet
    Placeholder,
}
