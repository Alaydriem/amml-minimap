use crate::structs::ui::MessageKind;

// a short note shown at the top of the screen until `remaining` seconds run out
#[derive(Debug, Clone, PartialEq)]
pub struct MessageLine {
    pub text: String,
    pub remaining: f32,
    pub kind: MessageKind,
}
