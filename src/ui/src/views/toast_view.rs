use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Align2, Context, LayerId, Order, Rect, pos2, vec2};
use minimap_core::structs::ui::{MessageKind, MessageLine};

use crate::id::MinimapUiId;

// the message line at the top centre, over either map: a check for something done, an info mark
// for a note. It takes no input, so it is painted straight on a foreground layer and shows in the
// frame that set it. The amml-ui toast has a dismiss button and a timer this line does not have, so it
// is built from amml-ui parts here. The words come from minimap-core and the runtime and stay English
// for now
pub struct ToastView;

impl ToastView {
    const PAD: (f32, f32) = (14.0, 9.0);
    const ICON_GAP: f32 = 22.0;
    const STYLE: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Regular);

    pub fn show(ctx: &Context, message: &MessageLine) {
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, MinimapUiId::Toast.id()));
        let text = <color::Text as ColorToken>::VALUE;
        let room =
            ctx.content_rect().width() - <spacing::L as SpacingToken>::VALUE * 2.0 - Self::ICON_GAP - Self::PAD.0 * 2.0;
        let galley = Draw::galley(&painter, &message.text, Self::STYLE, text, TextFit::Truncate(room));
        let size = vec2(galley.size().x + Self::ICON_GAP + Self::PAD.0 * 2.0, galley.size().y + Self::PAD.1 * 2.0);
        let top = pos2(ctx.content_rect().center().x - size.x * 0.5, <spacing::L as SpacingToken>::VALUE);
        let pill = Rect::from_min_size(top, size);
        let fill = <color::BgRaised as ColorToken>::VALUE;
        Draw::surface(
            &painter,
            pill,
            Corner::Full,
            fill,
            Some(<color::LineStrong as ColorToken>::VALUE),
            Elevation::Floating,
        );
        let (icon, colour) = match message.kind {
            MessageKind::Done => (Icon::Check, <color::Success as ColorToken>::VALUE),
            MessageKind::Info => (Icon::Info, <color::TextDim as ColorToken>::VALUE),
        };
        let y = pill.center().y;
        let side = IconSize::Small.points();
        let icon_rect = Rect::from_center_size(pos2(pill.left() + Self::PAD.0 + side * 0.5, y), vec2(side, side));
        icon.paint(&painter, icon_rect, IconSize::Small, colour);
        let at = Align2::LEFT_CENTER.anchor_size(pos2(pill.left() + Self::PAD.0 + Self::ICON_GAP, y), galley.size());
        painter.galley(at.min, galley, text);
    }
}
