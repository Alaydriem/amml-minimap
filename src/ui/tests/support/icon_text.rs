use amml_ui::icon::{Icon, IconShape};

// the text a font icon paints, to find it among a pass's texts
pub struct IconText;

impl IconText {
    pub fn of(icon: Icon) -> String {
        match icon.shape() {
            IconShape::Font(glyph) => glyph.to_string(),
            IconShape::Drawn(_) => panic!("{icon:?} is drawn, not a font glyph"),
        }
    }
}
