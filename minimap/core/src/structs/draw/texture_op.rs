// GPU work for the host: `buffer` indexes the frame's upload buffers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureOp {
    Upload { tex: u32, width: u32, height: u32, buffer: u32 },
    Release { tex: u32 },
}
