use crate::structs::draw::{DrawCommand, TextAlign};

// the commands of one frame; buffers keep their capacity between frames
#[derive(Debug, Clone, Default)]
pub struct DrawList {
    pub commands: Vec<DrawCommand>,
    pub text: Vec<u8>,
}

impl DrawList {
    pub fn new() -> Self {
        Self { commands: Vec::with_capacity(256), text: Vec::with_capacity(256) }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.text.clear();
    }

    pub fn push(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    pub fn text(&mut self, pos: (f32, f32), colour: u32, size: f32, align: TextAlign, text: &str) {
        let start = self.text.len() as u32;
        self.text.extend_from_slice(text.as_bytes());
        self.commands.push(DrawCommand::Text { pos, colour, size, align, start, len: text.len() as u32 });
    }

    pub fn text_of(&self, start: u32, len: u32) -> &str {
        std::str::from_utf8(&self.text[start as usize..(start + len) as usize]).unwrap_or("")
    }
}
