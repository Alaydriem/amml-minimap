use amml_ui::tokens::{SpacingToken, spacing};
use egui::{Align, Layout, Rect, Sense, TextWrapMode, Ui, UiBuilder, pos2, vec2};

/// A card row across the card's width: the trailing part (a switch, a close button, a short name) is laid out first
/// at the right, then the leading part gets the room that is left, with its labels cut. So a long translation never
/// widens the card or runs under the trailing part.
pub struct SplitRow;

impl SplitRow {
    /// Returns what `trailing` returned and what `leading` returned.
    pub fn show<T, L>(ui: &mut Ui, trailing: impl FnOnce(&mut Ui) -> T, leading: impl FnOnce(&mut Ui) -> L) -> (T, L) {
        let height = <spacing::Row as SpacingToken>::VALUE;
        let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
        let mut right = ui.new_child(UiBuilder::new().max_rect(row).layout(Layout::right_to_left(Align::Center)));
        let trailing = trailing(&mut right);
        let used = right.min_rect();
        let gap = if used.width() > 0.0 { <spacing::S as SpacingToken>::VALUE } else { 0.0 };
        let room = Rect::from_min_max(row.min, pos2((used.left() - gap).max(row.left()), row.bottom()));
        let mut left = ui.new_child(UiBuilder::new().max_rect(room).layout(Layout::left_to_right(Align::Center)));
        left.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        let leading = leading(&mut left);
        (trailing, leading)
    }
}
