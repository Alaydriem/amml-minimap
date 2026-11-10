use egui::Rect;

// what a layer pass tells the manager: the area it owns for the pointer, in points, and the cursor it wants
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerReply {
    pub rect: Rect,
    pub arrow_cursor: bool,
}

impl LayerReply {
    pub const EMPTY: LayerReply = LayerReply { rect: Rect::NOTHING, arrow_cursor: false };
}
