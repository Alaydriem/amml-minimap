use amml_i18n::text::Text;
use amml_ui::draw::{TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::Toggle;
use egui::{Label, RichText, Sense, Ui};

use crate::views::SplitRow;

/// A label with a amml-ui switch at the right of the card, and an optional note after the label (the tools card's
/// "zoom in"). A click on the label flips the switch too. The switch keeps its place; the label and the note are cut.
pub struct ToggleRow<'a> {
    label: Text,
    on: &'a mut bool,
    note: Option<Text>,
}

impl<'a> ToggleRow<'a> {
    const LABEL: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Regular);
    const NOTE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);

    pub fn new(label: Text, on: &'a mut bool) -> Self {
        Self { label, on, note: None }
    }

    pub fn note(mut self, note: Option<Text>) -> Self {
        self.note = note;
        self
    }

    /// True when the label or the switch was clicked; the switch has then flipped `on`.
    pub fn show(self, ui: &mut Ui) -> bool {
        let Self { label, on, note } = self;
        let (switched, label_clicked) = SplitRow::show(
            ui,
            |ui| Toggle::new(&mut *on).show(ui).clicked(),
            |ui| {
                let label = RichText::new(UiText::resolve(ui.ctx(), &label))
                    .font(Self::LABEL.font_id())
                    .color(<color::Text as ColorToken>::VALUE);
                let clicked = ui.add(Label::new(label).sense(Sense::click()).selectable(false)).clicked();
                if let Some(note) = note {
                    let note = RichText::new(UiText::resolve(ui.ctx(), &note))
                        .font(Self::NOTE.font_id())
                        .color(<color::TextFaint as ColorToken>::VALUE);
                    ui.label(note);
                }
                clicked
            },
        );
        if label_clicked {
            *on = !*on;
        }
        label_clicked || switched
    }
}
