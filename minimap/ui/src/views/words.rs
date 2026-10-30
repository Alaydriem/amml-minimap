use amml_i18n::text::Text;
use amml_ui::draw::TextStyle;
use amml_ui::text::UiText;
use egui::{Color32, RichText, Ui};

/// The cards' egui labels, from a `Text` in the context's language.
pub struct Words;

impl Words {
    pub fn rich(ui: &Ui, text: &Text, style: TextStyle, color: Color32) -> RichText {
        RichText::new(UiText::resolve(ui.ctx(), text)).font(style.font_id()).color(color)
    }
}
