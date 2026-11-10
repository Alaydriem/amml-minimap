// one laid-out text of a pass: its place in PassOutput::texts and its screen rect in physical pixels
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShownText {
    pub start: u32,
    pub len: u32,
    pub rect: [f32; 4],
}
