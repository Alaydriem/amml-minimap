use egui::epaint::Mesh;
use egui::{TextureId, TextureOptions};

use crate::testing::ShownText;

// what one test pass produced: its text with rects, the tessellated meshes, the texture changes it asked
// the manager for, and where egui wanted the pointer and keyboard
#[derive(Debug, Default)]
pub struct PassOutput {
    pub texts: String,
    pub text_rects: Vec<ShownText>,
    pub meshes: Vec<Mesh>,
    // textures set this pass: id, size and options
    pub textures_set: Vec<(TextureId, [usize; 2], TextureOptions)>,
    pub textures_freed: Vec<TextureId>,
    pub wants_pointer: bool,
    pub wants_keyboard: bool,
    // text the pass put on the clipboard
    pub clipboard: String,
}

impl PassOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.texts.clear();
        self.text_rects.clear();
        self.meshes.clear();
        self.textures_set.clear();
        self.textures_freed.clear();
        self.wants_pointer = false;
        self.wants_keyboard = false;
        self.clipboard.clear();
    }

    pub fn text_of(&self, text: &ShownText) -> &str {
        &self.texts[text.start as usize..(text.start + text.len) as usize]
    }

    pub fn draws_text(&self, text: &str) -> bool {
        self.text_rects.iter().any(|t| self.text_of(t) == text)
    }

    pub fn text_starting(&self, prefix: &str) -> bool {
        self.text_rects.iter().any(|t| self.text_of(t).starts_with(prefix))
    }

    // the rect of the last text that is exactly `text`
    pub fn text_rect(&self, text: &str) -> Option<[f32; 4]> {
        self.text_rects.iter().rev().find(|t| self.text_of(t) == text).map(|t| t.rect)
    }

    pub fn vertex_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.vertices.len()).sum()
    }

    // the meshes drawn with one of minimap's own textures, not egui's font atlas
    pub fn textured(&self) -> impl Iterator<Item = &Mesh> {
        self.meshes.iter().filter(|mesh| mesh.texture_id != TextureId::default())
    }
}
