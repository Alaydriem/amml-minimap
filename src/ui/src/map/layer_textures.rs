use std::collections::HashMap;

use egui::{ColorImage, Context, Id, TextureHandle, TextureId, TextureOptions};
use minimap_core::structs::draw::TextureOp;

// the map textures of one layer, as egui textures of the manager's context for that layer. A texture
// belongs to the context that made it, so a new context (after a lost device or a caught panic) clears them;
// bounded by `capacity`, which the residency budget sets
pub struct LayerTextures {
    handles: HashMap<u32, TextureHandle>,
    capacity: usize,
}

impl LayerTextures {
    // marks a context these handles were made in
    const MARKER: &'static str = "minimap.layer_textures";

    pub fn new(capacity: usize) -> Self {
        Self { handles: HashMap::with_capacity(capacity), capacity: capacity.max(1) }
    }

    // true when `ctx` is not the context the handles were made in: they are dropped, and every texture must
    // be uploaded again
    pub fn ensure_context(&mut self, ctx: &Context) -> bool {
        let id = Id::unique(Self::MARKER);
        if ctx.data(|data| data.get_temp::<bool>(id)).is_some() {
            return false;
        }
        ctx.data_mut(|data| data.insert_temp(id, true));
        self.handles.clear();
        true
    }

    // uploads and releases; an upload names its pixels by index into `buffers`, straight-alpha RGBA. Every
    // texture samples nearest, so a block stays a sharp square when the map scales it up
    pub fn apply(&mut self, ctx: &Context, ops: &[TextureOp], buffers: &[Vec<u8>]) {
        for op in ops {
            match *op {
                TextureOp::Upload { tex, width, height, buffer } => {
                    let Some(pixels) = buffers.get(buffer as usize) else { continue };
                    let size = [width as usize, height as usize];
                    if pixels.len() != size[0] * size[1] * 4 {
                        continue;
                    }
                    let image = ColorImage::from_rgba_unmultiplied(size, pixels);
                    if let Some(handle) = self.handles.get_mut(&tex) {
                        handle.set(image, TextureOptions::NEAREST);
                    } else if self.handles.len() < self.capacity {
                        self.handles
                            .insert(tex, ctx.load_texture(format!("minimap.{tex}"), image, TextureOptions::NEAREST));
                    }
                }
                TextureOp::Release { tex } => {
                    self.handles.remove(&tex);
                }
            }
        }
    }

    pub fn id(&self, tex: u32) -> Option<TextureId> {
        self.handles.get(&tex).map(TextureHandle::id)
    }

    pub fn clear(&mut self) {
        self.handles.clear();
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }
}
