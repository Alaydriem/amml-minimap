use amml_i18n::localize::Localizer;
use amml_i18n::text::{Args, Text, UserText};
use amml_ui::draw::{TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiLocale;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{Surface, SurfaceLevel};
use egui::{Align2, Area, Context, RichText, vec2};
use minimap_core::structs::ui::HintKeys;

use crate::id::MinimapUiId;
use crate::message::MinimapMessage;

// the pill at the bottom right that names the world map's controls; it takes no input
pub struct HintView;

impl HintView {
    const STYLE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);

    /// The hint in the language of `localizer`, with the world map's key chord as typed in the settings.
    pub fn text(keys: HintKeys<'_>, localizer: &Localizer) -> String {
        let args = Args::new().user("key", UserText::from_model(keys.toggle.to_owned()));
        Text::format(MinimapMessage::Hint, args).resolve(localizer).into_owned()
    }

    pub fn show(ctx: &Context, keys: HintKeys<'_>) {
        let gap = <spacing::L as SpacingToken>::VALUE;
        Area::new(MinimapUiId::Hint.id())
            .anchor(Align2::RIGHT_BOTTOM, vec2(-gap, -gap))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                // the pill never grows past the screen less the margins; a long key chord is cut
                ui.set_max_width(ctx.content_rect().width() - gap * 2.0);
                Surface::frame(SurfaceLevel::Pill, Elevation::Flat).show(ui, |ui| {
                    let text = RichText::new(Self::text(keys, &UiLocale::get(ctx))).font(Self::STYLE.font_id());
                    ui.add(egui::Label::new(text.color(<color::TextDim as ColorToken>::VALUE)).truncate());
                });
            });
    }
}
