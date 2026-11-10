// the world map overlays draw only while a chunk is big enough on screen to read; below that the
// lines and tints would fill the screen and cost frame time
pub struct OverlayGate;

impl OverlayGate {
    pub const MIN_CHUNK_PIXELS: f32 = 6.0;
    const CHUNK_BLOCKS: f32 = 16.0;

    pub fn open(zoom: f32) -> bool {
        zoom * Self::CHUNK_BLOCKS >= Self::MIN_CHUNK_PIXELS
    }
}
