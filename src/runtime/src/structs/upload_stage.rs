use minimap_core::structs::draw::TextureOp;

// one layer pass's texture work: uploads and releases, with the uploads' pixels in buffers that keep their
// capacity between passes
#[derive(Debug, Default)]
pub struct UploadStage {
    pub ops: Vec<TextureOp>,
    // pixel data for Upload ops, indexed by TextureOp::Upload::buffer; only the first `used` are live
    pub buffers: Vec<Vec<u8>>,
    pub used: usize,
    // bytes staged since the last clear
    pub bytes: u64,
}

impl UploadStage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.ops.clear();
        self.used = 0;
        self.bytes = 0;
    }

    pub fn upload(&mut self, tex: u32, width: u32, height: u32, pixels: &[u8]) {
        if self.used == self.buffers.len() {
            self.buffers.push(Vec::new());
        }
        let buffer = &mut self.buffers[self.used];
        buffer.clear();
        buffer.extend_from_slice(pixels);
        self.ops.push(TextureOp::Upload { tex, width, height, buffer: self.used as u32 });
        self.used += 1;
        self.bytes += pixels.len() as u64;
    }

    pub fn live_buffers(&self) -> &[Vec<u8>] {
        &self.buffers[..self.used]
    }
}
