use amml_i18n::text::Text;
use amml_ui::draw::{TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::selector::AxisColor;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Color32, Label, Layout, RichText, Ui};
use minimap_core::structs::ui::HoverInfo;

use crate::message::MinimapMessage;
use crate::views::Words;

// the block, chunk and biome under the cursor, as label and value rows; dashes when the cursor is
// not on the map
pub struct HoverView;

impl HoverView {
    pub const NO_DATA: &'static str = "\u{2014}";
    const VALUE_SIZE: f32 = 14.0;
    const LABEL: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Regular);
    const BIOME: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Semibold);

    pub fn show(ui: &mut Ui, hover: Option<HoverInfo<'_>>) {
        let block = hover.map(|h| Self::axes(ui, h.block));
        Self::row(ui, MinimapMessage::Block, block);
        let text = <color::Text as ColorToken>::VALUE;
        let chunk = hover.map(|h| Self::plain(&format!("{}, {}", h.chunk.0, h.chunk.1), text));
        Self::row(ui, MinimapMessage::Chunk, chunk);
        // the biome name comes from minimap-core and stays English for now
        let biome = hover.and_then(|h| h.biome).map(|name| {
            let mut job = LayoutJob::default();
            job.append(name, 0.0, TextFormat::simple(Self::BIOME.font_id(), text));
            job
        });
        Self::row(ui, MinimapMessage::Biome, biome);
    }

    fn row(ui: &mut Ui, label: MinimapMessage, value: Option<LayoutJob>) {
        ui.horizontal(|ui| {
            ui.set_min_height(<spacing::Row as SpacingToken>::VALUE);
            ui.label(Words::rich(ui, &Text::message(label), Self::LABEL, <color::TextDim as ColorToken>::VALUE));
            // a value wraps in the room right of the label (a long biome name), right-aligned
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| match value {
                Some(job) => ui.add(Label::new(job).wrap()),
                None => {
                    let dash = RichText::new(Self::NO_DATA).font(Self::LABEL.font_id());
                    ui.label(dash.color(<color::TextFaint as ColorToken>::VALUE))
                }
            });
        });
    }

    // "X 316  Z -58" with the axis letters in their colours
    fn axes(ui: &Ui, block: (i32, i32)) -> LayoutJob {
        let font = FontFace::Regular.id(Self::VALUE_SIZE);
        let text = <color::Text as ColorToken>::VALUE;
        let mut job = LayoutJob::default();
        for (axis, letter, value, gap) in
            [(AxisColor::X, MinimapMessage::AxisX, block.0, "  "), (AxisColor::Z, MinimapMessage::AxisZ, block.1, "")]
        {
            let letter = format!("{} ", UiText::resolve(ui.ctx(), &Text::message(letter)));
            job.append(&letter, 0.0, TextFormat::simple(font.clone(), axis.color()));
            job.append(&format!("{value}{gap}"), 0.0, TextFormat::simple(font.clone(), text));
        }
        job
    }

    fn plain(text: &str, colour: Color32) -> LayoutJob {
        let mut job = LayoutJob::default();
        job.append(text, 0.0, TextFormat::simple(FontFace::Regular.id(Self::VALUE_SIZE), colour));
        job
    }
}
