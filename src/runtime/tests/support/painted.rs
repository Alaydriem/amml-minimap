use minimap_core::structs::draw::{DrawList, TextureOp};
use minimap_runtime::structs::LayerReply;
use minimap_ui::testing::{PassOutput, ShownText};

// one frame of both manager layers, as the tests see it: the text of both, the draw list of the layer that
// drew the map, the texture work of both, and each layer's reply
pub struct Painted {
    pub mesh: PassOutput,
    pub draw: DrawList,
    pub textures: Vec<TextureOp>,
    pub hud: LayerReply,
    pub overlay: LayerReply,
}

impl Painted {
    pub fn new() -> Self {
        Self {
            mesh: PassOutput::new(),
            draw: DrawList::new(),
            textures: Vec::new(),
            hud: LayerReply::EMPTY,
            overlay: LayerReply::EMPTY,
        }
    }

    // the world map's layer drew this frame
    pub fn map_open(&self) -> bool {
        self.overlay.arrow_cursor
    }

    // puts the hud pass's text, then the overlay pass's, into `mesh`
    pub fn combine(&mut self, hud: &PassOutput, overlay: &PassOutput) {
        self.mesh.clear();
        for pass in [hud, overlay] {
            let offset = self.mesh.texts.len() as u32;
            self.mesh.texts.push_str(&pass.texts);
            self.mesh.text_rects.extend(pass.text_rects.iter().map(|t| ShownText { start: t.start + offset, ..*t }));
            self.mesh.meshes.extend(pass.meshes.iter().cloned());
            self.mesh.textures_set.extend_from_slice(&pass.textures_set);
        }
        self.mesh.wants_pointer = overlay.wants_pointer;
        self.mesh.wants_keyboard = overlay.wants_keyboard;
    }
}
